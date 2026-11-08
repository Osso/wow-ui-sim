# Crafted quality — final bounded existing runtime gate

**PASS: 12 unique named tests per profile, 24/24 candidate cases; two separate normal-binary startup passes.**

No new runtime/build/check/fmt/authentication work. All six artifacts per each of 16 runs read completely and retained verbatim in `proof.json`, including 14,223/14,147-byte ItemButton and 72,106/71,662-byte PlayerChoice stderr. Tool display limits did not restrict full-file reads or receipt contents. New actual wait: **0 seconds per run**; historical launchers recorded nonzero runtime.

## Exact 12-row coverage matrix

| Exact test name | Retail | PTR | Assertion reach / proof level |
|---|---|---|---|
| `crafted_quality_api::crafted_quality_required_item_info_errors_have_context` | PASS | PASS | Omitted/nil required input; public method/itemInfo context; ordinary failure not secret denial. Recorded simulator runtime, not native acceptance. |
| `crafted_quality_api::crafted_quality_secret_nil_authenticates_before_required_validation` | PASS | PASS | Genuine VM secret nil: untainted required validation; tainted denial before validation; wrapper/payload retained, denied unwrap, caller and outer taint recovery. Recorded simulator runtime, not native acceptance. |
| `crafted_quality_item_button::crafted_quality_original_item_button_observes_supplied_quality_and_visibility` | PASS | PASS | Whole original ItemButton cached TOC closure; real Texture atlas, TOPLEFT (-3,2), OVERLAY/7 and flags; host real Frame visibility (NOT original panel), explicit alwaysShow true/false, both clears and host removal. Recorded simulator runtime, not native acceptance. |
| `crafted_quality_owned::crafted_quality_detached_snapshots_survive_host_replacement_and_removal` | PASS | PASS | Mutate every returned field without host mutation; replacement affects new snapshot, old snapshots persist; removal one nil, other item isolated. Recorded simulator runtime, not native acceptance. |
| `crafted_quality_owned::crafted_quality_existing_item_selector_forms_share_the_supplied_record` | PASS | PASS | Number, decimal string, item string, hyperlink aliases; same supplied record, fresh result tables. Recorded simulator runtime, not native acceptance. |
| `crafted_quality_owned::crafted_quality_real_vm_secret_selectors_guard_lookup_and_recover_wrappers` | PASS | PASS | VM secret four present/four absent forms; authentication-before-lookup/type guard, preserved wrappers/payloads and taint recovery; retained selectors after removal. Recorded simulator runtime, not native acceptance. |
| `crafted_quality_owned::crafted_quality_simulator_unknown_is_one_nil_and_inputs_are_environment_owned` | PASS | PASS | Unknown one nil; separate environment-owned maps; recipe tuple cannot seed item map; independent quality 3 versus 5. Recorded simulator runtime, not native acceptance. |
| `crafted_quality_owned::crafted_quality_supplied_record_projects_exact_thirteen_fields_and_one_result` | PASS | PASS | Supplied record: exact 13 fields/values/types, no extras; one-result arity. Recorded simulator runtime, not native acceptance. |
| `item_dressability::item_dressability_empty_input_returns_exactly_one_boolean_false` | PASS | PASS | Existing numeric/item-link/generated-link empty misses: exactly one boolean false. Recorded simulator runtime, not native acceptance. |
| `item_dressability::item_dressability_projects_explicit_numeric_and_link_booleans` | PASS | PASS | Existing parser projection false/true/false for real and uncatalogued numeric/link IDs; exact boolean arity. Recorded simulator runtime, not native acceptance. |
| `mount_item_player_choice::item_dressability_original_player_choice_tracks_supplied_boolean_and_mount` | PASS | PASS | Whole original PlayerChoice/ItemButton invocation: live dressability false/true/false; mount relation true despite predicate false; mount removal and dressability removal return false. Recorded simulator runtime, not native acceptance. |
| `p1200_rest_catalogs::p1200_rest_crafting_quality_missing` | PASS | PASS | Existing recipe paired Data fields/isolated snapshots and reagentLink positive/missing tuple behavior survive shared projection. Recorded simulator runtime, not native acceptance. |

