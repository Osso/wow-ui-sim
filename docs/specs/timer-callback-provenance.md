# Timer callback timing provenance

Opt-in simulator diagnostics identify original timer callbacks rather than only the simulator's container invoker. Source: `src/lua_api/handler_timing.rs`, `timer_layout.rs`, `env_runtime.rs` and `workarounds/temporary/proxy_object_factories.rs`. [Timer system](../wiki/systems/lua-api.md#timer-system) describes capture and lifetime management.

## What it must do

- [x] With `WOW_SIM_LOG_HANDLER_TIMINGS` enabled before process initialization, capture the original callback's `source:linedefined` once per registration, including container-wrapped After/NewTimer/NewTicker callbacks; repeating invocations reuse that identity.
- [x] Emit `[handler-entry]` on stderr before each callback, regardless of duration threshold. Include `addon`, `handler=C_Timer`, `frame=#0`, existing invoker `source` when known, `timer_id` and original `callback_source`. Missing Lua source is `<unknown>`. With diagnostics enabled every wrapper is associated before native registration. An absent association key denotes a direct, unwrapped registration (for example TimedSignalMap), whose own source is authoritative, not a fallback. A present non-function association or invalid registry table produces `callback_source=<invalid-association>`; diagnostics never change invocation.
- [x] Preserve existing thresholded `[handler]` return records and field meanings (`addon`, `handler`, `frame`, `duration_ms`, optional invoker `source`); append matching `timer_id`, `callback_source` and `outcome=ok|error`. Callback failures retain the existing error-handler path. A missing return identifies an entered callback, not its failure cause.
- [x] With the variable unset, emit no new records and bypass provenance capture, metadata lookup and formatting. Preserve callback arguments, container/proxy identity, taint, cancellation, errors and scheduling in both modes. Ownership/source-capture frequency and unchanged invocation blocks are source-audited; bounded behavioral coverage is listed below. Diagnostic tracking must not extend callback lifetimes.
- [x] Retail and WoW Forever register modeled FunctionContainers directly in Rust. NewTicker returns the same container, keeps per-registration iteration counts, and cancels every timer backed by that container. Plain functions become containers; other profiles retain the Lua adapter. NewTimer container acceptance and same-object return are INFERRED (R88y; native evidence pending).
- [x] INFERRED (R88y; native evidence pending): After accepts a FunctionContainer, invokes it with no arguments, and Cancel suppresses its callback while the queued one-shot remains until due. This preserves the former adapter behavior; the 12.0.7 probe did not test After.

## How it works

