//! Persistent institutions that carry an officer's craft beyond one lifetime.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Apprenticeship {
    pub post_id: String,
    pub apprentice_name: String,
    #[serde(default)]
    pub faction_id: String,
    pub skill: u32,
    pub designated_year: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubsystemSchool {
    pub subsystem_id: String,
    pub founded_year: u32,
    pub supported_until_year: u32,
    #[serde(default)]
    pub custodian_faction_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcedureArchive {
    pub subsystem_id: String,
    pub compiled_year: u32,
    #[serde(default)]
    pub preserved_experts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstitutionRecordKind {
    Appointment,
    ExpertiseLost,
    ExpertisePreserved,
    SchoolFounded,
    SchoolLapsed,
    ArchiveCompiled,
    CustodianshipGranted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstitutionRecord {
    pub year: u32,
    pub kind: InstitutionRecordKind,
    pub subject: String,
    pub discipline: String,
    pub knowledge_change: f32,
}
