use serde::{Deserialize, Serialize};

pub type PurchaseOrderId = u64;

pub const RAW_UNIT_COST_EUR: f64 = 1.20;
pub const RAW_MIN_PURCHASE_QTY: u32 = 100;
pub const RAW_DELIVERY_DELAY_S: f64 = 30.0 * 60.0;
pub const RAW_RECEIVING_CAPACITY: u32 = 2_000;
pub const RAW_LINE_INPUT_CAPACITY: u32 = 120;
const RAW_TRANSFER_PER_CARISTE_PER_HOUR: f64 = 360.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StockItemKind {
    MatierePremiere,
    ProduitFini,
    Rebut,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: PurchaseOrderId,
    pub item_kind: StockItemKind,
    pub qty: u32,
    pub remaining_delivery_s: f64,
    pub unit_cost_eur: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockState {
    pub raw_receiving: u32,
    pub raw_line_input: u32,
    pub pending_orders: Vec<PurchaseOrder>,
    pub next_order_id: PurchaseOrderId,
    transfer_accum: f64,
}

impl Default for StockState {
    fn default() -> Self {
        Self {
            raw_receiving: 0,
            raw_line_input: 0,
            pending_orders: Vec::new(),
            next_order_id: 1,
            transfer_accum: 0.0,
        }
    }
}

impl StockState {
    #[allow(dead_code)]
    pub fn sandbox_start() -> Self {
        Self {
            raw_receiving: 300,
            raw_line_input: 60,
            ..Self::default()
        }
    }

    pub fn raw_purchase_cost(qty: u32) -> f64 {
        qty as f64 * RAW_UNIT_COST_EUR
    }

    pub fn pending_raw_qty(&self) -> u32 {
        self.pending_orders
            .iter()
            .filter(|order| order.item_kind == StockItemKind::MatierePremiere)
            .map(|order| order.qty)
            .sum()
    }

    pub fn raw_capacity_reserved(&self) -> u32 {
        self.raw_receiving
            .saturating_add(self.raw_line_input)
            .saturating_add(self.pending_raw_qty())
    }

    pub fn can_buy_raw(&self, qty: u32, cash: f64) -> Result<f64, String> {
        if qty < RAW_MIN_PURCHASE_QTY {
            return Err(format!("commande minimale: {RAW_MIN_PURCHASE_QTY} unites"));
        }
        let cost = Self::raw_purchase_cost(qty);
        if !cash.is_finite() || cash < cost {
            return Err(format!("tresorerie insuffisante: {cost:.0} EUR requis"));
        }
        let reserved = self.raw_capacity_reserved();
        if reserved.saturating_add(qty) > RAW_RECEIVING_CAPACITY {
            return Err(format!(
                "capacite reception insuffisante: {reserved}/{RAW_RECEIVING_CAPACITY} reserves"
            ));
        }
        Ok(cost)
    }

    pub fn place_raw_order(
        &mut self,
        qty: u32,
        cash: f64,
    ) -> Result<(PurchaseOrderId, f64), String> {
        let cost = self.can_buy_raw(qty, cash)?;
        let id = self.next_order_id;
        self.next_order_id = self.next_order_id.saturating_add(1).max(1);
        self.pending_orders.push(PurchaseOrder {
            id,
            item_kind: StockItemKind::MatierePremiere,
            qty,
            remaining_delivery_s: RAW_DELIVERY_DELAY_S,
            unit_cost_eur: RAW_UNIT_COST_EUR,
        });
        Ok((id, cost))
    }

    pub fn tick_purchase_orders(&mut self, dt_s: f64) -> u32 {
        if !dt_s.is_finite() || dt_s <= 0.0 {
            return 0;
        }

        let mut delivered = 0u32;
        for order in &mut self.pending_orders {
            order.remaining_delivery_s = (order.remaining_delivery_s - dt_s).max(0.0);
            if order.remaining_delivery_s <= f64::EPSILON {
                match order.item_kind {
                    StockItemKind::MatierePremiere => {
                        let room = RAW_RECEIVING_CAPACITY.saturating_sub(self.raw_receiving);
                        let accepted = order.qty.min(room);
                        self.raw_receiving = self.raw_receiving.saturating_add(accepted);
                        delivered = delivered.saturating_add(accepted);
                        order.qty -= accepted;
                    }
                    StockItemKind::ProduitFini | StockItemKind::Rebut => {}
                }
            }
        }
        self.pending_orders.retain(|order| order.qty > 0);
        delivered
    }

    pub fn tick_cariste_transfer(&mut self, dt_hours: f64, caristes: usize) -> u32 {
        if !dt_hours.is_finite() || dt_hours <= 0.0 || caristes == 0 {
            return 0;
        }
        let room = RAW_LINE_INPUT_CAPACITY.saturating_sub(self.raw_line_input);
        if room == 0 || self.raw_receiving == 0 {
            self.transfer_accum = 0.0;
            return 0;
        }

        self.transfer_accum += dt_hours * RAW_TRANSFER_PER_CARISTE_PER_HOUR * caristes as f64;
        let wanted = self.transfer_accum.floor() as u32;
        let moved = wanted.min(room).min(self.raw_receiving);
        if moved > 0 {
            self.raw_receiving -= moved;
            self.raw_line_input += moved;
            self.transfer_accum -= moved as f64;
        }
        moved
    }

    pub fn has_any_raw_for_line(&self) -> bool {
        self.raw_receiving > 0 || self.raw_line_input > 0 || self.pending_raw_qty() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buy_raw_stock_checks_cash_and_capacity() {
        let mut stock = StockState::default();
        assert!(stock.place_raw_order(50, 1_000.0).is_err());
        assert!(stock.place_raw_order(100, 10.0).is_err());
        let (id, cost) = stock.place_raw_order(100, 1_000.0).unwrap();
        assert_eq!(id, 1);
        assert_eq!(cost, 120.0);
        assert_eq!(stock.pending_raw_qty(), 100);
    }

    #[test]
    fn purchase_order_delivers_after_delay() {
        let mut stock = StockState::default();
        stock.place_raw_order(100, 1_000.0).unwrap();

        assert_eq!(stock.tick_purchase_orders(RAW_DELIVERY_DELAY_S - 1.0), 0);
        assert_eq!(stock.raw_receiving, 0);
        assert_eq!(stock.tick_purchase_orders(1.0), 100);
        assert_eq!(stock.raw_receiving, 100);
        assert_eq!(stock.pending_orders.len(), 0);
    }

    #[test]
    fn cariste_transfer_respects_line_capacity() {
        let mut stock = StockState {
            raw_receiving: 500,
            raw_line_input: RAW_LINE_INPUT_CAPACITY - 2,
            ..StockState::default()
        };
        let moved = stock.tick_cariste_transfer(1.0, 4);
        assert_eq!(moved, 2);
        assert_eq!(stock.raw_line_input, RAW_LINE_INPUT_CAPACITY);
    }
}
