# Forever current combat-event getter publication

Forever publishes the existing temporary combat-history fixture getter under its documented namespaces, not `C_CombatLog` or the deprecated global. The producer remains in `src/lua_api/workarounds/temporary/combat_log_state.rs` because it is a fixture, not a modeled combat-history subsystem. See the [addon comparison investigation](../wiki/investigations/forever-addon-comparison.md).

## What it must do

- [x] A fresh Forever environment exposes neither `C_CombatLog.GetCurrentEventInfo` nor `CombatLogGetCurrentEventInfo`.
- [x] Loading the unchanged `Blizzard_DeprecatedCombatLog` publisher with `loadDeprecationFallbacks` enabled does not create either missing getter; reapplying simulator bootstrap does not resurrect them.
- [x] `C_CombatLogInternal.GetCurrentEventInfo` and `C_CombatLogSecure.GetCurrentEventInfo` read the same existing concrete fixture entries, follow its current-entry navigation, and return nil after clearing entries.
- [x] EpicDamageMeter's unmodified capability predicate selects the meter API on Forever.
- [x] Other profiles retain their existing public getter and deprecated-global compatibility behavior.

## How it works

- [Cached addon comparison](../forever-addon-comparison.md) — investigation scope and evidence boundaries.
- Publication is selected from the compiled client profile before executing the temporary fixture's producer. No post-vendor global deletion or addon flag override occurs.
- The existing generic namespace synthesizer honors the C API's `__wow_removed_keys` declaration, so reading the deliberately absent public member cannot manufacture a replacement function.

## Implementation inventory

- `src/c_api/c_combat_log.rs` — documented public-member absence, preventing generic namespace synthesis.
- `src/c_api/mod.rs` and `src/c_api/registration.rs` — Forever-only publication-policy registration.
- `src/lua_api/workarounds/temporary/combat_log_state.rs` — profile-specific fixture getter publication and legacy aliases.
- `tests/c_namespace_noop_replacements.rs` — direct, actual deprecated-publisher, capability-selection, and shared-fixture regressions.

## Tests asserting this spec

Existing grouped `integration` target:

- `forever_combat_namespace::direct_environment_selects_meter_without_legacy_getter`
- `forever_combat_namespace::deprecated_publisher_does_not_restore_legacy_getter`
- `forever_combat_namespace::documented_getters_share_concrete_fixture_entries`
- Existing `combat_log_globals_have_stable_stub_behavior` and `combat_log_namespaces_iterate_seeded_entries_and_messages` retain profile-specific expectations.

The existing unit `installs_shared_combat_log_state_and_navigation` uses the getter appropriate to its profile.

## Evidence

Pinned Forever 1.60.1.69913 source cache:

- `Blizzard_APIDocumentationGenerated/CombatLogDocumentation.lua`: public `C_CombatLog` does not declare the getter.
- `CombatLogInternalDocumentation.lua`: `C_CombatLogInternal.GetCurrentEventInfo`, environment `All`.
- `CombatLogSecureDocumentation.lua`: `C_CombatLogSecure.GetCurrentEventInfo`, environment `SecureOnly`.
- `Blizzard_DeprecatedCombatLog/Deprecated_CombatLog.lua:18`: the global merely aliases the public member; it does not manufacture an implementation when the member is nil.
- `Blizzard_CombatLogProcessor/Blizzard_CombatLogProcessor.lua:105`: authored consumer of the secure getter.

Cached EpicDamageMeter file `8930362`, SHA-256 `46e66770c867f57a51de4726f858bfc6b88dfa2e7935312023d80e494700ab22`, `Core/Constants.lua`: checks the legacy getter's absence before selecting `C_DamageMeter` on non-Retail clients. This is the consumer motivation, not authority for removing other APIs.

## Development proof

At `9862dc7b3`, the three new Forever tests pass, as do the two existing shared combat-log tests under both Forever and Retail (7 executions total). Initial RED was 0/3. Moving the fixture getter alone reached 1/3: the generic namespace synthesizer still invented the missing public member. Declaring that member absent at C API registration completed GREEN 3/3.

Retail is the executed representative of the unchanged non-Forever publication branch; other non-Forever profiles were not separately run. The existing unit navigation test was updated for the profile-specific getter but not executed in this slice. A cancelled diagnostic has no proof credit. Commands, revisions, and full logs: `/tmp/forever-combat-namespace-development-ledger.json`.

## Known gaps (current cycle)

- [ ] Parent-owned full-addon reproduction and final verification remain separate.

## Out of scope

- Real combat history, combat-event production, payload secrecy, and enforcement of secure-only namespace visibility. Existing fixture limitations and namespace exposure remain explicit; publishing the secure getter does not prove security enforcement.
- Removing other combat-log compatibility methods, changing other profiles' API contracts, vendor/addon edits, or forcing EpicDamageMeter flags.

Retirement: replace this fixture publisher when a modeled combat-history subsystem owns these documented namespaces; retain the publication and consumer-selection regressions.
