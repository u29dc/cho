## 1. Documentation

cho is a Rust workspace (`cho-sdk`, `cho-cli`, `cho-tui`, `cho-mcp`) exposing Xero Accounting API operations for agent-first workflows (JSON stdout contract) and human CLI/TUI usage. This file is the compact execution contract; record deviations when implementation diverges.

- Primary references: `Xero Portal` https://developer.xero.com (app/account constraints), `OAuth2 PKCE` https://developer.xero.com/documentation/guides/oauth2/pkce-flow (interactive auth), `Rate Limits` https://developer.xero.com/documentation/guides/oauth2/limits (quota + headers), `OpenAPI` https://github.com/XeroAPI/Xero-OpenAPI (wire schema), `Changelog` https://developer.xero.com/changelog (breaking/non-breaking changes).
- Runtime/docs references: `reqwest` https://docs.rs/reqwest, `serde` https://serde.rs, `clap` https://docs.rs/clap, `ratatui` https://docs.rs/ratatui, `tokio` https://tokio.rs, `rust_decimal` https://docs.rs/rust_decimal.

## 2. Repository Structure

```text
.
├── Cargo.toml                 # workspace resolver=3, rust-version=1.93.0
├── AGENTS.md                  # compact operational contract
├── package.json               # bun quality scripts
├── commitlint.config.js       # scopes: sdk|cli|tui|mcp|config|deps
├── lint-staged.config.js      # runs bun run util:check
├── biome.json
├── rustfmt.toml
├── .husky/                    # pre-commit + commit-msg hooks
└── crates/
    ├── cho-sdk/               # publishable API client
    │   └── src/{api,auth,http,models,blocking,client,config,error}
    ├── cho-cli/               # command interface + json/table/csv rendering
    │   └── src/{commands,output,context,envelope,registry,error,main}
    ├── cho-tui/               # ratatui app (phase 4)
    └── cho-mcp/               # MCP server (phase 5)
```

- `cho-sdk`: core Xero transport/auth/models; no CLI/TUI/MCP deps; namespaced typed API handles.
- `cho-cli`: human + agent entrypoint; stable JSON envelope on stdout in JSON mode; stable error codes/hints.
- `cho-tui`: dashboard UX layer over SDK only.
- `cho-mcp`: MCP tool surface over SDK only.

## 3. Stack

| Layer         | Choice                                         | Notes                                      |
| ------------- | ---------------------------------------------- | ------------------------------------------ |
| Language      | Rust 2024                                      | rust-version 1.93.0                        |
| Async         | tokio 1.x                                      | multithread runtime; SDK blocking wrappers |
| HTTP          | reqwest 0.12+                                  | rustls, pooling, retries/backoff           |
| Serialization | serde + serde_json                             | wire PascalCase; CLI snake_case transform  |
| CLI           | clap 4 derive                                  | nested subcommands; env fallback           |
| TUI           | ratatui 0.30 + crossterm 0.29                  | phase 4 target                             |
| Money         | rust_decimal                                   | never use `f64` for money                  |
| Dates         | chrono                                         | `MsDate`, `MsDateTime` newtypes            |
| IDs           | uuid 1.x                                       | typed resource IDs                         |
| Errors        | thiserror 2.x                                  | per-crate enums                            |
| Secrets       | secrecy + keyring                              | in-memory secrecy + OS keychain            |
| Config        | toml                                           | XDG path `~/.config/cho`                   |
| Output tables | comfy-table                                    | human formatting path                      |
| Logging       | tracing + tracing-subscriber                   | `--verbose`, `RUST_LOG`                    |
| JS tooling    | bun + biome + commitlint + husky + lint-staged | hooks + contract checks                    |
| Testing       | wiremock + assert_cmd                          | SDK HTTP mocks + CLI integration           |

## 4. Commands

