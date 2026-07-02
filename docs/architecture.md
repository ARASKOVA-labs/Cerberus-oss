# Cerberus Architecture

Cerberus is built around a security agent kernel.

```text
operator input
  -> mission parser
  -> policy engine
  -> planner
  -> LLM provider
  -> tool proposal
  -> policy approval
  -> controlled execution
  -> evidence capture
  -> finding normalization
  -> remediation proposal
  -> verification
  -> memory
```

## Crates

| Crate | Role |
|-------|------|
| `cerberus-cli` | Terminal console, commands, operator interaction |
| `cerberus-core` | Agent kernel, mission model, plans, events |
| `cerberus-policy` | Risk classification, scope, action decisions |
| `cerberus-llm` | Anthropic, OpenAI, and local provider abstraction |
| `cerberus-memory` | Local sessions, evidence, findings, audit log |
| `cerberus-tools` | Tool registry, proposals, execution contracts |

## Non-Negotiables

- LLMs propose actions; Cerberus approves and executes.
- Every active action passes through policy.
- Findings need evidence.
- Fixes must be verified.
- Offline/local operation must remain possible.
- The terminal interface is first-class.