- [Timer system and diagnostic storage](../wiki/systems/lua-api.md#timer-system)
- [Related file-budget diagnostics](startup-file-instruction-diagnostics.md) — same opt-in, separate records and attribution scope.

## Implementation inventory

- `src/lua_api/workarounds/temporary/proxy_object_factories.rs`: FunctionContainer factory plus the legacy C_Timer adapter outside Retail and WoW Forever; adapter provenance association remains weak-key/weak-value (`__mode="kv"`).
- `src/lua_api/timer_layout.rs`: Retail/Forever callback-container normalization, backing callback records, timer-handle binding, and registration-time provenance capture; retire metadata with the stored callback.
- `src/lua_api/timer_dispatch.rs`: invoke stored container callbacks directly from their captured function while honoring container cancellation.
- `src/lua_api/state{,/sim_state}.rs`: per-environment timer-ID-to-source strings; no callback references.
- `src/lua_api/handler_timing.rs`: source capture and additive entry/return formatting; existing threshold parser unchanged.
- `src/lua_api/env_runtime.rs`: emit entry before invoking and return after existing error handling; shared TimedSignalMap firing retains its direct callback source.

## Tests asserting this spec

- `tests/timer_callback_provenance.rs` and `tests/fixtures/TimerProvenance/`: subprocess-captured stderr for After success/error, entry-before-body ordering, matching return ID/source, disabled gate, high threshold and repeated container-backed ticker. Fixture asserts zero After arguments, ticker/container identity, addon stack taint and unchanged error delivery. The GC regression covers the legacy adapter's weak wrapper association outside Retail/Forever. On Retail/Forever, it verifies a completed container callback cycle is collectible after the known backing-map callback edge is cleared; it does not prove unrelated container cycles collect.
- `registration_and_invocation_contracts_hold_in_both_diagnostic_modes`: subprocess gate-on/off After/NewTimer/NewTicker/direct TimedSignalMap matrix with cancellation, rescheduling, proxy/container identity, arguments, taint and error delivery. Mutating diagnostic associations after registration must not change captured source labels or invocation; ticker repeats reuse the registration snapshot. This observes snapshot stability, not internal capture counts or absence of gate-off work.
- `handler_timing::tests::missing_source_and_invalid_registry_states_are_explicit`: native callback unknown-source serialization, missing/non-table registry invalid markers, direct absent-key source and unchanged callback results. Execution acceptance is separate from this test inventory.
- `tests/timer_callback_container.rs`, `tests/timed_signal_map.rs` and `src/lua_api/handler_timing.rs::tests`: existing invocation/lifecycle/security and timing-format preservation controls. `tests/group_loot_timer_container.rs`: original GroupLootFrame event and hide/show lifecycle cancellation on Retail and WoW Forever. Resolver unit regressions distinguish a present numeric association (explicit invalid marker) from an absent key (the direct callback's own source). Normal public registration cannot create a numeric association; the unit fixture injects that invalid registry state directly.
- RED at `245f55610`: compile exit0; 1 PASS / 3 FAIL, missing original entry records, existing invoker-only sources `=(string):273/284`. Evidence: private `handoff/timer-provenance/red-retail-20261018T075507Z`.

## Bounded proof — 2026-10-18

Private evidence root: `handoff/timer-provenance/` (private handoff store, not tracked).

| Scope | Exact source and receipt | Result |
|---|---|---|
| Retail runtime | `52d074234`, `green-retail-20261018T081650Z` | 4 provenance + 5 container + 16 TimedSignalMap + 7 existing timing cases = **32 PASS**, exit0 |
| Forever runtime | `30b1c32c6`, `green-wowforever-20261018T084501Z` | Same **32 PASS**, exit0 |
| Retail check | `30b1c32c6`, `check-retail-20261018T084242Z` | exit0, no compiler messages |
| Forever check | `30b1c32c6`, `check-wowforever-20261018T090954Z` | exit0; existing `UnitRole` dead-code warning |
| Formatting | `30b1c32c6`, `fmt-check-30b.txt` | `cargo fmt --check`, exit0 |

All compilation uses `--no-default-features --features sound,gui,casc,client-<profile> -j1`, async user units with MemoryMax9G and slot0 after capacity ADD. Executables stay in `deps` (normal test CVar isolation); no standalone-copy runtime was launched. `30b1c32c6` removes only an import made unused by this feature; Retail behavior is unchanged from `52d074234`. Its old lib/lib-test warnings are fixed, not suppressed. Forever's unrelated `UnitRole` warning remains.

Final rebase onto `35822d5c7` completed without conflict. All eleven owning diagnostic/fixture/container/TimedSignalMap source hashes match the pre-rebase source (`pre-rebase-source-hashes.json`, `post-rebase-source-hashes.json`). Rebased feature commits are `aca5c8be6`, `705d5051a`, `e8ebcf711`; upstream Exec-capture/UnitExists changes are outside these fixtures' timer paths. Saved runtime receipts are pre-rebase evidence, not a newly executed whole-HEAD gate. No broad suites were repeated.

## Additional bounded proof — 2026-10-18

Source `f8afe3b3f9e23082aa7c44acef9feafb05ff4f59`, based on `29fac7076965dd53973168c47d972e4090f27b98`, uses one Retail lib+integration build with `--no-default-features --features sound,gui,casc,client-retail`:

| Exact selector | Executed result |
|---|---|
| `timer_callback_provenance::registration_and_invocation_contracts_hold_in_both_diagnostic_modes` | 1 PASS; same fixture in isolated disabled/enabled processes |
| `lua_api::handler_timing::tests::missing_source_and_invalid_registry_states_are_explicit` | 1 PASS; native unknown source, nil/non-table registry and direct callback controls |

Private-store-relative receipts `handoff/a1-timer-unit-tests/build-03-B3.json` and `build-03-B4.json` bind exact argv, revision, emitted features and executable SHA-256. Matching `cargo fmt --check` and Retail `cargo check --lib` passed. Earlier fixture failures were private-method access and an incorrect eval-chunk line oracle, not product divergences; only tests changed. Snapshot stability is executed proof, not a measurement of capture frequency or absence of gate-off work. No Forever rerun or broader scheduling guarantee is claimed.

## Known gaps (current cycle)

Diagnostic-map cycle collection has a dedicated GC regression; once-per-registration capture remains source-audited (string-only metadata and no per-tick closure lookup), without a source-lookup-counter regression. Pre-existing function-container backing-map ownership is outside the diagnostic-map fix. Verifier-fix revisions and command receipts are recorded in private `handoff/timer-provenance/fix-report.md` and appended to `proof-ledger.json`; the historical proof above is unchanged. No full startup, GUI, native parity or full-suite acceptance is claimed.

## Out of scope

Registration caller stacks, callback arguments/payloads/SavedVariables, per-line cost, vendor/addon edits, timer policy changes and diagnosis of the earlier 95-second callback. Source identities can contain private paths. No additional protected call, retry or error suppression is introduced.

## Original AsyncRequest correlation — bounded Retail proof

- [x] Two ordinary fixture addons register four original AsyncRequest timers. Timing-off/on children both report four requests, two responses, two once-only timeouts, no cancelled timeouts and zero callback errors. Cancelled deadlines are advanced before two extra drains.
- [x] Enabled records bind live public handle IDs to registering owners and original `@Interface/AddOns/Blizzard_AsyncRequest/Blizzard_AsyncRequest.lua:60`, with entry/body/return ordering. Cancelled IDs emit no invocation records; timing-off emits no provenance fields.
- [x] Wrong response payload and doubled fixture timeout increments are rejected in each timing mode. A disposable simulator diagnostic-table mutation associates each wrapper with itself: ordinary callbacks still pass, but enabled original-source correlation rejects wrapper provenance.

`tests/timer_async_request_provenance.rs` loads the original TOC directly in an isolated environment and rejects consumer drift using SHA-256 `1648c842902f6657ae1e10de7366cd0650c77a5eb41bc1c9c7e8030256fa8acb`. `tests/fixtures/AsyncRequestProvenance/Requests.lua` is supplied test data, not a replacement consumer. The panel smoke harness was rejected for this zero-error proof because its unrelated TimeManager listener fails during PLAYER_LOGIN; no error was suppressed and no timer producer changed. Three unchanged ordinary timeout/response/concurrent controls also pass. Proof covers source `5f5aa4563`, not native parity, full startup, other profiles or capture frequency.
