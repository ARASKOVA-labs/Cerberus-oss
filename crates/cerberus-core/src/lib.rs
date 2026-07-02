use cerberus_policy::{PolicyDecision, PolicyEngine, RiskLevel};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mission {
    pub id: Uuid,
    pub title: String,
    pub objective: String,
    pub scope_root: String,
    pub created_at: OffsetDateTime,
}

impl Mission {
    pub fn new(title: impl Into<String>, objective: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            objective: objective.into(),
            scope_root: ".".to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPlan {
    pub mission_id: Uuid,
    pub steps: Vec<AgentStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentStep {
    pub id: Uuid,
    pub kind: StepKind,
    pub summary: String,
    pub risk: RiskLevel,
    pub decision: PolicyDecision,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepKind {
    Observe,
    Analyze,
    ProposeTool,
    RequestApproval,
    Execute,
    CollectEvidence,
    ReportFinding,
    Patch,
    Verify,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentEvent {
    pub time: OffsetDateTime,
    pub message: String,
}

#[derive(Debug)]
pub struct AgentKernel {
    policy: PolicyEngine,
}

impl AgentKernel {
    pub fn new(policy: PolicyEngine) -> Self {
        Self { policy }
    }

    pub fn plan_mission(&self, mission: &Mission) -> AgentPlan {
        let templates = [
            (
                StepKind::Observe,
                "Map repository structure",
                RiskLevel::Passive,
            ),
            (
                StepKind::Analyze,
                "Identify security-relevant surfaces",
                RiskLevel::Passive,
            ),
            (
                StepKind::ProposeTool,
                "Propose safe scanners",
                RiskLevel::ActiveSafe,
            ),
            (
                StepKind::CollectEvidence,
                "Capture evidence for any finding",
                RiskLevel::Passive,
            ),
            (
                StepKind::Verify,
                "Verify fixes before closing findings",
                RiskLevel::Passive,
            ),
        ];

        let steps = templates
            .into_iter()
            .map(|(kind, summary, risk)| AgentStep {
                id: Uuid::new_v4(),
                kind,
                summary: format!("{summary}: {}", mission.objective),
                risk,
                decision: self.policy.decide(risk),
            })
            .collect();

        AgentPlan {
            mission_id: mission.id,
            steps,
        }
    }

    pub fn boot_events(&self) -> Vec<AgentEvent> {
        vec![
            AgentEvent {
                time: OffsetDateTime::now_utc(),
                message: "CERBERUS_CORE initialized agent kernel".to_string(),
            },
            AgentEvent {
                time: OffsetDateTime::now_utc(),
                message: "POLICY_ENGINE online".to_string(),
            },
            AgentEvent {
                time: OffsetDateTime::now_utc(),
                message: "MISSION_RUNTIME waiting for operator objective".to_string(),
            },
        ]
    }
}
