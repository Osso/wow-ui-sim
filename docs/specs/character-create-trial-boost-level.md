# Character-create trial boost level

This contract defines the `C_CharacterCreation.GetTrialBoostStartingLevel` result used by the character-create class-trial tooltip. The state model and profile registration are described in the [system page](../wiki/systems/character-create-trial-boost-level.md).

## What it must do

### Registration and result
- [ ] Retail and WowForever expose the getter on Glue screens and not on Game screens. **VERIFIED:** R61 finds the Glue catalog registration in both profiles. **INFERRED:** the getter is Glue-only because the checked Game catalogs do not install that namespace.
- [ ] The getter reads no Lua arguments and returns one numeric value on the ordinary path. **VERIFIED:** R61 reads no arguments; **INFERRED:** an ordinary successful call publishes an integer-valued number. Extra arguments are not asserted.
- [ ] **VERIFIED:** the boost-type selector starts at `2`. The host setter always stores the supplied integer and emits no event.
- [ ] **VERIFIED:** the getter returns the `BoostLevel` from the first matching row in table order: `ServiceType == 1` and `BoostType == selector`, or the `ServiceType == 0`, `BoostType == 0` row when the selector is zero. No row or no match returns `0`.

### Profile data
- [ ] **VERIFIED (local export):** Retail uses the `CharacterServiceInfo` table from installed build `12.1.5.70783`. It contains 13 rows, no unreadable records, and one selector-2 level-boost row with level `100`. **VERIFIED (separate R61 observation):** Retail `12.1.5.70688` has the same matching level; the installed export remains labeled `70783`.
- **VERIFIED (exporter and CSV):** the exporter writes records in ID order; the generated Rust rows preserve that CSV order. All 13 `(ServiceType, BoostType)` pairs are unique, so current exported lookups do not depend on row order. Future duplicate keys must use the host's actual table iteration order.
- [ ] WowForever ships no `CharacterServiceInfo` rows. **VERIFIED:** the checked Forever payload is encrypted and its key is missing. Forever's native table values remain **UNKNOWN**; the empty host table makes the getter return `0` by the verified no-match rule.
- [ ] PTR keeps its existing zero fallback. PTR native availability and native result behavior remain **UNKNOWN**.

### Original consumer
- [ ] The Retail class-trial tooltip at `Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:1454` receives the exported level. The Forever consumer is `Blizzard_CharacterCreate/Camelot/Blizzard_CharacterCreate.lua:1585`. No declaration was found in the pinned `Blizzard_APIDocumentationGenerated` cache files; the numeric type follows from these consumer call sites.
- [ ] The three Forever Glue character-create consumers proceed past `GetTrialBoostStartingLevel`; later failures in the separately owned `IsNameReservationOnly` path do not change this contract.

## How it works
- [Character-create trial boost level](../wiki/systems/character-create-trial-boost-level.md)

## Implementation inventory
- `src/c_api/c_character_creation_trial_boost.rs` — state-backed getter and Glue registration.
- `src/c_api/c_character_creation_trial_boost_data.rs` — generated Retail row tuples.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — per-environment backing state.
- `src/lua_api/env_runtime.rs` — refreshes screen-scoped registration when the screen changes.
- `src/lua_api/workarounds/temporary/character_create_defaults.rs` — retains the PTR-only fallback.
- `tools/gen_character_creation_trial_boost.py` — regenerates the Rust row tuples from the checked export.
- `data/character-service-info/` — Retail CSV, scanned DBD definition and provenance.

## Tests asserting this spec
- `src/c_api/c_character_creation_trial_boost_tests.rs`
- `tests/glue_character_select.rs`
- Existing Forever consumer tests in `tests/glue_character_select.rs`: `character_create_action_updates_player_name_without_lua_errors`, `character_create_screen_can_boot_directly`, and `character_select_can_transition_to_character_create_without_lua_errors`.

## Known gaps (current cycle)
- [ ] Forever native `CharacterServiceInfo` rows are unknown because the checked local payload is encrypted.
- [ ] PTR native getter availability and result behavior are unknown; the existing simulator fallback remains unchanged.

## Out of scope
- The separately owned `IsNameReservationOnly` implementation and `C_Login`.
- Native PTR availability or a Forever row value inferred from Retail data.
