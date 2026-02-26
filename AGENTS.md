## 1. Documentation

cho is a Rust workspace focused on FreeAgent. Current scope is SDK + CLI (`cho-sdk`, `cho-cli`) with `cho-tui` scaffolded for later work.

Primary API references:
- Docs hub: https://dev.freeagent.com/docs
- Introduction (auth, pagination, rate limits): https://dev.freeagent.com/docs/introduction
- OAuth details: https://dev.freeagent.com/docs/oauth
- Changes feed: https://dev.freeagent.com/docs/changes

## 2. Repository Structure

```text
.
├── AGENTS.md
├── Cargo.toml
├── package.json
├── .husky/
├── commitlint.config.js
├── lint-staged.config.js
├── rustfmt.toml
└── crates/
    ├── cho-sdk/
    │   └── src/{api,auth,client,config,error,home,models,blocking}
    ├── cho-cli/
    │   └── src/{commands,output,audit,context,envelope,error,registry,main}
    └── cho-tui/
```

## 3. Stack

| Layer | Choice | Notes |
| --- | --- | --- |
| Language | Rust 2024 | workspace-based |
| Runtime | tokio | async CLI + SDK |
| HTTP | reqwest + rustls | retries + 401 refresh + 429 handling |
| CLI | clap | command tree for agent primitives |
| Serialization | serde/serde_json | FreeAgent snake_case wire format |
| Secrets | secrecy + keyring | token fallback file when keyring unavailable |
| Logging | custom audit log + tracing | append-only history at `~/.tools/cho/history.log` |
| JS Tooling | bun + biome + commitlint + husky | quality-gate orchestration |

## 4. Commands

Core orientation commands:
- `cho tools --json`
- `cho tools <name> --json`
- `cho health --json`
- `cho config show --json`
- `cho config set <key> <value> --json`

Auth:
- `cho auth login [--port <n>] [--no-browser] --json`
- `cho auth status --json`
- `cho auth refresh --json`
- `cho auth logout --json`

Company and reports:
- `cho company {get|tax-timeline|business-categories} --json`
- `cho reports {profit-and-loss|balance-sheet|trial-balance|cashflow} --json`

Resource groups (agent primitives):
- `contacts {list|get|create|update|delete|search}`
- `invoices {list|get|create|update|delete|transition|send-email}`
- `bank-accounts {list|get|create|update|delete}`
- `bank-transactions {list|get|upload-statement}`
- `bank-transaction-explanations {list|get|create|update|delete}`
- `bills {list|get|create|update|delete}`
- `expenses {list|get|create|update|delete|mileage-settings}`
- `categories {list|get|create|update|delete}`
- `transactions {list|get}`
- `sales-tax-periods {list|get|create|update|delete}`
- `credit-notes {list|get|create|update|delete}`
- `estimates {list|get|create|update|delete}`
- `recurring-invoices {list|get}`
- `journal-sets {list|get|create|update|delete}`
- `users {list|get|create|update|delete}`
- `capital-assets {list|get}`
- `stock-items {list|get}`
- `projects {list|get|create|update|delete}`
- `timeslips {list|get|create|update|delete}`
- `attachments {get|delete}`

Tax and payroll:
- `corporation-tax-returns {list|get|mark-filed|mark-unfiled|mark-paid|mark-unpaid}`
- `self-assessment-returns {list|get|mark-filed|mark-unfiled|mark-payment-paid|mark-payment-unpaid}`
- `vat-returns {list|get|mark-filed|mark-unfiled|mark-payment-paid|mark-payment-unpaid}`
- `final-accounts-reports {list|get|mark-filed|mark-unfiled}`
- `payroll {periods|period|mark-payment-paid|mark-payment-unpaid}`
- `payroll-profiles list`

## 5. Architecture

Runtime flow:

```text
Agent/Human -> cho-cli
  -> cho-sdk::FreeAgentClient
    -> AuthManager (OAuth + token refresh)
    -> reqwest transport (retry/rate-limit handling)
      -> FreeAgent API
```

Invariants:
- `cho-sdk` is interface-agnostic and reusable.
- `cho-cli` is a thin command adapter around SDK primitives.
- JSON mode prints one compact envelope line to stdout.
- Error envelopes use stable codes + hints.
- Mutating operations require `[safety] allow_writes = true`.
- Every command start/input/output/end is appended to `history.log` with timestamp and run id.

## 6. Quality

Required checks before completion:
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo check --workspace`
- `cargo test --workspace`
- `bun run util:check`

Build/install helper:
- `bun run build` compiles release binaries and copies `cho` to `${CHO_HOME:-${TOOLS_HOME:-$HOME/.tools}/cho}/cho`.

Config path:
- `~/.tools/cho/config.toml` (or `CHO_HOME/config.toml`)

Audit path:
- `~/.tools/cho/history.log` (or `CHO_HOME/history.log`)
