# Cerberus: Automated AI Security Guardrail

**Cerberus is an automated CI/CD AI security review and testing engine.** 

Unlike interactive chat bots, Cerberus runs headless in your CI/CD pipeline, reading git diffs, surfacing security vulnerabilities exactly where they happen (CodeRabbit-style), and dynamically generating security test scripts to verify endpoints.

## 🚀 Quick Start (NPM Package)

The easiest way to integrate Cerberus into your workflow is via the blazing-fast NPM wrapper:

```bash
npm install -g @araskova/cerberus
```

Run an automated security code review on your local uncommitted changes:
```bash
cerberus review .
```

Generate automated dynamic security tests for an endpoint:
```bash
cerberus test "http://localhost:3000/api/login"
```

*(Note: If you prefer to build from source, you can use `cargo run -p cerberus-cli -- review .`)*

## 🧠 Bring Your Own Model (BYOM)

Cerberus is completely LLM agnostic. You are not forced to send proprietary source code to a black-box cloud API. 

Configure your preferred LLM engine:
```bash
cerberus setup
```
You can use:
- **Anthropic / Claude 3.5 Sonnet** (Best for deep semantic reviews)
- **OpenAI / GPT-4o** (Best for generating testing scripts)
- **Local Offline Models** via Ollama/LMStudio (Zero data leaves your laptop)

## 🏢 Monetization & Licensing (Open Core Strategy)

Araskova Labs operates Cerberus under an **Open Core** business model.

### Open Source (MIT)
The core AI engine, the CLI, the git diff parser, and the NPM packaging are 100% free and open-source. We believe every developer should have access to world-class automated AI security reviews natively in their terminal and pipelines.

### Proprietary Enterprise (Commercial)
We sell managed infrastructure and premium compliance packs to enterprises:
- **Cerberus Cloud (GitHub App)**: Let us run Cerberus on every Pull Request via our managed GitHub App ($20/dev/month). No infrastructure to manage.
- **Premium Policy Packs**: Advanced, proprietary prompt engineering rulesets used for strict regulatory compliance (e.g. SOC2, HIPAA, PCI-DSS compliance packs).
- **Cerberus Enterprise Vault**: Team-wide audit logs and centralized database hosting for security evidence and remediation reports.

Please read our [OPEN_CORE_STRATEGY.md](docs/OPEN_CORE_STRATEGY.md) for full licensing details.

## ⚙️ Architecture

Cerberus relies on a blazing fast Rust-native core, wrapped effortlessly in an NPM package for mass adoption.

```text
crates/
  cerberus-cli       Terminal interface, git parsers, automated commands
  cerberus-core      Agent kernel, mission data, memory schemas
  cerberus-llm       LLM provider connectors and strict JSON personas
npm/
  cerberus-cli       The NPM wrapper that manages cross-platform Rust binary downloads
docs/
  OPEN_CORE_STRATEGY.md
```
