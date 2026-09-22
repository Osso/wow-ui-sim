# Forever ComboFrame target-owned combo points

Enabling `comboPointLocation=1` exposes a missing `GetComboPoints` global in unchanged native `ComboFrame_Update`. The bounded correction reads the existing player combo-point pool through a target-GUID assignment; it does not replace the failure with a constant zero or alter native Lua.

## Two independent boundaries

Classic UI Forever enables the combo-point CVar. The audit initially shared persisted CVars between package runs, so later unrelated addons and a no-addons control reached the same native call. A fresh data-home control is clean because that path is inactive, not because the API exists. Audit isolation belongs to the harness; implementing the native API remains necessary for addons that enable it.

`UnitPower` alone is insufficient: cached EllesmereUI 9.2.2 reports a target-change observation with the old power snapshot at three but the target-aware query at zero. The simulator previously stored only `secondary_powers[4]`, without ownership. The new Forever-only optional target GUID is assigned by the existing admin combo-power input. Queries compare resolved target identity without mutating the power snapshot.

## Policy and proof limits

The [spec](../../specs/forever-combo-points.md) owns the explicit guessed lifecycle/ownership rules. Nonplayer-owned pools are unsupported and reported rather than replaced with fake counts. Other profiles retain existing behavior; no new native secrecy or combat-generation claim follows.

At `c1e830ffa`, the isolated Forever integration build passes and the six grouped regressions pass 6/6, including the target-change callback and real native ComboFrame CVar-enabled update. This is bounded simulator proof only; affected package paths still require isolated reruns.

## Sources

- Pinned Forever `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua` — global signature.
- Pinned Forever `Blizzard_UnitFrame/Mainline/ComboFrame.lua` — CVar gate and native consumer.
- Cached EllesmereUI file `8936131`, `EllesmereUI/CHANGELOG.md` — author-reported target-swap distinction, not native probe proof.
- `/tmp/forever-addon-audit/combo-native-producer-cause.md` — missing publication investigation.
- `/tmp/forever-addon-runtime/post-batch-isolated-data-control-ledger.json` — fresh data-home control.

## See Also

- [[synchronous-intrinsic-events]] — separate visibility/event lifecycle corrections.
- [[forever-addon-comparison]] — broader inventory evidence boundaries.
