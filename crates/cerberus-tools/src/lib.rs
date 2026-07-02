use cerberus_policy::RiskLevel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolProposal {
    pub id: Uuid,
    pub name: String,
    pub args: Vec<String>,
    pub risk: RiskLevel,
    pub reason: String,
}

impl ToolProposal {
    pub fn new(name: impl Into<String>, risk: RiskLevel, reason: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            args: Vec::new(),
            risk,
            reason: reason.into(),
        }
    }
}
