## 1. Overview

> **Development plan** -- note deviations and correct this spec as you implement.

cho is a Rust workspace (cho-sdk, cho-cli, cho-tui, cho-mcp) exposing the Xero accounting REST API as a local terminal tool. Consumers: AI agents (~65%, shell exec + JSON stdout) and humans (~35%, CLI/TUI). Auth via OAuth 2.0 PKCE (browser, multi-org) with Custom Connections (headless, client_credentials) added later. Read-only MVP expanding to writes. Entirely greenfield — no production Xero CLI/TUI exists.

| Resource | URL |
|---|---|
| Xero Developer Portal | https://developer.xero.com |
| Xero OAuth 2.0 PKCE | https://developer.xero.com/documentation/guides/oauth2/pkce-flow |
| Xero Rate Limits | https://developer.xero.com/documentation/guides/oauth2/limits |
| Xero OpenAPI Specs | https://github.com/XeroAPI/Xero-OpenAPI (MIT, v10.1.0, ~57k lines YAML) |
| Xero Changelog | https://developer.xero.com/changelog |
| reqwest | https://docs.rs/reqwest |
| serde | https://serde.rs |
| clap | https://docs.rs/clap |
| ratatui | https://docs.rs/ratatui |
| tokio | https://tokio.rs |
| rust_decimal | https://docs.rs/rust_decimal |

## 2. Repository Structure

```
.
├── Cargo.toml                  # workspace root, resolver = "3", rust-version = "1.93.0"
├── SPEC.md
├── AGENTS.md                   # condensed operational spec derived from SPEC.md
├── package.json                # bun scripts for quality gates
├── commitlint.config.js        # scopes: sdk|cli|tui|mcp|config|deps
├── lint-staged.config.js       # runs bun run util:check
├── rustfmt.toml                # edition = "2024"
├── biome.json                  # extends global biome config
├── .husky/
│   ├── pre-commit              # bunx lint-staged
│   └── commit-msg              # bunx --no-install commitlint --edit "$1"
├── .gitignore
└── crates/
    ├── cho-sdk/                # pure API client (publishable to crates.io)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── client.rs               # XeroClient builder, tenant header injection
    │       ├── auth/
    │       │   ├── mod.rs
    │       │   ├── pkce.rs             # PKCE flow: verifier/challenge, localhost callback, browser open
    │       │   ├── credentials.rs      # Custom Connections (client_credentials grant)
    │       │   ├── token.rs            # TokenPair, refresh logic, expiry check
    │       │   └── storage.rs          # keyring primary, encrypted file fallback (0600)
    │       ├── http/
    │       │   ├── mod.rs
    │       │   ├── rate_limit.rs       # token bucket (5 concurrent, 60/min), header tracking, 429 backoff
    │       │   ├── pagination.rs       # auto-paginate Stream<Item=Result<T>>, page-based + offset
    │       │   └── request.rs          # request builder: auth header, tenant header, query params
    │       ├── models/
    │       │   ├── mod.rs
    │       │   ├── invoice.rs          # Invoice (~40 fields), Invoices wrapper, InvoiceType, InvoiceStatus
    │       │   ├── contact.rs          # Contact (~35 fields), Contacts, ContactStatus, ContactPerson, Address, Phone
    │       │   ├── payment.rs          # Payment (~25 fields), Payments, PaymentStatus
    │       │   ├── bank_transaction.rs # BankTransaction (~25 fields), BankTransactions, BankTransactionType
    │       │   ├── credit_note.rs      # CreditNote (~30 fields), CreditNotes
    │       │   ├── account.rs          # Account (~15 fields), Accounts, AccountType, AccountClass
    │       │   ├── item.rs             # Item, Items, PurchaseDetails, SalesDetails
    │       │   ├── tax_rate.rs         # TaxRate, TaxRates, TaxComponent
    │       │   ├── quote.rs            # Quote, Quotes, QuoteStatus
    │       │   ├── purchase_order.rs   # PurchaseOrder, PurchaseOrders
    │       │   ├── prepayment.rs       # Prepayment, Prepayments
    │       │   ├── overpayment.rs      # Overpayment, Overpayments
    │       │   ├── manual_journal.rs   # ManualJournal, ManualJournals
    │       │   ├── organisation.rs     # Organisation (~30 fields), OrganisationType, enums
    │       │   ├── report.rs           # raw Report/ReportRow/ReportCell + typed BalanceSheet/PnL/TrialBalance
    │       │   ├── connection.rs       # Connection (Identity API)
    │       │   ├── common.rs           # LineItem, LineItemTracking, Allocation, Attachment, Pagination, ValidationError
    │       │   ├── enums.rs            # CurrencyCode (~170), CountryCode (~250), TaxType (~130), TimeZone (~140)
    │       │   └── dates.rs            # MsDate(NaiveDate), MsDateTime(DateTime<Utc>), custom serde
    │       ├── api/
    │       │   ├── mod.rs
    │       │   ├── invoices.rs         # client.invoices().list(params) / .get(id)
    │       │   ├── contacts.rs
    │       │   ├── payments.rs
    │       │   ├── bank_transactions.rs
    │       │   ├── accounts.rs
    │       │   ├── reports.rs
    │       │   └── identity.rs         # connections, tenant listing
    │       ├── error.rs                # ChoSdkError enum
    │       └── config.rs               # SdkConfig (base_url, timeout, retries)
    │
    ├── cho-cli/                # thin CLI layer for agent + human interaction
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── commands/
    │       │   ├── mod.rs
    │       │   ├── auth.rs             # login, status, refresh, tenants
    │       │   ├── invoices.rs         # list, get
    │       │   ├── contacts.rs         # list, get, search
    │       │   ├── payments.rs         # list, get
    │       │   ├── transactions.rs     # list, get
    │       │   ├── accounts.rs         # list
    │       │   ├── reports.rs          # balance-sheet, pnl, trial-balance, aged-payables, aged-receivables
    │       │   └── config.rs           # set, show
    │       ├── output/
    │       │   ├── mod.rs
    │       │   ├── json.rs             # snake_case re-serialization, --meta envelope, --raw, --precise
    │       │   ├── table.rs            # comfy-table formatter
    │       │   └── csv.rs
    │       └── error.rs                # CLI error formatting (JSON stderr vs human-readable)
    │
    ├── cho-tui/                # ratatui dashboard (Phase 4)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── app.rs                  # App state, event loop
    │       └── views/
    │           ├── mod.rs
    │           ├── dashboard.rs        # overview: recent invoices, overdue count, bank balances
    │           ├── invoices.rs         # invoice list + detail
    │           ├── contacts.rs         # contact browser
    │           └── reports.rs          # report viewer
    │
    └── cho-mcp/                # MCP server (Phase 5)
        ├── Cargo.toml
        └── src/
            ├── main.rs
            └── tools/
                ├── mod.rs
                ├── invoices.rs
                ├── contacts.rs
                └── reports.rs
```

