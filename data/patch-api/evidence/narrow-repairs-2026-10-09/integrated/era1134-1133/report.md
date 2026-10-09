# Independent integrated Era 1.13.4 / 1.13.3 proof

Scope: owned SOURCE, default replay, scratch-only serialized tamper controls, original/receipt seals, separate actual 1.13.4 successor inspection for frozen 1.13.3, and exactly one two-target offline/locked headless Cargo invocation. No runtime fix, historical signature/native/loaded-UI claim; no other Era targets, broad check, full suite, network, commit, deployment or delegation.

## revision
```json
{
  "label": "revision",
  "argv": [
    "git",
    "rev-parse",
    "HEAD"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566507.9794557,
  "end_epoch": 1791566507.9810216,
  "exit_code": 0,
  "environment_file": "revision.env.json",
  "environment_sha256": "f9a1d36f45ea0ca0d6d467d78ad19a6d7b0cfa61b116fb0c42dee321833a682a",
  "overrides": {},
  "stdout": "revision.stdout",
  "stderr": "revision.stderr"
}
```
### Full stdout
```text
9252c6cc93c087518f25e64987e095ed76312a51

```
### Full stderr
```text

```

## before-status
```json
{
  "label": "before-status",
  "argv": [
    "git",
    "status",
    "--short"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566507.9812574,
  "end_epoch": 1791566507.9982355,
  "exit_code": 0,
  "environment_file": "before-status.env.json",
  "environment_sha256": "f9a1d36f45ea0ca0d6d467d78ad19a6d7b0cfa61b116fb0c42dee321833a682a",
  "overrides": {},
  "stdout": "before-status.stdout",
  "stderr": "before-status.stderr"
}
```
### Full stdout
```text
?? .code-index.db

```
### Full stderr
```text

```

## integration-parents
```json
{
  "label": "integration-parents",
  "argv": [
    "git",
    "show",
    "--no-patch",
    "--format=%H %P %s",
    "03fa877e6",
    "0c5cecbf9"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566507.9985795,
  "end_epoch": 1791566508.0022225,
  "exit_code": 0,
  "environment_file": "integration-parents.env.json",
  "environment_sha256": "f9a1d36f45ea0ca0d6d467d78ad19a6d7b0cfa61b116fb0c42dee321833a682a",
  "overrides": {},
  "stdout": "integration-parents.stdout",
  "stderr": "integration-parents.stderr"
}
```
### Full stdout
```text
03fa877e60fb3dbafb0e6ac252837fd7019d51a8 76bdaa961bee34ce2bdbbf06f6feb242a586eb11 426a3eadfdfd0c5010973cec401a5cc5b8c221ba Integrate Era 1.13.4 source and bounded totem proof
0c5cecbf9cdc9c7334c9055ddae3bd2a03169c04 03fa877e60fb3dbafb0e6ac252837fd7019d51a8 8af171b49d8646cb989e5b00ff8a092d96a9c261 Integrate Era 1.13.3 source and bounded NPC health proof

```
### Full stderr
```text

```

## root-delta
```json
{
  "label": "root-delta",
  "argv": [
    "git",
    "show",
    "9252c6cc9",
    "--",
    "src/lua_api/workarounds/temporary/housing_catalog_state.rs"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566508.0024767,
  "end_epoch": 1791566508.0042229,
  "exit_code": 0,
  "environment_file": "root-delta.env.json",
  "environment_sha256": "f9a1d36f45ea0ca0d6d467d78ad19a6d7b0cfa61b116fb0c42dee321833a682a",
  "overrides": {},
  "stdout": "root-delta.stdout",
  "stderr": "root-delta.stderr"
}
```
### Full stdout
```text
commit 9252c6cc93c087518f25e64987e095ed76312a51
Author: Alessio Deiana <adeiana@gmail.com>
Date:   Fri Oct 9 12:18:16 2026 -0500

    Align housing compatibility probe with empty catalog model

diff --git a/src/lua_api/workarounds/temporary/housing_catalog_state.rs b/src/lua_api/workarounds/temporary/housing_catalog_state.rs
index 47761afd5..00fba27a3 100644
--- a/src/lua_api/workarounds/temporary/housing_catalog_state.rs
+++ b/src/lua_api/workarounds/temporary/housing_catalog_state.rs
@@ -1,8 +1,7 @@
-//! Temporary housing/catalog seeded state surface.
+//! Temporary housing compatibility surface.
 //!
-//! The housing service flag is Rust-backed, but catalog/decor/neighborhood
-//! data is still a seeded UI fixture. Keep those compatibility namespaces out
-//! of the generic runtime surface until housing has a real backing subsystem.
+//! Catalog variants/search are state-backed and start empty. Remaining legacy
+//! decor/neighborhood fixtures stay isolated here until their backing models exist.
 
 const HOUSING_CATALOG_STATE_LUA: &str = include_str!("housing_catalog_state.lua");
 
@@ -26,7 +25,7 @@ mod tests {
     use crate::lua_api::WowLuaEnv;
 
     #[test]
-    fn installs_seeded_housing_catalog_surface() {
+    fn installs_housing_catalog_surface() {
         let env = WowLuaEnv::new().expect("lua env should initialize");
 
         let result: String = env
@@ -49,8 +48,11 @@ mod tests {
                     return "bad_featured"
                 end
                 local searcher = C_HousingCatalog.CreateCatalogSearcher()
-                if type(searcher) ~= "table" or searcher:GetSearchCount() == 0 then
-                    return "bad_searcher"
+                if type(searcher) ~= "table" then
+                    return "bad_searcher_type"
+                end
+                if searcher:GetSearchCount() ~= 0 then
+                    return "searcher_count_not_empty"
                 end
                 return "ok"
                 "#,

```
### Full stderr
```text

```

## source-1.13.4
```json
{
  "label": "source-1.13.4",
  "argv": [
    "python3",
    "-B",
    "/tmp/era1134-1133-independent/evidence-1.13.4/test_source_accounting.py"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566508.0312924,
  "end_epoch": 1791566511.3791342,
  "exit_code": 0,
  "environment_file": "source-1.13.4.env.json",
  "environment_sha256": "e74967377c6a385f9d82f9cbff6e1c1e11bb26589e57990f67de3784648f67d5",
  "overrides": {
    "PYTHONDONTWRITEBYTECODE": "1"
  },
  "stdout": "source-1.13.4.stdout",
  "stderr": "source-1.13.4.stderr"
}
```
### Full stdout
```text

```
### Full stderr
```text
test_each_omission_and_invented_contract_rejected (__main__.SourceAccounting.test_each_omission_and_invented_contract_rejected) ... ok
test_headers_counts_caption_and_navigation (__main__.SourceAccounting.test_headers_counts_caption_and_navigation) ... ok
test_identity_and_response (__main__.SourceAccounting.test_identity_and_response) ... ok
test_inventory_and_unspecified_signatures_defaults (__main__.SourceAccounting.test_inventory_and_unspecified_signatures_defaults) ... ok
test_lossless_raw_and_default_extract (__main__.SourceAccounting.test_lossless_raw_and_default_extract) ... ok
test_model_and_history_credit_separate (__main__.SourceAccounting.test_model_and_history_credit_separate) ... ok
test_prose_links_templates_and_client_limits (__main__.SourceAccounting.test_prose_links_templates_and_client_limits) ... ok
test_source_tamper_rejected (__main__.SourceAccounting.test_source_tamper_rejected) ... ok

----------------------------------------------------------------------
Ran 8 tests in 3.300s

OK

```

