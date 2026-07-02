# Agent Adoption

## Goal

Agents should use Cerberus because it is the safest and most useful path for
security work, not because the README merely asks them nicely.

Cerberus needs to become an execution surface:

```text
agent intent -> cerberus policy -> approved tool execution -> evidence -> finding -> report
```

## Can LLMs Be Forced To Use Cerberus?

Not universally.

An arbitrary LLM in an arbitrary chat cannot be forced to use a local tool. It
can ignore instructions, lack tool access, or run in an environment where
Cerberus is unavailable.

Cerberus can be made mandatory inside controlled environments:

- coding agent workspaces
- CI pipelines
- internal developer machines
- MCP tool servers
- CLI wrappers
- pre-commit and pre-push checks
- security review templates
- organization policy

The practical product goal is:

> If an agent wants to perform security work in this environment, Cerberus is
> the only approved execution path.

## Enforcement Patterns

### AGENTS.md

Add repository instructions that tell coding agents to use Cerberus for security
tasks, evidence collection, and findings.

This helps with agents that honor repository guidance, but it is not a hard
control by itself.

### CLI Wrapper

Provide commands that agents can call directly:

```bash
cerberus mission "audit this repository"
cerberus plan --json
cerberus run --passive --json
cerberus finding add --json
cerberus report export --format markdown
```

Agents use tools when the tools are reliable, scriptable, and return structured
output.

### MCP Server

Expose Cerberus as an MCP server so compatible agents can discover tools such as:

- `create_mission`
- `generate_plan`
- `request_policy_decision`
- `run_passive_check`
- `capture_evidence`
- `create_finding`
- `export_report`
- `verify_fix`

This is the strongest path for agent adoption because the runtime becomes a
native tool surface instead of a pile of shell commands.

### CI Gate

Add CI checks that reject security-sensitive changes unless a Cerberus report or
verification artifact exists.

Examples:

- require `cerberus report export` for release branches
- require `cerberus verify` for security patches
- require no `Forbidden` policy events in the audit log
- require signed evidence for customer-facing reports

### Policy Files

Use a project policy file:

```toml
[scope]
targets = ["."]
forbidden = ["external-exploit", "credential-use"]

[approval]
intrusive = "manual"
exploit_validation = "manual"

[evidence]
required_for_findings = true
```

Agents can read policy, but Cerberus must enforce it.

## What Agents Need From Cerberus

Agents will use Cerberus if the interface gives them:

- JSON output
- stable command names
- predictable exit codes
- clear error messages
- machine-readable policies
- local evidence paths
- finding IDs
- report artifacts
- noninteractive flags

The human terminal UI matters, but agents need structured APIs.

## Required Product Work

To make Cerberus agent-native, build:

- `--json` output for every command
- a stable finding schema
- a stable evidence schema
- a policy decision API
- noninteractive execution modes
- MCP server mode
- CI examples
- AGENTS.md template
- examples for Codex, Claude Code, Cursor, Continue, and GitHub Actions

## Hard Rule

LLMs should never be trusted as the authority for action safety.

The model can say:

```text
I want to run this command because...
```

Cerberus must decide:

```text
allowed, denied, or requires approval
```

Then Cerberus executes, captures evidence, and logs the result.
