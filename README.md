# Cerberus OSS: Autonomous AI Security Guardrail

**Built by Araskova Labs**

Cerberus is an automated AI security review, patching, and testing engine.

Unlike interactive chat bots, Cerberus runs headless in your terminal or CI/CD pipeline. It reads your git diffs, surfaces security vulnerabilities exactly where they happen, **and autonomously patches them on disk.**

## 🚀 Quick Start (Cargo)

The easiest way to integrate Cerberus into your workflow is via the Rust CLI:

```bash
cargo install cerberus-cli
```

### 1. Zero-Config Setup
Run the setup wizard to securely configure your preferred AI provider globally (Ollama, OpenAI, Anthropic). You only need to do this once.
```bash
cerberus setup
```

### 2. Autonomous Fix Engine
Run an automated security code review on your local uncommitted changes, and let Cerberus automatically patch the vulnerable code inline:
```bash
cerberus review . --fix
```

### 3. Dynamic Test Generation
Generate automated dynamic security tests for an endpoint:
```bash
cerberus test "http://localhost:3000/api/login"
```

## 🧠 Bring Your Own Model (BYOM)

Cerberus is completely LLM agnostic. You are not forced to send proprietary source code to a black-box cloud API. 

You can use:
- **Anthropic / Claude 3.5 Sonnet** (Best for deep semantic reviews)
- **OpenAI / GPT-4o** (Best for generating testing scripts)
- **Local Offline Models** via Ollama/LMStudio (Zero data leaves your laptop)

## 🏢 Licensing (Open Core Strategy)

Araskova Labs operates Cerberus under an **Open Core** model.

### Open Source (MIT)
The core AI engine, the CLI, the autonomous fix engine, and the git diff parser are 100% free and open-source. We believe every developer should have access to world-class automated AI security reviews natively in their terminal and pipelines.

### Proprietary Enterprise (Commercial Hub)
We offer the **Operator Console** for enterprise teams — a fleet-wide management hub that provides:
- Centralized TUI dashboards for fleet actions
- Audit evidence accrual and real-time review
- Advanced, proprietary prompt engineering rulesets used for strict regulatory compliance (e.g. SOC2, HIPAA, PCI-DSS compliance packs).

## ⚙️ Architecture

Cerberus relies on a blazing fast Rust-native core.

```text
crates/
  cerberus-cli       Terminal interface, git parsers, automated commands, TUI dashboard
  cerberus-core      Agent kernel, mission data, memory schemas
  cerberus-llm       LLM provider connectors and strict JSON personas
```
