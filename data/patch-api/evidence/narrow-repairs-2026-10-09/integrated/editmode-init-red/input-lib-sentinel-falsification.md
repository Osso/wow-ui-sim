# Library sentinel falsification — `dd710c6fc469a53e3b77b97aa6340993ae83e46e`

**Evidence:** saved log and JSON under `/home/osso/Projects/wow/full-suite-results/`; exact suite SHA `dd710c6fc469a53e3b77b97aa6340993ae83e46e`, checkout `/home/osso/Projects/wow/full-suite-checkout`. Also reviewed `/tmp/fail-layout-lib-map.md`, relevant source/tests/specs in that recorded checkout. No tests run, edits to repo, network, or delegation.

## `debug_environment_defaults`

- **Failure:** panic at `src/lua_api/workarounds/temporary/debug_environment_defaults.rs:240:14`; Lua `(string):7: attempt to call a userdata value`.
- **Exact line/control flow:** raw probe starts with `local marker...` on line 1. Line 7 is `if GetErrorCallstackHeight() ~= 0 then return "error_callstack_height" end`. `CreateSecureDelegate(marker)()`, both metatable probes, `secretwrap(marker)()`, and `GetCallstackHeight()` all passed. Later `debugstack`/`debuglocals` checks were not reached.
- **Contract:** aggregate default-install test does not isolate which global/value backs `GetErrorCallstackHeight`. No repo spec establishes this debug-height contract. `tests/security_api.rs::test_securecallfunction*` verifies ordinary `securecallfunction` calls/returns, unrelated to this failing getter. Do not misdiagnose as secure-delegate or stack-format failure.
- **Minimal proposal:** inspect the published `GetErrorCallstackHeight` binding against its intended modeled callable/default; if it is intended callable, fix that binding; otherwise retire this unsupported assertion. Do not expand unrelated debug defaults or claim native parity.

## `housing_catalog_state`

- **Failure:** assertion at `src/lua_api/workarounds/temporary/housing_catalog_state.rs:60:9`, actual sentinel `"bad_searcher"` (expected `"ok"`). This sentinel follows the test’s `CreateCatalogSearcher()` check: either its result was not a table or `GetSearchCount()` was zero. Saved output does not distinguish those branches.
- **Contract comparison:** recorded checkout now constructs the searcher in `src/c_api/c_housing/catalog/searcher.lua`; rows come from host-backed variants via `queries.rs::search_items`. `docs/specs/housing-catalog-variants.md` says catalog starts empty absent explicit host input. It also records native wording that `GetSearchCount` counts owned instances, not result rows; row-length behavior is expressly not adopted. Thus `GetSearchCount()>0` is neither a valid empty-default assertion nor justified by adding fake product rows. Earlier checks in the same composite probe did not return their `bad_*` sentinels, but that does not validate their invented product/decor values as native data.
- **Minimal proposal:** remove the seeded-surface expectation. Test empty defaults as empty; test populated observable source/results only with explicit host entry/variant fixtures using `GetAllSearchItems` and `GetCatalogSearchResults`. Do not create runtime fallback seeds or assert row count via `GetSearchCount`.

## Three `apply_system_anchors` failures

All three fail at the same precondition, before their named behavior: saved error `(string):8: attempt to call method 'InitSystemAnchors' (a nil value)` during `env.exec(APPLY_SYSTEM_ANCHORS_LUA)`. Production `src/lua_api/workarounds/editmode/apply_system_anchors.lua` calls `emm:InitSystemAnchors()` before per-frame replay. Each fabricated manager omits that method.

| Failing test | Fixture and unreachable assertions | Falsification |
|---|---|---|
| `cast_and_player.rs:278-407` `...player_frame_size_without_cast_bar_side_effect` | Manager at 370–374 lacks `InitSystemAnchors`; panic at 380. Converted scale, saved `BOTTOM` geometry, one base SetPoint, zero `ApplySystemAnchor` calls never observed. | Test’s geometry/side-effect outcomes are observable; internal call count alone is not a native contract. `docs/specs/edit-mode-initial-anchors.md` does require initial anchors before replay/callbacks. This run proves neither outcome. |
| `singletons.rs:123-214` `...falls_back_to_minus_one_for_nil_singletons` | Manager at 157–161 lacks method; panic at 193. Getter requests and setting application never run. | The asserted `nil,-1` retry is fixture-invented here; no cited spec/native evidence grounds nil-to-minus-one. Separate nil-index-to-saved-index test does not prove this fallback. |
| `unit_frames.rs:291-467` `...batches_compact_unit_frame_startup_refreshes` | Manager at 395–403 lacks method; panic at 437. Exact callback-count summary never runs. | No evidence about refresh correctness. Exact batching/counts are implementation shape, not a demonstrated native contract. |

**Minimal proposal:** repair the test fixture boundary by supplying the required `InitSystemAnchors` method, then assess actual observable postconditions. Keep player geometry/no-cast-bar effects as outcomes; do not promote call counts to contract. Do not preserve nil-to-minus-one fallback or exact compact callback counts without separate literal model/spec/native evidence. No runtime patch is justified by these failures.

## Limit

These are saved failures at the pinned SHA, not results against current checkout. Housing sentinel does not isolate its two conditions; EditMode subject logic was not reached. No pass/fix claim.
