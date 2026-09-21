# Mainline spellbook lifecycle across profiles

Retail-only panel fixtures did not protect profiles that consume the same Mainline `PlayerSpells` lifecycle. A Forever GUI run proved the gap: the first spellbook keypress loaded the panel, while the second failed in `SpellBookFrameMixin:OnHide()` because `InClickBindingMode` was missing.

## Root causes

The actual open → tick → close path exposed five independent simulator producers:

1. Forever startup excluded `[Bootstrap]` files from LoadOnDemand Blizzard addons, so `Blizzard_ClickBindingUI_Bootstrap.lua` never published `InClickBindingMode`.
2. `C_SpellBook.GetClassSkillLineInfo()` was absent although the modeled Paladin spellbook already had a class skill line.
3. `GetPetIcon()` was absent despite existing modeled pet state.
4. Forever omitted documented `Constants.TransmogOutfitDataConsts.CLEAR_TRANSMOG_OUTFIT_MANUAL_SPELL_ID` and `Enum.ActionBarSet` values.
5. The compact spell lookup omitted exact spell `1247917`, consumed for the clear-transmog category icon.

Commits `d06147537` and `389c3a2d5` fix those producers. No Blizzard Lua guards, handler suppression, generic unknown-spell fallback, or test error filtering was added.

## Regression shape

`tests/blizzard_player_spells_loads.rs` now uses the production startup-addon discovery and load kinds, including profile FrameXML, bootstrap-only publishers, `ADDON_LOADED`, startup events, and real key dispatch. It clears startup errors, presses `S`, runs ten update ticks, verifies the spellbook is shown, presses `S` again, runs ten more ticks, verifies closure, and requires zero collected Lua errors.

## Profile coverage

| Profile | Contract | Proof |
|---|---|---|
| Retail | Mainline PlayerSpells | PASS: 1/1 |
| PTR | Mainline PlayerSpells | PASS: 1/1 |
| Forever | Camelot overrides on Mainline PlayerSpells | PASS: 1/1 |
| Mists | Cata PlayerSpells implementation | Not covered by this contract; attempted reuse exposed existing Cata panel/startup failures, including missing `GetDropdown` and `SkipResetOnShow`. |
| Wrath | Legacy SpellBookFrame | Separate regression required. |
| Era | Legacy SpellBookFrame | Separate regression required. |
| Anniversary | Legacy SpellBookFrame | Separate regression required. |

Artifacts: `/tmp/mainline-spellbook-final-{retail,ptr,wowforever}.{stdout,stderr}`. The original Forever RED is `/tmp/forever-spellbook-red.{stdout,stderr}`.

## Sources

- [Mainline spellbook lifecycle spec](../../specs/mainline-spellbook-lifecycle.md)
- [Forever report](../../wowforever-1.60.1.md)
- [[client-profiles]]

## See Also

- [[forever-clean-startup]] — sustained Forever runtime acceptance
- [[client-profiles]] — profile routing and cache selection
