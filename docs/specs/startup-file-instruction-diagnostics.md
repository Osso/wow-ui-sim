# Startup file instruction diagnostics

Simulator diagnostics expose successful tainted Lua-file owner-budget snapshots without changing execution or quota policy. Source: `src/loader/lua_file.rs` and `src/lua_api/execution_budget.rs`; [loading architecture](../addon-loading-pipeline.md#top-level-file-budget-diagnostics) describes the path. This is diagnostic attribution, not a startup-error repair or native quota contract.

## What it must do

- [x] Under `retail-12-0-5` and existing `WOW_SIM_LOG_HANDLER_TIMINGS` opt-in, emit one `[file-budget-success]` record per successfully executed tainted top-level Lua chunk; preserve existing `[file-budget-error]` records for failed execution. Duration threshold does not filter these records.
- [x] Report debug-escaped owner/chunk identities, limit and cumulative `used_before`/`used_after`; label unavailable snapshots `unavailable` and unlimited limit `none`. No arguments, payloads or error text.
- [x] Preserve successful return values, private-table effects and cumulative owner-meter continuity between files: the same concrete sequential loader fixture passed with instrumentation opt-in on and off.
- [x] Preserve tested sequential owner limits, error propagation and opt-in-off behavior. Broader quota initialization/reset/exemption preservation and excluded untainted/compilation/setup/Rust-work boundaries have unchanged-source audit evidence, not comprehensive runtime re-execution.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md#top-level-file-budget-diagnostics)
- [Startup evidence boundaries](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md)

## Implementation inventory

Source audit of `0e4bf4161` (2026-10-10; not runtime proof): environment presence opts in via a once-cached Unicode environment read; non-Unicode values count as absent. The value filters duration records only. Snapshots bracket `exec_addon_func` after owner-meter initialization, without resetting existing usage. Their difference is cumulative owner charges during the invocation's dynamic scope, including synchronous nested Lua charged to that owner, not instructions charged to other owners. Nested records can overlap; summing them is not exclusive attribution.

- `src/loader/lua_file.rs`: existing budgeted execution path selects success/error diagnostics without changing the result. Non-quota execution failures also emit `[file-budget-error]`; existing error conversion/contextualization remains in place. Untainted execution and pre-call taint/secure-environment/loading-environment setup failures do not enter this diagnostic path.
- `src/lua_api/execution_budget.rs`: success formatter reuses the existing counter encoding; `limit` comes from the before snapshot, usage from each respective snapshot. Debug escaping protects record structure, not privacy: owner/chunk metadata remains visible. `wow_chunk_name` normalizes paths containing `AddOns/`; other paths can retain an absolute path. No source text, arguments, payloads or error text is added to these records; other stderr/error reporting is not sanitized by this formatter.

## Tests asserting this spec

- `loader::lua_file::tests::startup_file_budget_success_preserves_meter_continuity_returns_and_effects`: returns19/42, final private total42, nested call and meter continuity.
- Adjacent original error/exhaustion loader test and `execution_budget::tests`: preserved failure semantics, escaped metadata, unavailable/unlimited counters.
- Actual GREEN receipts at epoch `20261010T183607Z`, independently audited for `0e4bf4161d58cbbf6572a5be08cac7f5019e316f`: 12 passing executions / 11 distinct tests across four subprocess invocations (the same success loader fixture runs opt-in on and off). Opt-in1000 emits exactly two success records, limit1000, cumulative counters0→4→16; opt-in off emits no file-budget records. Original error fixture: 1PASS, one prior success0→2 and exactly two error records2→100 and100→100, limit100. Nine formatter/error controls pass, covering escaped metadata, unavailable snapshots and unlimited meters.
- Independent receipt audit: fmt/default-check exit0; compile exit0; source and invoked artifact sealed and matched. Six vendor `iced_wgpu` manifest deprecation warnings remain; this is not warning-free proof. Preservation of quota rules and excluded logging boundaries is source-audit evidence, not comprehensive runtime quota-policy coverage.
- Evidence: private actual receipt `/home/osso/.local/state/wow-ui-sim/verification/success-file-budget-green-current/independent-report.md`, epoch `20261010T183607Z`. [Tracked independent report](../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/success-file-budget-green/independent-report.md) and [receipt hashes](../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/success-file-budget-green/retention-manifest.json) retain the bounded evidence. Provenance is bounded local source/receipt binding, not hermetic attestation or full-addon attribution.

## Known gaps (current cycle)

- [x] Execute current success/error/formatter controls and independent fmt/default-check proof: actual GREEN receipt above covers this bounded diagnostic gate.
- [x] Capture full-addon startup and verify exact file-record attribution: epoch `20261010T183739Z`, submission `b048251f38b2d53e33b3365590cd227d40fc07d0` with diagnostic code `0e4bf4161`, binds source/artifact and 4,044 vendor-cache paths. [Independent report](../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/success-file-budget-full-addon/independent-report.md): 3,064 file-success, four file-error and 91 handler-error records; exact private file joins across 50 owners, zero rejected records and zero observed continuity gaps. Gate covers bounded capture/parser attribution, not sealed third-party inputs or exclusive file costs. Historical105 actual error occurrences remain distinct from current95 diagnostic error records; no count-reduction or startup-fix claim.

## Out of scope

Quota changes, vendor optimization, per-line profiling, exclusive/self instruction costs, elapsed-time/Rust-work attribution and native quota/reset parity. Logged identities can contain paths; raw full-addon output remains private.
