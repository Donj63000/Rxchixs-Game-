pub mod commandes;
pub mod lignes;
pub mod personnel;
pub mod stock;
pub mod vente;

pub use commandes::SimCommand;
pub use lignes::{LineOperationalState, ProductionLineId, ProductionLineState};
pub use personnel::{EmployeeRole, PersonnelState};
pub use stock::{RAW_LINE_INPUT_CAPACITY, StockState};
pub use vente::SalesState;
