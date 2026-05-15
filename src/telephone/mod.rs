mod contacts;
mod etat;
mod interface;

pub use contacts::{ContactTelephone, contact_label, contacts_disponibles};
pub use etat::TelephoneEtat;
pub use interface::{
    draw_telephone_panel, process_telephone_panel_input, telephone_panel_contains_mouse,
};