## 3. Stack

| Layer | Choice | Notes |
|---|---|---|
| Language | Rust 2024 edition | rust-version = "1.93.0" |
| Async | tokio 1.x | multi-threaded; sync wrapper via block_on |
| HTTP | reqwest 0.12+ | rustls TLS, async, connection pooling |
| Serde | serde 1.x + serde_json 1.x | PascalCase wire, snake_case CLI output |
| CLI | clap 4.x (derive) | nested subcommands, env var fallbacks |
| TUI | ratatui 0.30+ / crossterm 0.29+ | cho-tui crate |
| Money | rust_decimal 1.x | serde feature, replaces all f64 money fields |
| Dates | chrono 0.4.x | MsDate/MsDateTime newtypes (NaiveDate/DateTime\<Utc\>) |
| UUIDs | uuid 1.x | all Xero resource IDs, serde feature |
| Errors | thiserror 2.x | per-crate error enums |
| Tokens | keyring 3.x + secrecy 0.10+ | OS keychain primary, file fallback |
| Config | toml 0.8.x | ~/.config/cho/config.toml, XDG-compliant |
| Tables | comfy-table 7.x | --format table rendering |
| Logging | tracing 0.1 + tracing-subscriber 0.3 | --verbose, RUST_LOG |
| Quality | bun + biome + commitlint + husky + lint-staged | JS tooling for git hooks |
| MCP | rmcp or mcp-server (TBD) | Phase 5 |
| Mocking | wiremock 0.6.x | test-only dev-dependency |

## 4. Architecture

cho-sdk is a pure API client (zero CLI/TUI/MCP deps, publishable to crates.io); cho-cli, cho-tui, cho-mcp are thin consumers adding their interface layers.

```
Agent/Human → cho-cli (clap) / cho-tui (ratatui) / cho-mcp (MCP tools)
  → cho-sdk XeroClient [auth, rate_limit, pagination]
    → reqwest → Xero REST API (api.xero.com)
      → JSON (PascalCase, MS dates, envelope) → SDK models (Option<T>, Decimal, MsDate)
        → cho-cli output (snake_case JSON / table / CSV) → stdout + stderr
```

**Namespaced API**: `client.invoices().list(params)`, `client.contacts().get(id)` — resource-specific handles with typed builder params.

**Auto-pagination**: `list()` transparently fetches pages; `limit` caps total items (default 100); page size 100.

