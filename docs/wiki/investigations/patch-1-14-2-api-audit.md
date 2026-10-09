# Patch 1.14.2 API audit

Frozen Era task SOURCE audit: page236101/revision2290155, timestamp2022-06-05T00:14:08Z. Source caption literally `1.14.1 (41030) &rarr; 1.14.2 (42214) Feb 3 2022`; TOC11402. Current configured Era/Anniversary11507 is not native1.14.2.

## Coverage matrix

| Scope | Literal accounting | Proof/remaining boundary |
|---|---|---|
| SOURCE | 39 physical/36 nonblank rows:26 metadata,10 UNPROVEN | Own RED8 failures/GREEN8 passes; lossless raw/response hashes |
| Inventory | Four added callables, three added CVars; removed0/0 | 14 UNPROVEN contracts; no callable arguments/returns specified |
| Headers/prose | Three section/four numeric headers; three hidden defaults/descriptions | Exact caption, seven links, navigation; no examples; linked content unexpanded |
| Configuration | Era/Anniversary11507 | Manifest pins separate; no native equivalence |
| Runtime/model/native | Zero measured in immutable SOURCE | Headless CVar observation pending, never LED effects |

`C_GamePad.ClearLedColor`, `GetLedColor`, `SetLedColor` lack native signatures on this page. Existing `src/c_api/c_game_pad.rs` implements mapped sticks/free-look only, gated to Forever in module and registration; no LED state or virtual-device LED implementation. `TradeSkillOnlyShowMakeable` exists as a no-op under legacy missing surface. No production change proposed beyond recording these gaps; main accepted no runtime edits.

Defaults retained literally: `GamePadFactionColor=1` (faction-color description), `GamePadVibrationStrength=1` (effect strength), `telemetryTargetPackage=Blizzard.Telemetry.Wow_Mainline` (package routing description). Current YAML contains first two; telemetry entry absent. YAML inspection alone grants no runtime/getter or physical-effect credit. Parent requested separately bounded getter measurements.

Successors:1.14.3 in-flight,1.14.4 queued,1.15.0–1.15.9 integrated-not-applied. Frozen same-Era task inputs are retained, not semantic supersession or native receipts. Shared tools unchanged; retained historical copies preserve default replay.

## Sources

- [Frozen source pin](../../../data/patch-api/evidence/1.14.2-session-2026-10-09/source-pin.json) — exact identity/hashes.
- [Literal ledger](../../../data/patch-api/evidence/1.14.2-session-2026-10-09/ledger.json) — full rows, inventory, prose, links and boundaries.
- [Own tests](../../../data/patch-api/evidence/1.14.2-session-2026-10-09/test_source_accounting.py) — bounded SOURCE controls.
- [Spec](../../specs/patch-1-14-2-source-accounting.md) — requirements.

## See Also

- [[patch-1-15-0-api-audit]] — later Era source context, not imported behavior.
