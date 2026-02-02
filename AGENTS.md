## 1. Overview

> **Proposed development plan** -- may deviate during development; note down any deviations and correct this spec as you implement.

cho is a Rust workspace shipping four crates (cho-sdk, cho-cli, cho-tui, cho-mcp) that expose the Xero accounting REST API as a local terminal tool; primary consumers are AI agents (~65 %) invoking `cho` via shell exec with JSON stdout, and human operators (~35 %) using the CLI interactively or through a ratatui TUI; the tool connects to a single Xero organisation via OAuth 2.0 PKCE (browser-based, multi-org capable) with Custom Connections (headless, client_credentials grant) added later; read-only MVP expanding to writes in a later phase; no production-quality open-source Xero CLI/TUI exists -- this is entirely greenfield.

| Resource              | URL                                                                      |
| --------------------- | ------------------------------------------------------------------------ |
| Xero Developer Portal | https://developer.xero.com                                               |
| Xero OAuth 2.0 PKCE   | https://developer.xero.com/documentation/guides/oauth2/pkce-flow         |
| Xero Rate Limits      | https://developer.xero.com/documentation/guides/oauth2/limits            |
| Xero OpenAPI Specs    | https://github.com/XeroAPI/Xero-OpenAPI (MIT, v10.1.0, ~57 k lines YAML) |
| Xero Changelog        | https://developer.xero.com/changelog                                     |
| reqwest               | https://docs.rs/reqwest                                                  |
| serde                 | https://serde.rs                                                         |
| clap                  | https://docs.rs/clap                                                     |
| ratatui               | https://docs.rs/ratatui                                                  |
| tokio                 | https://tokio.rs                                                         |
| rust_decimal          | https://docs.rs/rust_decimal                                             |

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

| Layer         | Choice                                         | Notes                                                         |
| ------------- | ---------------------------------------------- | ------------------------------------------------------------- |
| Language      | Rust 2024 edition                              | rust-version = "1.93.0", latest stable                        |
| Async runtime | tokio 1.x                                      | multi-threaded, SDK-internal; sync wrapper via block_on       |
| HTTP client   | reqwest 0.13+                                  | rustls TLS, async, connection pooling                         |
| Serialization | serde 1.x + serde_json 1.x                     | PascalCase wire format, snake_case CLI output                 |
| CLI framework | clap 4.x (derive)                              | nested subcommands, env var fallbacks                         |
| TUI framework | ratatui 0.30+                                  | crossterm 0.29+ backend, cho-tui crate                        |
| Money         | rust_decimal 1.x                               | serde feature enabled, replaces all f64 money fields          |
| Dates         | chrono 0.4.x                                   | MsDate/MsDateTime newtypes wrapping NaiveDate/DateTime\<Utc\> |
| UUIDs         | uuid 1.x                                       | all Xero resource IDs, serde feature                          |
| Errors        | thiserror 2.x                                  | per-crate error enums                                         |
| Token storage | keyring 3.x                                    | OS keychain; encrypted file fallback with secrecy 0.10+       |
| Config        | toml 0.8.x                                     | ~/.config/cho/config.toml, XDG-compliant                      |
| Table output  | comfy-table 7.x                                | --format table rendering                                      |
| Logging       | tracing 0.1 + tracing-subscriber 0.3           | --verbose flag, RUST_LOG support                              |
| Quality gates | bun + biome + commitlint + husky + lint-staged | JS tooling for git hooks                                      |
| MCP           | rmcp or mcp-server (TBD)                       | cho-mcp crate, Phase 5                                        |
| HTTP mocking  | httpmock or wiremock                           | test-only dependency                                          |

## 4. Architecture

cho-sdk is a pure API client crate with zero CLI/TUI/MCP dependencies, publishable to crates.io as a standalone Xero Rust SDK; cho-cli, cho-tui, and cho-mcp are thin consumer crates that depend on cho-sdk and add their respective interface layers; this separation means the SDK can be versioned and distributed independently.