**Rate limiting**: SDK-internal token bucket (5 concurrent, 60/min) tracking `X-MinLimit-Remaining`/`X-DayLimit-Remaining`; exponential backoff on 429 respecting `Retry-After`; configurable/disableable.

**Transparent auth**: every request checks token expiry, auto-refreshes; `secrecy::SecretString` wraps tokens in memory.

**Output separation**: SDK structs `#[serde(rename_all = "PascalCase")]` for wire compat; CLI re-serializes to snake_case; `--raw` preserves native dates; `--precise` emits money as strings.

**Sync wrapper**: `_blocking()` variants via `tokio::runtime::Runtime::block_on`; async is primary API.

## 5. Xero API Reference

**Base URLs**: Accounting `https://api.xero.com/api.xro/2.0/`, Identity `https://api.xero.com/connections`, authorize `https://login.xero.com/identity/connect/authorize`, token `https://identity.xero.com/connect/token`.

**PKCE flow**: generate `code_verifier` (43-128 chars, URL-safe), `code_challenge = base64url(sha256(verifier))`, redirect to authorize with challenge + `S256` + scopes + `redirect_uri=http://localhost:PORT/callback`, localhost server receives callback, exchange code + verifier at token endpoint. No device flow — browser mandatory. Scopes: `openid offline_access accounting.transactions.read accounting.contacts.read accounting.settings.read accounting.reports.read accounting.journals.read files.read assets.read projects.read payroll.employees payroll.timesheets payroll.settings`.

**Token lifecycle**: access 30min, refresh 60 days (non-use), refresh single-use (each returns new pair), `offline_access` required for refresh tokens.

**Custom Connections** (Phase 3+): `client_credentials` grant, `client_id`+`client_secret`, single org, paid feature, no refresh (new token each time, 30min TTL).

**Required headers**: `xero-tenant-id` (from `GET /connections`), `Authorization: Bearer <token>`, `Content-Type: application/json`.

**Rate limits**: Concurrent 5 in-flight, 60/min, 5000/day (per app+org); 10000/min app-wide. Headers: `X-DayLimit-Remaining`, `X-MinLimit-Remaining`, `X-AppMinLimit-Remaining`; 429 with `Retry-After` (seconds).

**Response envelope**: `{ "ResourceName": [...], "pagination": {...}, "Warnings": [...] }` — PascalCase plural key; single GETs same wrapper with 1-element array; mutating adds `Id`, `Status`, `ProviderName`, `DateTimeUTC`.

**Pagination**: page-based (`page=1`, `pageSize=100`) for 12 endpoints (BankTransactions, Contacts, CreditNotes, Invoices, Payments, Prepayments, Overpayments, PurchaseOrders, Quotes, ManualJournals, LinkedTransactions, RepeatingInvoices); response: `pagination: {page, pageSize, pageCount, itemCount}`; offset-based for Journals only; non-paginated: Accounts, Currencies, TaxRates, Items.

**Date formats** — three wire formats:

| Spec marker | Wire (response) | Request | Example | Fields |
|---|---|---|---|---|
| `x-is-msdate: true` | `/Date(epoch_ms+offset)/` | `YYYY-MM-DD` | `/Date(1539993600000+0000)/` | 31 |
| `x-is-msdate-time: true` | `/Date(epoch_ms)/` | not writable | `/Date(1573755038314)/` | 26 |
| `format: date` | ISO `YYYY-MM-DD` | ISO `YYYY-MM-DD` | `"2019-10-31"` | 16 |

MS Date regex: `/\/Date\((-?\d+)(\+\d{4})?\)\//`; epoch ms since Unix epoch; offset `+HHMM`.

**Where filter**: OData-like on ~21 endpoints (`Status=="ACTIVE" AND Type=="BANK"`); cho exposes as raw `--where` pass-through.

**Query params**: `where` (~21), `order` (~23), `page` (~12), `pageSize`, `If-Modified-Since` (header, ~20), `unitdp`, `summaryOnly` (Contacts/Invoices), `searchTerm`, `Idempotency-Key` (writes), `xero-tenant-id` (all).

**API stability**: v2.0, no v3, 6-month deprecation policy. **AI/ML**: Xero prohibits training on API data; querying/displaying for agents is compliant.

## 6. SDK Models

**Organization**: one file per resource in `models/`, shared types in `common.rs`, enums in `enums.rs`, date newtypes in `dates.rs`. Each resource has entity struct (`Invoice`) + collection wrapper (`Invoices`) containing `Option<Vec<Invoice>>` + `Option<Pagination>` + `Option<Vec<ValidationError>>`.

**Serde**: `#[serde(rename_all = "PascalCase")]` for wire compat; all fields `Option<T>` with `skip_serializing_if = "Option::is_none"` (except BankTransaction: required Type/LineItems/BankAccount); money `Decimal` (never f64); IDs `Uuid`; dates `MsDate`/`MsDateTime`/`NaiveDate` per spec marker.