## successors-1.13.4
```json
{
  "label": "successors-1.13.4",
  "argv": [
    "python3",
    "-B",
    "/tmp/era1134-1133-independent/evidence-1.13.4/test_successors.py"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566511.379703,
  "end_epoch": 1791566512.7805426,
  "exit_code": 0,
  "environment_file": "successors-1.13.4.env.json",
  "environment_sha256": "e74967377c6a385f9d82f9cbff6e1c1e11bb26589e57990f67de3784648f67d5",
  "overrides": {
    "PYTHONDONTWRITEBYTECODE": "1"
  },
  "stdout": "successors-1.13.4.stdout",
  "stderr": "successors-1.13.4.stderr"
}
```
### Full stdout
```text

```
### Full stderr
```text
test_frozen_queue_and_source_ledger_unchanged (__main__.SuccessorAccounting.test_frozen_queue_and_source_ledger_unchanged) ... ok
test_omission_reordering_and_foreign_credit_rejected (__main__.SuccessorAccounting.test_omission_reordering_and_foreign_credit_rejected) ... ok
test_serialized_exact_precedence (__main__.SuccessorAccounting.test_serialized_exact_precedence) ... ok

----------------------------------------------------------------------
Ran 3 tests in 1.357s

OK

```

## portable-1.13.4
```json
{
  "label": "portable-1.13.4",
  "argv": [
    "python3",
    "-B",
    "/tmp/era1134-1133-independent/evidence-1.13.4/test_portable.py",
    "--archive",
    "/tmp/era1134-1133-independent/evidence-1.13.4/originals.tar.gz",
    "--scratch",
    "/tmp/era1134-1133-independent/scratch-1.13.4",
    "--receipts",
    "/tmp/era1134-1133-independent/portable-1.13.4.json"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566512.781065,
  "end_epoch": 1791566518.1453657,
  "exit_code": 0,
  "environment_file": "portable-1.13.4.env.json",
  "environment_sha256": "e74967377c6a385f9d82f9cbff6e1c1e11bb26589e57990f67de3784648f67d5",
  "overrides": {
    "PYTHONDONTWRITEBYTECODE": "1"
  },
  "stdout": "portable-1.13.4.stdout",
  "stderr": "portable-1.13.4.stderr"
}
```
### Full stdout
```text

```
### Full stderr
```text
test_copied_source_and_default_register (__main__.PortableSource.test_copied_source_and_default_register) ... ok
test_serialized_ledger_omission_rejected_and_restored (__main__.PortableSource.test_serialized_ledger_omission_rejected_and_restored) ... ok
test_serialized_log_fabrication_rejected_and_restored (__main__.PortableSource.test_serialized_log_fabrication_rejected_and_restored) ... ok

----------------------------------------------------------------------
Ran 3 tests in 5.321s

OK

```

## validator-1.13.4
```json
{
  "label": "validator-1.13.4",
  "argv": [
    "python3",
    "-B",
    "/tmp/era1134-1133-independent/evidence-1.13.4/validator.py"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566518.1456625,
  "end_epoch": 1791566518.2348566,
  "exit_code": 0,
  "environment_file": "validator-1.13.4.env.json",
  "environment_sha256": "e74967377c6a385f9d82f9cbff6e1c1e11bb26589e57990f67de3784648f67d5",
  "overrides": {
    "PYTHONDONTWRITEBYTECODE": "1"
  },
  "stdout": "validator-1.13.4.stdout",
  "stderr": "validator-1.13.4.stderr"
}
```
### Full stdout
```text
{
  "original_seals": 112,
  "source_totals": {
    "physical_lines": 63,
    "source_rows": 57,
    "extracted_rows": 10,
    "inventory": 23,
    "signatures": 23,
    "headers": 6,
    "inventory_headers": 3,
    "captions": 1,
    "prose": 2,
    "links": 5,
    "templates": 26,
    "references": 0,
    "contracts": 30,
    "configured_profiles": 2,
    "successors": 18,
    "model_review": 6,
    "source_statuses": {
      "metadata-only": 30,
      "UNPROVEN": 27
    },
    "inventory_by_section": {
      "global-api": 17,
      "events": 5,
      "cvars": 1
    },
    "contract_statuses": {
      "UNPROVEN": 30
    },
    "count_conflicts": 0,
    "declared_signatures": 0,
    "defaults": 0,
    "examples": 0,
    "registry_pages": 101,
    "omission_controls": 212
  },
  "successor_totals": {
    "actual_registers": 18,
    "source_inventory": 23,
    "overlap_occurrences": 1,
    "source_occurrences_with_overlap": 1,
    "meaningful_historical_closures": 0,
    "native_closures": 0
  },
  "source_tests": 8,
  "successor_tests": 3,
  "current_era_tests": 1,
  "native_historical_closures": 0,
  "scope": "Historical copied inputs/receipts only; no rerun of current Era or final gates."
}

```
### Full stderr
```text

```

## source-1.13.3
```json
{
  "label": "source-1.13.3",
  "argv": [
    "python3",
    "-B",
    "/tmp/era1134-1133-independent/evidence-1.13.3/test_source_accounting.py"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566518.2457042,
  "end_epoch": 1791566525.1601906,
  "exit_code": 0,
  "environment_file": "source-1.13.3.env.json",
  "environment_sha256": "e74967377c6a385f9d82f9cbff6e1c1e11bb26589e57990f67de3784648f67d5",
  "overrides": {
    "PYTHONDONTWRITEBYTECODE": "1"
  },
  "stdout": "source-1.13.3.stdout",
  "stderr": "source-1.13.3.stderr"
}
```
### Full stdout
```text

```
### Full stderr
```text
test_all_occurrence_omissions_and_fabricated_credit_rejected (__main__.SourceAccounting.test_all_occurrence_omissions_and_fabricated_credit_rejected) ... ok
test_current_state_and_successor_limits (__main__.SourceAccounting.test_current_state_and_successor_limits) ... ok
test_every_raw_row_and_historical_default_boundary (__main__.SourceAccounting.test_every_raw_row_and_historical_default_boundary) ... ok
test_exact_frozen_identity (__main__.SourceAccounting.test_exact_frozen_identity) ... ok
test_heading_counts_captions_navigation (__main__.SourceAccounting.test_heading_counts_captions_navigation) ... ok
test_identity_and_content_tamper_rejected (__main__.SourceAccounting.test_identity_and_content_tamper_rejected) ... ok
test_literal_inventory_not_truncated_default_register (__main__.SourceAccounting.test_literal_inventory_not_truncated_default_register) ... ok
test_prose_and_reference_boundaries (__main__.SourceAccounting.test_prose_and_reference_boundaries) ... ok
test_signature_fragments_do_not_invent_parameters (__main__.SourceAccounting.test_signature_fragments_do_not_invent_parameters) ... ok

----------------------------------------------------------------------
Ran 9 tests in 6.864s

OK

```

## portable-1.13.3
```json
{
  "label": "portable-1.13.3",
  "argv": [
    "python3",
    "-B",
    "/tmp/era1134-1133-independent/evidence-1.13.3/test_portable.py",
    "--archive",
    "/tmp/era1134-1133-independent/evidence-1.13.3/originals.tar.gz",
    "--scratch",
    "/tmp/era1134-1133-independent/scratch-1.13.3",
    "--receipts",
    "/tmp/era1134-1133-independent/portable-1.13.3.json",
    "--cwd",
    "/home/osso/Projects/wow/wow-ui-sim"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566525.1607034,
  "end_epoch": 1791566531.6280293,
  "exit_code": 0,
  "environment_file": "portable-1.13.3.env.json",
  "environment_sha256": "e74967377c6a385f9d82f9cbff6e1c1e11bb26589e57990f67de3784648f67d5",
  "overrides": {
    "PYTHONDONTWRITEBYTECODE": "1"
  },
  "stdout": "portable-1.13.3.stdout",
  "stderr": "portable-1.13.3.stderr"
}
```
### Full stdout
```text

```
### Full stderr
```text
test_copied_source_and_historical_default_bytes (__main__.PortableSource.test_copied_source_and_historical_default_bytes) ... ok
test_serialized_ledger_omission_rejected_and_restored (__main__.PortableSource.test_serialized_ledger_omission_rejected_and_restored) ... ok
test_serialized_log_fabrication_rejected_and_restored (__main__.PortableSource.test_serialized_log_fabrication_rejected_and_restored) ... ok

----------------------------------------------------------------------
Ran 3 tests in 6.427s

OK

```