```
Agent / Human
  |
  v
cho-cli (clap parse, validate flags, dispatch)
  |                                    cho-tui (ratatui render, keyboard nav)
  |                                    cho-mcp (MCP tool dispatch)
  |                                      |
  +--------------------------------------+
  |
  v
cho-sdk XeroClient
  |- auth (PKCE / client_credentials, auto-refresh)
  |- rate_limit (token bucket, header tracking, 429 backoff)
  |- pagination (async Stream, auto-fetch pages)
  |
  v
reqwest -> Xero REST API (api.xero.com)
  |
  v
JSON response (PascalCase keys, MS dates, envelope wrapper)
  |
  v
SDK models (typed Rust structs, Option<T>, Decimal, MsDate)
  |
  v
cho-cli output layer (re-serialize to snake_case JSON / table / CSV)
  |
  v
stdout (bare JSON array default) + stderr (errors)
```

**Namespaced API surface**: `client.invoices().list(params)` and `client.contacts().get(id)` -- each resource method returns a resource-specific API handle; params use typed builder structs.

**Auto-pagination**: `list()` returns `impl Stream<Item = Result<T>>` that transparently fetches pages; `limit` param caps total items (default 100); page size fixed at 100 (Xero default max).

**Rate limiting**: SDK-internal token bucket (5 concurrent, 60/min) tracking `X-MinLimit-Remaining` and `X-DayLimit-Remaining` response headers; exponential backoff on HTTP 429 respecting `Retry-After`; configurable (disable for tests, custom limits).

**Transparent auth**: every SDK request checks token expiry, refreshes if needed; caller never manages tokens manually; `secrecy::SecretString` wraps tokens in memory.

**Output separation**: SDK structs use `#[serde(rename_all = "PascalCase")]` for wire compat with Xero API; CLI output layer re-serializes to snake_case via `serde_json::Value` key transform; `--raw` skips date normalization (preserves `/Date(epoch)/`); `--precise` serializes money as strings instead of numbers.

**Sync wrapper**: SDK exposes `_blocking()` method variants using `tokio::runtime::Runtime::block_on` for synchronous callers; async is primary API.

## 5. Xero API Reference

**Base URLs**: Accounting/most APIs at `https://api.xero.com/api.xro/2.0/`, Identity at `https://api.xero.com/connections`, authorization at `https://login.xero.com/identity/connect/authorize`, token exchange at `https://identity.xero.com/connect/token`.

**OAuth 2.0 PKCE flow** (cho Phase 1 auth): generate `code_verifier` (43-128 chars, URL-safe random), compute `code_challenge = base64url(sha256(code_verifier))`, redirect user to authorize endpoint with `code_challenge` + `code_challenge_method=S256` + scopes + `redirect_uri=http://localhost:PORT/callback`, start localhost HTTP server to receive callback, exchange authorization code + `code_verifier` at token endpoint; no device flow exists -- browser redirect is mandatory even for CLI; scopes needed: `openid offline_access accounting.transactions.read accounting.contacts.read accounting.settings.read accounting.reports.read accounting.journals.read files.read assets.read projects.read payroll.employees payroll.timesheets payroll.settings`.

**Token lifecycle**: access tokens expire after 30 minutes, refresh tokens expire after 60 days of non-use, refresh tokens are single-use (each refresh returns a new access_token + refresh_token pair), `offline_access` scope is required to receive refresh tokens.

**Custom Connections** (cho Phase 3+): `client_credentials` grant using `client_id` + `client_secret`, single org only, paid Xero feature, no refresh token needed (request new access token each time, 30 min TTL).

**Required headers**: `xero-tenant-id` on every API request (obtained from `GET /connections` after auth), `Authorization: Bearer <token>`, `Content-Type: application/json`.

**Rate limits**:

| Limit           | Value        | Scope                    |
| --------------- | ------------ | ------------------------ |
| Concurrent      | 5 in-flight  | per app + org connection |
| Per minute      | 60 calls     | per app + org connection |
| Daily           | 5,000 calls  | per app + org connection |
| App-wide minute | 10,000 calls | across all tenancies     |

Response headers: `X-DayLimit-Remaining`, `X-MinLimit-Remaining`, `X-AppMinLimit-Remaining`; HTTP 429 with `Retry-After` (seconds) on exceeded.