**Modeling challenges**:

| Challenge | Solution |
|---|---|
| Circular refs (Payment↔Invoice) | `Box<T>` or ID-only in nested position |
| Hyphenated enums (RECEIVE-OVERPAYMENT) | `#[serde(rename = "RECEIVE-OVERPAYMENT")]` per variant |
| Mixed-case enums (LineAmountTypes) | Per-enum `#[serde(rename_all)]`; most SCREAMING_SNAKE |
| Unknown enum variants | `#[serde(other)]` catch-all on every enum |
| Polymorphic Payment target | 4 optional fields, not union |
| Inline validation errors | `ValidationErrors` + `HasErrors` fields on entities |
| Nearly-all-optional fields | Accept `Option<T>` reality; builder for construction |

**Coverage tiers**:

| Tier | Resources | Phase |
|---|---|---|
| 1 (core) | Invoice, Contact, BankTransaction, Payment, Account, Connection, BalanceSheet/P&L/TrialBalance reports | 1 |
| 2 (important) | CreditNote, Quote, PurchaseOrder, Item, TaxRate, Currency, TrackingCategory, Organisation, ManualJournal, remaining reports | 3 |
| 3 (completeness) | Prepayment, Overpayment, LinkedTransaction, Budget, RepeatingInvoice, BankFeed, FixedAsset, Files, Payroll UK | 3+ |

**Reports**: Xero returns tabular Rows/Cells/Attributes; SDK has raw `Report` struct + typed `BalanceSheetReport`/`ProfitAndLossReport`/`TrialBalanceReport` with parsed sections (assets/liabilities/equity, income/expenses/net profit); typed models walk the Row/Cell tree.

**Large enums**: CurrencyCode (~170, ISO 4217 + EMPTY_CURRENCY), CountryCode (~250, ISO 3166), TaxType (~130, year-suffixed like INPUTY23/INPUTY24), TimeZone (~140); all with `#[serde(other)]`.

**MsDate/MsDateTime**: `MsDate(NaiveDate)` deserializes `/Date(epoch_ms+offset)/` → extract epoch_ms → seconds → NaiveDateTime → date; serializes `YYYY-MM-DD`. `MsDateTime(DateTime<Utc>)` similar. Round-trip tests: negative epochs, zero offset, various timezones.

## 7. CLI Design

**Command structure**: `cho <resource> <action> [flags]` -- nested two-level subcommands via clap derive; resource names are plural lowercase nouns; actions are imperative verbs.

```
cho auth login [--client-credentials]
cho auth status
cho auth refresh
cho auth tenants

cho invoices list [--where EXPR] [--order EXPR] [--from DATE] [--to DATE] [--summary] [--limit N] [--all]
cho invoices get <id-or-number>

cho contacts list [--where EXPR] [--limit N] [--all]
cho contacts get <id>
cho contacts search <term>

cho payments list [--where EXPR] [--limit N] [--all]
cho payments get <id>

cho transactions list [--where EXPR] [--from DATE] [--to DATE] [--limit N] [--all]
cho transactions get <id>

cho accounts list [--where EXPR]

cho reports balance-sheet [--date DATE] [--periods N] [--timeframe MONTH|QUARTER|YEAR]
cho reports pnl [--from DATE] [--to DATE] [--periods N] [--timeframe MONTH|QUARTER|YEAR]
cho reports trial-balance [--date DATE]
cho reports aged-payables [--contact ID] [--date DATE]
cho reports aged-receivables [--contact ID] [--date DATE]

cho config set <key> <value>
cho config show
```

**Global flags**:

| Flag                        | Default     | Description                                                           |
| --------------------------- | ----------- | --------------------------------------------------------------------- |
| `--format json\|table\|csv` | `json`      | output format                                                         |
| `--meta`                    | off         | wrap JSON output with `{"data": [...], "pagination": {...}}` envelope |
| `--raw`                     | off         | preserve Xero-native date format, skip ISO normalization              |
| `--precise`                 | off         | emit money as strings ("1234.56") instead of numbers                  |
| `--tenant <id>`             | from config | override default tenant ID                                            |
| `--verbose`                 | off         | enable tracing output (HTTP requests, auth, rate limits)              |
| `--quiet`                   | off         | suppress non-essential output                                         |
| `--no-color`                | off         | disable terminal colors                                               |
| `--limit <N>`               | 100         | max items for list commands (auto-pagination)                         |
| `--all`                     | off         | fetch all pages, no limit                                             |

