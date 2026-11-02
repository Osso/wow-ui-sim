# Regional unique names

Regional name policy projects explicit player inputs. The [consumer investigation](../wiki/investigations/ptr-regional-names.md) records source selection and sequential failure boundaries.

## What it must do

- [x] Forever and PTR publish `RegionalUniqueNamesEnabled()` with one boolean from `PlayerState.regional_unique_names_enabled`; other profiles do not gain the global.
- [x] The regional flag defaults to false as explicit simulator policy, not an inferred native default. Changes apply immediately and environments remain independent.
- [x] PTR publishes `C_PlayerInfo.ShouldDisplaySurname()` with one boolean from supplied `PlayerState.should_display_surname`. Unsupplied input raises an explicit error; it does not guess native policy.
- [x] The selected original PTR `UnitFrame_Update` displays the whole local name when regional names are disabled or surnames are enabled, and the first name when regional names are enabled and surnames are disabled. Both policy transitions and nonplayer full names are preserved.
- [x] PTR/Forever graveyard-port behavior remains unchanged. [Explicit imported
  identity](character-identity.md) can additionally supply a separate Forever
  surname; that contract records its original-consumer coverage and remaining
  regional first-name limitation.

## How it works

- [Original consumer and declaration evidence](../wiki/investigations/ptr-regional-names.md).

## Implementation inventory

- `src/lua_api/state_types/character_world.rs`: player policy inputs.
- `src/lua_api/globals/real/player_identity.rs`: regional-name global and independent graveyard query.
- `src/c_api/c_player_info_name_policy.rs`: supplied surname query.
- `src/c_api/registration.rs`: PTR-only surname publication.

## Tests asserting this spec

- `tests/ptr_regional_names.rs`: runtime-selected original UnitFrame, actual names, both policy transitions, unknown-policy errors, and environment isolation.
- `tests/wowforever_name_policy.rs`: default/arity, original Forever NameUtil transitions, isolation, and excluded-profile publication.
- `tests/ptr_i4_green1.rs`: supplied graveyard-port projection and original PTR GhostFrame.

## Known gaps (current cycle)

The bounded contract passes sequential RED/GREEN and preservation. PTR stock improves from 296/410 to 272/348; Forever 70532 remains exactly 26/26. Retail/PTR/Forever all-target checks pass within their zero/known-three warning limits. Native policy remains outside this supplied-input contract.

## Out of scope

Native default policy, native surname separators, Admin inputs, character creation, and unrelated GameRules, RecentAllies and housing behavior are not inferred or modified. The tests supply a separator fixture; they do not claim native separator publication. Forever's earlier declaration evidence is 1.60.1.69913 UnitDocumentation.lua; preservation runs on the pinned 70532 cache.
