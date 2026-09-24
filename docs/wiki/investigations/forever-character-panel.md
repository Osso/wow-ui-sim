# Forever character panel investigation

Opening the Forever character panel reproduces the reported `GetUnitSpeed` nil crash in a cached binary with unproven source provenance. Asset lookup evidence records separate unresolved icon and cache boundaries; fixes and final regression proof remain pending.

## Evidence

The cached binary with SHA-256 `2ef368d3a4587a2a1ae820afc62e64ad9c4138c21612a155dbbd838accff0b72` exits 1 when `ToggleCharacter` reaches the exact user `GetUnitSpeed` nil traceback. This proves the cached Camelot runtime behavior, not the binary's source revision or native Forever behavior.

Commit `5ca35ab62` adds a Forever-only regression test for opening, closing, and reopening the character panel; it exercises movement-stat discovery plus run/swim updates. Final proof for that test is pending, so it is not credited as passing.

The active installation identifies as Forever `1.60.1.69977`, build `3bd89ce2721f7c75e7525dc83741076f`; cached UI provenance still identifies `1.60.1.69913`, build `6c0df97e8e481a9a41600e373367c200`. That difference is not by itself proof of the texture failures. A fresh upstream community CSV regenerated 146,353 rows identical to the tracked limited listfile, but it lacks `inv_sidetab_currency_c60`, `inv_sidetab_honor_alliance_c60`, `inv_sidetab_reputation2_c60`, and `inv_sidetab_stats_c60`.

The local resolution database contains matching FDIDs `8175455`, `8245174`, and `8254784` for user-supplied encoding keys. Parsing their content entries from active `encoding.bin`, with page MD5 verification, finds exactly one encoding key each: no alternate encoding was discarded for these failures. Their nine-byte key prefixes occur in none of 40 local `.idx` files; control FDID `2447783` matches two indices. Although 220 local `data.*` archives exist, these three textures have no indexed local archive location. No resolver substitution or speculative FDID override was added.

The user's panel log spent 15.4 seconds building the resolution cache during an 18.4-second draw. This timing is user-supplied evidence, not a newly measured benchmark.

## Limits and next boundary

Commits `7be534fff` and `57ffc3d01` add the player-speed and GUI resolution-cache-preparation slices, respectively; their tests have no recorded GREEN result. The cache-preparation contract is [CASC asset loading](../../specs/casc-loading.md). No complete-panel, successful-regression, native, source-provenance, asset-resolution, or pixel claim follows from this record.

## Sources

- `/tmp/wow-character-bug/proof-ledger.json` — cached-binary reproduction, listfile regeneration, build, resolution, index, and archive observations.
- `/tmp/wow-character-bug/all-encoding-keys.json` and `local-index-evidence.json` — complete encoding-key lists, page checksum validation and indexed control.
- `5ca35ab62` — pending Forever character movement-stat regression test.

## See Also

- [[forever-addon-comparison]] — bounded cached-addon compatibility evidence.
- [[tick-cooldown-scan]] — separate asset-resolver cache investigation.
- [CASC asset loading](../../specs/casc-loading.md) — GUI resolution-cache-preparation contract.