**Output behavior**: bare JSON array to stdout by default; `--meta` wraps with `{"data": [...], "pagination": {...}}`; snake_case keys (re-serialized from PascalCase); dates ISO 8601 (`--raw` preserves `/Date(epoch)/`); money as numbers (`--precise` for strings); no prompts when stdin not TTY; auto-detect table (TTY) vs JSON (pipe), override with `--format`.

**Exit codes**: 0 success, 1 API/data error, 2 auth error, 3 usage error.

**Error output**: `--format json` emits `{"error": "...", "code": "AUTH_EXPIRED", "details": {...}}` on stderr; otherwise human-readable text.

## 8. Configuration

**Config file**: `~/.config/cho/config.toml` (XDG-compliant, respects `XDG_CONFIG_HOME`).

```toml
[auth]
tenant_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
client_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
# client_secret stored in OS keychain, never in config file

[defaults]
format = "json"
limit = 100

[sdk]
base_url = "https://api.xero.com/api.xro/2.0/"
timeout_secs = 30
max_retries = 3
```

**Token storage**: keyring (service `cho`, username `access_token`/`refresh_token`) primary; file fallback `~/.config/cho/tokens.enc` at 0600 when keychain unavailable; `secrecy::SecretString` in memory.

**Precedence** (high→low): CLI flags > env vars (`CHO_TENANT_ID`, `CHO_CLIENT_ID`, `CHO_CLIENT_SECRET`, `CHO_FORMAT`, `CHO_BASE_URL`) > config file > defaults.

## 9. Error Handling

**SDK error enum** (`ChoSdkError`):

| Variant        | Trigger                    | Fields                                                         |
| -------------- | -------------------------- | -------------------------------------------------------------- |
| `AuthRequired` | no valid token available   | message: String                                                |
| `TokenExpired` | refresh failed or 401      | message: String                                                |
| `RateLimited`  | HTTP 429                   | retry_after: u64                                               |
| `ApiError`     | 4xx/5xx (not 401/429)      | status: u16, message: String, validation_errors: Vec\<String\> |
| `NotFound`     | HTTP 404                   | resource: String, id: String                                   |
| `Network`      | connection/timeout         | #[from] reqwest::Error                                         |
| `Parse`        | deserialization failure    | message: String                                                |
| `Config`       | config file/keychain error | message: String                                                |

**CLI error codes** (structured for agent consumption):

| Code               | Exit | Meaning                         |
| ------------------ | ---- | ------------------------------- |
| `AUTH_REQUIRED`    | 2    | no valid token, login needed    |
| `TOKEN_EXPIRED`    | 2    | refresh failed, re-login needed |
| `RATE_LIMITED`     | 1    | retry after N seconds           |
| `NOT_FOUND`        | 1    | resource does not exist         |
| `VALIDATION_ERROR` | 1    | Xero rejected the request       |
| `API_ERROR`        | 1    | server error (5xx)              |
| `NETWORK_ERROR`    | 1    | connection/timeout failure      |
| `PARSE_ERROR`      | 1    | response deserialization failed |
| `USAGE_ERROR`      | 3    | invalid arguments/flags         |

## 10. Testing

**Mock HTTP**: wiremock dev-dependency; API modules have test modules starting mock server with expected requests/responses using recorded JSON fixtures.

**Fixtures**: `crates/cho-sdk/tests/fixtures/` per-resource subdirs; real Xero responses with data redacted.

**Date serde**: round-trip every MsDate/MsDateTime variant (positive/negative/zero epoch, with/without offset, large epoch); verify ISO 8601 output.

**Decimal precision**: money round-trip without loss; test 0.01, 999999999.99, 0.00, negatives.

**Pagination**: mock multi-page (3+), verify all items in order, `limit` caps correctly, single-page works.

**Rate limits**: mock 429 + `Retry-After`, verify retry; mock `X-MinLimit-Remaining: 0`, verify pre-emptive delay.

**CLI integration**: `assert_cmd` subprocess, verify parseable JSON stdout, exit codes, `--format table` alignment, `--meta` wrapping, `--raw` date preservation, error JSON on stderr.

**Live tests** (optional): `#[cfg(feature = "live")]`, requires credentials from a Xero developer account; contract validation only; not in CI.

### Live Testing Procedure

Live tests validate API contracts against the real Xero API. These are not run in CI to avoid rate limits and credential exposure.

**1. Create a Xero Developer Account**

1. Sign up at https://developer.xero.com
2. Create a new App (API type: "OAuth 2.0")
3. Note your Client ID from the app configuration page
4. For PKCE flow (interactive): no secret needed, just the Client ID
5. For Custom Connections (headless): enable Custom Connections and note the Client Secret

**2. Configure Environment**

