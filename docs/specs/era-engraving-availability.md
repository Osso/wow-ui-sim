# Era and Anniversary engraving availability

Era and Anniversary expose engraving availability through `src/c_api/c_engraving.rs`. The user decided that the default stock non-season scenario is disabled; native behavior remains unverified. See [the investigation](../wiki/investigations/era-engraving-availability.md) for original consumer selection and proof limits.

## What it must do

- [x] Publish `C_Engraving.IsEngravingEnabled` on Era and Anniversary through the shared C API registrar, selected through `client_profile::ACTIVE`.
- [x] Return one environment-local boolean: supplied availability takes precedence; an unsupplied value returns false for the user-decided stock non-season scenario.
- [x] Keep Season of Discovery as an explicit supplied scenario input, without inferring seasonal policy.
- [x] Drive the selected original `RuneFrameControlButton` XML-bound OnLoad with supplied true/false and the stock default, including transitions and independent environments.
- [x] Attempt the complete original settings closure as preservation and report its actual boundary, without claiming full-settings behavior acceptance.

## How it works

- [Selection, state and bounded evidence](../wiki/investigations/era-engraving-availability.md)
- [Shared publication and Rust input supply; no Lua admin setter](../wiki/investigations/mists-engraving-inputs.md#shared-availability-and-input-supply)

## Implementation inventory

- `src/c_api/c_engraving.rs` — shared availability reader; remaining six members stay Mists-only by the registrar's second profile gate.
- `src/c_api/engraving_inputs.rs` — existing environment-local `enabled: Option<bool>` input.
- `src/lua_api/workarounds/temporary/classic_startup_defaults.rs` — no engraving policy remains.

## Tests asserting this spec

- `tests/era_engraving.rs` — original XML OnLoad, supplied-query arity, stock default and settings preservation attempt.

## Known gaps (current cycle)

The earlier supplied-only implementation exposed an unset error and its ScriptErrors echo in both stock maps. The user-decided default replaces that unset error for Era and Anniversary. Stock and full-profile verification of this change is pending integration.

## Out of scope

Bags and full LoD EngravingUI are excluded. Native availability values remain unknown; the stock default is a user decision, not a native observation. No complete engraving lifecycle is claimed. Mists' existing seven-member model is retained; no other profile gains publication. The other25 declared members remain outside Era/Anniversary's modeled surface:

| Domain | Excluded members |
|---|---|
| Filters | AddCategoryFilter, AddExclusiveCategoryFilter, ClearAllCategoryFilters, ClearCategoryFilter, ClearExclusiveCategoryFilter, EnableEquippedFilter, GetExclusiveCategoryFilter, HasCategoryFilter, IsEquippedFilterEnabled, SetSearchFilter |
| Catalog | GetNumRunesKnown, GetRuneCategories, GetRunesForCategory, IsKnownRuneSpell, IsRuneEquipped |
| Equipment | GetRuneForEquipmentSlot, IsEquipmentSlotEngravable |
| Inventory | GetRuneForInventorySlot, IsInventorySlotEngravable, IsInventorySlotEngravableByCurrentRuneCast |
| Casting/mode/refresh | CastRune, GetCurrentRuneCast, GetEngravingModeEnabled, RefreshRunesList, SetEngravingModeEnabled |

## Pinned declaration

Both Era and Anniversary `Blizzard_APIDocumentationGenerated/EngravingInfoDocumentation.lua:179–186` independently declare the non-null bool query. Both select `Blizzard_CharacterFrame/Vanilla/PaperDollFrame.xml:716,722` and `PaperDollFrame.lua:950–954`; settings reads availability at `Classic/InterfaceOverrides.lua:136–138`.