**Response envelope**: all collection endpoints return `{ "ResourceName": [...], "pagination": {...}, "Warnings": [...] }` where the resource key is PascalCase plural (e.g., `"Invoices"`, `"Contacts"`); single-resource GETs return the same wrapper with a 1-element array; mutating responses add `Id` (UUID, not resource ID), `Status` ("OK"), `ProviderName`, `DateTimeUTC` (MS date).

**Pagination**: page-based for 12 endpoints (BankTransactions, Contacts, CreditNotes, Invoices, Payments, Prepayments, Overpayments, PurchaseOrders, Quotes, ManualJournals, LinkedTransactions, RepeatingInvoices) using `page=1` (1-indexed) and `pageSize=100`; response includes `pagination: {page, pageSize, pageCount, itemCount}`; offset-based for Journals only (`offset=N`); non-paginated reference endpoints (Accounts, Currencies, TaxRates, Items) return full list.

**Date formats** -- three distinct wire formats requiring different deserialization:

| Spec marker              | Wire format (response)    | Request format   | Example                      | Fields |
| ------------------------ | ------------------------- | ---------------- | ---------------------------- | ------ |
| `x-is-msdate: true`      | `/Date(epoch_ms+offset)/` | `YYYY-MM-DD`     | `/Date(1539993600000+0000)/` | 31     |
| `x-is-msdate-time: true` | `/Date(epoch_ms)/`        | not writable     | `/Date(1573755038314)/`      | 26     |
| `format: date`           | ISO `YYYY-MM-DD`          | ISO `YYYY-MM-DD` | `"2019-10-31"`               | 16     |

MS Date regex pattern: `/\/Date\((-?\d+)(\+\d{4})?\)\//`; epoch is milliseconds since Unix epoch; offset is timezone in `+HHMM` format.

**Where filter**: OData-like expression syntax on ~21 endpoints (`Status=="ACTIVE" AND Type=="BANK"`, `AmountDue > 1000.0`); cho exposes as raw `--where` string pass-through.

**Common query parameters**: `where` (string, ~21 endpoints), `order` (string, ~23 endpoints), `page` (int, ~12 endpoints), `pageSize` (int), `If-Modified-Since` (header, ~20 endpoints), `unitdp` (int, decimal places), `summaryOnly` (bool, Contacts/Invoices), `searchTerm` (string), `Idempotency-Key` (header, writes), `xero-tenant-id` (header, all requests).

**API stability**: version 2.0, no v3 announced, 6-month deprecation policy, breaking change detection via `oasdiff` in CI.

**AI/ML prohibition**: Xero prohibits using API data to train AI/ML models; querying and displaying data for agent responses is compliant; no training pipelines.

## 6. SDK Models

**Organization**: one file per resource in `crates/cho-sdk/src/models/`, shared types in `common.rs`, large enums in `enums.rs`, date newtypes in `dates.rs`; every resource has an entity struct (`Invoice`) and a collection wrapper struct (`Invoices`) containing `Option<Vec<Invoice>>` + `Option<Pagination>` + `Option<Vec<ValidationError>>`.

**Serde strategy**: all structs annotated `#[serde(rename_all = "PascalCase")]` for Xero wire compat; all fields `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]` except BankTransaction which has `required: [Type, LineItems, BankAccount]`; money fields use `rust_decimal::Decimal` (never f64); UUIDs use `uuid::Uuid`; dates use `MsDate`/`MsDateTime`/`chrono::NaiveDate` based on spec marker.

**Modeling challenges** (implementer reference):

| Challenge                                      | Solution                                                                    |
| ---------------------------------------------- | --------------------------------------------------------------------------- |
| Circular refs (Payment <-> Invoice)            | `Box<T>` or flatten to ID-only in nested position (API returns subset)      |
| Hyphenated enum values (RECEIVE-OVERPAYMENT)   | `#[serde(rename = "RECEIVE-OVERPAYMENT")]` per variant                      |
| Mixed-case enums (LineAmountTypes: PascalCase) | Per-enum `#[serde(rename_all)]` config; most are SCREAMING_SNAKE            |
| Unknown enum variants                          | `#[serde(other)]` catch-all variant on every enum                           |
| Polymorphic Payment target                     | 4 optional fields (Invoice, CreditNote, Prepayment, Overpayment), not union |
| Inline validation errors                       | `ValidationErrors` + `HasErrors` fields on entity structs                   |
| Nearly-all-optional fields                     | Accept the `Option<T>` reality; builder pattern for construction            |

