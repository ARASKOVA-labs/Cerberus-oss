# Cerberus Product Strategy

## Positioning

Cerberus is an automated, AI-driven security review and testing engine.

Short pitch:

> Drop Cerberus into your CI/CD pipeline to automatically enforce OWASP and SOC2 compliance on every pull request with AI code reviews and generated dynamic tests.

Longer pitch:

> Cerberus is an automated security guardrail for modern development teams. Unlike interactive chat bots, Cerberus runs headless in your CI/CD pipeline, reading git diffs, surfacing security vulnerabilities exactly where they happen (CodeRabbit-style), and dynamically generating security test scripts to verify endpoints.

## Category

Cerberus should not be positioned as a chatbot, REPL, or prompt pack.

The category is:

```text
Automated CI/CD AI Security Review & Testing Engine
```

This keeps the product away from crowded "AI scanner" messaging and makes the
core difference clear: this is a fully automated guardrail, not an interactive toy.

## Differentiation

Most competing tools emphasize one of these:

- static analysis
- dependency scanning
- cloud security posture
- code scanning autofix
- autonomous offensive testing
- AI chat over security findings

Cerberus should emphasize the workflow around all of them:

- local terminal-first operation
- model-agnostic providers
- policy-gated tool execution
- explicit mission scope
- evidence-first findings
- remediation plus verification
- auditable operator history
- exportable reports

The strongest claim is not "better AI." The strongest claim is "better control
around AI-assisted security work."

## Best First Customers

Primary early market:

- solo security consultants
- small AppSec teams
- boutique pentest firms
- bug bounty hunters
- dev agencies that include security reviews
- startups without dedicated security staff

These users already spend time on planning, reproducing issues, collecting
evidence, writing findings, and verifying fixes. Cerberus should compress that
work into a repeatable terminal workflow.

## Paid Product Shape

The first paid version should be a pro CLI, not an enterprise web platform.

Recommended packages:

- Founder License: one-time early purchase for v1.x
- Pro License: annual license for independent operators
- Team License: annual license for small teams
- Consulting/Enterprise License: private support, offline builds, custom policy

The free layer can remain Specter Toolkit: skills, checklists, references, and
small scripts.

Cerberus should be paid when it provides orchestration:

- mission state
- policy decisions
- LLM provider abstraction
- evidence store
- finding lifecycle
- report generation
- remediation workflow
- verification workflow

## Minimum Sellable Workflow

The first version worth selling should make this path feel solid for any CI/CD pipeline or local workflow:

```bash
cerberus setup
cerberus review .
cerberus test "http://localhost:3000/api"
```

Minimum feature set:

- local project ingestion
- passive checks first
- secret scan
- dependency/package review hooks
- LLM-generated audit plan
- policy gate before active actions
- evidence capture to local storage
- finding normalization
- Markdown/JSON report export
- verification status per finding

## What Is Missing

Cerberus needs these before it should be sold seriously:

- real `plan`, `run`, `findings`, `report`, and `verify` commands
- persistent mission files
- evidence directory format
- finding schema with stable IDs
- report export template
- tool allowlist and denylist
- policy prompts for risky actions
- passive scanner integrations
- local config file
- provider test suite
- installable binary release
- private distribution strategy
- license and update mechanism
- demo recording and sample report

## Market Notes

The market is active and crowded, but not identical to Cerberus:

- GitHub Copilot Autofix focuses on code scanning alerts and suggested fixes.
- Snyk positions around AI-native security across developer workflows.
- Semgrep focuses on AI-assisted application security and code understanding.
- Replit has integrated Semgrep Community Edition into agentic app creation.
- XBOW is focused on autonomous offensive security and pentesting depth.

Cerberus should not fight these products head-on. It should become the local
operator runtime that can coordinate scanners, LLMs, evidence, policy, and
reports from the terminal.

## Pitch

One-line:

> Governed AI security audits from your terminal.

Website headline:

> Run AI-assisted security audits without giving the model the keys.

Subheadline:

> Cerberus turns LLMs into governed security operators with scoped missions,
> policy gates, controlled tools, evidence-first findings, reports, and
> verification.

Consultant pitch:

> Finish audits faster, produce cleaner evidence, and verify fixes from one
> terminal workflow.

Team pitch:

> Let engineers and AppSec use AI for security review while keeping risky
> actions scoped, approved, logged, and reproducible.

## Roadmap

Phase 1: Credible CLI

- mission state
- passive run mode
- evidence store
- finding schema
- report export
- provider status

Phase 2: Governed Agent Loop

- LLM plan generation
- policy-gated tool proposals
- approval prompts
- tool execution contracts
- audit log

Phase 3: Commercial Build

- binary releases
- license activation
- update channel
- sample audits
- documentation site
- paid founder launch

Phase 4: Team/Enterprise

- shared policy packs
- offline builds
- signed reports
- CI integration
- MCP/server mode
- team evidence archive