## validator-1.13.3
```json
{
  "label": "validator-1.13.3",
  "argv": [
    "python3",
    "-B",
    "/tmp/era1134-1133-independent/evidence-1.13.3/audit.py"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566531.6283355,
  "end_epoch": 1791566531.68864,
  "exit_code": 0,
  "environment_file": "validator-1.13.3.env.json",
  "environment_sha256": "e74967377c6a385f9d82f9cbff6e1c1e11bb26589e57990f67de3784648f67d5",
  "overrides": {
    "PYTHONDONTWRITEBYTECODE": "1"
  },
  "stdout": "validator-1.13.3.stdout",
  "stderr": "validator-1.13.3.stderr"
}
```
### Full stdout
```text
{
  "seals": 84,
  "totals": {
    "physical_lines": 62,
    "source_rows": 56,
    "extracted_rows": 0,
    "inventory": 27,
    "default_inventory": 22,
    "default_omissions": 5,
    "signatures": 31,
    "headers": 8,
    "inventory_headers": 2,
    "captions": 2,
    "prose": 4,
    "links": 7,
    "templates": 35,
    "references": 2,
    "contracts": 38,
    "configured_profiles": 2,
    "successors": 18,
    "model_review": 7,
    "source_statuses": {
      "metadata-only": 23,
      "UNPROVEN": 33
    },
    "inventory_by_section": {
      "global-api": 22,
      "events": 1,
      "cvars": 4
    },
    "inventory_by_direction": {
      "added": 17,
      "removed": 10
    },
    "contract_statuses": {
      "UNPROVEN": 38
    },
    "count_conflicts": 0,
    "declared_signatures": 0,
    "defaults": 0,
    "examples": 0,
    "registry_pages": 101,
    "omission_controls": 266
  }
}

```
### Full stderr
```text

```

## Independently hashed original/receipt/archive seals
```json
{
  "1.13.4": {
    "seals.json": {
      "count": 112,
      "failures": [],
      "map_sha256": "543edf3bd874f54ae190e2df92254db6f808a986728512098a8ad8620171f66f"
    },
    "receipt-seals.json": {
      "count": 9,
      "failures": [],
      "map_sha256": "528a8ff833c682fb4b08e60165ca0975594957ceed6430edda6c5e9b2327ed71"
    },
    "archive": {
      "members": 113,
      "sealed_count": 112,
      "failures": [],
      "map_matches_original": true
    }
  },
  "1.13.3": {
    "seals.json": {
      "count": 84,
      "failures": [],
      "map_sha256": "8325cb7578f6007d1533fa06360001be6a0b2667ce3100925814e5e7c623ed9e"
    },
    "receipt-seals.json": {
      "count": 12,
      "failures": [],
      "map_sha256": "aec7cf31ea19144f1f3bb2c0b5629843cd3ad657c1e253af642956bb96e5a9be"
    },
    "archive": {
      "members": 85,
      "sealed_count": 84,
      "failures": [],
      "map_matches_original": true
    }
  }
}
```

## combined-era
```json
{
  "label": "combined-era",
  "argv": [
    "cargo",
    "test",
    "--offline",
    "--locked",
    "--no-default-features",
    "--features",
    "client-era",
    "--test",
    "patch_1_13_4_totems",
    "--test",
    "patch_1_13_3_npc_health",
    "--",
    "--nocapture"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566531.6891332,
  "end_epoch": 1791566608.3245497,
  "exit_code": 0,
  "environment_file": "combined-era.env.json",
  "environment_sha256": "2fb2143307a1410c5c294dd18a6db30b4af46876a8313f484405c4917d66cbd8",
  "overrides": {
    "CARGO_NET_OFFLINE": "true",
    "WOW_SIM_NO_ADDONS": "1",
    "WOW_SIM_NO_SAVED_VARS": "1",
    "WOW_SIM_P1133_HEALTH_OUT": "/tmp/era1134-1133-independent/npc-health-observations.json"
  },
  "stdout": "combined-era.stdout",
  "stderr": "combined-era.stderr"
}
```
### Full stdout
```text

running 1 test
test current_era_npc_health_reads_values_across_state_changes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s


running 1 test
test existing_era_totem_info_tracks_slot_replacement_expiry_and_removal ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s


```
### Full stderr
```text
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.large-enum-variant` is deprecated in favor of `lints.clippy.large_enum_variant` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.map-entry` is deprecated in favor of `lints.clippy.map_entry` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.match-wildcard-for-single-variants` is deprecated in favor of `lints.clippy.match_wildcard_for_single_variants` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.redundant-closure-for-method-calls` is deprecated in favor of `lints.clippy.redundant_closure_for_method_calls` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.trivially-copy-pass-by-ref` is deprecated in favor of `lints.clippy.trivially_copy_pass_by_ref` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.type-complexity` is deprecated in favor of `lints.clippy.type_complexity` and will not work in a future edition
warning: `iced_wgpu` (manifest) generated 6 warnings
    Blocking waiting for file lock on build directory
   Compiling wow-ui-sim v0.1.0 (/home/osso/Projects/wow/wow-ui-sim)
warning: constant `PROVENANCE_SCHEMA` is never used
  --> src/blizzard_ui_sync.rs:28:7
   |
28 | const PROVENANCE_SCHEMA: &str = "1";
   |       ^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: associated function `new` is never used
   --> src/blizzard_ui_sync.rs:180:8
    |
179 | impl CacheProvenance {
    | -------------------- associated function in this implementation
180 |     fn new(
    |        ^^^

warning: function `remove_missing_marker` is never used
   --> src/blizzard_ui_sync.rs:663:4
    |
663 | fn remove_missing_marker(path: &Path) {
    |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `ensure_known_asset_cached` is never used
  --> src/casc_asset_fallback.rs:93:15
   |
93 | pub(crate) fn ensure_known_asset_cached(_path: &str, _out_path: &Path) -> Option<PathBuf> {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `encoding_key_hex` is never read
  --> src/render/font.rs:44:5
   |
41 | struct WowFontFile {
   |        ----------- field in this struct
...
44 |     encoding_key_hex: Option<&'static str>,
   |     ^^^^^^^^^^^^^^^^

warning: methods `cooldown_elapsed_since_start` and `cooldown_remaining_seconds` are never used
   --> src/widget/frame.rs:601:19
    |
600 | impl Frame {
    | ---------- methods in this implementation
601 |     pub(crate) fn cooldown_elapsed_since_start(&self, elapsed_secs: f64) -> f64 {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
610 |     pub(crate) fn cooldown_remaining_seconds(&self, elapsed_secs: f64) -> Option<f64> {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `wow-ui-sim` (lib) generated 6 warnings
warning: unused import: `wow_ui_sim::saved_variables::SavedVariablesManager`
  --> src/bin/wow_sim/main.rs:20:5
   |
20 | use wow_ui_sim::saved_variables::SavedVariablesManager;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: `wow-ui-sim` (bin "wow-sim") generated 1 warning (run `cargo fix --bin "wow-sim" -p wow-ui-sim` to apply 1 suggestion)
    Finished `test` profile [optimized + debuginfo] target(s) in 1m 16s
     Running patch-tests/patch_1_13_3_npc_health.rs (target/debug/deps/patch_1_13_3_npc_health-76b78918e1a37849)
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.002s] [Startup] SimState::default complete in 2.43ms
[  0.003s] [Startup] rilua VM created in 119.66µs
[  0.003s] [Startup] builtin frames initialized in 71.23µs
[  0.003s] [Startup] template registry cleared in 6.41µs
[  0.003s] [Startup] intrinsic templates registered in 7.73µs
[  0.003s] [Startup] initial app_data lua handle installed in 391.00ns
[  0.141s] [Startup] init_lua_state complete in 137.91ms
[  0.141s] [Startup] final app_data lua handle installed in 300.00ns
[  0.141s] [Startup] initial screen globals installed in 117.12µs
[  0.141s] [Startup] WowLuaEnv::new complete
P1133_NPC_HEALTH {"configured_interface":11507,"historical_signature_credit":false,"native_credit":false,"observations":[{"observed":[7501,16003],"percentage_control":46.87246141348497,"seed":{"health":7501,"health_max":16003,"is_player":false}},{"observed":[93,24005],"percentage_control":0.3874192876484066,"seed":{"health":93,"health_max":24005,"is_player":false}},{"observed":[18007,24005],"percentage_control":75.01353884607373,"seed":{"health":18007,"health_max":24005,"is_player":false}}],"raw_types":["function","function"],"runtime_changes":0,"scope":"current bare Era NPC numeric values across three snapshot mutations","source_interface":11303,"source_ledger_mutated":false}
     Running patch-tests/patch_1_13_4_totems.rs (target/debug/deps/patch_1_13_4_totems-96c59d6788a3c97c)
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.002s] [Startup] SimState::default complete in 2.02ms
[  0.002s] [Startup] rilua VM created in 112.59µs
[  0.002s] [Startup] builtin frames initialized in 66.73µs
[  0.002s] [Startup] template registry cleared in 11.08µs
[  0.002s] [Startup] intrinsic templates registered in 9.93µs
[  0.002s] [Startup] initial app_data lua handle installed in 631.00ns
[  0.142s] [Startup] init_lua_state complete in 140.22ms
[  0.142s] [Startup] final app_data lua handle installed in 320.00ns
[  0.143s] [Startup] initial screen globals installed in 94.14µs
[  0.143s] [Startup] WowLuaEnv::new complete

```

