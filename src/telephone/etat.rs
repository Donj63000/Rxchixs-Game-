use super::ContactTelephone;

#[derive(Clone, Debug, Default)]
pub struct TelephoneEtat {
    pub ouvert: bool,
    requete_appel: Option<ContactTelephone>,
    pub dernier_statut: Option<String>,
}

impl TelephoneEtat {
    pub fn basculer_ouverture(&mut self) {
        self.ouvert = !self.ouvert;
        if self.ouvert {
            self.dernier_statut = Some("Contacts ouverts".to_string());
        } else {
            self.dernier_statut = Some("Contacts fermes".to_string());
        }
    }

    pub fn demander_appel(&mut self, contact: ContactTelephone) {
        self.requete_appel = Some(contact);
    }

    pub fn prendre_requete_appel(&mut self) -> Option<ContactTelephone> {
        self.requete_appel.take()
    }

    pub fn definir_statut(&mut self, statut: impl Into<String>) {
        self.dernier_statut = Some(statut.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_request_is_queued_and_consumed_once() {
        let mut etat = TelephoneEtat::default();
        etat.demander_appel(ContactTelephone::PereBatisseur);
        assert_eq!(
            etat.prendre_requete_appel(),
            Some(ContactTelephone::PereBatisseur)
        );
        assert_eq!(etat.prendre_requete_appel(), None);
    }
}