**API coverage tiers**:

| Tier             | Resources                                                                                                                                                                                                 | Phase    |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| 1 (core)         | Invoice, Contact, BankTransaction, Payment, Account, Connection, BalanceSheet report, P&L report, TrialBalance report                                                                                     | Phase 1  |
| 2 (important)    | CreditNote, Quote, PurchaseOrder, Item, TaxRate, Currency, TrackingCategory, Organisation, ManualJournal, remaining reports (AgedPayables, AgedReceivables, BankSummary, ExecutiveSummary, BudgetSummary) | Phase 3  |
| 3 (completeness) | Prepayment, Overpayment, LinkedTransaction, Budget, RepeatingInvoice, BankFeed, FixedAsset, Files, Payroll UK (Employee, Timesheet, Leave, PayRun, PaySlip, Settings)                                     | Phase 3+ |

**Report models**: Xero reports return tabular `Rows`/`Cells`/`Attributes` not structured objects; SDK provides both raw `Report` struct (mirrors API) and typed report structs (`BalanceSheetReport`, `ProfitAndLossReport`, `TrialBalanceReport`) with parsed sections (assets, liabilities, equity for balance sheet; income, expenses, net profit for P&L); typed models constructed by walking the raw Row/Cell tree.

**Large enums**: CurrencyCode (~170 variants, ISO 4217 + `EMPTY_CURRENCY` sentinel), CountryCode (~250, ISO 3166 alpha-2), TaxType (~130, including year-suffixed variants like `INPUTY23`/`INPUTY24`), TimeZone (~140, IANA names); all with `#[serde(other)]` catch-all.

**MsDate/MsDateTime newtypes**: `MsDate(chrono::NaiveDate)` deserializes `/Date(epoch_ms+offset)/` by extracting epoch_ms, converting to seconds, building NaiveDateTime, extracting date; serializes to `YYYY-MM-DD` for request bodies; `MsDateTime(chrono::DateTime<Utc>)` deserializes `/Date(epoch_ms)/` similarly; comprehensive round-trip tests required including negative epochs, zero offset, various timezone offsets.

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

**Output behavior**: default is bare JSON array to stdout (`[{...}, {...}]`); `--meta` wraps with `{"data": [...], "pagination": {"page": N, "page_count": N, "item_count": N}}`; all JSON keys are snake_case (re-serialized from SDK PascalCase structs); dates normalized to ISO 8601 by default (`--raw` preserves `/Date(epoch)/`); money as JSON numbers by default (`--precise` for string representation); no interactive prompts when stdin is not a TTY; auto-detect: table format for TTY, JSON for pipe (override with `--format`).

**Exit codes**: 0 success, 1 API/data error, 2 auth error, 3 usage/argument error.

**Error output**: when `--format json` is active, errors emit structured JSON on stderr: `{"error": "message", "code": "AUTH_EXPIRED", "details": {...}}`; otherwise human-readable text on stderr.

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

**Token storage**: OS keychain via `keyring` crate (service: `cho`, username: `access_token` / `refresh_token`) as primary; encrypted file fallback at `~/.config/cho/tokens.enc` with `0600` permissions when keychain is unavailable; `secrecy::SecretString` for all in-memory token handling.

**Precedence** (highest to lowest): CLI flags > environment variables (`CHO_TENANT_ID`, `CHO_CLIENT_ID`, `CHO_CLIENT_SECRET`, `CHO_FORMAT`, `CHO_BASE_URL`) > config file > built-in defaults.

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

