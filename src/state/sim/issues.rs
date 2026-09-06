//! Persisted actionable aftermath and maintenance notices.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    #[default]
    Notice,
    Vulnerable,
    Critical,
}

impl IssueSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Notice => "NOTICE",
            Self::Vulnerable => "VULNERABLE",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Issue {
    pub food_production_penalty: f32,
    pub id: String,
    pub source: String,
    pub target: String,
    pub created_month: u32,
    pub severity: IssueSeverity,
    pub due_month: Option<u32>,
    pub recovery_project_ids: Vec<String>,
    pub acknowledged: bool,
    pub overdue_effect_applied: bool,
    pub resolved_month: Option<u32>,
    pub resolution: Option<String>,
}

impl Default for Issue {
    fn default() -> Self {
        Self {
            food_production_penalty: 0.0,
            id: String::new(),
            source: String::new(),
            target: String::new(),
            created_month: 0,
            severity: IssueSeverity::Notice,
            due_month: None,
            recovery_project_ids: Vec::new(),
            acknowledged: false,
            overdue_effect_applied: false,
            resolved_month: None,
            resolution: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct IssueState {
    pub active: Vec<Issue>,
    pub resolved: Vec<Issue>,
}

impl IssueState {
    pub fn active_issue(&self, id: &str) -> Option<&Issue> {
        self.active.iter().find(|issue| issue.id == id)
    }

    pub fn has_active(&self, id: &str) -> bool {
        self.active_issue(id).is_some()
    }
}
