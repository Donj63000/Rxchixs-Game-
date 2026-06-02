use serde::{Deserialize, Serialize};

use super::personnel::EmployeeId;

pub type ProductionLineId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineOperationalState {
    Active,
    Bloquee,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductionLineState {
    pub id: ProductionLineId,
    pub label: String,
    pub assigned_lead_id: Option<EmployeeId>,
    pub status: LineOperationalState,
    pub block_reason: String,
    pub target_boxes_per_hour: f64,
    pub staffing_factor: f64,
    pub active_temps: u8,
}

impl ProductionLineState {
    pub fn main_line() -> Self {
        Self {
            id: 1,
            label: "Deshydratation 1".to_string(),
            assigned_lead_id: None,
            status: LineOperationalState::Bloquee,
            block_reason: "aucun chef assigne".to_string(),
            target_boxes_per_hour: 6.0,
            staffing_factor: 0.0,
            active_temps: 0,
        }
    }

    pub fn set_blocked(&mut self, reason: impl Into<String>) {
        self.status = LineOperationalState::Bloquee;
        self.block_reason = reason.into();
        self.staffing_factor = 0.0;
    }

    pub fn set_active(&mut self, lead_id: EmployeeId, active_temps: usize) {
        self.status = LineOperationalState::Active;
        self.assigned_lead_id = Some(lead_id);
        self.active_temps = active_temps.min(3) as u8;
        self.staffing_factor = staffing_factor_for_temps(active_temps);
        self.block_reason = "aucune".to_string();
    }
}

pub fn staffing_factor_for_temps(active_temps: usize) -> f64 {
    match active_temps.min(3) {
        0 => 0.70,
        1 => 0.85,
        2 => 1.00,
        _ => 1.15,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staffing_factor_is_bounded() {
        assert_eq!(staffing_factor_for_temps(0), 0.70);
        assert_eq!(staffing_factor_for_temps(2), 1.00);
        assert_eq!(staffing_factor_for_temps(9), 1.15);
    }
}