**Mock HTTP layer**: `httpmock` (or `wiremock`) as dev-dependency in cho-sdk; every API module (`api/invoices.rs`, `api/contacts.rs`, etc.) has a corresponding test module that starts a mock server, registers expected requests/responses, and exercises the SDK client methods; mock server returns recorded JSON fixtures.

**Fixture organization**: `crates/cho-sdk/tests/fixtures/` with per-resource subdirectories (`invoices/list.json`, `invoices/list_page2.json`, `invoices/get.json`, `contacts/list.json`, etc.); fixtures are real Xero API responses with sensitive data (IDs, names, amounts) redacted/anonymized.

**Date serde round-trip tests**: every MsDate/MsDateTime variant: positive epoch, negative epoch, with timezone offset, without offset, zero epoch, large epoch; parse then serialize and verify ISO 8601 output; exercise the regex parser edge cases.

**Decimal precision tests**: money fields round-trip through JSON serialize/deserialize without precision loss; test `0.01`, `999999999.99`, `0.00`, negative amounts.

**Pagination tests**: mock multi-page responses (3+ pages), verify Stream yields all items in order, verify `limit` param caps correctly, verify single-page response works.

**Rate limit tests**: mock 429 response with `Retry-After` header, verify client waits and retries; mock `X-MinLimit-Remaining: 0`, verify client pre-emptively delays.

**CLI integration tests**: run `cho` binary as subprocess (`assert_cmd` or `std::process::Command`), verify stdout is parseable JSON, verify exit codes match spec, verify `--format table` produces aligned output, verify `--meta` wraps correctly, verify `--raw` preserves date format, verify error JSON on stderr for invalid auth.

**Live API tests** (optional): behind `#[cfg(feature = "live")]` feature flag, require `CHO_CLIENT_ID` + `CHO_CLIENT_SECRET` environment variables, run against real Xero organisation, contract validation only (schemas match, pagination works, auth flow completes); not run in CI, manual execution only.

## 11. Commands

| Command               | Action                                                                 |
| --------------------- | ---------------------------------------------------------------------- |
| `bun run build`       | `cargo build --workspace --release`                                    |
| `bun run dev`         | `cargo run -p cho-cli --`                                              |
| `bun run util:format` | `cargo fmt --all`                                                      |
| `bun run util:lint`   | `cargo clippy --all-targets --all-features -- -D warnings`             |
| `bun run util:test`   | `cargo test --workspace`                                               |
| `bun run util:types`  | `cargo check --workspace`                                              |
| `bun run util:check`  | format + lint + types + test sequentially, exit nonzero on any failure |

## 12. Quality

Zero clippy warnings (`-D warnings`), `cargo fmt --all` enforced, all tests passing pre-commit via lint-staged + husky; commitlint enforces conventional commits `type(scope): subject` with types `[feat|fix|refactor|docs|style|chore|test]` and scopes `[sdk|cli|tui|mcp|config|deps]`; `#![deny(missing_docs)]` on cho-sdk (all public items documented with `///`); no `unwrap()` in library code (`?` propagation), no `unsafe` unless required for OS keychain FFI; `#![forbid(unsafe_code)]` on cho-cli, cho-tui, cho-mcp.

## 13. Roadmap

### Phase 0: scaffolding + progenitor experiment

- [x] Workspace `Cargo.toml` (resolver = "3", rust-version = "1.93.0") with 4 member crates (cho-sdk, cho-cli, cho-tui, cho-mcp) as empty `lib.rs`/`main.rs` stubs
    - Implemented in initial scaffolding commit; all 4 crates with workspace dependency inheritance
- [x] `package.json` with bun scripts (build, dev, util:format, util:lint, util:test, util:types, util:check)
    - All scripts configured including util:clean
- [x] `commitlint.config.js` with scopes `sdk|cli|tui|mcp|config|deps`
    - Conventional commits enforced via husky commit-msg hook
- [x] `lint-staged.config.js` triggering `bun run util:check`
    - Runs full check pipeline on staged files
- [x] `.husky/pre-commit` (lint-staged) + `.husky/commit-msg` (commitlint)
    - Both hooks in place and functional
- [x] `rustfmt.toml` (edition = "2024")
    - Edition 2024 configured