## after-revision
```json
{
  "label": "after-revision",
  "argv": [
    "git",
    "rev-parse",
    "HEAD"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566608.3762894,
  "end_epoch": 1791566608.3776786,
  "exit_code": 0,
  "environment_file": "after-revision.env.json",
  "environment_sha256": "f9a1d36f45ea0ca0d6d467d78ad19a6d7b0cfa61b116fb0c42dee321833a682a",
  "overrides": {},
  "stdout": "after-revision.stdout",
  "stderr": "after-revision.stderr"
}
```
### Full stdout
```text
9252c6cc93c087518f25e64987e095ed76312a51

```
### Full stderr
```text

```

## after-status
```json
{
  "label": "after-status",
  "argv": [
    "git",
    "status",
    "--short"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "revision": "9252c6cc93c087518f25e64987e095ed76312a51",
  "start_epoch": 1791566608.377902,
  "end_epoch": 1791566608.397093,
  "exit_code": 0,
  "environment_file": "after-status.env.json",
  "environment_sha256": "f9a1d36f45ea0ca0d6d467d78ad19a6d7b0cfa61b116fb0c42dee321833a682a",
  "overrides": {},
  "stdout": "after-status.stdout",
  "stderr": "after-status.stderr"
}
```
### Full stdout
```text
?? .code-index.db

```
### Full stderr
```text

```

## Separate actual integrated 1.13.4 successor for 1.13.3
```json
{
  "scope": "Separate actual integrated 1.13.4 exact-name/section publication precedence over 1.13.3; not persisted into frozen queue",
  "earlier_source_head": "8af171b49d8646cb989e5b00ff8a092d96a9c261",
  "later_source_head": "426a3eadfdfd0c5010973cec401a5cc5b8c221ba",
  "count": 4,
  "overlaps": [
    {
      "symbol": "C_SummonInfo.ConfirmSummon",
      "section": "global-api",
      "prior_line": 24,
      "prior_direction": "added",
      "prior_literal": ": {{api|C_SummonInfo.ConfirmSummon}}",
      "later_line": 21,
      "later_direction": "added",
      "later_literal": ": {{api|C_SummonInfo.ConfirmSummon}}",
      "semantic_status": "UNPROVEN",
      "model_credit": false,
      "native_credit": false
    },
    {
      "symbol": "GetTotemInfo",
      "section": "global-api",
      "prior_line": 43,
      "prior_direction": "removed",
      "prior_literal": ": {{api|GetTotemInfo}}",
      "later_line": 25,
      "later_direction": "added",
      "later_literal": ": {{api|GetTotemInfo}}",
      "semantic_status": "UNPROVEN",
      "model_credit": false,
      "native_credit": false
    },
    {
      "symbol": "GetTotemTimeLeft",
      "section": "global-api",
      "prior_line": 44,
      "prior_direction": "removed",
      "prior_literal": ": {{api|GetTotemTimeLeft}}",
      "later_line": 26,
      "later_direction": "added",
      "later_literal": ": {{api|GetTotemTimeLeft}}",
      "semantic_status": "UNPROVEN",
      "model_credit": false,
      "native_credit": false
    },
    {
      "symbol": "TargetTotem",
      "section": "global-api",
      "prior_line": 46,
      "prior_direction": "removed",
      "prior_literal": ": {{api|TargetTotem}}",
      "later_line": 34,
      "later_direction": "added",
      "later_literal": ": {{api|TargetTotem}}",
      "semantic_status": "UNPROVEN",
      "model_credit": false,
      "native_credit": false
    }
  ],
  "foreign_history_credit": false,
  "historical_signature_credit": false,
  "contracts_remaining_unproven": {
    "1.13.3": 38,
    "1.13.4": 30
  }
}
```

