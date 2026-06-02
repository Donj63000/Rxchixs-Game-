use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SalesBlockReason {
    Operationnel,
    BureauManquant,
    AdministrateurManquant,
    StockFiniAbsent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SalesState {
    pub sale_accum: f64,
    pub last_units_per_hour: f64,
    pub last_revenue_per_hour: f64,
    pub last_block_reason: SalesBlockReason,
}

impl Default for SalesState {
    fn default() -> Self {
        Self {
            sale_accum: 0.0,
            last_units_per_hour: 0.0,
            last_revenue_per_hour: 0.0,
            last_block_reason: SalesBlockReason::StockFiniAbsent,
        }
    }
}

impl SalesState {
    pub fn capacity_units_per_hour(admins: usize, offices: usize) -> f64 {
        let seats = admins.min(offices);
        seats as f64 * 6.0
    }

    pub fn tick(
        &mut self,
        dt_hours: f64,
        finished_stock: &mut u32,
        admins: usize,
        offices: usize,
        sale_price: f64,
    ) -> (u32, f64) {
        self.last_units_per_hour = 0.0;
        self.last_revenue_per_hour = 0.0;

        if *finished_stock == 0 {
            self.sale_accum = 0.0;
            self.last_block_reason = SalesBlockReason::StockFiniAbsent;
            return (0, 0.0);
        }
        if offices == 0 {
            self.sale_accum = 0.0;
            self.last_block_reason = SalesBlockReason::BureauManquant;
            return (0, 0.0);
        }
        if admins == 0 {
            self.sale_accum = 0.0;
            self.last_block_reason = SalesBlockReason::AdministrateurManquant;
            return (0, 0.0);
        }
        if !dt_hours.is_finite() || dt_hours <= 0.0 {
            self.last_block_reason = SalesBlockReason::Operationnel;
            return (0, 0.0);
        }

        let capacity = Self::capacity_units_per_hour(admins, offices);
        self.sale_accum += capacity * dt_hours;
        let sellable = self.sale_accum.floor() as u32;
        let sold = sellable.min(*finished_stock);
        if sold > 0 {
            *finished_stock -= sold;
            self.sale_accum -= sold as f64;
        }
        let revenue = sold as f64 * sale_price.max(0.0);
        self.last_units_per_hour = capacity;
        self.last_revenue_per_hour = capacity * sale_price.max(0.0);
        self.last_block_reason = SalesBlockReason::Operationnel;
        (sold, revenue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sales_are_progressive_and_require_admin_and_office() {
        let mut sales = SalesState::default();
        let mut stock = 3;

        let (sold, revenue) = sales.tick(1.0, &mut stock, 0, 1, 10.0);
        assert_eq!(sold, 0);
        assert_eq!(revenue, 0.0);
        assert_eq!(
            sales.last_block_reason,
            SalesBlockReason::AdministrateurManquant
        );

        let (sold, revenue) = sales.tick(0.5, &mut stock, 1, 1, 10.0);
        assert_eq!(sold, 3);
        assert_eq!(revenue, 30.0);
        assert_eq!(stock, 0);
    }
}