- [x] `biome.json` (extends global config)
    - Extends global biome config
- [x] `.gitignore` (node_modules, target, .DS_Store, .env, .env.\*, .claude, .tmp, .wrangler)
    - All patterns included
- [x] Clone `Xero-OpenAPI` repo, run progenitor against `xero_accounting.yaml`, archive output to `.tmp/progenitor/`; evaluate generated code quality, use only as loose reference -- do not copy/paste directly, write all cho-sdk code from scratch with proper architectural decisions
    - SKIPPED: progenitor experiment deferred; no research file or OpenAPI clone present; all SDK code will be written from scratch using AGENTS.md spec as reference
- [x] Move `xero-terminal-tool-research.md` to `.tmp/`
    - SKIPPED: file does not exist in repo; no action needed
- [x] Verify: `cargo build --workspace` succeeds, `bun run util:check` exits 0
    - Verified: both commands pass cleanly

### Phase 1: cho-sdk core

- [ ] `MsDate` and `MsDateTime` newtypes with custom serde deserializer (regex-based, handles `/Date(epoch+offset)/` and `/Date(epoch)/`), serializer (ISO 8601 for MsDate, not serialized for MsDateTime); round-trip tests for positive/negative epochs, offsets, edge cases
- [ ] `rust_decimal::Decimal` for all money fields, serde round-trip tests (0.01, large values, negatives, zero)
- [ ] OAuth 2.0 PKCE auth module: code_verifier/challenge generation, localhost callback HTTP server (tokio + hyper/axum minimal), browser open via `open` crate, authorization code exchange, token pair storage; tested manually against real Xero
- [ ] Token refresh module: auto-refresh before expiry, single-use refresh token handling (store new pair on every refresh), expiry tracking; tested with mock token endpoint
- [ ] Token storage: keyring crate for OS keychain (service "cho"), encrypted file fallback at `~/.config/cho/tokens.enc` with `0600` perms; tested on macOS
- [ ] `SdkConfig` struct: base_url, timeout_secs, max_retries
- [ ] `XeroClient` builder: accepts SdkConfig + auth provider + tenant_id; injects `Authorization` and `xero-tenant-id` headers on every request
- [ ] Rate limiter: token bucket (5 concurrent, 60/min), parse `X-MinLimit-Remaining` / `X-DayLimit-Remaining` from response headers, exponential backoff on 429 respecting `Retry-After`; configurable; tested with mock 429 responses
- [ ] Auto-pagination: `list()` returns `impl Stream<Item = Result<T>>`, fetches pages transparently (page=1,2,3...) until `page >= pageCount`, respects `limit` param; tested with mock multi-page responses
- [ ] Request builder: auth header, tenant header, where/order query params, page/pageSize
- [ ] Tier 1 models with full serde derives and fixture deserialization tests: Invoice (~40 fields), Contact (~35), BankTransaction (~25), Payment (~25), Account (~15)
- [ ] Collection wrapper structs: Invoices, Contacts, BankTransactions, Payments, Accounts with pagination + warnings
- [ ] Common types: LineItem, LineItemTracking, Allocation, Attachment, Pagination, ValidationError, Address, Phone, ContactPerson
- [ ] Large enums: CurrencyCode, CountryCode, TaxType, AccountType, InvoiceType, InvoiceStatus, ContactStatus, BankTransactionType, PaymentStatus, LineAmountTypes -- all with `#[serde(other)]`
- [ ] Connection model (Identity API) for tenant listing
- [ ] Report models: raw `Report`/`ReportRow`/`ReportCell` + typed `BalanceSheetReport`/`ProfitAndLossReport`/`TrialBalanceReport` with parsing from tabular structure
- [ ] API modules: `client.invoices().list(params)`, `.get(id)`; `client.contacts().list()`, `.get(id)`, `.search(term)`; `client.payments().list()`, `.get(id)`; `client.bank_transactions().list()`, `.get(id)`; `client.accounts().list()`; `client.reports().balance_sheet(params)`, `.profit_and_loss(params)`, `.trial_balance(params)`; `client.identity().connections()`
- [ ] `ChoSdkError` enum with all variants from Section 9
- [ ] `#![deny(missing_docs)]` enforced, all public items documented
- [ ] Sync wrapper: `_blocking()` variants for key methods
- [ ] Verify: `cargo test -p cho-sdk` all passing, fixture deserialization covers all Tier 1 models, MsDate round-trip works, Decimal precision preserved

