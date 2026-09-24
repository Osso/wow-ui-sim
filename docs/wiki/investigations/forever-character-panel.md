# Forever character panel investigation

Opening the Forever character panel reproduces the reported `GetUnitSpeed` nil crash in a cached binary with unproven source provenance. Four formerly unmapped sidetab names are now authoritative; content extraction and runtime proof remain pending.

## Evidence

The cached binary with SHA-256 `2ef368d3a4587a2a1ae820afc62e64ad9c4138c21612a155dbbd838accff0b72` exits 1 when `ToggleCharacter` reaches the exact user `GetUnitSpeed` nil traceback. This proves the cached Camelot runtime behavior, not the binary's source revision or native Forever behavior.

Commit `5ca35ab62` adds a Forever-only regression test for opening, closing, and reopening the character panel; it exercises movement-stat discovery plus run/swim updates. Final proof for that test is pending, so it is not credited as passing.

The active installation identifies as Forever `1.60.1.69977`, build `3bd89ce2721f7c75e7525dc83741076f`; cached UI provenance still identifies `1.60.1.69913`, build `6c0df97e8e481a9a41600e373367c200`. That provenance difference does not itself explain the texture failures.

Complete active-root parsing verified its header against `56,735,783` bytes, `1,167` blocks, `2,741,566` records, and `235,575` names. Uppercase-backslash direct Jenkins hashes resolve controls `134400` and `2447783`, then authoritatively resolve the formerly unmapped names: Currency `8197078`, Honor Alliance `8197097`, Reputation `8197103`, and Stats `8197104`. This supersedes the earlier claim that all four were mapping blockers.

Commit `34975b1c2` records those four overrides and changes generation so every explicit override FDID is requested even absent from community data, literal scans, atlas entries, and Blizzard UI manifests. Regeneration adds five override-backed rows, including existing `8187495`; the four sidetab mappings appear in the limited listfile. The fresh 146,353-row upstream community CSV still lacks all four names, so it remains insufficient as their source.

Mapping is not content availability. Currency, Honor Alliance, and Reputation now have local index entries. Stats `8197104` has one verified encoding key, `4c7bd5a0d7e07775ff13ac62c07e3e18`, and no local index entry. The original unmapped FDIDs `8175455`, `8245174`, and `8254784` also each have one verified encoding key and no local index entry. Thus the remaining content blockers are Stats plus those original three; no alternate encoding was discarded and no resolver substitution is credited.

The user's panel log spent 15.4 seconds building the resolution cache during an 18.4-second draw. This timing is user-supplied evidence, not a newly measured benchmark.

## Limits and next boundary

Commits `7be534fff` and `57ffc3d01` add the player-speed and GUI resolution-cache-preparation slices, respectively; their tests remain pending verifier confirmation. The mapping generator's behavioral RED/GREEN passed, but its Python module suite has two reported pre-existing canonical-case expectation failures that are not independently confirmed here; no all-suite pass is claimed. The cache-preparation contract is [CASC asset loading](../../specs/casc-loading.md).

| Boundary | Status |
|---|---|
| Four sidetab name-to-FDID mappings | committed in `34975b1c2` |
| Generator behavioral RED/GREEN | passed |
| Icon extraction and character-panel runtime | pending GREEN |
| Remaining content availability | blocked: Stats `8197104`, plus `8175455`, `8245174`, `8254784` |
| API/cache slice tests | pending verifier |
| Native, pixels, full-panel pass | not run |

No complete-panel, successful-regression, native, source-provenance, asset-extraction, or pixel claim follows from this record.

## Sources

- `/tmp/wow-character-bug/proof-ledger.json` — cached-binary reproduction and prior cache observations.
- `/tmp/wow-character-bug/root-name-variants.json` and `name-hash-variants.tsv` — complete active-root totals, control FDIDs, and authoritative hash variant.
- `/tmp/wow-character-bug/stats-encoding-keys.json`, `all-encoding-keys.json`, and `local-index-evidence.json` — verified encoding cardinality and local-index evidence.
- `34975b1c2` — explicit overrides and generator request preservation.
- `5ca35ab62` — pending Forever character movement-stat regression test.

## See Also

- [[forever-addon-comparison]] — bounded cached-addon compatibility evidence.
- [[tick-cooldown-scan]] — separate asset-resolver cache investigation.
- [CASC asset loading](../../specs/casc-loading.md) — GUI resolution-cache-preparation contract.