- Workspace commands: `bun run build` -> `cargo build --workspace --release`; `bun run cho:dev` -> `cargo run -p cho-cli --`; `bun run cho` -> `./target/release/cho`.
- Quality commands: `bun run util:format` -> `cargo fmt --all`; `bun run util:lint` -> `cargo clippy --all-targets --all-features -- -D warnings`; `bun run util:types` -> `cargo check --workspace`; `bun run util:test` -> `cargo test --workspace`; `bun run util:check` -> format+lint+types+test.

```text
cho tools [<name>] [--json]
cho health [--json]

cho auth login [--client-credentials]
cho auth status
cho auth refresh
cho auth tenants

cho invoices {list|get|create|update}
cho contacts {list|get|search|create|update}
cho payments {list|get|create|delete}
cho transactions {list|get|create}
cho accounts list
cho reports {balance-sheet|pnl|trial-balance|aged-payables|aged-receivables}
cho config {set|show}
```

- Global output/behavior flags: `--json` (alias for `--format json`), `--format json|table|csv` (auto: JSON when piped, table on TTY), `--raw` (preserve PascalCase wire keys), `--precise` (money as strings), `--quiet`, `--verbose`, `--no-color`.
- Global scope flags: `--tenant <uuid>` (override config tenant), `--limit <N>` (default 100, hard cap 10,000), `--all` (fetch all pages).
- Deprecated flag: `--meta` is hidden; envelope is default in JSON mode.

- JSON success envelope: `{"ok": true, "data": <payload>, "meta": {"tool": "<category.action>", "elapsed": <ms>, ...}}`.
- JSON error envelope: `{"ok": false, "error": {"code": "...", "message": "...", "hint": "..."}, "meta": {...}}`.
- Tool catalog command: `cho tools --json` returns `{version, tools[], globalFlags[]}` (~55 tools).
- Health command: `cho health --json` checks `config|auth|tenant|keyring`, status `ready|degraded|blocked`, blocked exits `2`.

- Config path: `~/.config/cho/config.toml` (`XDG_CONFIG_HOME` respected).
- Precedence: CLI flags > env > config > defaults.
- Key env vars: `CHO_TENANT_ID`, `CHO_CLIENT_ID`, `CHO_CLIENT_SECRET`, `CHO_FORMAT`, `CHO_BASE_URL`.
- Write safety contract: mutating operations require `[safety] allow_writes = true`; default deny.

```toml
[auth]
tenant_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
client_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

[defaults]
format = "json"
limit = 100

[sdk]
base_url = "https://api.xero.com/api.xro/2.0/"
timeout_secs = 30
max_retries = 3

[safety]
allow_writes = false
```

- Exit code contract: `0` success; `1` runtime/validation/usage/API/network failure; `2` blocked prerequisites (`AUTH_REQUIRED`, `TOKEN_EXPIRED`, `WRITE_NOT_ALLOWED`).
- Error code mapping: `AUTH_REQUIRED` -> run `cho auth login`; `TOKEN_EXPIRED` -> re-authenticate; `WRITE_NOT_ALLOWED` -> enable safety gate; `RATE_LIMITED` -> wait/retry (`--verbose` for headers); `NOT_FOUND` -> verify ID/number; `VALIDATION_ERROR` -> fix payload; `API_ERROR` -> retry/check Xero status; `NETWORK_ERROR` -> verify connectivity; `PARSE_ERROR` -> rerun verbose/report; `CONFIG_ERROR` -> repair config; `USAGE_ERROR` -> run command help.

## 5. Architecture

```text
Agent/Human -> cho-cli / cho-tui / cho-mcp
  -> cho-sdk::XeroClient (auth + rate-limit + pagination + request builder)
    -> reqwest -> Xero API
      -> wire JSON (PascalCase, envelope, MS dates)
        -> typed SDK models
          -> CLI transform (snake_case/table/csv) -> stdout/stderr
```

- Invariant: `cho-sdk` remains interface-agnostic and publishable; UI crates stay thin adapters.
- Invariant: API namespace pattern is `client.<resource>().<action>()`; blocking wrappers mirror async APIs.
- Invariant: pagination is iterative (not async stream), supports cap enforcement and metadata propagation.
- Invariant: token refresh is transparent and serialized (single-use refresh token race prevention).
- Invariant: rate-limiter enforces Xero constraints (5 concurrent, 60/min), tracks limit headers, respects `Retry-After`, adds jitter.
- Invariant: write retries require idempotency keys; write gate is SDK-level, not CLI-only.
- Invariant: `--raw` preserves PascalCase wire keys; non-raw JSON is snake_case transformed.

