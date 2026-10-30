# Forever spell data

The Forever profile uses a selected spell-data file derived from the pinned Forever client.

## What it must do

- [ ] Verify the CDN BLTE encoding key and each chunk checksum before decoding DB2 data.
- [ ] Zero-fill only chunks whose encryption key is unknown and retain their decoded byte ranges.
- [ ] Reject selected data that overlaps a zero-filled range.
- [ ] When chunks are zero-filled, do not claim whole-file content-key verification; the strict Blizzard UI downloader remains unchanged.

## How it works

- The pinned CDN transport is described in [PTR CDN content index](../ptr-cdn-content-index.md).

## Implementation inventory

- `src/blizzard_ui_sync/pinned_download.rs`: pinned CDN transport and DB2 decoding.
- `src/forever_spell_data.rs`: selected-record cache generation and source-range validation.

## Tests asserting this spec

- `src/blizzard_ui_sync/pinned_download.rs`: encrypted chunk decoding and corruption rejection.
- `src/forever_spell_data.rs`: selection, deterministic output and zero-filled-range rejection.

## Known gaps (current cycle)

- [ ] Generate and bundle the selected data, and load it in the Forever runtime.

## Out of scope

- A complete Forever spell catalog.
