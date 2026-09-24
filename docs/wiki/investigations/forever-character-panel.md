# Forever character panel investigation

Opening the Forever character panel reproduces the reported `GetUnitSpeed` nil crash in a cached binary with unproven source provenance. Asset lookup evidence records separate unresolved icon and cache boundaries; fixes and final regression proof remain pending.

## Evidence

The cached binary with SHA-256 `2ef368d3a4587a2a1ae820afc62e64ad9c4138c21612a155dbbd838accff0b72` exits 1 when `ToggleCharacter` reaches the exact user `GetUnitSpeed` nil traceback. This proves the cached Camelot runtime behavior, not the binary's source revision or native Forever behavior.

Commit `5ca35ab62` adds a Forever-only regression test for opening, closing, and reopening the character panel; it exercises movement-stat discovery plus run/swim updates. Final proof for that test is pending, so it is not credited as passing.

The active installation identifies as Forever `1.60.1.69977`, build `3bd89ce2721f7c75e7525dc83741076f`. A fresh upstream community CSV regenerated 146,353 rows identical to the tracked limited listfile, but it lacks `inv_sidetab_currency_c60`, `inv_sidetab_honor_alliance_c60`, `inv_sidetab_reputation2_c60`, and `inv_sidetab_stats_c60`.

The local resolution database contains matching FDIDs `8175455`, `8245174`, and `8254784` for user-supplied encoding keys, while their raw nine-byte key prefixes occur in none of 40 local `.idx` files. This is not evidence that the files are unavailable: 220 local `data.*` archives exist. The observed panel run spent 15.4 seconds building cache during an 18.4-second draw.

## Limits and next boundary

The missing API and startup-cache fixes are pending. No complete-panel, successful-regression, native, source-provenance, asset-resolution, or pixel claim follows from this record.

## Sources

- `/tmp/wow-character-bug/proof-ledger.json` — cached-binary reproduction, listfile regeneration, build, resolution, index, and archive observations.
- `5ca35ab62` — pending Forever character movement-stat regression test.

## See Also

- [[forever-addon-comparison]] — bounded cached-addon compatibility evidence.
- [[tick-cooldown-scan]] — separate asset-resolver cache investigation.