```bash
# Required: OAuth client ID from your Xero app
export CHO_CLIENT_ID="YOUR_CLIENT_ID_HERE"

# Optional: for headless/CI testing with Custom Connections (paid Xero feature)
export CHO_CLIENT_SECRET="YOUR_CLIENT_SECRET_HERE"

# Optional: override default tenant (multi-org accounts)
export CHO_TENANT_ID="YOUR_TENANT_UUID"
```

**3. Authenticate**

```bash
# Interactive PKCE flow (opens browser)
cargo run -p cho-cli -- auth login

# Or headless with Custom Connections
cargo run -p cho-cli -- auth login --client-credentials
```

**4. Run Manual Live Tests**

```bash
# List invoices (basic smoke test)
cargo run -p cho-cli -- invoices list --limit 5

# Test pagination
cargo run -p cho-cli -- invoices list --limit 150 --format json | jq length

# Test reports
cargo run -p cho-cli -- reports balance-sheet

# Verify auth status
cargo run -p cho-cli -- auth status
```

**5. Validation Checklist**

- [ ] Authentication flow completes without error
- [ ] `auth status` shows valid token with reasonable expiry
- [ ] `invoices list` returns valid JSON with expected fields
- [ ] Pagination correctly fetches multiple pages (test with data-rich account)
- [ ] Reports return structured data matching typed models
- [ ] Rate limit headers are tracked (visible with `--verbose`)
- [ ] Token refresh works (wait 30+ minutes, then retry a command)

## 11. Commands

| Command               | Action                                                                 |
| --------------------- | ---------------------------------------------------------------------- |
| `bun run build`       | `cargo build --workspace --release`                                    |
| `bun run cho:dev`     | `cargo run -p cho-cli --` (debug build)                                |
| `bun run cho`         | `./target/release/cho` (release binary alias)                          |
| `bun run util:format` | `cargo fmt --all`                                                      |
| `bun run util:lint`   | `cargo clippy --all-targets --all-features -- -D warnings`             |
| `bun run util:test`   | `cargo test --workspace`                                               |
| `bun run util:types`  | `cargo check --workspace`                                              |
| `bun run util:check`  | format + lint + types + test sequentially, exit nonzero on any failure |

## 12. Quality

Zero clippy warnings (`-D warnings`), `cargo fmt --all` enforced, tests pass pre-commit via lint-staged + husky. Conventional commits `type(scope): subject` — types: feat|fix|refactor|docs|style|chore|test, scopes: sdk|cli|tui|mcp|config|deps. `#![deny(missing_docs)]` on cho-sdk; no `unwrap()` in library code (`?` propagation); `#![forbid(unsafe_code)]` on cho-cli, cho-tui, cho-mcp.

## 13. Roadmap

### Phase 0: scaffolding + progenitor experiment

- [x] Workspace Cargo.toml (resolver 3, rust-version 1.93.0), 4 member crates with workspace dep inheritance
- [x] Quality tooling: package.json bun scripts, commitlint, lint-staged, husky hooks, rustfmt.toml, biome.json, .gitignore
- [x] Progenitor experiment — SKIPPED; all SDK code written from scratch using spec as reference
- [x] Verify: `cargo build --workspace` + `bun run util:check` pass

### Phase 1: cho-sdk core

- [x] MsDate/MsDateTime newtypes (`models/dates.rs`) — regex parser for `/Date(epoch+offset)/`, ISO 8601 serialization (deviation: MsDateTime also serializes to ISO for CLI use)
- [x] rust_decimal::Decimal for all money fields with round-trip serde tests
- [x] OAuth 2.0 PKCE auth (`auth/pkce.rs`) — SHA-256 challenge, base64url, TcpListener callback, browser open; TokenPair with SecretString wrapping + expiry tracking
- [x] Token auto-refresh (`auth/mod.rs` AuthManager) — transparent refresh via `get_access_token()`, 5-min safety margin, single-use refresh token rotation
- [x] Token storage (`auth/storage.rs`) — keyring primary + JSON file fallback at 0600 perms (deviation: plaintext JSON, not encrypted .enc)
- [x] SdkConfig (`config.rs`) — base_url, timeout, max_retries with builder pattern
- [x] XeroClient (`client.rs`) — builder, auto-retry with exponential backoff, 401 auto-refresh, 429 handling, namespaced API handles (`client.invoices()` etc.)
- [x] Rate limiter (`http/rate_limit.rs`) — Semaphore concurrency (5), sliding-window MinuteTracker (60/min), header-based limits, configurable/disableable
- [x] Auto-pagination (`http/pagination.rs`) — `PaginatedResponse` trait, `PaginationParams`, iterative page fetch via `get_all_pages()` (deviation: iterative instead of async Stream)
- [x] Request builder (`http/request.rs`) — `ListParams`, `ReportParams`, `build_headers()` for auth/tenant/content-type headers
- [x] Tier 1 models: Invoice (~40 fields), Contact (~35), BankTransaction (~25), Payment (~25), Account (~15) with full serde and nested reference types
- [x] Collection wrappers (Invoices, Contacts, etc.) with `pagination` (camelCase serde) and `warnings` fields
- [x] Common types (`models/common.rs`) — LineItem, LineItemTracking, Allocation, Attachment, Pagination, ValidationError, Address, Phone, ContactPerson
- [x] Enums with `#[serde(other)]` — CurrencyCode (~170), TaxType, AccountType, InvoiceType/Status, ContactStatus, BankTransactionType, PaymentStatus, LineAmountTypes (deviation: CountryCode/TimeZone deferred to Phase 3.1)
- [x] Connection model (`models/connection.rs`) — camelCase serde for Identity API
- [x] Report models — raw Report/ReportRow/ReportCell + typed BalanceSheetReport, ProfitAndLossReport, TrialBalanceReport with Row/Cell tree parsing
- [x] 7 API modules (`api/`) — invoices (list/get/get_by_number), contacts (list/get/search), payments, bank_transactions, accounts, reports (raw + typed + aged), identity
- [x] ChoSdkError enum — all 8 variants from Section 9 in `error.rs`
- [x] `#![deny(missing_docs)]` enforced on cho-sdk
- [x] Sync wrapper (`blocking.rs`) — BlockingClient with internal tokio Runtime, BlockingClientBuilderExt trait, sync methods for all APIs

