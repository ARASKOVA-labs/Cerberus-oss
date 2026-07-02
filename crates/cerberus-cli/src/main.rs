use anyhow::{bail, Context, Result};
use cerberus_core::{AgentKernel, AgentPlan, Mission};
use cerberus_llm::{LlmClient, LlmConfig};
use cerberus_memory::EvidenceRecord;
use cerberus_policy::{PolicyDecision, PolicyEngine, RiskLevel};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
};
use time::OffsetDateTime;
use uuid::Uuid;

mod tui;

#[derive(Debug, Parser)]
#[command(name = "cerberus")]
#[command(about = "Araskova Labs terminal-first agentic security framework")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Review code changes for security issues.
    Review(ReviewArgs),
    /// Run automated security tests on a target.
    Test(TestArgs),
    /// Interactive configuration setup.
    Setup,
    /// Platform gateway commands.
    Gateway {
        #[command(subcommand)]
        command: GatewayCommand,
    },
    /// Show runtime health.
    Doctor,
    /// Create a deterministic mission plan.
    Mission {
        /// Mission objective.
        objective: String,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Create or show the current mission plan.
    Plan {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Run governed checks.
    Run(RunArgs),
    /// Finding commands.
    Findings {
        #[command(subcommand)]
        command: FindingsCommand,
    },
    /// Export reports.
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    /// Mark findings as verified after retest.
    Verify(VerifyArgs),
    /// Policy commands.
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },
    /// LLM provider commands.
    Llm {
        #[command(subcommand)]
        command: LlmCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GatewayCommand {
    /// Start a platform gateway.
    Start {
        /// Platform to start (telegram or whatsapp).
        #[arg(long)]
        platform: String,
        /// Optional bot token (for telegram).
        #[arg(long)]
        token: Option<String>,
    },
}

#[derive(Debug, Args)]
struct ReviewArgs {
    /// Target path to review.
    #[arg(default_value = ".")]
    target: String,
    
    /// Run in CI mode (disables interactive dashboard and prints raw text).
    #[arg(long)]
    ci: bool,
}

#[derive(Debug, Args)]
struct TestArgs {
    /// Target URL or module to test.
    target: String,
}

#[derive(Debug, Args)]
struct RunArgs {
    /// Run passive checks only.
    #[arg(long)]
    passive: bool,
    /// Print machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct VerifyArgs {
    /// Finding ID to verify. If omitted, verifies all open findings.
    #[arg(long)]
    finding: Option<Uuid>,
    /// Evidence note for the verification.
    #[arg(long, default_value = "manual verification completed")]
    evidence: String,
    /// Print machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum FindingsCommand {
    /// List findings.
    List {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Add a finding with evidence.
    Add {
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "medium")]
        severity: Severity,
        #[arg(long)]
        evidence: String,
        #[arg(long, default_value = "Needs remediation.")]
        remediation: String,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ReportCommand {
    /// Export a Markdown or PDF report.
    Export {
        /// Report format.
        #[arg(long, default_value = "markdown")]
        format: ReportFormat,
        /// Output path.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum PolicyCommand {
    /// Check a risk level against policy.
    Check {
        #[arg(long, default_value = "passive")]
        risk: RiskLevel,
    },
}

#[derive(Debug, Subcommand)]
enum LlmCommand {
    /// Show configured provider status.
    Status {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        model: Option<String>,
    },
    /// Ask the configured model.
    Ask {
        prompt: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        model: Option<String>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct Vulnerability {
    severity: String,
    description: String,
    remediation: String,
    file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Review(args) => {
            let llm = LlmConfig::from_env(None, None)?;
            println!("Analyzing target: {} ...", args.target);
            
            // Execute `git diff` on the target path
            let mut diff_cmd = ProcessCommand::new("git");
            diff_cmd.arg("diff").arg("HEAD").arg(&args.target);
            let mut output = diff_cmd.output()?;
            
            if !output.status.success() {
                // Fallback for brand new repos without a HEAD commit
                diff_cmd = ProcessCommand::new("git");
                diff_cmd.arg("diff").arg(&args.target);
                output = diff_cmd.output()?;
            }
                
            let diff = String::from_utf8_lossy(&output.stdout);
            if diff.trim().is_empty() {
                println!("No uncommitted changes found for target.");
                return Ok(());
            }

            let client = cerberus_llm::LlmClient::new(llm);
            let prompt = format!("Review this code diff for security vulnerabilities based on OWASP standards:\n\n{}", diff);
            
            match client.ask(&prompt).await {
                Ok(res) => {
                    // Extract JSON array between [ and ]
                    let start_idx = res.find('[').unwrap_or(0);
                    let end_idx = res.rfind(']').map(|i| i + 1).unwrap_or(res.len());
                    let clean_json = if start_idx < end_idx {
                        &res[start_idx..end_idx]
                    } else {
                        res.trim()
                    };

                    match serde_json::from_str::<Vec<Vulnerability>>(clean_json) {
                        Ok(vulns) => {
                            if args.ci {
                                if vulns.is_empty() {
                                    println!("\n[CERBERUS SECURITY REVIEW]\n✅ No vulnerabilities found!");
                                } else {
                                    println!("\n[CERBERUS SECURITY REVIEW]\n🚨 Found {} vulnerabilities:\n", vulns.len());
                                    for v in vulns {
                                        println!("- [{}] {}: {}", v.severity.to_uppercase(), v.file, v.description);
                                        println!("  Remediation: {}\n", v.remediation);
                                    }
                                }
                            } else {
                                if let Err(e) = tui::run_vulnerability_dashboard(vulns) {
                                    eprintln!("TUI Error: {}", e);
                                }
                            }
                        }
                        Err(_) => {
                            println!("\n[CERBERUS SECURITY REVIEW - RAW OUTPUT]\n\n{}", res);
                        }
                    }
                }
                Err(e) => eprintln!("Failed to generate review: {}", e),
            }
        }
        Command::Test(args) => {
            let llm = LlmConfig::from_env(None, None)?;
            println!("Generating security tests for target: {} ...", args.target);
            
            let client = cerberus_llm::LlmClient::new(llm);
            let prompt = format!("Generate an automated security test script (like Selenium or Python requests) to test this target for common vulnerabilities: {}", args.target);
            
            match client.ask(&prompt).await {
                Ok(res) => println!("\n[CERBERUS AUTOMATED TEST GENERATOR]\n\n{}", res),
                Err(e) => eprintln!("Failed to generate tests: {}", e),
            }
        }
        Command::Setup => {
            let selections = &["Anthropic (Claude 3.5 Sonnet)", "OpenAI (GPT-4o)", "Local (Ollama / LMStudio)"];
            let selection = dialoguer::Select::new()
                .with_prompt("Select LLM Provider")
                .default(0)
                .items(&selections[..])
                .interact()?;
            
            let provider = match selection {
                0 => "anthropic",
                1 => "openai",
                2 => "openai-compatible",
                _ => "offline",
            };
            
            let config_content = format!("[llm]\nprovider = \"{}\"\n", provider);
            let workspace = Workspace::default();
            workspace.ensure()?;
            std::fs::write(workspace.root.join("config.toml"), config_content)?;
            println!("Configuration saved to .cerberus/config.toml");
        }
        Command::Gateway { command } => match command {
            GatewayCommand::Start { platform, token } => {
                let workspace = Workspace::default();
                workspace.ensure()?;
                if platform == "telegram" {
                    if let Some(t) = token.or_else(|| std::env::var("TELOXIDE_TOKEN").ok()) {
                        println!("Starting Telegram gateway...");
                        let db_path = workspace.root.join("state.db");
                        let db = std::sync::Arc::new(tokio::sync::Mutex::new(
                            cerberus_memory::StateDB::new(db_path).unwrap()
                        ));
                        cerberus_gateway::run_telegram_bot(t, db).await?;
                    } else {
                        eprintln!("Error: Telegram token required (via --token or TELOXIDE_TOKEN env var)");
                    }
                } else if platform == "whatsapp" {
                    println!("Starting WhatsApp gateway...");
                    let mut bridge = cerberus_gateway::whatsapp::WhatsappBridge::start(std::env::current_dir()?)?;
                    tokio::signal::ctrl_c().await?;
                    println!("Stopping WhatsApp bridge...");
                    bridge.stop()?;
                } else {
                    eprintln!("Unknown platform: {}. Use 'telegram' or 'whatsapp'.", platform);
                }
            }
        }
        Command::Doctor => {
            let logo = r#"
             ___________________
            < Cerberus v0.1.0   >
             -------------------
                    \   ^__^
                     \  (oo)\_______
                        (__)\       )\/\
                            ||----w |
                            ||     ||
            "#;
            println!("{}", logo);
            println!("Cerberus: initialized");
            println!("Agent kernel: ready");
            println!("Policy engine: guarded");
            println!("Interface: terminal-only");
            println!("Memory engine: SQLite WAL + FTS5");
            println!("Gateways: Telegram, WhatsApp");
            println!("Workspace: {}", Workspace::default().root.display());
        }
        Command::Mission { objective, json } => {
            let mission = Mission::new("operator mission", objective);
            let kernel = AgentKernel::new(PolicyEngine::default());
            let plan = kernel.plan_mission(&mission);
            let mut workspace = Workspace::default();
            workspace.save_mission(&mission)?;
            workspace.save_plan(&plan)?;
            if json {
                print_json(&MissionPlanOutput { mission, plan })?;
            } else {
                print_plan(&mission, &plan);
                println!("saved: {}", workspace.root.display());
            }
        }
        Command::Plan { json } => {
            let workspace = Workspace::default();
            let mission = workspace.load_mission()?;
            let plan = match workspace.load_plan() {
                Ok(plan) => plan,
                Err(_) => {
                    let kernel = AgentKernel::new(PolicyEngine::default());
                    let plan = kernel.plan_mission(&mission);
                    workspace.save_plan(&plan)?;
                    plan
                }
            };
            if json {
                print_json(&MissionPlanOutput { mission, plan })?;
            } else {
                print_plan(&mission, &plan);
            }
        }
        Command::Run(args) => {
            let workspace = Workspace::default();
            let mission = workspace.load_mission()?;
            let run = run_active_plan(&workspace, &mission)?;
            if args.json {
                print_json(&run)?;
            } else {
                println!("run: {}", run.id);
                println!("mode: {}", run.mode);
                for event in &run.events {
                    println!("- {} [{:?}] {}", event.name, event.decision, event.summary);
                }
                println!("evidence: {}", workspace.evidence_file.display());
            }
        }
        Command::Findings { command } => match command {
            FindingsCommand::List { json } => {
                let workspace = Workspace::default();
                let findings = workspace.load_findings()?;
                if json {
                    print_json(&findings)?;
                } else if findings.is_empty() {
                    println!("no findings");
                } else {
                    for finding in findings {
                        println!(
                            "{} [{:?}] {:?} {}",
                            finding.id, finding.severity, finding.status, finding.title
                        );
                    }
                }
            }
            FindingsCommand::Add {
                title,
                severity,
                evidence,
                remediation,
                json,
            } => {
                let workspace = Workspace::default();
                let mission = workspace.load_mission()?;
                let evidence_record =
                    workspace.add_evidence(&mission, "finding", evidence.clone())?;
                let finding = Finding {
                    id: Uuid::new_v4(),
                    mission_id: mission.id,
                    title,
                    severity,
                    status: FindingStatus::Open,
                    evidence_ids: vec![evidence_record.id],
                    evidence,
                    remediation,
                    created_at: OffsetDateTime::now_utc(),
                    verified_at: None,
                };
                workspace.add_finding(finding.clone())?;
                if json {
                    print_json(&finding)?;
                } else {
                    println!("finding: {}", finding.id);
                    println!("evidence: {}", evidence_record.id);
                }
            }
        },
        Command::Report { command } => match command {
            ReportCommand::Export { format, out, json } => {
                let workspace = Workspace::default();
                let mission = workspace.load_mission()?;
                let findings = workspace.load_findings()?;
                let out = out.unwrap_or_else(|| default_report_path(format));
                match format {
                    ReportFormat::Markdown => {
                        let report = render_markdown_report(&mission, &findings);
                        write_report_file(&out, report.as_bytes())?;
                    }
                    ReportFormat::Pdf => {
                        export_pdf_report(&workspace, &out)?;
                    }
                }
                let output = ReportOutput {
                    path: out,
                    format,
                    finding_count: findings.len(),
                };
                if json {
                    print_json(&output)?;
                } else {
                    println!("report: {}", output.path.display());
                    println!("findings: {}", output.finding_count);
                }
            }
        },
        Command::Verify(args) => {
            let workspace = Workspace::default();
            let mission = workspace.load_mission()?;
            let evidence = workspace.add_evidence(&mission, "verification", args.evidence)?;
            let verified = workspace.verify_findings(args.finding, evidence.id)?;
            if args.json {
                print_json(&verified)?;
            } else {
                println!("verified findings: {}", verified.len());
                for finding in verified {
                    println!("- {}", finding);
                }
            }
        }
        Command::Policy { command } => match command {
            PolicyCommand::Check { risk } => {
                let decision = PolicyEngine::default().decide(risk);
                println!("{decision}");
            }
        },
        Command::Llm { command } => match command {
            LlmCommand::Status { provider, model } => {
                let config = LlmConfig::from_env(provider.as_deref(), model.as_deref())?;
                for line in config.status_lines() {
                    println!("{line}");
                }
            }
            LlmCommand::Ask {
                prompt,
                provider,
                model,
            } => {
                let config = LlmConfig::from_env(provider.as_deref(), model.as_deref())?;
                let client = LlmClient::new(config);
                println!("{}", client.ask(&prompt).await?);
            }
        },
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MissionPlanOutput {
    mission: Mission,
    plan: AgentPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunOutput {
    id: Uuid,
    mission_id: Uuid,
    mode: String,
    events: Vec<RunEvent>,
    created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunEvent {
    name: String,
    summary: String,
    decision: PolicyDecision,
    evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportOutput {
    path: PathBuf,
    format: ReportFormat,
    finding_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
enum ReportFormat {
    Markdown,
    Pdf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum FindingStatus {
    Open,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Finding {
    id: Uuid,
    mission_id: Uuid,
    title: String,
    severity: Severity,
    status: FindingStatus,
    evidence_ids: Vec<Uuid>,
    evidence: String,
    remediation: String,
    created_at: OffsetDateTime,
    verified_at: Option<OffsetDateTime>,
}

pub(crate) struct Workspace {
    pub(crate) root: PathBuf,
    pub(crate) db: cerberus_memory::StateDB,
    pub(crate) mission_file: PathBuf,
    pub(crate) plan_file: PathBuf,
    pub(crate) evidence_file: PathBuf,
    pub(crate) findings_file: PathBuf,
}

impl Default for Workspace {
    fn default() -> Self {
        let root = PathBuf::from(".cerberus");
        Self {
            db: cerberus_memory::StateDB::new(root.join("state.db")).unwrap(),
            mission_file: root.join("mission.json"),
            plan_file: root.join("plan.json"),
            evidence_file: root.join("evidence.json"),
            findings_file: root.join("findings.json"),
            root,
        }
    }
}

impl Workspace {
    pub(crate) fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("failed to create {}", self.root.display()))
    }

    pub(crate) fn save_mission(&mut self, mission: &Mission) -> Result<()> {
        self.ensure()?;
        let json = serde_json::to_string(mission)?;
        self.db.set_mission("default", &json).map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
        write_json(&self.mission_file, mission)?;
        Ok(())
    }

    pub(crate) fn load_mission(&self) -> Result<Mission> {
        let data = self.db.get_mission("default").map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
        if let Some(json) = data {
            let m = serde_json::from_str(&json)?;
            Ok(m)
        } else {
            bail!("no mission found in db; run `cerberus mission \"...\"` first")
        }
    }

    pub(crate) fn save_plan(&self, plan: &AgentPlan) -> Result<()> {
        self.ensure()?;
        let json = serde_json::to_string(plan)?;
        self.db.set_plan("default", &json).map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
        write_json(&self.plan_file, plan)?;
        Ok(())
    }

    pub(crate) fn load_plan(&self) -> Result<AgentPlan> {
        let data = self.db.get_plan("default").map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
        if let Some(json) = data {
            let p = serde_json::from_str(&json)?;
            Ok(p)
        } else {
            bail!("no plan found in db")
        }
    }

    pub(crate) fn load_evidence(&self) -> Result<Vec<cerberus_memory::EvidenceRecord>> {
        let evs = self.db.get_evidence("default").map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
        Ok(evs)
    }

    pub(crate) fn add_evidence(
        &self,
        mission: &Mission,
        kind: impl Into<String>,
        summary: impl Into<String>,
    ) -> Result<cerberus_memory::EvidenceRecord> {
        self.ensure()?;
        let record = cerberus_memory::EvidenceRecord {
            id: Uuid::new_v4(),
            mission_id: mission.id,
            kind: kind.into(),
            summary: summary.into(),
            captured_at: OffsetDateTime::now_utc(),
        };
        self.db.insert_evidence("default", &record).map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
        
        let evs = self.load_evidence()?;
        write_json(&self.evidence_file, &evs)?;
        Ok(record)
    }

    pub(crate) fn load_findings(&self) -> Result<Vec<Finding>> {
        let rows = self.db.get_findings("default").map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
        let mut results = Vec::new();
        for r in rows {
            results.push(serde_json::from_str(&r)?);
        }
        Ok(results)
    }

    pub(crate) fn add_finding(&self, finding: Finding) -> Result<()> {
        self.ensure()?;
        let json = serde_json::to_string(&finding)?;
        self.db.save_finding("default", &finding.id.to_string(), &json).map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
        
        let f = self.load_findings()?;
        write_json(&self.findings_file, &f)?;
        Ok(())
    }

    pub(crate) fn verify_findings(&self, finding_id: Option<Uuid>, evidence_id: Uuid) -> Result<Vec<Uuid>> {
        let mut findings = self.load_findings()?;
        let mut verified = Vec::new();
        for finding in &mut findings {
            let matches = finding_id.map_or(true, |id| finding.id == id);
            if matches && finding.status == FindingStatus::Open {
                finding.status = FindingStatus::Verified;
                finding.verified_at = Some(OffsetDateTime::now_utc());
                finding.evidence_ids.push(evidence_id);
                verified.push(finding.id);
                
                let json = serde_json::to_string(finding)?;
                self.db.save_finding("default", &finding.id.to_string(), &json).map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
            }
        }
        if finding_id.is_some() && verified.is_empty() {
            bail!("no open finding matched the requested ID");
        }
        
        let f = self.load_findings()?;
        write_json(&self.findings_file, &f)?;
        Ok(verified)
    }
}

pub(crate) fn run_active_plan(workspace: &Workspace, mission: &Mission) -> Result<RunOutput> {
    let plan = workspace.load_plan()?;
    let policy = PolicyEngine::default();
    let mut events = Vec::new();

    for step in plan.steps {
        let decision = policy.decide(step.risk);
        
        let mut final_decision = decision.clone();
        if let PolicyDecision::RequireApproval { risk, reason } = &decision {
            use std::io::{self, Write};
            print!("\n[POLICY GATE] Action requires approval:\nStep: {}\nRisk: {:?}\nReason: {}\nApprove? [y/N]: ", step.summary, risk, reason);
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().eq_ignore_ascii_case("y") {
                final_decision = PolicyDecision::Allow { risk: *risk };
            } else {
                final_decision = PolicyDecision::Deny { risk: *risk, reason: "Operator denied via CLI".to_string() };
            }
        }

        let evidence_id = match final_decision {
            PolicyDecision::Allow { .. } => {
                Some(workspace.add_evidence(mission, format!("{:?}", step.kind), format!("Executed: {}", step.summary))?.id)
            }
            _ => None,
        };

        events.push(RunEvent {
            name: format!("{:?}", step.kind),
            summary: step.summary,
            decision: final_decision,
            evidence_id,
        });
    }

    Ok(RunOutput {
        id: Uuid::new_v4(),
        mission_id: mission.id,
        mode: "active".to_string(),
        events,
        created_at: OffsetDateTime::now_utc(),
    })
}

pub(crate) async fn generate_llm_plan(mission: &Mission) -> Result<AgentPlan> {
    let config = LlmConfig::from_env(None, None)?;
    if config.provider == cerberus_llm::LlmProvider::Offline {
        bail!("Offline mode, falling back to static plan");
    }
    
    let client = cerberus_llm::LlmClient::new(config);
    let prompt = format!(
        "Plan an active security mission for this objective: {}. 
        Respond with a raw JSON array of objects. Each object must have:
        - \"kind\": string (Observe, Analyze, ProposeTool, RequestApproval, Execute, CollectEvidence, ReportFinding, Patch, Verify)
        - \"summary\": string (brief description of the step)
        - \"risk\": string (Passive, ActiveSafe, Intrusive, ExploitValidation, Forbidden)
        Do not wrap the JSON in markdown blocks.", 
        mission.objective
    );
    let response = client.ask(&prompt).await?;
    
    // Parse the JSON array
    #[derive(serde::Deserialize)]
    struct LlmStep {
        kind: cerberus_core::StepKind,
        summary: String,
        risk: cerberus_policy::RiskLevel,
    }

    // Find the JSON array bounds to ignore preambles
    let start_idx = response.find('[').unwrap_or(0);
    let end_idx = response.rfind(']').map(|i| i + 1).unwrap_or(response.len());
    let clean_json = if start_idx < end_idx {
        &response[start_idx..end_idx]
    } else {
        response.trim()
    };
    
    let parsed_steps: Vec<LlmStep> = serde_json::from_str(clean_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse LLM JSON: {}\nRAW RESPONSE:\n{}", e, response))?;

    let policy = PolicyEngine::default();
    let steps = parsed_steps
        .into_iter()
        .map(|s| cerberus_core::AgentStep {
            id: Uuid::new_v4(),
            kind: s.kind,
            summary: s.summary,
            risk: s.risk,
            decision: policy.decide(s.risk),
        })
        .collect();

    Ok(AgentPlan {
        mission_id: mission.id,
        steps,
    })
}

fn render_markdown_report(mission: &Mission, findings: &[Finding]) -> String {
    let mut report = String::new();
    report.push_str("# Cerberus Security Audit Report\n\n");
    report.push_str(&format!("Mission: `{}`\n\n", mission.id));
    report.push_str(&format!("Objective: {}\n\n", mission.objective));
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- Findings: {}\n", findings.len()));
    report.push_str(&format!(
        "- Open: {}\n",
        findings
            .iter()
            .filter(|finding| finding.status == FindingStatus::Open)
            .count()
    ));
    report.push_str(&format!(
        "- Verified: {}\n\n",
        findings
            .iter()
            .filter(|finding| finding.status == FindingStatus::Verified)
            .count()
    ));

    report.push_str("## Findings\n\n");
    if findings.is_empty() {
        report.push_str("No findings recorded.\n");
    }
    for finding in findings {
        report.push_str(&format!("### {}\n\n", finding.title));
        report.push_str(&format!("- ID: `{}`\n", finding.id));
        report.push_str(&format!("- Severity: `{:?}`\n", finding.severity));
        report.push_str(&format!("- Status: `{:?}`\n", finding.status));
        report.push_str(&format!("- Evidence IDs: `{:?}`\n\n", finding.evidence_ids));
        report.push_str("Evidence:\n\n");
        report.push_str(&finding.evidence);
        report.push_str("\n\nRemediation:\n\n");
        report.push_str(&finding.remediation);
        report.push_str("\n\n");
    }

    report
}

fn default_report_path(format: ReportFormat) -> PathBuf {
    match format {
        ReportFormat::Markdown => PathBuf::from(".cerberus/report.md"),
        ReportFormat::Pdf => PathBuf::from(".cerberus/report.pdf"),
    }
}

fn write_report_file(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, data).with_context(|| format!("failed to write {}", path.display()))
}

fn export_pdf_report(workspace: &Workspace, out: &Path) -> Result<()> {
    let script = find_reportlab_script()?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let status = ProcessCommand::new("python")
        .arg(&script)
        .arg("--mission")
        .arg(&workspace.mission_file)
        .arg("--findings")
        .arg(&workspace.findings_file)
        .arg("--evidence")
        .arg(&workspace.evidence_file)
        .arg("--out")
        .arg(out)
        .status()
        .with_context(|| {
            "failed to launch Python; install Python and run `pip install -r requirements-report.txt`"
        })?;

    if !status.success() {
        bail!(
            "ReportLab PDF export failed; run `pip install -r requirements-report.txt` and retry"
        );
    }

    Ok(())
}

fn find_reportlab_script() -> Result<PathBuf> {
    let cwd_script = PathBuf::from("scripts").join("reportlab_report.py");
    if cwd_script.exists() {
        return Ok(cwd_script);
    }

    let exe = std::env::current_exe().context("failed to locate current executable")?;
    let exe_script = exe
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("scripts")
        .join("reportlab_report.py");
    if exe_script.exists() {
        return Ok(exe_script);
    }

    bail!("ReportLab script not found at scripts/reportlab_report.py")
}

fn print_plan(mission: &Mission, plan: &AgentPlan) {
    println!("mission: {}", mission.id);
    println!("objective: {}", mission.objective);
    for (index, step) in plan.steps.iter().enumerate() {
        println!(
            "{:02}. {:?} [{:?}] {} -> {}",
            index + 1,
            step.kind,
            step.risk,
            step.summary,
            step.decision
        );
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let data = serde_json::to_string_pretty(value)?;
    fs::write(path, data).with_context(|| format!("failed to write {}", path.display()))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let data =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))
}

fn read_json_or_empty<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    read_json(path)
}
