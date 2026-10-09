# Patch 1.14.2 API audit

Frozen Era task SOURCE audit: page236101/revision2290155, timestamp2022-06-05T00:14:08Z. Source caption literally `1.14.1 (41030) &rarr; 1.14.2 (42214) Feb 3 2022`; TOC11402. Current configured Era/Anniversary11507 is not native1.14.2.

## Coverage matrix

| Scope | Literal accounting | Proof/remaining boundary |
|---|---|---|
| SOURCE | 39 physical/36 nonblank rows:26 metadata,10 UNPROVEN | Own RED8 failures/GREEN8 passes; lossless raw/response hashes |
| Inventory | Four added callables, three added CVars; removed0/0 | 14 UNPROVEN contracts; no callable arguments/returns specified |
| Headers/prose | Three section/four numeric headers; three hidden defaults/descriptions | Exact caption, seven links, navigation; no examples; linked content unexpanded |
| Configuration | Era/Anniversary11507 | Manifest pins separate; no native equivalence |
| Runtime/model/native | Zero measured in immutable SOURCE | Separate current Era getters1/1 at55c6fe1a9: two exact defaults/current values, one missing; unknown control nil/nil. No LED/native proof |

`C_GamePad.ClearLedColor`, `GetLedColor`, `SetLedColor` lack native signatures on this page. Existing `src/c_api/c_game_pad.rs` implements mapped sticks/free-look only, gated to Forever in module and registration; no LED state or virtual-device LED implementation. `TradeSkillOnlyShowMakeable` exists as a no-op under legacy missing surface. No production change proposed beyond recording these gaps; main accepted no runtime edits.

Defaults retained literally: `GamePadFactionColor=1` (faction-color description), `GamePadVibrationStrength=1` (effect strength), `telemetryTargetPackage=Blizzard.Telemetry.Wow_Mainline` (package routing description). Current YAML contains first two; telemetry entry absent. YAML inspection alone grants no runtime/getter or physical-effect credit. Separate current headless Era getter test at55c6fe1a9 returned `("1","1")` for each gamepad CVar, `(nil,nil)` for telemetry and unknown control. Missing telemetry default remains a precise gap, not invented or registered. Seven pre-existing headless warnings retained (six library, one binary); no changes to suppress/fix unrelated warnings.

Successors:1.14.3 in-flight,1.14.4 queued,1.15.0–1.15.9 integrated-not-applied. Frozen same-Era task inputs are retained, not semantic supersession or native receipts. Shared tools unchanged; retained historical copies preserve default replay.

## Portable SOURCE proof

Original SOURCE seals/map remain immutable. Fresh archive replay without Git/target/current tools passes SOURCE8/8 and reproduces default register bytes. Own portable RED3 failures (archive absent), GREEN3/3; serialized ledger omission and fabricated GREEN log each reject by seal, then exact restoration passes. Current receipts are separately sealed, never backfilled into original GREEN. Main owns successor integration, native and final gates.

## Sources

- [Frozen source pin](../../../data/patch-api/evidence/1.14.2-session-2026-10-09/source-pin.json) — exact identity/hashes.
- [Literal ledger](../../../data/patch-api/evidence/1.14.2-session-2026-10-09/ledger.json) — full rows, inventory, prose, links and boundaries.
- [Own tests](../../../data/patch-api/evidence/1.14.2-session-2026-10-09/test_source_accounting.py) — bounded SOURCE controls.
- [Spec](../../specs/patch-1-14-2-source-accounting.md) — requirements.
- [Portable controls](../../../data/patch-api/evidence/1.14.2-session-2026-10-09/portable-controls.json) — fresh replay, serialized tamper/restoration.
- [Current Era observations](../../../data/patch-api/evidence/1.14.2-session-2026-10-09/current-era-observations.json) — bounded getters, separate from SOURCE.

## See Also

- [[patch-1-15-0-api-audit]] — later Era source context, not imported behavior.
