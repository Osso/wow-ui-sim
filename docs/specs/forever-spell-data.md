# Forever spell data

The Forever profile uses a selected spell-data file derived from the pinned Forever client.

## What it must do

- [x] Verify the CDN BLTE encoding key and each chunk checksum before decoding DB2 data.
- [x] Zero-fill only chunks whose encryption key is unknown and retain their decoded byte ranges.
- [x] Reject selected data that overlaps a zero-filled range.
- [ ] `wow-cli casc sync-forever-spells` writes the selected data to the Forever profile cache, using the installed Forever build when present or pinned CDN build 1.60.1.70685 otherwise.
- [ ] The file contains only selected IDs, full table columns, layout hashes and unreadable-record counts.
- [x] Forever runtime initialization fails with the generation command if the data file is missing. Bundled data or the profile cache supplies the file.
- [ ] Selected spell names, ranks, icons, schools, targets, descriptions and power costs come from the Forever data. Retail behavior is unchanged.
- [ ] Forever release packages and Docker images generate and bundle the pinned spell data beside the simulator.
- [ ] When chunks are zero-filled, do not claim whole-file content-key verification; the strict Blizzard UI downloader remains unchanged.

## Source equivalence

For build `1.60.1.70685`, local CASC and CDN generation each produced 189,706 bytes with SHA-256 `13122873d813481e85815946ddf964068b04132d4d2b2b44ba5ce61491595ad0`. Both outputs contain 72 selected spell IDs. The selected rows were also compared with the matching DB2 CSV exports; all fields matched at float32 precision.

## How it works

- The pinned CDN transport is described in [PTR CDN content index](../ptr-cdn-content-index.md).

## Implementation inventory

- `src/blizzard_ui_sync/pinned_download.rs`: pinned CDN transport and DB2 decoding.
- `src/forever_spell_data.rs`: selected-record cache generation and source-range validation.
- `src/forever_db2.rs`: pinned WDC5 spell layouts.
- `src/blizzard_ui_sync/forever_spells.rs`: pinned DB2 sources and cache population.
- `src/forever_spell_runtime.rs`: cache validation and runtime catalog.
- `src/spells.rs`, `src/spell_descriptions.rs`, `src/spell_power.rs`: profile-aware lookups.
- `.github/workflows/release.yml`, `Dockerfile`: generate and bundle the data.

## Tests asserting this spec

- `src/blizzard_ui_sync/pinned_download.rs`: encrypted chunk decoding and corruption rejection.
- `src/forever_spell_data.rs`: selection, deterministic output and zero-filled-range rejection.
- `src/forever_db2.rs`: WDC5 name record and zero-fill rejection.
- `tests/issue13_spell_range.rs`: missing-file failure and reporter lookup.

## Known gaps (current cycle)

- [ ] Verify release and Docker packaging in their build environments.

## Out of scope

- A complete Forever spell catalog.