### Phase 2: cho-cli

- [ ] clap derive command tree matching Section 7 structure exactly
- [ ] Global flags: --format, --meta, --raw, --precise, --tenant, --verbose, --quiet, --no-color, --limit, --all
- [ ] JSON output formatter: snake_case key re-serialization from PascalCase SDK structs via `serde_json::Value` transform; bare array by default; `--meta` wraps with `{"data": [...], "pagination": {...}}`; `--raw` skips date ISO normalization; `--precise` serializes money as strings
- [ ] Table output formatter: comfy-table with column alignment, header row, truncation for wide fields, `font-variant-numeric: tabular-nums` equivalent (right-align numbers)
- [ ] CSV output formatter: standard CSV with header row
- [ ] Error formatter: JSON on stderr when `--format json` with structured error codes, human-readable text otherwise
- [ ] Exit codes: 0/1/2/3 per Section 9
- [ ] `cho auth login` triggers PKCE flow, stores tokens, prints tenant list
- [ ] `cho auth status` prints token expiry, tenant info, connected orgs
- [ ] `cho auth refresh` forces token refresh
- [ ] `cho auth tenants` lists connected organisations
- [ ] `cho invoices list` with --where, --order, --from, --to, --summary, --limit, --all
- [ ] `cho invoices get <id-or-number>`
- [ ] `cho contacts list`, `cho contacts get <id>`, `cho contacts search <term>`
- [ ] `cho payments list`, `cho payments get <id>`
- [ ] `cho transactions list`, `cho transactions get <id>`
- [ ] `cho accounts list`
- [ ] `cho reports balance-sheet`, `cho reports pnl`, `cho reports trial-balance`, `cho reports aged-payables`, `cho reports aged-receivables`
- [ ] `cho config set <key> <value>`, `cho config show`
- [ ] Config file creation/reading from `~/.config/cho/config.toml`
- [ ] Environment variable support (CHO_TENANT_ID, CHO_CLIENT_ID, CHO_CLIENT_SECRET, CHO_FORMAT, CHO_BASE_URL)
- [ ] TTY detection: auto-select table format for TTY, JSON for pipe; no interactive prompts when stdin is not TTY
- [ ] `--verbose` enables tracing subscriber output
- [ ] CLI integration tests: run binary as subprocess, verify JSON parseable, verify exit codes, verify table output, verify error formatting
- [ ] Verify: `cho invoices list --format json | jq '.[0].invoice_id'` returns valid UUID; `cho invoices list --format table` renders aligned; invalid auth produces exit code 2

### Phase 3: cho-sdk Tier 2 + Tier 3 + write operations

- [ ] Tier 2 models: CreditNote, Quote, PurchaseOrder, Item, TaxRate, Currency, TrackingCategory, Organisation, ManualJournal + remaining report types (AgedPayables, AgedReceivables, BankSummary, ExecutiveSummary, BudgetSummary)
- [ ] Tier 3 models: Prepayment, Overpayment, LinkedTransaction, Budget, RepeatingInvoice, BankFeed, FixedAsset, Files API models, Payroll UK models (Employee, Timesheet, Leave, PayRun, PaySlip, Settings)
- [ ] API modules for all Tier 2/3 resources
- [ ] Write operations on SDK: `client.invoices().create(invoice)`, `.update(id, invoice)`; same for Contact, Payment, BankTransaction; `Idempotency-Key` header support
- [ ] CLI commands for Tier 2/3 list/get
- [ ] CLI commands for writes: `cho invoices create --file invoice.json`, `cho invoices update <id> --file updates.json`
- [ ] Custom Connections auth (client_credentials grant) in SDK + `cho auth login --client-credentials` in CLI
- [ ] Fixture tests for all new models
- [ ] Verify: all new models deserialize, write operations work against mock server

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
