> `cho` is a Rust workspace for FreeAgent automation: [`crates/cho-sdk/`](crates/cho-sdk/) owns OAuth, transport, resources, and finance heuristics; [`crates/cho-cli/`](crates/cho-cli/) exposes a compact JSON-first command surface; [`crates/cho-tui/`](crates/cho-tui/) is a read-first ratatui navigator over the same config, auth, and runtime state.

## 1. Documentation

- Primary external references: [FreeAgent docs hub](https://dev.freeagent.com/docs), [Introduction](https://dev.freeagent.com/docs/introduction), [OAuth](https://dev.freeagent.com/docs/oauth), [Changes feed](https://dev.freeagent.com/docs/changes)
- Local source-of-truth files: [`crates/cho-cli/src/main.rs`](crates/cho-cli/src/main.rs), [`crates/cho-cli/src/registry.rs`](crates/cho-cli/src/registry.rs), [`crates/cho-sdk/src/api/specs.rs`](crates/cho-sdk/src/api/specs.rs), [`crates/cho-sdk/src/client.rs`](crates/cho-sdk/src/client.rs), [`crates/cho-sdk/src/liabilities.rs`](crates/cho-sdk/src/liabilities.rs), [`crates/cho-tui/src/routes.rs`](crates/cho-tui/src/routes.rs), [`crates/cho-tui/src/api.rs`](crates/cho-tui/src/api.rs)
- Contract-heavy tests: [`crates/cho-cli/tests/cli_contract.rs`](crates/cho-cli/tests/cli_contract.rs), [`crates/cho-cli/tests/cli_drift.rs`](crates/cho-cli/tests/cli_drift.rs), [`crates/cho-sdk/tests/http_contract.rs`](crates/cho-sdk/tests/http_contract.rs)
- Canonical repo instructions live in [`AGENTS.md`](AGENTS.md); [`CLAUDE.md`](CLAUDE.md) and [`README.md`](README.md) mirror it in this repository

## 2. Repository Structure

```text
.
├── crates/
│   ├── cho-sdk/            reusable FreeAgent client, auth, resource specs, liabilities logic
│   ├── cho-cli/            clap CLI, audit log, output envelopes, command registry, tests
│   └── cho-tui/            ratatui app, route catalog, background fetch worker, cache
├── AGENTS.md               canonical repo-level instructions
├── Cargo.toml              workspace manifest and shared Rust deps/lints
├── package.json            Bun quality gate and release/install wrapper
└── .husky/                 pre-commit and commit-msg hooks
```

- Start in [`crates/cho-cli/src/commands/`](crates/cho-cli/src/commands/) for user-facing command changes, [`crates/cho-sdk/src/`](crates/cho-sdk/src/) for API/auth/transport behavior, and [`crates/cho-tui/src/`](crates/cho-tui/src/) for interactive navigation
- Treat [`crates/cho-cli/src/registry.rs`](crates/cho-cli/src/registry.rs), [`crates/cho-sdk/src/api/specs.rs`](crates/cho-sdk/src/api/specs.rs), and [`crates/cho-tui/src/routes.rs`](crates/cho-tui/src/routes.rs) as catalog files that often need coordinated edits

## 3. Stack

| Layer | Choice | Notes |
| --- | --- | --- |
| Workspace | Rust 2024 | three-crate workspace with shared deps in [`Cargo.toml`](Cargo.toml) |
| SDK/runtime | `tokio` + `reqwest` + `rustls` | async transport, pagination, retries, 401 refresh, 429 handling |
| CLI | `clap` + custom JSON envelope/output adapters | default stdout is one compact JSON object; table/csv/text are opt-in |
| TUI | `ratatui` + `crossterm` | direct SDK consumer with background fetch worker and persisted cache |
| Auth/secrets | OAuth code flow + `secrecy` | tokens stored in `tokens.json`, login callback defaults to `127.0.0.1:53682` |
| JS tooling | Bun + Husky + commitlint + Biome | hooks, lockfile, and `util:*` quality orchestration only |

## 4. Commands

- `bun install` installs JS tooling and Husky hooks
- `cargo run -p cho-cli -- tools` and `cargo run -p cho-cli -- tools <name>` are the authoritative machine-readable command catalog
- `cargo run -p cho-cli -- health` checks home/config/credentials/audit/token readiness before trying real work
- `cargo run -p cho-cli -- --help`, `cargo run -p cho-cli -- start`, and `cargo run -p cho-tui` are the fastest local iteration paths
- `cargo test --workspace` runs all Rust tests; use the crate-specific contract suites when narrowing failures
- `bun run util:check` is the required full gate before completion
- `bun run build` builds release binaries and installs `cho` plus `cho-tui` into `${CHO_HOME:-${TOOLS_HOME:-$HOME/.tools}/cho}/`

## 5. Architecture

- [`crates/cho-cli/src/main.rs`](crates/cho-cli/src/main.rs) bootstraps `config -> audit -> auth -> FreeAgentClient`; early commands `tools`, `health`, `config`, and `start` intentionally bypass full API bootstrap
- [`crates/cho-cli/src/audit.rs`](crates/cho-cli/src/audit.rs) is safety-critical: it records `command.start/input/output/end` plus HTTP request/response events, redacts secrets, and hard-fails bootstrap when the audit log is unavailable
- [`crates/cho-sdk/src/client.rs`](crates/cho-sdk/src/client.rs) enforces same-origin absolute URLs, clamps pagination, follows `Link` pagination, retries rate limits/transient failures, refreshes on 401, and blocks mutating requests unless `allow_writes` is enabled
- [`crates/cho-sdk/src/liabilities.rs`](crates/cho-sdk/src/liabilities.rs) is the non-trivial finance layer behind `tax-calendar`, `taxes reconcile`, and `summary`; it merges company, payroll, bank, and optional self-assessment data and adds derived `status_trust` fields
- [`crates/cho-cli/src/registry.rs`](crates/cho-cli/src/registry.rs) and [`crates/cho-sdk/src/api/specs.rs`](crates/cho-sdk/src/api/specs.rs) are hand-maintained, not generated; command/resource additions usually also require test updates and TUI route decisions
- [`crates/cho-tui/src/api.rs`](crates/cho-tui/src/api.rs) talks to `cho-sdk` directly rather than shelling out to `cho`; [`crates/cho-cli/src/commands/start.rs`](crates/cho-cli/src/commands/start.rs) only launches a sibling or `PATH` `cho-tui` binary

## 6. Runtime and State

- Home resolution order is `CHO_HOME` -> `TOOLS_HOME/cho` -> `$HOME/.tools/cho` via [`crates/cho-sdk/src/home.rs`](crates/cho-sdk/src/home.rs)
- Runtime files live outside the repo: `config.toml`, `history.log`, `tokens.json`, and `tui-cache.json` under the resolved `cho` home
- CLI credential precedence is `--client-id/--client-secret` -> `CHO_CLIENT_ID` / `CHO_CLIENT_SECRET` -> `config.toml` `auth.*`; SDK base URL precedence is `CHO_BASE_URL` -> `config.toml` `sdk.base_url` -> FreeAgent default
- `auth status`, `health`, CLI bootstrap, and TUI startup all call trusted session checks that can refresh tokens and rewrite `tokens.json`; these are not read-only inspections
- TUI route data uses stale-while-revalidate caching in [`crates/cho-tui/src/cache.rs`](crates/cho-tui/src/cache.rs); preview and full payloads persist to `tui-cache.json`, oversized cache files are rejected, and stale cached data may be shown while a refresh is in flight
- JSON mode writes only the compact envelope to stdout; `--verbose` enables tracing to stderr, `--text` or `--format table|csv` switch stdout into human output

## 7. Conventions

- `cho tools` is the authoritative contract surface; [`crates/cho-cli/tests/cli_contract.rs`](crates/cho-cli/tests/cli_contract.rs) and [`crates/cho-cli/tests/cli_drift.rs`](crates/cho-cli/tests/cli_drift.rs) reject duplicate names, stale help/output metadata, and any reintroduction of the removed `--json` flag
- Generic resource writes consume JSON files up to 50 MB, auto-wrap unwrapped payloads under the singular resource key, and support repeated `--query key=value` pairs for FreeAgent edge cases
- Bank transaction explanation updates can attach local files; attachments are base64-encoded client-side, MIME-sniffed by extension, and capped at FreeAgent's 5 MB limit
- Several surfaces are client-composed rather than direct endpoint pass-through: cross-account bank transaction listing, invoice `--unpaid-only` filtering, grouped category flattening, tax-calendar assembly, HMRC reconciliation, and TUI bank annotations
- `--precise` preserves decimal-like JSON values as strings; JSON output also compact-redacts signed company logo URLs instead of dumping volatile query tokens

## 8. Constraints

- Never commit or hand-edit runtime state under `CHO_HOME`, including `config.toml`, `tokens.json`, `history.log`, `tui-cache.json`, or binaries installed by `bun run build`
- Never bypass `[safety] allow_writes = true`; write blocking is enforced in both [`crates/cho-cli/src/context.rs`](crates/cho-cli/src/context.rs) and [`crates/cho-sdk/src/client.rs`](crates/cho-sdk/src/client.rs)
- Never trust arbitrary absolute resource URLs; the SDK only accepts `http(s)` URLs on the configured FreeAgent origin and base path
- Treat [`crates/cho-cli/src/registry.rs`](crates/cho-cli/src/registry.rs), [`crates/cho-sdk/src/api/specs.rs`](crates/cho-sdk/src/api/specs.rs), [`crates/cho-tui/src/routes.rs`](crates/cho-tui/src/routes.rs), [`crates/cho-cli/tests/cli_contract.rs`](crates/cho-cli/tests/cli_contract.rs), and [`crates/cho-cli/tests/cli_drift.rs`](crates/cho-cli/tests/cli_drift.rs) as a coordinated change set for command-surface work
- Keep stdout clean in JSON mode; ad-hoc diagnostics break the machine contract and belong on stderr or in the audit log

## 9. Validation

- Required gate: `bun run util:check`
- If you change CLI command surface, registry metadata, help text, output envelopes, or write gating, run `cargo test -p cho-cli --test cli_contract --test cli_drift` and smoke `cargo run -p cho-cli -- tools`
- If you change SDK auth, transport, pagination, absolute URL handling, or resource behavior, run `cargo test -p cho-sdk --test http_contract`
- If you change TUI routes, fetch context, cache logic, or palette/navigation behavior, run `cargo run -p cho-tui` and verify startup warnings, palette prompts, stale-cache revalidation, and route refresh with a seeded `CHO_HOME`
- For mutating workflows, validate both the default blocked path and an explicit `[safety] allow_writes = true` path
- There is no checked-in CI workflow; the local commands above are the completion bar