## Portable 1.13.4 full worker streams and restoration receipts
```json
{
  "scope": "copied historical SOURCE; no runtime/model/native credit",
  "runs": [
    {
      "test": "__main__.PortableSource.test_copied_source_and_default_register",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e/validator.py"
      ],
      "cwd": "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "{\n  \"original_seals\": 112,\n  \"source_totals\": {\n    \"physical_lines\": 63,\n    \"source_rows\": 57,\n    \"extracted_rows\": 10,\n    \"inventory\": 23,\n    \"signatures\": 23,\n    \"headers\": 6,\n    \"inventory_headers\": 3,\n    \"captions\": 1,\n    \"prose\": 2,\n    \"links\": 5,\n    \"templates\": 26,\n    \"references\": 0,\n    \"contracts\": 30,\n    \"configured_profiles\": 2,\n    \"successors\": 18,\n    \"model_review\": 6,\n    \"source_statuses\": {\n      \"metadata-only\": 30,\n      \"UNPROVEN\": 27\n    },\n    \"inventory_by_section\": {\n      \"global-api\": 17,\n      \"events\": 5,\n      \"cvars\": 1\n    },\n    \"contract_statuses\": {\n      \"UNPROVEN\": 30\n    },\n    \"count_conflicts\": 0,\n    \"declared_signatures\": 0,\n    \"defaults\": 0,\n    \"examples\": 0,\n    \"registry_pages\": 101,\n    \"omission_controls\": 212\n  },\n  \"successor_totals\": {\n    \"actual_registers\": 18,\n    \"source_inventory\": 23,\n    \"overlap_occurrences\": 1,\n    \"source_occurrences_with_overlap\": 1,\n    \"meaningful_historical_closures\": 0,\n    \"native_closures\": 0\n  },\n  \"source_tests\": 8,\n  \"successor_tests\": 3,\n  \"current_era_tests\": 1,\n  \"native_historical_closures\": 0,\n  \"scope\": \"Historical copied inputs/receipts only; no rerun of current Era or final gates.\"\n}\n",
      "stderr": ""
    },
    {
      "test": "__main__.PortableSource.test_copied_source_and_default_register",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e/test_source_accounting.py"
      ],
      "cwd": "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "",
      "stderr": "test_each_omission_and_invented_contract_rejected (__main__.SourceAccounting.test_each_omission_and_invented_contract_rejected) ... ok\ntest_headers_counts_caption_and_navigation (__main__.SourceAccounting.test_headers_counts_caption_and_navigation) ... ok\ntest_identity_and_response (__main__.SourceAccounting.test_identity_and_response) ... ok\ntest_inventory_and_unspecified_signatures_defaults (__main__.SourceAccounting.test_inventory_and_unspecified_signatures_defaults) ... ok\ntest_lossless_raw_and_default_extract (__main__.SourceAccounting.test_lossless_raw_and_default_extract) ... ok\ntest_model_and_history_credit_separate (__main__.SourceAccounting.test_model_and_history_credit_separate) ... ok\ntest_prose_links_templates_and_client_limits (__main__.SourceAccounting.test_prose_links_templates_and_client_limits) ... ok\ntest_source_tamper_rejected (__main__.SourceAccounting.test_source_tamper_rejected) ... ok\n\n----------------------------------------------------------------------\nRan 8 tests in 3.288s\n\nOK\n"
    },
    {
      "test": "__main__.PortableSource.test_copied_source_and_default_register",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e/test_successors.py"
      ],
      "cwd": "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "",
      "stderr": "test_frozen_queue_and_source_ledger_unchanged (__main__.SuccessorAccounting.test_frozen_queue_and_source_ledger_unchanged) ... ok\ntest_omission_reordering_and_foreign_credit_rejected (__main__.SuccessorAccounting.test_omission_reordering_and_foreign_credit_rejected) ... ok\ntest_serialized_exact_precedence (__main__.SuccessorAccounting.test_serialized_exact_precedence) ... ok\n\n----------------------------------------------------------------------\nRan 3 tests in 1.430s\n\nOK\n"
    },
    {
      "test": "__main__.PortableSource.test_copied_source_and_default_register",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e/historical-tools/gen_patch_wikitext_register.py",
        "1.13.4",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e/source.wikitext",
        "3216451",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e/reproduced-register.json"
      ],
      "cwd": "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "",
      "stderr": ""
    },
    {
      "test": "__main__.PortableSource.test_copied_source_and_default_register",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e/audit.py",
        "default-extract"
      ],
      "cwd": "/tmp/era1134-1133-independent/scratch-1.13.4/tmp0akyw_2e",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "Patch 1.13.4 API changes\n\n== Diffs ==\n1.13.3 (32790) to 1.13.4 (33491)\n* FrameXML: https://github.com/Gethe/wow-ui-source/commit/1977612d0fd23d01a80d44b504bee70039df1782\n* API resources: https://github.com/Ketho/BlizzardInterfaceResources/commit/30ee4c00d5620733b5b7f2ff4f743cefc998c558\n\n== Changes ==\n* TOC version: 11304\n* Reinstated Totem API\n* Supports WoW Tokens in China\n\n== References ==\n",
      "stderr": ""
    },
    {
      "test": "__main__.PortableSource.test_serialized_ledger_omission_rejected_and_restored",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmp2dmip01j/validator.py"
      ],
      "cwd": "/tmp/era1134-1133-independent/scratch-1.13.4/tmp2dmip01j",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 1,
      "stdout": "",
      "stderr": "Traceback (most recent call last):\n  File \"/tmp/era1134-1133-independent/scratch-1.13.4/tmp2dmip01j/validator.py\", line 44, in <module>\n    print(json.dumps(validate(), indent=2))\n                     ~~~~~~~~^^\n  File \"/tmp/era1134-1133-independent/scratch-1.13.4/tmp2dmip01j/validator.py\", line 28, in validate\n    seals = audit.check_seals()\n  File \"/tmp/era1134-1133-independent/scratch-1.13.4/tmp2dmip01j/audit.py\", line 134, in check_seals\n    assert digest((E / name).read_bytes()) == expected, f'seal: {name}'\n           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\nAssertionError: seal: ledger.json\n"
    },
    {
      "test": "__main__.PortableSource.test_serialized_ledger_omission_rejected_and_restored",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmp2dmip01j/validator.py"
      ],
      "cwd": "/tmp/era1134-1133-independent/scratch-1.13.4/tmp2dmip01j",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "{\n  \"original_seals\": 112,\n  \"source_totals\": {\n    \"physical_lines\": 63,\n    \"source_rows\": 57,\n    \"extracted_rows\": 10,\n    \"inventory\": 23,\n    \"signatures\": 23,\n    \"headers\": 6,\n    \"inventory_headers\": 3,\n    \"captions\": 1,\n    \"prose\": 2,\n    \"links\": 5,\n    \"templates\": 26,\n    \"references\": 0,\n    \"contracts\": 30,\n    \"configured_profiles\": 2,\n    \"successors\": 18,\n    \"model_review\": 6,\n    \"source_statuses\": {\n      \"metadata-only\": 30,\n      \"UNPROVEN\": 27\n    },\n    \"inventory_by_section\": {\n      \"global-api\": 17,\n      \"events\": 5,\n      \"cvars\": 1\n    },\n    \"contract_statuses\": {\n      \"UNPROVEN\": 30\n    },\n    \"count_conflicts\": 0,\n    \"declared_signatures\": 0,\n    \"defaults\": 0,\n    \"examples\": 0,\n    \"registry_pages\": 101,\n    \"omission_controls\": 212\n  },\n  \"successor_totals\": {\n    \"actual_registers\": 18,\n    \"source_inventory\": 23,\n    \"overlap_occurrences\": 1,\n    \"source_occurrences_with_overlap\": 1,\n    \"meaningful_historical_closures\": 0,\n    \"native_closures\": 0\n  },\n  \"source_tests\": 8,\n  \"successor_tests\": 3,\n  \"current_era_tests\": 1,\n  \"native_historical_closures\": 0,\n  \"scope\": \"Historical copied inputs/receipts only; no rerun of current Era or final gates.\"\n}\n",
      "stderr": ""
    },
    {
      "restored": "ledger.json",
      "sha256": "7cf2ddbf813c1eb45e10f241e627bef00ecb24812071be98ac827e399ab0785d",
      "all_original_seals": 112
    },
    {
      "test": "__main__.PortableSource.test_serialized_log_fabrication_rejected_and_restored",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmpj3dgvd4_/validator.py"
      ],
      "cwd": "/tmp/era1134-1133-independent/scratch-1.13.4/tmpj3dgvd4_",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 1,
      "stdout": "",
      "stderr": "Traceback (most recent call last):\n  File \"/tmp/era1134-1133-independent/scratch-1.13.4/tmpj3dgvd4_/validator.py\", line 44, in <module>\n    print(json.dumps(validate(), indent=2))\n                     ~~~~~~~~^^\n  File \"/tmp/era1134-1133-independent/scratch-1.13.4/tmpj3dgvd4_/validator.py\", line 28, in validate\n    seals = audit.check_seals()\n  File \"/tmp/era1134-1133-independent/scratch-1.13.4/tmpj3dgvd4_/audit.py\", line 134, in check_seals\n    assert digest((E / name).read_bytes()) == expected, f'seal: {name}'\n           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\nAssertionError: seal: source-green.log\n"
    },
    {
      "test": "__main__.PortableSource.test_serialized_log_fabrication_rejected_and_restored",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.4/tmpj3dgvd4_/validator.py"
      ],
      "cwd": "/tmp/era1134-1133-independent/scratch-1.13.4/tmpj3dgvd4_",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "{\n  \"original_seals\": 112,\n  \"source_totals\": {\n    \"physical_lines\": 63,\n    \"source_rows\": 57,\n    \"extracted_rows\": 10,\n    \"inventory\": 23,\n    \"signatures\": 23,\n    \"headers\": 6,\n    \"inventory_headers\": 3,\n    \"captions\": 1,\n    \"prose\": 2,\n    \"links\": 5,\n    \"templates\": 26,\n    \"references\": 0,\n    \"contracts\": 30,\n    \"configured_profiles\": 2,\n    \"successors\": 18,\n    \"model_review\": 6,\n    \"source_statuses\": {\n      \"metadata-only\": 30,\n      \"UNPROVEN\": 27\n    },\n    \"inventory_by_section\": {\n      \"global-api\": 17,\n      \"events\": 5,\n      \"cvars\": 1\n    },\n    \"contract_statuses\": {\n      \"UNPROVEN\": 30\n    },\n    \"count_conflicts\": 0,\n    \"declared_signatures\": 0,\n    \"defaults\": 0,\n    \"examples\": 0,\n    \"registry_pages\": 101,\n    \"omission_controls\": 212\n  },\n  \"successor_totals\": {\n    \"actual_registers\": 18,\n    \"source_inventory\": 23,\n    \"overlap_occurrences\": 1,\n    \"source_occurrences_with_overlap\": 1,\n    \"meaningful_historical_closures\": 0,\n    \"native_closures\": 0\n  },\n  \"source_tests\": 8,\n  \"successor_tests\": 3,\n  \"current_era_tests\": 1,\n  \"native_historical_closures\": 0,\n  \"scope\": \"Historical copied inputs/receipts only; no rerun of current Era or final gates.\"\n}\n",
      "stderr": ""
    },
    {
      "restored": "source-green.log",
      "sha256": "95cb0368fe062ff93eecbb6137628eba70d9be0972db7ae26919e5908e0c8558",
      "all_original_seals": 112
    }
  ],
  "tests_run": 3,
  "failures": 0,
  "errors": 0
}
```

