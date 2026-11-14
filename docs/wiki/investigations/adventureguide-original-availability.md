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

The fallback is omitted only for Retail. Namespace allocation, all eight
C_AdventureJournal fallbacks, the four other AdventureGuideUtil functions,
and every non-Retail fallback remain unchanged. The retained hyperlink
fallback is not independently proven: its Game consumer normally runs after
the original owner. No vendor Lua is changed.

## Proof boundary

`tests/stub_l_adventureguide_ownership.rs` uses normal selected Game startup,
checks selected TOCs and loaded records, original helper/consumer source and
mixin identity, and empty unfiltered diagnostics. It repeatedly disables the
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
with other-profile behavior retained. The synthetic startup helper probe
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
