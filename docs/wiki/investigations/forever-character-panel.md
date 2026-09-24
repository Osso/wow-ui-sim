# Forever character panel investigation

Opening the Forever character panel first exposed `GetUnitSpeed`, then a separate Camelot primary-stat contribution failure at `GetCritChanceFromStat` after the speed fix. Four formerly unmapped sidetab names are now authoritative; content extraction and complete runtime proof remain pending.

## Evidence

The cached binary with SHA-256 `2ef368d3a4587a2a1ae820afc62e64ad9c4138c21612a155dbbd838accff0b72` exits 1 when `ToggleCharacter` reaches the exact user `GetUnitSpeed` nil traceback. This proves the cached Camelot runtime behavior, not the binary's source revision or native Forever behavior.

Commit `5ca35ab62` adds a Forever-only regression test for opening, closing, and reopening the character panel; it exercises movement-stat discovery plus run/swim updates. The previous speed-path tests passed 5/5, but the same full-UI replay then reached `PaperDollFrameStats.lua:291` and failed on missing `GetCritChanceFromStat`. Commits `6a076ed46` and `b871b7083` add that bounded API model and its borrow-order correction; post-change replay and complete-panel proof remain pending.

The active installation identifies as Forever `1.60.1.69977`, build `3bd89ce2721f7c75e7525dc83741076f`; cached UI provenance still identifies `1.60.1.69913`, build `6c0df97e8e481a9a41600e373367c200`. That provenance difference does not itself explain the texture failures.

Complete active-root parsing verified its header against `56,735,783` bytes, `1,167` blocks, `2,741,566` records, and `235,575` names. Uppercase-backslash direct Jenkins hashes resolve controls `134400` and `2447783`, then authoritatively resolve the formerly unmapped names: Currency `8197078`, Honor Alliance `8197097`, Reputation `8197103`, and Stats `8197104`. This supersedes the earlier claim that all four were mapping blockers.

Commit `34975b1c2` records those four overrides and changes generation so every explicit override FDID is requested even absent from community data, literal scans, atlas entries, and Blizzard UI manifests. Regeneration adds five override-backed rows, including existing `8187495`; the four sidetab mappings appear in the limited listfile. The fresh 146,353-row upstream community CSV still lacks all four names, so it remains insufficient as their source.

Mapping is not content availability. Currency, Honor Alliance, and Reputation now have local index entries. Stats `8197104` has one verified encoding key, `4c7bd5a0d7e07775ff13ac62c07e3e18`, and no local index entry. The original unmapped FDIDs `8175455`, `8245174`, and `8254784` also each have one verified encoding key and no local index entry. Thus the remaining content blockers are Stats plus those original three; no alternate encoding was discarded and no resolver substitution is credited.

Cached PlayerScriptDocumentation supplies signatures and return counts for six missing Camelot globals: `GetCritChanceFromStat`, `GetSpellCritChanceFromStat`, `GetRangedAttackPowerForStat`, `GetHealthRegenFromSpirit`, `GetManaRegenFromSpirit`, and `GetHealthRegen`. The tooltip multiplies crit fractions by 100 and displays spirit regeneration; the previous `GetManaRegen` intellect baseline does not incorporate spirit. The [bounded model contract](../../specs/forever-character-stat-contributions.md) defines `100` primary-stat points per `1%` crit, class-specific ranged-AP coefficients, `0.2` health and `0.1` mana spirit rates, and default spirit `0` as explicit simulator policies, not native-verified values. Behavioral tests and the unchanged replay still require GREEN verification.

The user's earlier panel log spent 15.4 seconds building the resolution cache during an 18.4-second draw. New cold GUI evidence records a 19.6-second resolution-cache build after post-load workarounds at 17.5 seconds and before font initialization at 47.0 seconds. Initial first draw then spent 7.3 seconds in texture work (8.7 seconds total), attributed to the first community-catalog lookup; commit `79382f3b6` prewarms that catalog before GUI drawing. A later forced fresh CASC extract for FDID `2447783` took 266 ms. The next cold run is pending, so no full startup-stall fix is claimed.

## Limits and next boundary

Commits `7be534fff` and `57ffc3d01` add the player-speed and GUI resolution-cache-preparation slices, respectively; their tests remain pending verifier confirmation. The mapping generator's behavioral RED/GREEN passed, but its Python module suite has two reported pre-existing canonical-case expectation failures that are not independently confirmed here; no all-suite pass is claimed. The cache-preparation contract is [CASC asset loading](../../specs/casc-loading.md).

| Boundary | Status |
|---|---|
| Four sidetab name-to-FDID mappings | committed in `34975b1c2` |
| Generator behavioral RED/GREEN | passed |
| Icon extraction and character-panel runtime | pending GREEN |
| Remaining content availability | blocked: Stats `8197104`, plus `8175455`, `8245174`, `8254784` |
| Speed API tests | 5/5 GREEN; full UI hits next missing stat global |
| Six stat/regen global model tests | committed; batched GREEN pending |
| API/cache slice tests and next cold GUI run | pending verifier |
| Native, pixels, full-panel pass | not run |

No complete-panel, successful-regression, native, source-provenance, asset-extraction, or pixel claim follows from this record.

## Sources

- `/tmp/wow-character-bug/proof-ledger.json` — cached-binary reproduction and prior cache observations.
- `/tmp/wow-character-bug/root-name-variants.json` and `name-hash-variants.tsv` — complete active-root totals, control FDIDs, and authoritative hash variant.
- `/tmp/wow-character-bug/stats-encoding-keys.json`, `all-encoding-keys.json`, and `local-index-evidence.json` — verified encoding cardinality and local-index evidence.
- `34975b1c2` — explicit overrides and generator request preservation.
- `5ca35ab62` — Forever character movement-stat regression test.
- `6a076ed46`, `b871b7083` — bounded Forever stat/regen model and borrow-order correction.
- `79382f3b6` — community-catalog prewarm before GUI drawing.
- `/tmp/wow-character-bug/verify-speed.stderr` and `stat-surface-red.stdout` — previous full-UI failure and six missing-global probes.
- `/tmp/wow-character-bug/cold-gui/{stderr,result.json}` — bounded cold GUI timing evidence.
- Cached `Blizzard_APIDocumentationGenerated/PlayerScriptDocumentation.lua` and `Blizzard_UIPanels_Game/Camelot/PaperDollFrameStats.lua` — signatures, return counts, and tooltip consumption.

## See Also

- [[forever-addon-comparison]] — bounded cached-addon compatibility evidence.
- [[tick-cooldown-scan]] — separate asset-resolver cache investigation.
- [CASC asset loading](../../specs/casc-loading.md) — GUI resolution-cache-preparation contract.
