#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactTelephone {
    PereBatisseur,
}

const CONTACTS_DISPONIBLES: [ContactTelephone; 1] = [ContactTelephone::PereBatisseur];

pub fn contacts_disponibles() -> &'static [ContactTelephone] {
    &CONTACTS_DISPONIBLES
}

pub fn contact_label(contact: ContactTelephone) -> &'static str {
    match contact {
        ContactTelephone::PereBatisseur => "Pere batisseur",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_pere_batisseur_is_available() {
        assert_eq!(contacts_disponibles(), &[ContactTelephone::PereBatisseur]);
        assert_eq!(
            contact_label(ContactTelephone::PereBatisseur),
            "Pere batisseur"
        );
    }
}
