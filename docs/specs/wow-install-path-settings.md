# WoW install path settings

`wow-sim` and CASC tooling find the local WoW installation through `asset-resolver`. Users
who install WoW outside a default location must otherwise set `WOW_INSTALL_PATH` (or
`WOW_DATA_PATH`) on every launch. This contract makes the resolved location persistent per
CASC product. Implementation lives in `src/install_paths.rs` and
`src/asset_resolver_config.rs`.

## What it must do

- [x] Store install locations in `<platform config dir>/wow-ui-sim/install-paths.json`:
      `%APPDATA%\wow-ui-sim\install-paths.json` on Windows,
      `$XDG_CONFIG_HOME/wow-ui-sim/install-paths.json` (default `~/.config`) on Linux.
- [x] Key entries by CASC product: `wow`, `wowxptr`, `wowf`, `wow_classic`,
      `wow_classic_era`.
- [x] Store an optional `install_path` (install root that contains `Data`) and an optional
      `data_path` (`Data` directory). A `data_path`-only entry resolves to its parent root.
- [x] Resolve in this order: CLI override, `WOW_INSTALL_PATH`, `WOW_DATA_PATH`, the stored
      entry for the active product, then automatic detection. Automatic detection and its
      built-in default candidates remain owned by `asset-resolver`. The settings API
      accepts a CLI override; `wow-sim` currently passes none, so environment and settings
      are the shipped surfaces.
- [x] Accept a candidate only when `<root>/Data/data` is an existing directory, matching
      the resolver's own acceptance check.
- [x] Persist a valid CLI or environment path and a valid auto-detected path for the active
      product. A valid stored entry needs no rewrite.
- [x] Ignore an invalid stored entry, report why on stderr, and continue with detection.
- [x] Export the resolved root as `WOW_INSTALL_PATH` before the resolver caches its install
      path, so stored and detected paths need no additional resolver plumbing.
- [x] Name the settings file and the key `install_paths.<product>.install_path` in the
      `Could not find a World of Warcraft installation` startup error.
- [x] Treat a missing settings file as empty settings. Report unreadable or malformed
      content and continue startup.
- [x] Accept the settings file as an explicit input. Entrypoints pass the platform default
      path; library paths pass none, so library and test processes never read or write the
      user's settings file.
- [x] Leave settings and resolver state untouched when `WOW_SIM_CASC=0`.

## Out of scope

- The settings file does not store player, addon, or CVar preferences.
- No new resolver search paths or default install locations; those remain in
  `asset-resolver`.
- No installer or GUI editor for the file.

## How it works

- [CASC asset cache](../wiki/systems/casc-asset-cache.md#install-path-settings)
- [CASC loading](casc-loading.md)

## Implementation inventory

- `src/install_paths.rs` — settings types, JSON load/save, resolution order, warnings.
- `src/asset_resolver_config.rs` — product selection, install resolution before resolver
  caching, `WOW_INSTALL_PATH` export.
- `src/bin/wow_sim/main.rs` — startup configuration and the not-found error message.
- `src/blizzard_ui_sync.rs` — CASC-unavailable message names the settings file.

## Tests asserting this spec

Library tests in `src/install_paths.rs`:

- `resolution_order_prefers_cli_then_env_then_stored_then_detection`
- `detection_is_stored_when_no_override_or_stored_path_exists`
- `invalid_stored_path_is_ignored_and_replaced`
- `explicit_path_is_persisted_for_the_next_launch`
- `persistence_round_trip_preserves_both_paths`
- `products_keep_separate_install_paths`
- `data_path_entry_resolves_to_parent_root`
- `missing_settings_handle_uses_environment_without_touching_settings`
- `missing_settings_file_is_empty_settings`
- `env_names_are_derived_from_entry_field_names`

They use temporary directories and injected detection results; no test reads or writes the
real platform config directory, and none requires a local WoW install.

## Verification status (2026-11-04)

All builds used the canonical `scripts/slot-build.py`, slot 3, `-j4`, with Cargo
1.99.0 and rustc 1.99.0.

- RED `fa2e96fe8`: 7 of 9 targeted tests failed at `src/install_paths.rs:76`,
  `:87`, and `:164` with `not implemented: install path ...`.
- Targeted GREEN `c25f97580`: 9 passed, 0 failed. Unit
  `install-path-persist-green2`, CPUUsageNSec 214461214000.
- Retail library `c25f97580`: 2213 passed, 0 failed, 1 ignored (includes the 9
  tests above), zero compiler warnings/errors. Unit
  `install-path-persist-lib-full`, CPUUsageNSec 611184564000.
- PTR library check `c25f97580`: success, zero compiler warnings/errors. Unit
  `install-path-persist-ptr-check`, CPUUsageNSec 299311267000.
- Retail binaries check `3de2fcc8e` (`--bins --features tools`): success, zero
  compiler warnings/errors. Unit `install-path-persist-bins-check`,
  CPUUsageNSec 31060229000.
- Windows cross-check was not run: rustup is absent and no
  `x86_64-pc-windows-gnu` target is installed.
