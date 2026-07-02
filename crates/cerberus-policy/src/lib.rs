use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum RiskLevel {
    Passive,
    ActiveSafe,
    Intrusive,
    ExploitValidation,
    Forbidden,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow { risk: RiskLevel },
    RequireApproval { risk: RiskLevel, reason: String },
    Deny { risk: RiskLevel, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyEngine {
    pub max_auto_risk: RiskLevel,
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self {
            max_auto_risk: RiskLevel::Passive,
        }
    }
}

impl PolicyEngine {
    pub fn decide(&self, risk: RiskLevel) -> PolicyDecision {
        match risk {
            RiskLevel::Passive => PolicyDecision::Allow { risk },
            RiskLevel::ActiveSafe => PolicyDecision::RequireApproval {
                risk,
                reason: "active target interaction requires in-scope authorization".to_string(),
            },
            RiskLevel::Intrusive => PolicyDecision::RequireApproval {
                risk,
                reason: "intrusive testing requires explicit rules of engagement".to_string(),
            },
            RiskLevel::ExploitValidation => PolicyDecision::RequireApproval {
                risk,
                reason: "exploit validation requires written authorization".to_string(),
            },
            RiskLevel::Forbidden => PolicyDecision::Deny {
                risk,
                reason: "forbidden actions cannot be executed".to_string(),
            },
        }
    }
}

impl fmt::Display for PolicyDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow { risk } => write!(f, "allow:{risk:?}"),
            Self::RequireApproval { risk, reason } => write!(f, "approval:{risk:?}:{reason}"),
            Self::Deny { risk, reason } => write!(f, "deny:{risk:?}:{reason}"),
        }
    }
}
