use serde::{Deserialize, Serialize};

use super::personnel::{EmployeeId, EmployeeRole, ProductionLineId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimCommand {
    HireEmployee {
        role: EmployeeRole,
    },
    FireEmployee {
        employee_id: EmployeeId,
    },
    AssignEmployeeToLine {
        employee_id: EmployeeId,
        line_id: ProductionLineId,
    },
    SetLineTempPolicy {
        line_id: ProductionLineId,
        enabled: bool,
        max_temps: u8,
    },
    BuyRawStock {
        qty: u32,
    },
}
