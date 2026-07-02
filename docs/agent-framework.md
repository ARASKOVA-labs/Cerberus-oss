# Agent Framework

Cerberus is centered on missions.

## Mission

A mission is the operator's authorized security objective.

Examples:

- audit this repository for auth bugs
- scan this project for secrets
- validate this suspected SSRF finding
- patch and verify this SQL injection

## Agent Kernel

The kernel owns the loop:

```text
Mission -> Plan -> Step -> PolicyDecision -> Execution -> Evidence -> Memory
```

The kernel is model-agnostic. Claude, OpenAI, or a local model can provide
reasoning, but Cerberus owns state, policy, tools, and evidence.

## Step Kinds

- `Observe`
- `Analyze`
- `ProposeTool`
- `RequestApproval`
- `Execute`
- `CollectEvidence`
- `ReportFinding`
- `Patch`
- `Verify`

## Policy Risk Levels

- `Passive`
- `ActiveSafe`
- `Intrusive`
- `ExploitValidation`
- `Forbidden`

## First Milestone

The first complete milestone is:

```bash
cerberus console
cerberus mission "audit this repository"
cerberus llm status
cerberus policy check --risk exploit-validation
```

Then:

1. project ingestion
2. secret scanner
3. findings store
4. LLM plan generation
5. controlled tool proposals
6. fix and verify
