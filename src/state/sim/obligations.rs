//! Persistent duties: promises with owners, deadlines, stakes, and history.

use serde::{Deserialize, Serialize};

use super::SimState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationVisibility {
    Public,
    Private,
    Disputed,
}

impl ObligationVisibility {
    pub fn label(self) -> &'static str {
        match self {
            Self::Public => "PUBLIC",
            Self::Private => "PRIVATE",
            Self::Disputed => "DISPUTED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationStatus {
    Pending,
    Fulfilled,
    Renegotiated,
    Defaulted,
    Void,
}

impl ObligationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Fulfilled => "FULFILLED",
            Self::Renegotiated => "RENEGOTIATED",
            Self::Defaulted => "DEFAULTED",
            Self::Void => "VOID",
        }
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Pending | Self::Renegotiated)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObligationStakes {
    pub material: String,
    pub reputation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObligationHistoryEntry {
    pub year: u32,
    pub captain: String,
    pub status: ObligationStatus,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Obligation {
    pub id: String,
    pub authored_id: String,
    pub title: String,
    pub source: String,
    pub creator: String,
    pub responsible: String,
    pub beneficiary: String,
    pub created_year: u32,
    pub due_year: Option<u32>,
    #[serde(default)]
    pub resolution_event: String,
    pub visibility: ObligationVisibility,
    pub status: ObligationStatus,
    pub stakes: ObligationStakes,
    pub successions_crossed: u32,
    #[serde(default)]
    pub history: Vec<ObligationHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObligationCreate {
    pub authored_id: String,
    pub title: String,
    pub source: String,
    pub beneficiary: String,
    pub due_in_years: Option<u32>,
    pub resolution_event: String,
    pub visibility: ObligationVisibility,
    pub material_stakes: String,
    pub reputation_stakes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ObligationOperation {
    Create(ObligationCreate),
    Fulfil {
        authored_id: String,
        note: String,
    },
    Renegotiate {
        authored_id: String,
        due_in_years: Option<u32>,
        note: String,
    },
    Default {
        authored_id: String,
        note: String,
    },
    Void {
        authored_id: String,
        note: String,
    },
}

impl SimState {
    pub fn active_obligations(&self) -> impl Iterator<Item = &Obligation> {
        self.obligations.iter().filter(|o| o.status.is_active())
    }

    pub fn due_obligations(&self) -> Vec<&Obligation> {
        let year = self.year();
        let mut due: Vec<_> = self
            .active_obligations()
            .filter(|o| o.due_year.is_some_and(|due| due <= year))
            .collect();
        due.sort_by_key(|o| (o.due_year, o.created_year, o.id.as_str()));
        due
    }

    pub fn apply_obligation_operation(&mut self, operation: &ObligationOperation) {
        let year = self.year();
        let captain = self
            .dynasty
            .leader()
            .map(|leader| leader.name.clone())
            .unwrap_or_else(|| "The vacant office".to_owned());
        match operation {
            ObligationOperation::Create(spec) => {
                if self.obligations.iter().any(|obligation| {
                    obligation.authored_id == spec.authored_id && obligation.status.is_active()
                }) {
                    return;
                }
                let id = format!("obligation-{:06}", self.next_obligation_id.max(1));
                self.next_obligation_id = self.next_obligation_id.max(1) + 1;
                let due_year = spec.due_in_years.map(|years| year.saturating_add(years));
                self.obligations.push(Obligation {
                    id,
                    authored_id: spec.authored_id.clone(),
                    title: spec.title.clone(),
                    source: spec.source.clone(),
                    creator: captain.clone(),
                    responsible: captain.clone(),
                    beneficiary: spec.beneficiary.clone(),
                    created_year: year,
                    due_year,
                    resolution_event: spec.resolution_event.clone(),
                    visibility: spec.visibility,
                    status: ObligationStatus::Pending,
                    stakes: ObligationStakes {
                        material: spec.material_stakes.clone(),
                        reputation: spec.reputation_stakes.clone(),
                    },
                    successions_crossed: 0,
                    history: vec![ObligationHistoryEntry {
                        year,
                        captain,
                        status: ObligationStatus::Pending,
                        note: "Promise entered in the ship's ledger.".to_owned(),
                    }],
                });
                if let Some(fire_year) = due_year.filter(|_| !spec.resolution_event.is_empty()) {
                    self.scheduled_events.push(super::ScheduledEvent {
                        template_id: spec.resolution_event.clone(),
                        fire_year,
                    });
                }
            }
            ObligationOperation::Fulfil { authored_id, note } => {
                self.resolve_obligation(authored_id, ObligationStatus::Fulfilled, note, None)
            }
            ObligationOperation::Renegotiate {
                authored_id,
                due_in_years,
                note,
            } => self.resolve_obligation(
                authored_id,
                ObligationStatus::Renegotiated,
                note,
                Some(due_in_years.map(|years| year.saturating_add(years))),
            ),
            ObligationOperation::Default { authored_id, note } => {
                self.resolve_obligation(authored_id, ObligationStatus::Defaulted, note, None)
            }
            ObligationOperation::Void { authored_id, note } => {
                self.resolve_obligation(authored_id, ObligationStatus::Void, note, None)
            }
        }
    }

    fn resolve_obligation(
        &mut self,
        authored_id: &str,
        status: ObligationStatus,
        note: &str,
        due_year: Option<Option<u32>>,
    ) {
        let year = self.year();
        let captain = self
            .dynasty
            .leader()
            .map(|leader| leader.name.clone())
            .unwrap_or_else(|| "The vacant office".to_owned());
        if let Some(obligation) = self
            .obligations
            .iter_mut()
            .rev()
            .find(|o| o.authored_id == authored_id && o.status.is_active())
        {
            obligation.status = status;
            obligation.responsible = captain.clone();
            if let Some(due_year) = due_year {
                obligation.due_year = due_year;
            }
            obligation.history.push(ObligationHistoryEntry {
                year,
                captain,
                status,
                note: note.to_owned(),
            });
            if status == ObligationStatus::Renegotiated {
                if let Some(fire_year) = obligation
                    .due_year
                    .filter(|_| !obligation.resolution_event.is_empty())
                {
                    self.scheduled_events.push(super::ScheduledEvent {
                        template_id: obligation.resolution_event.clone(),
                        fire_year,
                    });
                }
            }
        }
    }

    pub fn inherit_obligations(&mut self) {
        let year = self.year();
        let Some(captain) = self.dynasty.leader().map(|leader| leader.name.clone()) else {
            return;
        };
        let mut inherited = 0;
        for obligation in self.obligations.iter_mut().filter(|o| o.status.is_active()) {
            if obligation.responsible == captain {
                continue;
            }
            obligation.responsible = captain.clone();
            obligation.successions_crossed += 1;
            inherited += 1;
            obligation.history.push(ObligationHistoryEntry {
                year,
                captain: captain.clone(),
                status: obligation.status,
                note: "Responsibility inherited with the first chair.".to_owned(),
            });
        }
        if let Some(reign) = self.dynasty.reigns.last_mut() {
            reign.inherited_obligations = inherited;
        }
    }
}

#[cfg(test)]
mod tests;