- Xero endpoints: accounting `https://api.xero.com/api.xro/2.0/`; identity `https://api.xero.com/connections`; auth `https://login.xero.com/identity/connect/authorize`; token `https://identity.xero.com/connect/token`.
- Required request headers: `Authorization: Bearer ...`, `xero-tenant-id`, `Content-Type: application/json`, `Accept: application/json`.
- PKCE flow: verifier length 43-128, challenge `S256`, localhost callback, explicit `state`, browser mandatory.
- Token lifecycle: access token ~30 minutes; refresh token single-use and 60-day inactivity window; `offline_access` required.
- Custom Connections: `client_credentials`, single org, paid Xero feature, no refresh token.
- Limits: per org/app 5 concurrent + 60/min + 5000/day; app minute limit header also tracked; 429 uses `Retry-After`.
- Response shape: wrapper arrays + pagination + warnings; single GET still wrapped.
- Pagination modes: page-based for invoices/contacts/payments/etc.; offset-like journals; some endpoints non-paginated.
- Filtering: pass-through OData-like `where` and `order`; suspicious patterns warned before dispatch.
- Date wire variants: `/Date(epoch+offset)/`, `/Date(epoch)/`, ISO `YYYY-MM-DD`; regex `/\/Date\((-?\d+)(\+\d{4})?\)\//`.
- Stability: API v2.0 with deprecation windows; no v3 currently.

- Model layout: one resource per `models/*.rs`; shared types in `common.rs`; enums in `enums.rs`; date wrappers in `dates.rs`.
- Serde policy: `rename_all = "PascalCase"`; mostly optional fields with `skip_serializing_if`.
- Numeric policy: all money in `Decimal`; no float money fields.
- Enum policy: `#[serde(other)]` catch-all for forward compatibility.
- Known complexity coverage: hyphenated enums, circular refs, polymorphic payment targets, inline validation errors.
- Reports coverage: raw row/cell tree + typed parsed reports (balance sheet, P&L, trial balance).

## 6. Quality

- Mandatory gates before completion: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo check --workspace`; `cargo test --workspace`.
- Safety/style constraints: no `unwrap()` in library paths; `cho-sdk` uses `#![deny(missing_docs)]`; all crates enforce `#![forbid(unsafe_code)]`.
- Test strategy: SDK wiremock contract tests (retry/pagination/401/429/validation extraction), serialization edge tests (MS date + Decimal + enum forward-compat), CLI `assert_cmd` tests (parsing/format/envelope/exits/env).
- Live tests policy: manual and optional; never CI-bound due credential/rate-limit risk.

```bash
export CHO_CLIENT_ID="..."
# optional for custom-connections flow
export CHO_CLIENT_SECRET="..."
export CHO_TENANT_ID="..."

cargo run -p cho-cli -- auth login
cargo run -p cho-cli -- auth status
cargo run -p cho-cli -- invoices list --limit 5
cargo run -p cho-cli -- reports balance-sheet
```

- Roadmap: `P0 done` scaffold+tooling; `P1 done` SDK core; `P2 done` CLI core; `P3 done` Tier2/3 + writes + safety gate; `P3.1 done` contract correctness fixes; `P3.2 done` security/observability hardening; `P3.3 done` code-audit fixes; `P3.4 done` agent-native contract/tool registry/health; `P4 pending` TUI; `P5 pending` MCP server.
- Recorded deviations: progenitor generation skipped (manual SDK implementation); pagination uses iterative fetch (not stream); write gate moved SDK-side; plaintext token fallback removed (fail-closed keyring strategy with legacy migration support); Tier3 intentionally excludes BankFeed/FixedAsset/Files/Payroll UK for now.