### Phase 2: cho-cli

- [x] Clap derive command tree — Auth, Invoices, Contacts, Payments, Transactions, Accounts, Reports, Config subcommands
- [x] 10 global flags: --format, --meta, --raw, --precise, --tenant, --verbose, --quiet, --no-color, --limit, --all (with env var fallbacks)
- [x] JSON output (`output/json.rs`) — pascal_to_snake_keys transform, bare array default, --meta envelope, --precise money-as-strings
- [x] Table output (`output/table.rs`) — comfy-table with Column, format_table(), right-aligned numbers
- [x] CSV output (`output/csv.rs`) — format_csv() with header row and proper quoting
- [x] Error formatting (`error.rs`) — ErrorCode enum, JSON on stderr when --format json, exit codes 0/1/2/3 per Section 9
- [x] Auth commands: login (PKCE + --client-credentials), status, refresh, tenants
- [x] Invoice commands: list (--where, --order, --from, --to, --summary), get (auto-detects UUID vs invoice number)
- [x] Contact commands: list, get, search — all with pagination
- [x] Payment commands: list (--where), get
- [x] Transaction commands: list (--where, --from, --to), get
- [x] Accounts command: list (--where, non-paginated)
- [x] Report commands: balance-sheet, pnl, trial-balance, aged-payables, aged-receivables with appropriate date/period flags
- [x] Config commands: set (section.key dotted format), show — TOML at `~/.config/cho/config.toml`
- [x] Env var support: CHO_TENANT_ID, CHO_CLIENT_ID, CHO_FORMAT, CHO_BASE_URL wired in main.rs
- [x] TTY detection via `std::io::IsTerminal` — auto-select table (TTY) vs JSON (pipe)
- [x] --verbose enables tracing_subscriber with debug filter
- [x] 25 CLI integration tests (assert_cmd + predicates) — help, subcommand help, flag parsing, argument validation, exit codes, env vars
- [x] Verify: argument parsing and exit codes verified; live API verification deferred to manual testing with Xero credentials

### Phase 3: cho-sdk Tier 2 + Tier 3 + write operations

- [x] Tier 2 models: CreditNote, Quote, PurchaseOrder, Item, TaxRate, Currency, TrackingCategory, Organisation, ManualJournal + 10 new enums; remaining report types (BankSummary, ExecutiveSummary, BudgetSummary) use raw Report model
- [x] Tier 3 models: Prepayment, Overpayment, LinkedTransaction, Budget, RepeatingInvoice with hyphenated type enums (deviation: BankFeed, FixedAsset, Files API, Payroll UK deferred — separate API endpoints/versions)
- [x] 14 API modules for Tier 2/3 — credit_notes, quotes, purchase_orders, manual_journals, prepayments, overpayments, linked_transactions, repeating_invoices (paginated); items, tax_rates, currencies, tracking_categories, organisations, budgets (non-paginated); blocking wrappers for all
- [x] Write operations — put/post/request_with_body on XeroClient with Idempotency-Key; create/update for invoices, contacts, bank_transactions; create/delete for payments (Xero payments immutable); blocking wrappers
- [x] Write safety gate — SDK-level `allow_writes` on SdkConfig, config-file-only (NO CLI flag, NO env var), reads `[safety] allow_writes` from config.toml
- [x] 14 CLI command files for Tier 2/3 list/get — kebab-case multi-word commands, paginated resources support --where/--order
- [x] CLI write commands — create/update for invoices, contacts, transactions; create/delete for payments; --file and --idempotency-key flags, gated behind require_writes_allowed()
- [x] Custom Connections auth — client_credentials grant in SDK (`credentials::authenticate()`), `cho auth login --client-credentials` reads CHO_CLIENT_SECRET env var
- [x] All 24 model files have inline deserialization tests (99 test functions); every Tier 2/3 model tested with realistic Xero JSON fixtures

