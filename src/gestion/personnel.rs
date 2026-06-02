use serde::{Deserialize, Serialize};

pub type EmployeeId = u64;
pub type ProductionLineId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeRole {
    Patron,
    ChefEquipe,
    Cariste,
    AdministrateurVente,
    Interimaire,
}

impl EmployeeRole {
    pub fn label(self) -> &'static str {
        match self {
            Self::Patron => "Patron",
            Self::ChefEquipe => "Chef d'equipe",
            Self::Cariste => "Cariste",
            Self::AdministrateurVente => "Administrateur vente",
            Self::Interimaire => "Interimaire",
        }
    }

    pub fn hourly_wage_eur(self) -> f64 {
        match self {
            Self::Patron => 0.0,
            Self::ChefEquipe => 32.0,
            Self::Cariste => 24.0,
            Self::AdministrateurVente => 27.0,
            Self::Interimaire => 30.0,
        }
    }

    pub fn hiring_cost_eur(self) -> f64 {
        match self {
            Self::Patron | Self::Interimaire => 0.0,
            Self::ChefEquipe => 900.0,
            Self::Cariste => 450.0,
            Self::AdministrateurVente => 520.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeStatus {
    Disponible,
    Occupe,
    EnPause,
    Absent,
    Termine,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmployeeSkills {
    pub management: u8,
    pub logistique: u8,
    pub vente: u8,
    pub technique: u8,
    pub fiabilite: u8,
}

impl EmployeeSkills {
    pub fn for_role(role: EmployeeRole) -> Self {
        match role {
            EmployeeRole::Patron => Self {
                management: 70,
                logistique: 45,
                vente: 55,
                technique: 45,
                fiabilite: 80,
            },
            EmployeeRole::ChefEquipe => Self {
                management: 72,
                logistique: 45,
                vente: 35,
                technique: 66,
                fiabilite: 78,
            },
            EmployeeRole::Cariste => Self {
                management: 30,
                logistique: 76,
                vente: 20,
                technique: 52,
                fiabilite: 74,
            },
            EmployeeRole::AdministrateurVente => Self {
                management: 45,
                logistique: 28,
                vente: 78,
                technique: 38,
                fiabilite: 76,
            },
            EmployeeRole::Interimaire => Self {
                management: 28,
                logistique: 48,
                vente: 25,
                technique: 45,
                fiabilite: 58,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TempPolicy {
    pub enabled: bool,
    pub max_temps: u8,
    pub min_cash_reserve: f64,
}

impl Default for TempPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            max_temps: 3,
            min_cash_reserve: 5_000.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Employee {
    pub id: EmployeeId,
    pub name: String,
    pub role: EmployeeRole,
    pub status: EmployeeStatus,
    pub skills: EmployeeSkills,
    pub hourly_wage: f64,
    pub hired_at_s: f64,
    pub assigned_line: Option<ProductionLineId>,
    pub supervisor_id: Option<EmployeeId>,
    pub temp_contract_remaining_s: f64,
    pub temp_policy: Option<TempPolicy>,
    pub fatigue: f32,
    pub morale: f32,
}

impl Employee {
    fn new(id: EmployeeId, role: EmployeeRole, now_s: f64) -> Self {
        Self {
            id,
            name: deterministic_name(role, id),
            role,
            status: EmployeeStatus::Disponible,
            skills: EmployeeSkills::for_role(role),
            hourly_wage: role.hourly_wage_eur(),
            hired_at_s: now_s.max(0.0),
            assigned_line: None,
            supervisor_id: None,
            temp_contract_remaining_s: 0.0,
            temp_policy: matches!(role, EmployeeRole::ChefEquipe).then(TempPolicy::default),
            fatigue: 0.0,
            morale: 72.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonnelState {
    pub employees: Vec<Employee>,
    pub next_employee_id: EmployeeId,
}

impl Default for PersonnelState {
    fn default() -> Self {
        Self {
            employees: Vec::new(),
            next_employee_id: 1,
        }
    }
}

impl PersonnelState {
    pub fn sandbox_start(now_s: f64, line_id: ProductionLineId) -> Self {
        let mut state = Self::default();
        let lead = state
            .hire(EmployeeRole::ChefEquipe, now_s)
            .expect("sandbox lead should be hireable");
        state
            .assign_to_line(lead, line_id)
            .expect("sandbox lead should be assignable");
        state
            .hire(EmployeeRole::Cariste, now_s)
            .expect("sandbox cariste should be hireable");
        state
            .hire(EmployeeRole::AdministrateurVente, now_s)
            .expect("sandbox admin should be hireable");
        state
    }

    pub fn hire(&mut self, role: EmployeeRole, now_s: f64) -> Result<EmployeeId, String> {
        if role == EmployeeRole::Interimaire {
            return Err("Les interimaires sont geres par un chef d'equipe".to_string());
        }
        let id = self.next_employee_id;
        self.next_employee_id = self.next_employee_id.saturating_add(1).max(1);
        self.employees.push(Employee::new(id, role, now_s));
        Ok(id)
    }

    pub fn hire_temp_for_lead(
        &mut self,
        lead_id: EmployeeId,
        line_id: ProductionLineId,
        now_s: f64,
        contract_s: f64,
    ) -> Result<EmployeeId, String> {
        let lead = self
            .employee(lead_id)
            .ok_or_else(|| format!("chef introuvable: {lead_id}"))?;
        if lead.role != EmployeeRole::ChefEquipe {
            return Err("un interimaire doit etre rattache a un chef d'equipe".to_string());
        }

        let id = self.next_employee_id;
        self.next_employee_id = self.next_employee_id.saturating_add(1).max(1);
        let mut temp = Employee::new(id, EmployeeRole::Interimaire, now_s);
        temp.assigned_line = Some(line_id);
        temp.supervisor_id = Some(lead_id);
        temp.temp_contract_remaining_s = contract_s.max(0.0);
        self.employees.push(temp);
        Ok(id)
    }

    pub fn fire(&mut self, id: EmployeeId) -> Result<(), String> {
        let Some(employee) = self.employee(id) else {
            return Err(format!("employe introuvable: {id}"));
        };
        if employee.role == EmployeeRole::Patron {
            return Err("le patron ne peut pas etre licencie".to_string());
        }

        self.employees
            .retain(|employee| employee.id != id && employee.supervisor_id != Some(id));
        Ok(())
    }

    pub fn assign_to_line(
        &mut self,
        id: EmployeeId,
        line_id: ProductionLineId,
    ) -> Result<(), String> {
        let employee = self
            .employee_mut(id)
            .ok_or_else(|| format!("employe introuvable: {id}"))?;
        if employee.role != EmployeeRole::ChefEquipe {
            return Err("seul un chef d'equipe peut etre assigne a une ligne".to_string());
        }
        employee.assigned_line = Some(line_id);
        Ok(())
    }

    pub fn set_temp_policy(
        &mut self,
        lead_id: EmployeeId,
        enabled: bool,
        max_temps: u8,
    ) -> Result<(), String> {
        let employee = self
            .employee_mut(lead_id)
            .ok_or_else(|| format!("chef introuvable: {lead_id}"))?;
        if employee.role != EmployeeRole::ChefEquipe {
            return Err("politique interim reservee aux chefs d'equipe".to_string());
        }
        let policy = employee.temp_policy.get_or_insert_with(TempPolicy::default);
        policy.enabled = enabled;
        policy.max_temps = max_temps.min(3);
        Ok(())
    }

    pub fn tick_temp_contracts(&mut self, dt_s: f64) {
        if !dt_s.is_finite() || dt_s <= 0.0 {
            return;
        }
        for employee in &mut self.employees {
            if employee.role == EmployeeRole::Interimaire {
                employee.temp_contract_remaining_s =
                    (employee.temp_contract_remaining_s - dt_s).max(0.0);
            }
        }
    }

    pub fn release_finished_temps_without_stock(&mut self, line_id: ProductionLineId) -> usize {
        let before = self.employees.len();
        self.employees.retain(|employee| {
            !(employee.role == EmployeeRole::Interimaire
                && employee.assigned_line == Some(line_id)
                && employee.temp_contract_remaining_s <= f64::EPSILON)
        });
        before.saturating_sub(self.employees.len())
    }

    #[cfg(test)]
    pub fn count_role(&self, role: EmployeeRole) -> usize {
        self.employees
            .iter()
            .filter(|employee| employee.role == role && employee.status != EmployeeStatus::Termine)
            .count()
    }

    pub fn available_role_count(&self, role: EmployeeRole) -> usize {
        self.employees
            .iter()
            .filter(|employee| {
                employee.role == role && employee.status == EmployeeStatus::Disponible
            })
            .count()
    }

    pub fn team_lead_for_line(&self, line_id: ProductionLineId) -> Option<&Employee> {
        self.employees.iter().find(|employee| {
            employee.role == EmployeeRole::ChefEquipe
                && employee.status != EmployeeStatus::Termine
                && employee.assigned_line == Some(line_id)
        })
    }

    pub fn active_temps_for_lead(&self, lead_id: EmployeeId) -> usize {
        self.employees
            .iter()
            .filter(|employee| {
                employee.role == EmployeeRole::Interimaire
                    && employee.status != EmployeeStatus::Termine
                    && employee.supervisor_id == Some(lead_id)
            })
            .count()
    }

    pub fn hourly_payroll_eur(&self) -> f64 {
        self.employees
            .iter()
            .filter(|employee| employee.status != EmployeeStatus::Termine)
            .map(|employee| employee.hourly_wage.max(0.0))
            .sum()
    }

    pub fn employee(&self, id: EmployeeId) -> Option<&Employee> {
        self.employees.iter().find(|employee| employee.id == id)
    }

    pub fn employee_mut(&mut self, id: EmployeeId) -> Option<&mut Employee> {
        self.employees.iter_mut().find(|employee| employee.id == id)
    }
}

fn deterministic_name(role: EmployeeRole, id: EmployeeId) -> String {
    let names = match role {
        EmployeeRole::Patron => &["Patron"][..],
        EmployeeRole::ChefEquipe => &["Nadia", "Romain", "Lea", "Karim"][..],
        EmployeeRole::Cariste => &["Karim", "Maya", "Sofiane", "Ines"][..],
        EmployeeRole::AdministrateurVente => &["Alice", "Hugo", "Salma", "Theo"][..],
        EmployeeRole::Interimaire => &["Interim A", "Interim B", "Interim C", "Interim D"][..],
    };
    let index = (id.saturating_sub(1) as usize) % names.len();
    names[index].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hire_assign_and_fire_are_deterministic() {
        let mut personnel = PersonnelState::default();
        let lead = personnel.hire(EmployeeRole::ChefEquipe, 0.0).unwrap();
        assert_eq!(lead, 1);
        assert_eq!(personnel.count_role(EmployeeRole::ChefEquipe), 1);

        personnel.assign_to_line(lead, 1).unwrap();
        assert_eq!(personnel.team_lead_for_line(1).map(|e| e.id), Some(lead));

        personnel.fire(lead).unwrap();
        assert_eq!(personnel.count_role(EmployeeRole::ChefEquipe), 0);
    }

    #[test]
    fn payroll_uses_real_active_employees() {
        let mut personnel = PersonnelState::default();
        personnel.hire(EmployeeRole::ChefEquipe, 0.0).unwrap();
        personnel.hire(EmployeeRole::Cariste, 0.0).unwrap();
        assert_eq!(personnel.hourly_payroll_eur(), 56.0);
    }

    #[test]
    fn temps_are_limited_by_supervisor_contracts() {
        let mut personnel = PersonnelState::default();
        let lead = personnel.hire(EmployeeRole::ChefEquipe, 0.0).unwrap();
        personnel.assign_to_line(lead, 1).unwrap();
        personnel
            .hire_temp_for_lead(lead, 1, 0.0, 10.0)
            .expect("temp should be hireable");
        assert_eq!(personnel.active_temps_for_lead(lead), 1);

        personnel.tick_temp_contracts(20.0);
        assert_eq!(personnel.release_finished_temps_without_stock(1), 1);
        assert_eq!(personnel.active_temps_for_lead(lead), 0);
    }
}
