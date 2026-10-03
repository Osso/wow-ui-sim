# Delve instance state

Retail 12.0.5 `IsInInstance()` includes explicit active-Delve host state in its boolean result. [API change source](../../data/patch-api/sources/12.0.5-api-changes.txt), line 181 (`prose-2026-03-31-181`), says: “Fixed an issue where the IsInInstance API would not always return true when the player was in certain instances (mostly delves).” Reuse the existing [Delves host inputs](delves-api-inputs.md); do not infer membership from map arguments.

Cached retail `Blizzard_APIDocumentationGenerated/InstanceDocumentation.lua:170–177` declares no arguments and exactly two nonnil returns: boolean `isInInstance`, cstring `instanceType`. `DelvesUIDocumentation.lua:373–379` declares a no-argument boolean `HasActiveDelve`. Cached `Blizzard_FrameXML/InstanceDifficulty.lua:48–52` reads `HasActiveDelve` in `InstanceDifficultyMixin:IsInDelve`; this supports using the same host input, not a new unrelated Delve flag. Declarations and vendor consumers are not native execution proof.

## What it must do

- [ ] When `retail-12-0-5` is enabled, return true for an explicitly active Delve even if `world.in_instance` remains false; read host activation/deactivation live, without mutating either flag.
- [ ] INFERRED composition policy: return `world.in_instance || has_active_delve`; ending a Delve must not clear an independently active dungeon. Without the feature, preserve the existing world-only behavior (older profiles untested).
- [ ] Preserve the exact live `world.instance_type` string as the second result; do not manufacture a Delve type. An inconsistent host input with active Delve and type `none` consequently returns `(true, 'none')`; no native claim for that pair.
- [ ] Return exactly two public values of declared types, ignore all undeclared extra arguments including authentic secret wrappers, and preserve caller taint, wrapper secrecy, and host state.
- [ ] Keep instance observations isolated between environments, including after activation and deactivation.

## How it works

- [Lua API and shared host state](../lua-api.md)
- [Existing Delves host inputs](delves-api-inputs.md)

## Implementation inventory

- `src/lua_api/globals/real/combat_probes.rs` — registered non-C_* global producer; feature-gated participation of existing active-Delve input.
- `src/lua_api/state/sim_state.rs` — existing `has_active_delve` field, unchanged.
- `src/lua_api/state.rs` — existing false default, unchanged.
- `src/lua_api/state_types/character_world.rs` — existing world instance boolean/type fields, unchanged.

## Tests asserting this spec

`tests/delve_instance_state.rs`, auto-included into `integration`:

- `active_delve_entry_and_exit_drive_instance_flag_without_legacy_flag_changes` — registered Delve and instance queries agree across explicit host entry/exit; legacy flag remains false.
- `ending_delve_preserves_independently_active_dungeon` — inferred composition and unchanged legacy type read-through.
- `delve_query_preserves_live_host_type_and_does_not_repair_host_state` — concrete type replacement and repeated nonmutating reads.
- `delve_query_ignores_extra_arguments_and_preserves_secret_wrappers_and_taint` — public types/arity, malformed extras, authentic secret number/string extras, GC, actual tainted closure, secure-context recovery.
- `active_delve_instance_state_is_environment_local` — two independent environments and opposite transitions.

## Development proof and independent bounded acceptance — 2026-10-03

Commit `a4cce2db1`. RED: 1 PASS / 4 FAIL. GREEN: 5/5 inside a 404/404 run with control suites; `cargo fmt --check` exit0; startup `lua-errors` `[]`. This section supersedes any wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-events-commands.md) SHA256 `72c3dbad682b576a045792d498f279ba54e7ef7b6094d5364da3d8a5817b4ac3`. Composition with the world flag and the unchanged type string are inferred. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-03-31-181 bounded-coverage under capability `delve-instance-state`; **119 capabilities/362 IDs; 42 pending /262 bounded /23 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Staging-only authoring: tests, formatting, compilation, RED/GREEN and acceptance not run. All requirements remain unchecked.
- [ ] Native instance membership acquisition and native-client behavior of the inferred composition/type policy are not proven.

## Out of scope

Native map coverage, automatic world entry/exit acquisition, events, instance metadata normalization/catalogs, new admin APIs, existing Delves producers, older-profile proof and page accounting. Explicit host changes are test inputs, not evidence that native zone transitions are reproduced. This slice fixes the instance-query producer only; it adds no wrapped placeholders or alternate lookup fallback.