#### Phase 3.1: contract verification + fixes

- [x] Fix Pagination struct casing to camelCase matching Xero wire format; update 9 test fixtures
- [x] Add PKCE `state` parameter for OAuth CSRF protection
- [x] Guard write retries behind idempotency key presence (prevent duplicate creation)
- [x] Move write safety gate from CLI-only to SDK-level (`allow_writes` on SdkConfig, `WriteNotAllowed` variant)
- [x] Wire `If-Modified-Since` header through `build_headers()` and `get_all_pages()`
- [x] Expand PKCE/client_credentials scopes (files.read, assets.read, projects.read, payroll)
- [x] Implement `--raw` flag to skip pascal_to_snake_keys, preserve PascalCase keys
- [x] Security warning + `tracing::warn!` for plaintext token file fallback
- [x] Add `Accept: application/json` header to all API requests
- [x] Fix UTF-8 safe truncation using `char_indices()` boundary detection
- [x] Input validation for InvoiceNumber where filter (OData injection prevention)
- [x] Parse `ValidationErrors[].Message` from 400 responses into `ApiError.validation_errors`
- [x] Idempotency-Key length validation (128 char max per Xero spec)
- [x] Unit tests for write safety gate, validation error extraction, UTF-8 truncate
- [x] Document 401 refresh during pagination as known limitation (partial result loss)
- [x] Wire `extract_validation_errors()` into `request_with_body()` error path
- [x] Wire table/CSV formatters through CliContext format dispatch
- [x] Only write file fallback when keyring storage fails (was unconditionally writing)
- [x] Use `refresh_attempted` flag for 401 retry (fixes 401-after-429 skipping refresh)
- [x] Track `X-AppMinLimit-Remaining` header in rate limiter
- [x] Thread pagination metadata through list output for `--meta` envelope (`ListResult<T>`)
- [x] URL scheme validation on `SdkConfig.base_url` (SSRF mitigation)
- [x] Replace `blocking_write()` in `load_stored_tokens()` with async `.write().await`
- [x] Wiremock integration tests for retry, pagination, 404, validation errors, rate limit headers
- [x] Remove `#[allow(dead_code)]` from write safety functions
- [x] Add CountryCode (62 variants) and TimeZone (35 Windows identifiers) enums

#### Phase 3.2: security hardening + observability

- [ ] Encrypt token file fallback using age/ChaCha20 OR fail closed when keyring unavailable (currently plaintext JSON at `~/.config/cho/tokens.json`)
- [ ] Add basic OData injection detection to `--where` filter — warn on suspicious patterns (`'`, `--`, `/*`, etc.) before passing to Xero
- [ ] Add random jitter to 429 retry backoff (prevent thundering herd on shared rate limits)
- [ ] Expand invoice number validation beyond `"` and `\` to cover `'`, `==`, `&&`, `||` OData operators
- [ ] Add structured request/response logging with correlation IDs (trace_id in tracing spans)
- [ ] Document live testing procedure in AGENTS.md with CHO_CLIENT_ID setup example

### Phase 4: cho-tui

- [ ] ratatui + crossterm setup in cho-tui crate
- [ ] App state machine, event loop, clean terminal restore on exit/panic
- [ ] Dashboard view: recent invoices, overdue count, bank account balances, quick stats
- [ ] Invoice list view: scrollable table, status filtering, detail panel
- [ ] Contact browser: searchable list, detail view
- [ ] Report viewer: formatted balance sheet / P&L display
- [ ] Keyboard navigation: j/k scroll, Enter detail, Esc back, q quit, / search
- [ ] Status bar: connection status, rate limit remaining, current tenant
- [ ] Verify: TUI launches, renders data from SDK, navigation works, clean shutdown

### Phase 5: cho-mcp

- [ ] MCP server crate using rmcp or equivalent
- [ ] Tool definitions for all Tier 1 resources (list, get) with configurable page size
- [ ] Tool definitions for reports (balance-sheet, pnl, trial-balance)
- [ ] Auth via config file or environment variables (same as CLI)
- [ ] Rate limiting inherited from SDK (transparent)
- [ ] Verify: MCP server starts, responds to tool list, returns valid JSON for invoice list tool