Initial 2 + owned 5 + original ItemButton 1 + preservation 4 = 12 per profile. Zero failed/ignored/measured; filtered cases are not coverage. Assertions/aliases/environments/iterations and historical RED are not extra test names. All 16 launcher exits 0, allowed admissions, result success/status=0. Runtime result/argv/admission/launcher data remain verbatim in proof. No per-run memory.events delta or native budget parity claim.

## Full diagnostics — separate from warnings/errors

| Profile / original consumer | Nil observations | Missing requirements | Loader warnings | Collected Lua errors / records / count entries |
|---|---:|---:|---:|---|
| Retail ItemButton | 19 | 1 (`C_Reveal`) | 0 | 0 / 0 / 0 |
| PTR ItemButton | 19 | 1 (`C_Reveal`) | 0 | 0 / 0 / 0 |
| Retail PlayerChoice | 77 | 18 | 0 | 0 / 0 / 0 |
| PTR PlayerChoice | 77 | 18 | 0 | 0 / 0 / 0 |

Counts independently computed from entire stderr, not assumed from prior Retail gate. Full attributed nil/missing diagnostics and all empty warning/error lists retained; zero recorded errors does NOT mean zero diagnostics.

## Normal-binary startup — two passes, not tests

Each stdout is exactly 50 bytes:
```text
CRAFTED_QUALITY_EMPTY_DEFAULT_RECIPE_PRESERVED
[]
```
The exec-Lua assertion checks exactly one nil crafted miss, recipe absence, and dressability false. JSON `[]` and complete stderr independently show 0 unique/0 occurrence(s), 0 unattributed errors. Retail stderr 14,690 bytes; PTR 14,697 bytes. CASC=0, sound disabled, no third-party addons/SavedVariables; read-only EditMode cache and post-load workarounds remain. This is not clean-room/no-workaround, native, rendered-pixel, or original professions-panel acceptance.

## Provenance and bounded reuse

Retail source `cd9f42756472b8f8d706fd3e592f1977b3d89868`; PTR source `a5397a2d60ebdd2dc813309671bfb26534e3621c`; observed HEAD `51b64e72fd0ad5891135b6220441aa82eac0f492`; production `05665ddad6aa083406777f75887504a8e54c5725`. Git object diffs show Retail→PTR docs only and PTR/production→HEAD docs/evidence-publication only; production getter body unchanged. Authored test/production bodies retained in proof; no source-cache contents/vendor files read. Source representation shows real public registration and shared 13-field serializer, not auto-nil-only behavior. Header comments saying unexecuted are historical, superseded only for these exact recorded runtime cases.

Reused four existing PASS receipts with current dev/inode/size/mtime matching before/after and all runtime receipt_stat_reused snapshots. **Stat only: zero new hashes, binary/library/archive reads, source-tree authentication, or vendor/cache content authentication.** This associates the existing historical runs, NOT current-HEAD binary execution. Initial broad enumeration inspected source-cache names/stat metadata only; no source-cache contents opened.

## Preserved history and separate check gate

Historical initial/owned/consumer RED and build-admission denials remain in original locations; their full available runtime logs/results are copied to proof as history. Initial required/secret-nil RED failed at first required-validation assertion; later security recovery was unreached. Positive RED returned nil and original consumer failed at positive supplied-map boundary, not a setup acceptance. Two Retail GREEN build denials have allowed=false and zero runtime directories, not executions/pass counts. No historical artifact modified.

Default check/full format: existing separate PASS gate at `<verification>/crafted-quality-default-check-format-current/`, six manifest deprecations retained. Imported separately, not rerun, not 24-test coverage.

## OPEN / excluded

**Native catalog/classification, rendered pixels, item-name lookup, earlier profiles, full original professions panel, and full-parent goal remain OPEN.** No EventRegistry callback-delivery/unregistration assertions exist here; no such acceptance is claimed. Scope ends at this bounded runtime/preservation/startup gate.
