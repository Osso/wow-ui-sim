# UI file asset queries

`C_UIFileAsset` reports shipped file IDs and selected addon-local loose files. Source: `src/c_api/c_ui_file_asset.rs`. Addon selection follows the real TOC loader; see [addon loading](../addon-loading-pipeline.md).

## What it must do

- [x] Preserve numeric file IDs and shipped listfile path lookups, including their existing slash/case normalization.
- [x] Recognize an existing regular addon-local `.ogg` file during its TOC load and on later queries, with case-insensitive folder/path components and either slash convention.
- [x] Report selected addon-local files as known and loose, with `GetFileID` returning `nil`. **Inferred simulator policy:** cached API documentation does not specify loose-file ID behavior.
- [x] Resolve only against the loader-selected addon directory, including after an earlier scan registered the same folder from another root; missing files in the selected root must not be rescued from alternate roots.
- [x] Probe supported texture extensions for extensionless addon-local texture queries, without guessing extensions for sounds.
- [x] Reject traversal, absolute paths, and symlink escapes outside the selected addon directory.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/c_ui_file_asset.rs` — C API classification and selected-root file lookup.
- `src/lua_api/state_types/runtime.rs` — selected addon directory in addon metadata.
- `src/loader/addon.rs` — associates loaded TOC with its selected directory.
- `src/lua_api/addon_scan.rs` — records scanned addon directory before loading.
- `src/bin/wow_sim/addon_loading.rs` — preserves the selected discovered TOC directory before runtime loading.

## Tests asserting this spec

- `tests/ui_file_assets.rs` — grouped real-loader file queries and selected-root boundaries.
- `src/c_api/c_ui_file_asset.rs` — shipped listfile/numeric path regression test.

## Known gaps (current cycle)

- [ ] Independently verify frozen `963a3b791` BigWigs sound-registration replay. The observed replay has zero reset warnings; audio playback remains out of scope.

## Out of scope

- Native fidelity for existence/openability: cached API docs explicitly do not guarantee these; discovered regular files are a bounded simulator policy.
- Renderer changes, CASC asset extraction, playback, and sound-device availability.
