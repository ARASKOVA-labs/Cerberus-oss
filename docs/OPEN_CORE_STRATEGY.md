# Cerberus: Open Core Business Strategy

Araskova Labs operates Cerberus under an **Open Core** business model. This document strictly defines the licensing and architectural split between the open-source community engine and our proprietary enterprise features.

## 1. Open Source (OSS)
All code within this repository (`araskova-labs/cerberus`) is strictly open-source and governed by the **MIT License**. This includes:
- **`cerberus-core`**: The mission state, memory logic, and vulnerability schemas.
- **`cerberus-cli`**: The command-line interface and Git diff extraction logic.
- **`cerberus-llm`**: The AI adapters that communicate with OpenAI, Anthropic, or local offline models.
- **`@araskova/cerberus` (NPM Package)**: The Node.js binary wrapper used to seamlessly integrate Cerberus into standard CI/CD pipelines.

We give this away for free because we believe every developer on the planet should have access to world-class automated AI security reviews natively in their terminal and pipelines.

## 2. Proprietary (Commercial)
Araskova Labs restricts access to enterprise-grade management features, premium compliance rulesets, and managed infrastructure. These components are kept in private repositories and are strictly proprietary:
- **Cerberus Cloud (GitHub App)**: A hosted SaaS platform that automatically runs `cerberus review` on every Pull Request in your organization, functioning exactly like CodeRabbit but for deep security testing. (Revenue: $20/developer/month).
- **Premium Policy Packs**: Advanced, proprietary prompt engineering rulesets used for strict regulatory compliance (e.g. SOC2, HIPAA, PCI-DSS compliance packs).
- **Cerberus Enterprise Vault**: Team-wide audit logs and centralized database hosting for security evidence and remediation reports.

**Note to Contributors:** Do not merge proprietary policy packs or cloud authentication logic into this public repository. All commercial features must be built via external integrations or private repositories using the `cerberus-core` public API.
