# Forever 70707 addon startup compatibility

A Forever 70707 game startup with SavedVariables enabled reached the first
frame and exposed missing queries and duplicate secure state dispatch.

## Content

### Cooldown Viewer in Edit Mode

The Game Menu's Edit Mode button refreshes the Forever Cooldown Viewer.
`CooldownViewerItemData.lua` enumerates totem slots with `GetNumTotemSlots()`.
Forever now reads that count from the same slot state as `GetTotemInfo()`.
Active slots include the supplied spell ID and rate. When two totems share a
spell ID, the Blizzard cache keeps the slot with the latest expiration.

The next refresh step scans legacy talent slots for icons. Forever exposes
`C_SpecializationInfo.GetTalentInfo()` over a
[supplied character grid](../systems/forever-talents.md). The default snapshot
has no rows, so the icon provider skips them; supplied rows produce WM90 talent
records. No populated Forever database export is assumed.

The Edit Mode regression enters through the Game Menu and checks the Blizzard
cache with populated totem slots. In the Windows 70707 check, Edit Mode became
active, the default slot count returned four, and the `lua-errors` report was
empty.

### Startup failures

- AuraContainer needed the shared caster, carryover and enchantment queries.
  RXPGuides needed the predicate for Forever's public aura policy.
  [Carryover](../../specs/aura-container-carryover-private-caster.md) ·
  [Weapon enchantments](../../specs/forever-weapon-enchants.md#temporary-enchantment-display-query) ·
  [Aura secrecy](../../specs/spell-aura-secrecy.md#current-aura-output-predicate)
- In the startup capture, a missing surname separator interrupted
  `CompactRaidFrameManager` before its divider pool initialized. The focused
  test covers the name formatter. [Name formatter test](../../../tests/wowforever_character_name_constants.rs)
- The partial `C_Widget` namespace bypassed MoveAny's fallback. AuraContainer
  also needed registered subtypes to report `Animation` ancestry.
  [Widget recognition](../../specs/widget-recognition.md) ·
  [Animation ancestry](../../specs/animation-factory.md#animation-ancestry)
- The secure setter replayed Blizzard's state snippet with a raw frame after
  the restricted callback ran, breaking `RunAttribute` and frame references.
  [State snippet ownership](../../specs/secure-attribute-delegation.md#state-snippet-ownership)
- The gamepad action bar needed pending pet orders, LockInTools needed local
  channel membership, and EllesmereUIRaidFrames needed Blizzard's encounter
  wrapper to read live state.

### World map after startup

On Forever 70707, the World Map key binding reached Blizzard's map and quest
panels. Those panels called queries registered for Retail or Retail and PTR.
The missing zone-story query interrupted `WorldMapMixin:OnShow` before map
selection and zoom reset, leaving `targetScale` nil for `IsZoomingOut()`.

Forever now exposes bounty, threat, and outdoor PvP queries over existing
simulator state. The map and Settings share an in-memory Trivial Quests value
that defaults to off. Campaign-header updates and zone-story queries use empty
results while their backing data is unavailable.

Quest titles read the supplied elite tag and use the existing quest difficulty
model. The model's level bands for Forever are inferred. Party quest counts
use supplied party quest membership, and world quest pins use the supplied
short-expiration flag to select a countdown threshold.

The map regression uses the `M` binding and checks map selection, zoom, and
Lua errors across 60 updates. A Windows 70707 capture used `ToggleWorldMap()`
with populated session data. It selected map 2248, reported a positive zoom
target, and showed the quest list and pins. The capture reported terrain
texture lookup failures.

### Tooltip baseline fixtures and producers

The Camelot character panel uses mode tabs, not Mainline `numTabs`. Tooltip
fixtures now enter through its original `ToggleCharacter` and load the complete
Forever startup inventory, including `Blizzard_Statistics`. This removes two
fixture errors without editing vendor Lua or assertions. Original item-button
and paper-doll consumers then exposed missing publication of the existing
[crafted-quality map](../../specs/crafted-item-quality-info.md#forever-publication)
and [Azerite location query](../../specs/retail-corrupted-azerite-items.md).
The five character-slot hover/render cases pass after those publications.

Forever spellbook fixtures explicitly supply an empty outfit list and unavailable
transmog. Both inputs are required by the existing strict
[outfit](../../specs/forever-camelot-spellbook.md) and
[availability](../../specs/transmog-availability.md) contracts; no production
fallback is added. The original spellbook hover and layout cases then pass.

`Blizzard_GroupFinder.toc:10` excludes camelot. Named test registration now omits
its consumer cases on Forever and tests the original loader's false/MISSING
result instead. Neither builtin frames nor the legacy LFG workaround synthesizes
`LFGListFrame` on Forever. Source-bound stock startup JSON remained `[]` before
and after that removal; unrelated legacy LFG helpers are unchanged.

WM99 adds the independent supplied [off-hand hit modifier](../../specs/forever-character-remaining-stats.md)
and [spec-group configuration selection](../../specs/forever-spec-group-config.md).
WM101 adds the resolved-player/class ammunition predicate and supplied
[trait-group display rows](../../specs/forever-trait-group-display.md), preserving
provider order and missing SkillLine joins. The original character-panel flow
now passes. The remaining tab-switch failure calls `C_Traits.GetGroupCurrencyInfo`;
WM102 is pending. The [glue character name contract](../../specs/forever-glue-character-name.md)
follows WM100's separate surname and supplied separator, uses the pinned localized
format and preserves zero-result unknown GUID lookup. Only absent localized
format data retains a named EMULATED seam. Original supplied-name and surname
queries pass.

The bounded Forever integration `tooltip` filter now reports 363 passed,
1 pre-existing failure and 1 ignored. This is not full-profile acceptance.

### Verification

The focused tests vary simulator state and exercise selected consumers.
Set `MOVEANY_UI_PATH` and `LOCKINTOOLS_UI_PATH` for their installed-addon cases.
For a startup comparison, use `lua-errors` with matching caches, addons and
read-only WTF imports in separate writable sessions. Compare failure causes
because Blizzard can report the same error twice. [Runtime inputs](../../runtime-inputs.md)

## Sources

- [Map open and update test](../../../tests/blizzard_map_canvas_loads.rs),
  [map filter and PvP tests](../../../tests/blizzard_world_map_loads.rs), and
  [quest title test](../../../tests/wowforever_quest_difficulty.rs).
- [Name constants](../../../src/c_api/forever_finite_constants.rs) and
  [caster GUID tests](../../../tests/forever_aura_caster_guid.rs).
- [Pet-order tests](../../../tests/forever_pet_move_order.rs),
  [channel tests](../../../tests/forever_permanent_channels.rs), and
  [encounter tests](../../../src/c_api/c_instance_encounter.rs).
- [Totem queries](../../../src/lua_api/globals/real/totems.rs),
  [talent query](../../../src/c_api/c_spec.rs), and
  [Edit Mode regression](../../../tests/edit_mode_api.rs).

## See Also

- [[forever-login-startup]] — graphics initialization before game startup.
- [[consumer-selection-and-red-proof]] — selecting the original consumer.