## Portable 1.13.3 full worker streams and restoration receipts
```json
{
  "scope": "copied historical SOURCE; no runtime/model/native credit",
  "runs": [
    {
      "test": "__main__.PortableSource.test_copied_source_and_historical_default_bytes",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmpxe5g823o/audit.py"
      ],
      "cwd": "/home/osso/Projects/wow/wow-ui-sim",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "{\n  \"seals\": 84,\n  \"totals\": {\n    \"physical_lines\": 62,\n    \"source_rows\": 56,\n    \"extracted_rows\": 0,\n    \"inventory\": 27,\n    \"default_inventory\": 22,\n    \"default_omissions\": 5,\n    \"signatures\": 31,\n    \"headers\": 8,\n    \"inventory_headers\": 2,\n    \"captions\": 2,\n    \"prose\": 4,\n    \"links\": 7,\n    \"templates\": 35,\n    \"references\": 2,\n    \"contracts\": 38,\n    \"configured_profiles\": 2,\n    \"successors\": 18,\n    \"model_review\": 7,\n    \"source_statuses\": {\n      \"metadata-only\": 23,\n      \"UNPROVEN\": 33\n    },\n    \"inventory_by_section\": {\n      \"global-api\": 22,\n      \"events\": 1,\n      \"cvars\": 4\n    },\n    \"inventory_by_direction\": {\n      \"added\": 17,\n      \"removed\": 10\n    },\n    \"contract_statuses\": {\n      \"UNPROVEN\": 38\n    },\n    \"count_conflicts\": 0,\n    \"declared_signatures\": 0,\n    \"defaults\": 0,\n    \"examples\": 0,\n    \"registry_pages\": 101,\n    \"omission_controls\": 266\n  }\n}\n",
      "stderr": ""
    },
    {
      "test": "__main__.PortableSource.test_copied_source_and_historical_default_bytes",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmpxe5g823o/test_source_accounting.py"
      ],
      "cwd": "/home/osso/Projects/wow/wow-ui-sim",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "",
      "stderr": "test_all_occurrence_omissions_and_fabricated_credit_rejected (__main__.SourceAccounting.test_all_occurrence_omissions_and_fabricated_credit_rejected) ... ok\ntest_current_state_and_successor_limits (__main__.SourceAccounting.test_current_state_and_successor_limits) ... ok\ntest_every_raw_row_and_historical_default_boundary (__main__.SourceAccounting.test_every_raw_row_and_historical_default_boundary) ... ok\ntest_exact_frozen_identity (__main__.SourceAccounting.test_exact_frozen_identity) ... ok\ntest_heading_counts_captions_navigation (__main__.SourceAccounting.test_heading_counts_captions_navigation) ... ok\ntest_identity_and_content_tamper_rejected (__main__.SourceAccounting.test_identity_and_content_tamper_rejected) ... ok\ntest_literal_inventory_not_truncated_default_register (__main__.SourceAccounting.test_literal_inventory_not_truncated_default_register) ... ok\ntest_prose_and_reference_boundaries (__main__.SourceAccounting.test_prose_and_reference_boundaries) ... ok\ntest_signature_fragments_do_not_invent_parameters (__main__.SourceAccounting.test_signature_fragments_do_not_invent_parameters) ... ok\n\n----------------------------------------------------------------------\nRan 9 tests in 6.035s\n\nOK\n"
    },
    {
      "test": "__main__.PortableSource.test_copied_source_and_historical_default_bytes",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmpxe5g823o/historical-tools/gen_patch_wikitext_register.py",
        "1.13.3",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmpxe5g823o/source.wikitext",
        "6472111",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmpxe5g823o/reproduced-register.json"
      ],
      "cwd": "/home/osso/Projects/wow/wow-ui-sim",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "",
      "stderr": ""
    },
    {
      "test": "__main__.PortableSource.test_copied_source_and_historical_default_bytes",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmpxe5g823o/audit.py",
        "default-extract"
      ],
      "cwd": "/home/osso/Projects/wow/wow-ui-sim",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 1,
      "stdout": "",
      "stderr": "Traceback (most recent call last):\n  File \"/tmp/era1134-1133-independent/scratch-1.13.3/tmpxe5g823o/audit.py\", line 196, in <module>\n    print(load_tool('extract_patch_non_inventory.py').extract_text(raw), end='')\n          ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^^^^\n  File \"/tmp/era1134-1133-independent/scratch-1.13.3/tmpxe5g823o/historical-tools/extract_patch_non_inventory.py\", line 279, in extract_text\n    raise ValueError(f\"unhandled template: {line}\")\nValueError: unhandled template: * Removed the \"CHANNEL\" chat type from {{api|SendAddonMessage}}() and added \"SAY\" and \"YELL\" types. <ref>{{ref web|url=https://us.forums.blizzard.com/en/wow/t/classic-patch-1-13-3-lua-api-change|author=[[Kaivax]]|date=2019-12-09|title=Classic Patch 1.13.3 Lua API Change}}</ref>\n"
    },
    {
      "test": "__main__.PortableSource.test_serialized_ledger_omission_rejected_and_restored",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmp086x2_lg/audit.py"
      ],
      "cwd": "/home/osso/Projects/wow/wow-ui-sim",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 1,
      "stdout": "",
      "stderr": "Traceback (most recent call last):\n  File \"/tmp/era1134-1133-independent/scratch-1.13.3/tmp086x2_lg/audit.py\", line 202, in <module>\n    seals = check_seals()\n  File \"/tmp/era1134-1133-independent/scratch-1.13.3/tmp086x2_lg/audit.py\", line 184, in check_seals\n    assert digest((E / name).read_bytes()) == expected, f'seal: {name}'\n           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\nAssertionError: seal: ledger.json\n"
    },
    {
      "test": "__main__.PortableSource.test_serialized_ledger_omission_rejected_and_restored",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmp086x2_lg/audit.py"
      ],
      "cwd": "/home/osso/Projects/wow/wow-ui-sim",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "{\n  \"seals\": 84,\n  \"totals\": {\n    \"physical_lines\": 62,\n    \"source_rows\": 56,\n    \"extracted_rows\": 0,\n    \"inventory\": 27,\n    \"default_inventory\": 22,\n    \"default_omissions\": 5,\n    \"signatures\": 31,\n    \"headers\": 8,\n    \"inventory_headers\": 2,\n    \"captions\": 2,\n    \"prose\": 4,\n    \"links\": 7,\n    \"templates\": 35,\n    \"references\": 2,\n    \"contracts\": 38,\n    \"configured_profiles\": 2,\n    \"successors\": 18,\n    \"model_review\": 7,\n    \"source_statuses\": {\n      \"metadata-only\": 23,\n      \"UNPROVEN\": 33\n    },\n    \"inventory_by_section\": {\n      \"global-api\": 22,\n      \"events\": 1,\n      \"cvars\": 4\n    },\n    \"inventory_by_direction\": {\n      \"added\": 17,\n      \"removed\": 10\n    },\n    \"contract_statuses\": {\n      \"UNPROVEN\": 38\n    },\n    \"count_conflicts\": 0,\n    \"declared_signatures\": 0,\n    \"defaults\": 0,\n    \"examples\": 0,\n    \"registry_pages\": 101,\n    \"omission_controls\": 266\n  }\n}\n",
      "stderr": ""
    },
    {
      "restored": "ledger.json",
      "sha256": "050632dc0adee2629b702338c109bdbdda25d04975d841b45edb4ce0a68df3c8",
      "all_original_seals": 84
    },
    {
      "test": "__main__.PortableSource.test_serialized_log_fabrication_rejected_and_restored",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmp2pp49n3l/audit.py"
      ],
      "cwd": "/home/osso/Projects/wow/wow-ui-sim",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 1,
      "stdout": "",
      "stderr": "Traceback (most recent call last):\n  File \"/tmp/era1134-1133-independent/scratch-1.13.3/tmp2pp49n3l/audit.py\", line 202, in <module>\n    seals = check_seals()\n  File \"/tmp/era1134-1133-independent/scratch-1.13.3/tmp2pp49n3l/audit.py\", line 184, in check_seals\n    assert digest((E / name).read_bytes()) == expected, f'seal: {name}'\n           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\nAssertionError: seal: green.log\n"
    },
    {
      "test": "__main__.PortableSource.test_serialized_log_fabrication_rejected_and_restored",
      "command": [
        "/usr/bin/python3",
        "-B",
        "/tmp/era1134-1133-independent/scratch-1.13.3/tmp2pp49n3l/audit.py"
      ],
      "cwd": "/home/osso/Projects/wow/wow-ui-sim",
      "path": "empty; Git/current commands unavailable",
      "exit_code": 0,
      "stdout": "{\n  \"seals\": 84,\n  \"totals\": {\n    \"physical_lines\": 62,\n    \"source_rows\": 56,\n    \"extracted_rows\": 0,\n    \"inventory\": 27,\n    \"default_inventory\": 22,\n    \"default_omissions\": 5,\n    \"signatures\": 31,\n    \"headers\": 8,\n    \"inventory_headers\": 2,\n    \"captions\": 2,\n    \"prose\": 4,\n    \"links\": 7,\n    \"templates\": 35,\n    \"references\": 2,\n    \"contracts\": 38,\n    \"configured_profiles\": 2,\n    \"successors\": 18,\n    \"model_review\": 7,\n    \"source_statuses\": {\n      \"metadata-only\": 23,\n      \"UNPROVEN\": 33\n    },\n    \"inventory_by_section\": {\n      \"global-api\": 22,\n      \"events\": 1,\n      \"cvars\": 4\n    },\n    \"inventory_by_direction\": {\n      \"added\": 17,\n      \"removed\": 10\n    },\n    \"contract_statuses\": {\n      \"UNPROVEN\": 38\n    },\n    \"count_conflicts\": 0,\n    \"declared_signatures\": 0,\n    \"defaults\": 0,\n    \"examples\": 0,\n    \"registry_pages\": 101,\n    \"omission_controls\": 266\n  }\n}\n",
      "stderr": ""
    },
    {
      "restored": "green.log",
      "sha256": "5ed89477f9651075e1ed7db36331542f09a739b8f4794c5e38c7167c1cf0c981",
      "all_original_seals": 84
    }
  ],
  "tests_run": 3,
  "failures": 0,
  "errors": 0
}
```

