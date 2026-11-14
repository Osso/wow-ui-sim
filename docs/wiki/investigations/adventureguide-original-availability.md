# AdventureGuideUtil original availability ownership

Retail original FrameXMLUtil owns `AdventureGuideUtil.IsAvailable` before the
original XML MicroMenu consumer. Omitting its temporary Retail bootstrap
fallback is default-state PRESERVATION, not native availability modeling.

## Ownership and scope

`Blizzard_FrameXMLUtil.toc` allows Game and loads `AdventureGuideUtil.lua`.
The source replaces the helper namespace at line 8 and defines availability
at lines 10–12. `Blizzard_MicroMenu_Mainline.toc` selects Game/mainline and its
Communities dependency leads to FrameXMLUtil. The actual `EJMicroButton` XML
frame mixes in `EJMicroButtonMixin`; its `UpdateDisplay` calls the original
helper at line 1605, enabling the button at lines 1610–1611 when available.

The fallback is omitted only for Retail Game bootstrap. Retail Glue screens
retain their previous availability fallback through the screen-selection hook.
Returning to Game removes only the recorded fallback identity, never an original
or replacement function. Namespace allocation, all eight
C_AdventureJournal fallbacks, the four other AdventureGuideUtil functions,
and every non-Retail fallback remain textually unchanged. A synthetic Retail
Game call to the retained OpenHyperLink fallback before the original owner loads
now errors instead of returning false, because IsAvailable is absent. This is an
intentional unsupported pre-owner boundary, accepted by main in tracker #962.
The source search found no shipped consumer doing this: the sole original
OpenHyperLink callsite is ItemRefHandlers.lua:95 in Game-only UIPanels, whose
TOC explicitly depends on FrameXMLUtil. No vendor Lua is changed.

## Proof boundary

`tests/stub_l_adventureguide_ownership.rs` uses normal selected Game startup,
checks selected TOCs and loaded records, original helper/consumer source and
mixin identity, and empty unfiltered diagnostics. Before UIPanels' first normal
load it checks original FrameXMLUtil availability ownership and that the journal
handler is not yet registered; afterward the original SetItemRef/LinkUtil
entrypoints and journal registration are present. Glue discovery excludes both
FrameXMLUtil and UIPanels_Game. This is load-order proof, not full hyperlink
behavior coverage: native EJ_HandleLinkPath currently yields no results, so no
requested journal-instance selection is claimed. It repeatedly disables the
real XML button and invokes its original callback to restore enabled state.
The before-change selector passed with empty startup diagnostics.

Current C_AdventureJournal.CanBeShown is constant true and Kiosk.IsEnabled
is constant false. This proof covers that existing default state only. It
rejects a no-op consumer but does not reject a constant availability producer,
so it is PRESERVATION, not RED or native state coverage. No fake provider,
manual owner load, or synthetic availability state is used. Final build and
runtime receipts must be checked for the exact candidate; historical baseline
success is not final-candidate acceptance.

Bootstrap tests check the Retail helper is absent before original loading,
with other-profile behavior retained. Boundary tests preserve Login,
CharacterSelect and CharacterCreate availability and hyperlink defaults, then
check Game removes only the owned fallback. A replacement-function fixture tests
identity safety only; it is not a native-behavior or original-consumer proof. The synthetic startup helper probe
now reflects that ownership boundary rather than requiring the removed shim.

## Sources

- [Temporary fallback](../../../src/lua_api/workarounds/temporary/adventure_journal_fallbacks.rs)
- [Original consumer selector](../../../tests/stub_l_adventureguide_ownership.rs)
- [Bootstrap probe](../../../tests/startup_api_glue.rs)
- Retail 12.1.5.70575 runtime cache: FrameXMLUtil AdventureGuideUtil.lua,
  MicroMenu Mainline TOC/Lua/XML, Communities Mainline TOC.
- Tracker lt-88db: synthesis #934; restored pinned Retail delivery #958.

## See Also

- [[consumer-selection-and-red-proof]] — distinguish preservation from RED.
- [[uimap-point-original-ownership]] — separate Retail ownership boundary.
