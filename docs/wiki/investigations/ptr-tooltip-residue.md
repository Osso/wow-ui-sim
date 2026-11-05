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

## Counted action-combat follow-up

The rule-enabled original OnEnter selector is now GREEN with the separate
[WM93 counted channel](../../specs/ptr-action-combat-range-check.md). It previously
failed at the missing `C_ActionBar.EnableActionCombatActionRangeCheck` callback
(ActionButton.lua:433,1492,570,942,403). Its assertions and original consumer path
remain unchanged: no callback stub, texture removal or registration bypass.

Registration only changes counts; explicit supplied-result sweeps drive observer
notifications. The WM93 contract is the source of truth for the UNKNOWN
scheduling, target-selection, security and native diagnostic/trap seams.

## Source-bound slot test execution

Slot builds compile `CARGO_MANIFEST_DIR` into the fixed source mount path. Outside
the build unit, that host directory is empty. Direct test runs from the lane cwd
alone therefore cannot resolve manifest-relative bundled SimCommands/TestFramework
inputs. WM93 runs use the same `BindPaths=worktree:slot-source-mount` and working
directory as the build, with a fresh child CVar copy. The earlier minimap failure
and addon-test exit do not recur; no source or test patch was needed.

## Sources

- [PTR contract and evidence authority](../../specs/ptr-bank-reagents-action-hover.md)
- [WM93 counted-channel contract](../../specs/ptr-action-combat-range-check.md)
- [Original keybinding fixture](../../../tests/test_keybindings.rs)
- [Original consumer and direct selectors](../../../tests/ptr_tooltip_residue.rs)
- [Independent Leave selector](../../../tests/ptr_tooltip_hover_leave.rs)

## See Also

- [[lua-api]] — namespace and tooltip architecture
- [[consumer-selection-and-red-proof]] — preservation versus discriminating proof
