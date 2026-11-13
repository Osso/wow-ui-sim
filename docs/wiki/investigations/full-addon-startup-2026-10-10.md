# Full third-party startup — 2026-10-10

**Completion PASS; errorless full-third-party startup FAIL.** Existing receipt `20261010T173544Z` records exit 0, completion marker seen and unchanged sealed artifact. This docs-only record retains sanitized evidence; no rerun, build, source change, operation, delegation or commit occurred. SSOT, index and log remain untouched for main integration.

## Recorded coverage

| Observation | Evidence / boundary |
|---|---|
| Completion | Exit 0 and `[FullAddonStartupComplete]`; not errorless acceptance |
| Third-party loading | 79 addon attempts, 49 loaded; not all 79 failed |
| Lua errors | 105 occurrences across seven displayed signatures, including suppression summaries |
| Owner budget | 104 occurrences: EnhanceQoL 100, AllTheThings 4 |
| Nil call | One EllesmereUI occurrence at UICore line 1224 |
| Runtime warnings | Four reported; individual details unavailable |

Four file-load budget signatures contribute one occurrence each; EnhanceQoL OnEvent contributes 96 and AllTheThings OnEvent four. Seven first-occurrence headers plus 98 suppressed occurrences yield 105. No handler/file budget diagnostic counters were present: entry exhaustion versus callback consumption and native quota parity remain unestablished.

Loading separately reports one failed during loading, zero load failures and one loaded with Lua errors. Those categories do not explain every nonloaded addon. Sound and SavedVariables were disabled; cache hits demonstrate warm-cache execution, not runtime-input provenance. Inherited `WOW_SIM_NO_ADDONS` was not recorded/unset, but actual third-party loading is demonstrated by the saved streams.

## Nil-call metadata, not input semantics

The marker reports `nil`, `table`, `nil` for `type(IsUsingGamepad)`, `type(C_GamePad)` and the type of raw `IsEnabled`. Current local EllesmereUI source corroborates **IsUsingGamepad as callee one only**. It is not a receipt-hashed runtime asset. The first nil call prevents establishing that the second call was reached.

The raw-member probe does not establish metamethod-resolved lookup, native active-device semantics or gamepad enablement behavior. Native semantics remain unknown. No guessed default or repair is proposed; no vendor source is retained.

## Artifact association and separate stock proof

The full-addon submission revision is `d6f767b11b11a072a3bc3e8a1dd3395e7c4d97df`; it reused the seal from original submission `9635468d994be53d1f3571a8005115a49864c18a`. Independently verified seal SHA-256: `c9dd3209fbc3c10e8c40dc930cdf3890e8725b4bbe7a0ff8704fe0168956c552`. The independent report checked 3,853 recorded source/input paths: zero changed or missing. This is bounded source-byte association, not external dependency or all-runtime-input provenance.

Two separate clean-stock results remain separate evidence; neither establishes full-third-party acceptance. The inspected report specifically describes the original `--no-addons` completion receipt. This page does not elevate that receipt into third-party success or independently attest the other stock result.

## Retention

Only sanitized report, small outcome, marker summary and derivative SHA-256 manifest are retained. Original stream hashes are metadata, not retained raw streams. No raw vendor stderr, addon payload, private character/realm or cache identity is published. Original build warnings are separate from the four runtime warnings.

## Sources

- /home/osso/.local/state/wow-ui-sim/verification/full-addon-startup-current/independent-report.md — existing independent artifact inspection.
- /home/osso/.local/state/wow-ui-sim/verification/full-addon-startup-current/20261010T173544Z/outcome.json — exit and completion outcome.
- /home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/full-addon-startup-current/independent-report.md — sanitized retained report.
- /home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/full-addon-startup-current/marker-summary.json — bounded counts, shapes and original artifact hashes.
- /home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/full-addon-startup-current/SHA256SUMS — retained derivative hashes.

## New diagnostic capture

Epoch `20261010T183739Z`: [bounded independent attribution report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/success-file-budget-full-addon/independent-report.md), submission `b048251f38b2d53e33b3365590cd227d40fc07d0` (diagnostic code `0e4bf4161`). Source/artifact and 4,044 vendor-cache paths bound; completion observed. Records: 3,064 file-success, four file-error, 91 handler-error; exact private file attribution across 50 owners, zero rejected records and zero observed continuity gaps. Raw owner/file joins remain private. Current95 diagnostic error records are not historical105 actual error occurrences or evidence of a numeric count fix. Third-party inputs remain unsealed; full startup remains unfixed/unaccepted.

## Retail EllesmereUI v9.8 load reproduction (2026-11-12)

**VERIFIED:** the earlier `3/20` observation came from the active addon-enable configuration, which enabled only three compatible EllesmereUI directories; it did not show that the other 17 failed to load. A read-only staging directory and temporary `WOW_SIM_ADDONS_TXT` enabled all 20 installed Retail-compatible, non-LOD EllesmereUI addons without changing installed addon files or SavedVariables.

**VERIFIED (pre-fix baseline):** `EllesmereUI_UICore.lua:1499` calls `IsUsingGamepad()` before the following `C_GamePad` expression. The baseline simulator registered its existing `IsUsingGamepad` implementation only under `client-wowforever`; the pinned Retail `InputDocumentation.lua` declares a non-nil boolean return. The Retail consumer test at `tests/retail_ellesmere_core_gamepad_consumer.rs` reproduced the missing-global call before this lane added the Retail/PTR producer.

**VERIFIED (pre-fix baseline):** a local diagnostic addon that supplied only `IsUsingGamepad = function() return false end` let all 20 EllesmereUI addons reach loaded state without defining or replacing `C_GamePad`; the load still rejected an addon-local media path through `C_UIFileAsset.IsKnownFile`.