## Final bounded result: PASS

| Scope | Fresh evidence | Result |
|---|---|---|
| 1.13.4 SOURCE | Copied owned scripts, full output | 8/8; exit 0 |
| 1.13.3 SOURCE | Copied owned scripts, full output | 9/9; exit 0 |
| 1.13.4 separate successors | Owned 18-register precedence tests | 3/3; exit 0; one 1.14.0 removal overlap, zero semantic/native closure |
| Portable | Separate original archives; fresh empty-PATH subprocesses | 3/3 each; 9/8 worker runs; both serialized ledger/log tamper rejections and exact restorations each |
| Original/receipt seals | Independently hashed files and archive members | 112+9 / 84+12, no mismatches; archive 113/85 members, exact original maps |
| Current bare Era model | One combined Cargo invocation only | Totem 1/1 and NPC health 1/1; exit 0 |

**Observable behavior:** existing GetTotemInfo host slot active -> replacement -> removal; expired slot and invalid slots 0/99 inactive. NPC absolute health pairs 7501/16003 -> 93/24005 -> 18007/24005; raw callable registration and explicit percentage controls. Totem replacement mutates name in the existing host slot, not a full new Totem object; expiry uses a seeded already-expired slot, not waiting for live expiry. No stronger lifecycle claim.

**Historical defaults:** both generator registers reproduce exact saved bytes. 1.13.4 extractor reproduces saved text byte-for-byte. 1.13.3 preserves owned historical ValueError on unhandled SendAddonMessage/ref template, exit 1, empty stdout, no default-extract.txt; that expected rejection is not an unexpected verification failure.

**Separate integrated successor:** actual 1.13.4 inventory has four exact-name/section overlaps with 1.13.3: C_SummonInfo.ConfirmSummon added -> added; GetTotemInfo, GetTotemTimeLeft, TargetTotem removed -> added. Literal rows independently checked at saved line numbers. Frozen 1.13.3 queued-successor remains null/unapplied and later_registers stays empty. No GetTotemCannotDismiss overlap or inferred alias, foreign-history credit, native runtime proof or historical signatures. All 30/38 historical contracts remain UNPROVEN.

