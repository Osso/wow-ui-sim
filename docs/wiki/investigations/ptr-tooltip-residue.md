# PTR tooltip residue

Three original tooltip tests exposed two missing native producers and a partial
addon fixture. The [contract](../../specs/ptr-bank-reagents-action-hover.md)
records the bounded WM92 implementation, evidence-build mismatch and current
verification results. It is the source of truth for native limitations.

## Root causes and corrections

The keybinding fixture omitted the separate PTR `Blizzard_StatusTrackingBar`.
Its TOC:30 loads Mainline/StatusTrackingBar.xml:35, defining the manager required
by CharacterFrame.lua:256. The fixture loads it after TextStatusBar/EditMode.
Retail retains those files inside ActionBar, so it must not load a nonexistent
separate addon. The unchanged inventory-tooltip flow passes on both profiles.

The recipe form reached missing `ReagentsFromBankAllowed` at ItemUtil.lua:315.
Its producer now reads the supplied object flag and existing shared game-rule
store. The original recipe-source hover passes; the direct matrix distinguishes
missing object, missing rule and both restrictive-rule flag values. ItemUtil's
count smoke is preservation coverage: GetItemCount currently does not distinguish
bank-source arguments, so that consumer alone does not reject a constant reagent
predicate. The direct matrix does.

ActionButton.lua:1501/1512 reached missing Enter/Leave members. Both now use the
bounded parser and supplied visualization inputs; ordinary original hover and
independent original Leave pass. Direct tests distinguish state lookup gates,
matching/different Leave slots, parser errors, return arity and the explicitly
emulated argument-access policy.

## Remaining original-consumer boundary

The new rule-enabled OnEnter selector remains failing pending WM93. Enabling
rule 204 makes the original UpdateAction path call the missing
`C_ActionBar.EnableActionCombatActionRangeCheck` through its registered callback
(ActionButton.lua:433,1492,570,942,403). The test is retained with its error oracle;
no callback stub, texture removal, vendor modification or assertion weakening
was used. Main accepted this as a separate follow-up slice, not part of WM92.

## Sources

- [PTR contract and evidence authority](../../specs/ptr-bank-reagents-action-hover.md)
- [Original keybinding fixture](../../../tests/test_keybindings.rs)
- [Original consumer and direct selectors](../../../tests/ptr_tooltip_residue.rs)
- [Independent Leave selector](../../../tests/ptr_tooltip_hover_leave.rs)

## See Also

- [[lua-api]] — namespace and tooltip architecture
- [[consumer-selection-and-red-proof]] — preservation versus discriminating proof