### Native contract and post-fix Retail run (2026-11-12)

**VERIFIED (G1, Retail 12.1.5.70688):** `IsUsingGamepad` reads a gamepad-input timestamp that starts at zero, does not read mouse time, and reads state different from `C_GamePad.IsEnabled`; one direct timestamp writer exists, but its conditions remain unknown.

**VERIFIED (G1c, Retail 12.1.5.70688):** `C_UIFileAsset.IsKnownFile` tests two in-memory tables by resolved file ID; `IsLooseFile` reads a nonzero marker from the first table. Paths are upper-cased, `/` becomes `\\`, exact misses retry a `.BLP` path, ID zero returns false, and neither predicate checks disk state.

**INFERRED (simulator policy):** Retail/PTR use an empty-default last-input timestamp and `GamePadOverlapMouseMs`; no writer, event, or mouse reset is modeled. **INFERRED (simulator policy):** before add-on Lua runs, the loader indexes regular files under that add-on's root in `known_loose_asset_paths`. **UNKNOWN:** whether native Retail asset tables include loose files from add-on roots, and the exact native table identities.

**VERIFIED (isolated Retail run at code SHA `6845ba42e`):** all 20 installed EllesmereUI add-ons loaded, with zero load-time Lua errors; post-startup `lua-errors` reported three unique errors across 72 occurrences: missing `GetLandingPageGarrisonType` in Blizzard minimap code (1), `SetNamePlateHitTestInsets requires an untainted caller` (1), and secret access without an untainted caller (70). The captured error set contained no `IsUsingGamepad` nil call or LibSharedMedia asset rejection.

### Issue #22 symptoms

The following observations come from isolated 1600x1200 or 3840x2160 simulator runs with fresh session paths; they do not establish equivalence to the screenshots or SavedVariables.

| Symptom | Current evidence |
|---|---|
| a. Stacked movers | **VERIFIED:** Bar2/Bar3, Bar4/Bar5, Bar9/Bar10, and cooldown/utility mover coordinates overlap in the probe; the corresponding underlying frames were hidden. **UNVERIFIED:** visible stacked mover overlays. |
| b. Truncated mover labels | **UNVERIFIED:** no matching mover-label fields appeared in the sampled tree, including after the unlock-mode probe; truncation was not reproduced. |
| c. Quest overflow/guide overlap | **VERIFIED:** Retail's `UseCustomTracker()` was false and `EUI_CustomQuestTracker` was absent in this run. **UNVERIFIED:** overflow or guide overlap; the run had no matching quest data. |
| d. Off-screen unit frames | **VERIFIED:** the sampled player unit frame was within the 1600x1200 viewport. **UNVERIFIED:** the screenshot's saved positions and UI scale. |
| e. Dim/under-bar health text | **VERIFIED:** sampled health text had alpha 1, white color, `OVERLAY` draw layer and frame level 5, above the Health `ARTWORK` layer at level 5. The dim/under-bar symptom was not reproduced. |
| f. Detached keybind row | **VERIFIED:** the HotKey anchor span measured 55 px while its resolved text width was 37 px after the local probe resized/anchored the button. **UNVERIFIED:** visible detachment; the button was hidden in the sampled tree. |

**VERIFIED (baseline at `fdf247810`):** the Retail consumer test was RED on the missing global. **VERIFIED (post-fix at `6845ba42e`):** the Retail `--lib --test integration --no-run` build completed with zero warnings; six targeted tests passed 1/1 each, including the original consumer and add-on-file pre-load boundary. A later seven-profile compile receipt is tracked in the lane handoff; neither compile nor startup output establishes full native parity or visual acceptance.

### Retail follow-up sources

- [Retail input consumer test](../../../tests/retail_ellesmere_core_gamepad_consumer.rs) — original missing-global and loaded-addon asset boundaries.
- [Input API contract](../../specs/is-using-gamepad.md) — VERIFIED timestamp facts and the separate INFERRED CVar window.
- [Asset API contract](../../specs/patch-12-0-7-vehicle-sound-assets.md) — G1c facts and UNKNOWN native addon-file catalog membership.
- [Input interface registration](../../../src/lua_api/globals/real/mod.rs) — Retail/PTR and Forever profile gates.
- [C_UIFileAsset implementation](../../../src/c_api/c_ui_file_asset.rs), [loader registration](../../../src/loader/addon.rs), and [state fields](../../../src/lua_api/state/sim_state.rs) — current shipped/loose lookup inputs.
- `/syncthing/World of Warcraft/_retail_/Interface/AddOns/EllesmereUI/EllesmereUI_UICore.lua`, `EllesmereUI_Profiles.lua`, and `Libs/LibSharedMedia-3.0/LibSharedMedia-3.0.lua` — installed read-only addon sources.
- `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua` and `UIFileAssetAPIDocumentation.lua` — pinned Retail declarations.
- `/var/tmp/osso/ellesmere-core-enabled-baseline-dump.log`, `/var/tmp/osso/ellesmere-core-input-only-baseline.log`, and `/var/tmp/osso/ellesmere-core-unlock-1600-dumptree.log` — local diagnostic outputs.
- `/home/osso/.local/state/wow-ui-sim/handoff/issue22-layout/REPORT.md` — prior six-symptom baseline and evidence limits.

### Related pages

- [[gamepad-mapped-state]] — existing, separate gamepad capability and mapped-stick behavior.
- [[ellesmereui-forever]] — prior Forever compatibility investigation.
- [[addon-loading]] — addon selection, enablement and load phases.
