# Distribution And Source Protection

## Current Reality

If Cerberus is pushed to a public GitHub repository, the source code is public.
There is no reliable way to make already-public source private again for anyone
who has cloned or cached it.

If Cerberus is intended to be proprietary, the main product repository should be
private before serious development continues.

## Recommended Split

Use two products:

```text
Specter Toolkit
  open-source skills, checklists, references, scripts, examples

Cerberus
  proprietary runtime, policy engine, memory, agent loop, reports, licensing
```

The open-source layer creates trust and distribution. The proprietary layer
contains the workflow engine customers pay for.

## What To Keep Private

Keep these in the private Cerberus repository:

- agent kernel
- policy engine
- tool execution contracts
- evidence store
- report generator
- LLM orchestration
- license checks
- update logic
- commercial policy packs
- customer templates

Keep these public in Specter Toolkit:

- security skills
- checklists
- references
- simple scripts
- example findings
- documentation that markets the workflow

## Binary Distribution

Ship compiled binaries instead of source:

- Windows `.exe`
- macOS universal binary
- Linux x64/arm64 binary
- npm package that downloads or wraps the binary
- signed GitHub releases from a private build pipeline

For npm, avoid publishing Rust source if source secrecy matters. Publish a thin
JavaScript launcher plus platform binaries.

## License Model

Start simple:

- local signed license file
- machine activation limit
- offline activation for consultants
- annual license for pro/team
- founder license for early one-time buyers

Do not make license enforcement the core security boundary. Licenses discourage
casual copying; they do not stop determined reverse engineering.

## Source Leakage Controls

Practical controls:

- private GitHub organization repository
- least-privilege collaborators
- branch protection
- signed release artifacts
- CI secrets locked to protected branches
- no customer secrets in tests or examples
- no `.env` files committed
- separate public docs from private implementation
- strip symbols in release builds
- avoid publishing debug builds
- watermark licensed binaries if needed

## What Not To Rely On

Do not rely on these as primary protection:

- obfuscation
- minification
- hiding code in npm packages
- license checks alone
- asking users not to share binaries

Compiled binaries can still be reverse engineered. The real protection is a
private repo, signed releases, license terms, and keeping commercial advantage
in execution quality, updates, support, and policy packs.

## Deployment Options

### Private CLI

Best first product.

Users install Cerberus locally. Their code stays on their machine. LLM provider
use is configurable and can be disabled.

### Private Enterprise Build

For high-trust customers:

- offline mode
- no telemetry by default
- local model support
- custom policy packs
- signed reports

### Hosted Control Plane Later

Only add a hosted service after the CLI is valuable.

Possible hosted features:

- license management
- update channels
- team policy sync
- report archive
- organization dashboard

Do not start with hosted SaaS. It adds trust, privacy, and compliance burden
before the core workflow is proven.