**Revision/delta:** canonical HEAD remained 9252c6cc93c087518f25e64987e095ed76312a51. Parent inspection confirms 03fa877e6 integrates source head 426a3eadf, then 0c5cecbf9 integrates 8af171b49. Current root delta edits housing module comments, test name and empty-search fixture expectations only; no non-test runtime body altered. New Rust targets manually read under rust-readability skill; no violations identified. No fresh fmt/check, other Era targets, broad checks or full suite run. No repository edit/commit/ops/network/delegation. Cargo generated ordinary build artifacts; original inputs were not altered. Existing untracked .code-index.db remained before/after.

**Portability limits:** source worktrees remained present; this does not prove portability under their deletion. 1.13.4 workers use copied scratch cwd, absolute interpreter, empty PATH and only copied archive inputs; inspected scripts resolve inputs via __file__, not source worktree. 1.13.3 workers use explicit canonical cwd, NOT old source worktree cwd, while data/tool reads resolve via copied __file__. Inherited environment is retained except documented overrides, not a fully isolated OS sandbox. Full outer environments captured locally in *.env.json; worker environments inherit those with PATH empty and PYTHONDONTWRITEBYTECODE=1. Outer command epoch intervals are captured; historical portable harness does not timestamp each worker separately. Full worker argv/cwd/exit/stdout/stderr and restoration receipts are included above.

**Warnings:** six existing iced vendor-manifest deprecated lint names, six headless library dead-code warnings, one binary unused-import warning retained. Full warnings with file:line above. Build emitted a file-lock wait; completed successfully in 76.635s; no timeout, cancellation or rerun. Not warning-free. No model runtime fix, historical/native or loaded-Blizzard-UI parity claim.

**Scoped immutability:** 1,873 scoped file hashes identical before/after: both entire owned evidence trees, src Rust/Lua, Cargo.toml/lock, and the two new Rust targets. No additions/removals/changes in that scope. Full maps: before-hashes.json and after-hashes.json; hash-delta.json contains comparison. This is scoped, not a claim that build target/cache artifacts never changed.

### Scoped map hashes
- `before-hashes.json` SHA256 `1e8d958746d8b3dcb62ff986a9ebc9592dcdb508184660ab269860a9b1a800dc`
- `after-hashes.json` SHA256 `1e8d958746d8b3dcb62ff986a9ebc9592dcdb508184660ab269860a9b1a800dc`
- `hash-delta.json` SHA256 `6b264202f23a3fc1073090483e750cdab0c4a1ca1fd38f28ba52a85d4d81b6cf`

### Exact relevant before/after hashes
```json
{
  "Cargo.lock": {
    "before": "859f258fa21eb92f7f0c83c77d526ea0ce3e7535b0970df2e39c459023c0b9cf",
    "after": "859f258fa21eb92f7f0c83c77d526ea0ce3e7535b0970df2e39c459023c0b9cf"
  },
  "Cargo.toml": {
    "before": "b1fd7660e2562019c74a063fe82b2d3ba15937b36636c828277b33f8f38a8f7b",
    "after": "b1fd7660e2562019c74a063fe82b2d3ba15937b36636c828277b33f8f38a8f7b"
  },
  "data/patch-api/evidence/1.13.3-session-2026-10-09/receipt-seals.json": {
    "before": "aec7cf31ea19144f1f3bb2c0b5629843cd3ad657c1e253af642956bb96e5a9be",
    "after": "aec7cf31ea19144f1f3bb2c0b5629843cd3ad657c1e253af642956bb96e5a9be"
  },
  "data/patch-api/evidence/1.13.3-session-2026-10-09/seals.json": {
    "before": "8325cb7578f6007d1533fa06360001be6a0b2667ce3100925814e5e7c623ed9e",
    "after": "8325cb7578f6007d1533fa06360001be6a0b2667ce3100925814e5e7c623ed9e"
  },
  "data/patch-api/evidence/1.13.4-session-2026-10-09/receipt-seals.json": {
    "before": "528a8ff833c682fb4b08e60165ca0975594957ceed6430edda6c5e9b2327ed71",
    "after": "528a8ff833c682fb4b08e60165ca0975594957ceed6430edda6c5e9b2327ed71"
  },
  "data/patch-api/evidence/1.13.4-session-2026-10-09/seals.json": {
    "before": "543edf3bd874f54ae190e2df92254db6f808a986728512098a8ad8620171f66f",
    "after": "543edf3bd874f54ae190e2df92254db6f808a986728512098a8ad8620171f66f"
  },
  "patch-tests/patch_1_13_3_npc_health.rs": {
    "before": "6e882ab84b4009f71ded9b6b8e299d63c4578b792df82d7ef52e77470047dbcf",
    "after": "6e882ab84b4009f71ded9b6b8e299d63c4578b792df82d7ef52e77470047dbcf"
  },
  "patch-tests/patch_1_13_4_totems.rs": {
    "before": "9511117f4bc584a68be35dad4da9481ba8e699b0a69f16144235ce023849ec40",
    "after": "9511117f4bc584a68be35dad4da9481ba8e699b0a69f16144235ce023849ec40"
  },
  "src/lua_api/workarounds/temporary/housing_catalog_state.rs": {
    "before": "9f144f60a845ba4f0f9f4a910c249b94dcd6ceb2c13a81abc47166675799538d",
    "after": "9f144f60a845ba4f0f9f4a910c249b94dcd6ceb2c13a81abc47166675799538d"
  }
}
```

### Command intervals (UTC)
- `after-revision` 2026-10-09T17:23:28.376289+00:00 -> 2026-10-09T17:23:28.377679+00:00; exit 0
- `after-status` 2026-10-09T17:23:28.377902+00:00 -> 2026-10-09T17:23:28.397093+00:00; exit 0
- `before-status` 2026-10-09T17:21:47.981257+00:00 -> 2026-10-09T17:21:47.998235+00:00; exit 0
- `combined-era` 2026-10-09T17:22:11.689133+00:00 -> 2026-10-09T17:23:28.324550+00:00; exit 0
- `integration-parents` 2026-10-09T17:21:47.998580+00:00 -> 2026-10-09T17:21:48.002223+00:00; exit 0
- `portable-1.13.3` 2026-10-09T17:22:05.160703+00:00 -> 2026-10-09T17:22:11.628029+00:00; exit 0
- `portable-1.13.4` 2026-10-09T17:21:52.781065+00:00 -> 2026-10-09T17:21:58.145366+00:00; exit 0
- `revision` 2026-10-09T17:21:47.979456+00:00 -> 2026-10-09T17:21:47.981022+00:00; exit 0
- `root-delta` 2026-10-09T17:21:48.002477+00:00 -> 2026-10-09T17:21:48.004223+00:00; exit 0
- `source-1.13.3` 2026-10-09T17:21:58.245704+00:00 -> 2026-10-09T17:22:05.160191+00:00; exit 0
- `source-1.13.4` 2026-10-09T17:21:48.031292+00:00 -> 2026-10-09T17:21:51.379134+00:00; exit 0
- `successors-1.13.4` 2026-10-09T17:21:51.379703+00:00 -> 2026-10-09T17:21:52.780543+00:00; exit 0
- `validator-1.13.3` 2026-10-09T17:22:11.628335+00:00 -> 2026-10-09T17:22:11.688640+00:00; exit 0
- `validator-1.13.4` 2026-10-09T17:21:58.145663+00:00 -> 2026-10-09T17:21:58.234857+00:00; exit 0
