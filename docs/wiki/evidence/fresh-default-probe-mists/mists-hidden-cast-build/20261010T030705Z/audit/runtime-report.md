# Existing runtime evidence audit — 20261010T030705Z

**Ordinary controls PASS: three distinct selectors, each one nested fixture / exit 0. Profile acceptance FAIL: 37 passed / 6 failed across eight complete targets.**

Source epoch `903711da8b9d6472f718e407af350c576fe4a7e0`; 3839 before/after mappings and bytes equal. All selected Cargo artifacts enable `client-mists` with casc/gui/sound/rodio, no retail/default feature; Cargo test profile enabled. Invocations, saved stream digests/counts, summaries, selected Cargo artifact metadata and independent stable executable hashes reconcile. Runtime binding passes; build-end content hash seal remains unproven. Source revision is submission-recorded, not independently matched to Git tree. Recorded environment override WOW_SIM_NO_SOUND=1; each execution uses timeout 90, nocapture and one test thread.

## Ordinary selectors

| Exact selector | Distinct fixture pass | Summary records | Exit |
|---|---:|---:|---:|
| `spell_casting::cast_bar_respects_edit_mode_lock_setting_after_startup_fix` | 1 | 2 | 0 |
| `chat_frame::test_chat_editbox_click_type_and_submit` | 1 | 2 | 0 |
| `chat_frame::test_chat_editbox_text_color_after_activation` | 1 | 2 | 0 |

Each selector emits one child and one parent summary, each 1 passed / 0 failed / 8181 filtered. Three distinct fixtures, not six tests. Ordinary nested fixture execution is NOT Mists original-prefork proof or full-startup proof.

## Eight profile targets

| Target | Passed | Failed | Exit |
|---|---:|---:|---:|
| `pandaria_installed_addons` | 10 | 0 | 0 |
| `mists_compat_bootstrap` | 22 | 1 | 101 |
| `mists_forbidden_frames` | 1 | 0 | 0 |
| `mists_world_map_opacity` | 1 | 0 | 0 |
| `mists_nameplate_scale` | 0 | 2 | 101 |
| `mists_currency_list` | 0 | 2 | 101 |
| `mists_dialog_helpers` | 1 | 1 | 101 |
| `mists_class_colors` | 2 | 0 | 0 |

All eight completed, zero missing. Four targets exit 0; four exit 101. Per-test status lines match 43 selected / 37 passed / 6 failed. Failure locations and safe observed classes:

- `mists_honor_frame_shared_reproduces_missing_honor_system_enabled` — `tests/mists_compat_bootstrap.rs:381:5`; nil-call.
- `nameplate_options_default_vertical_scale_updates_sizes` — `tests/mists_nameplate_scale.rs:12:6`; missing-profile-source-file.
- `nameplate_options_reproduce_nil_vertical_scale_arithmetic` — `tests/mists_nameplate_scale.rs:12:6`; missing-profile-source-file.
- `legacy_currency_list_size_wraps_c_currency_info` — `tests/mists_currency_list.rs:12:6`; missing-profile-source-file.
- `token_frame_update_reproduces_missing_currency_list_size` — `tests/mists_currency_list.rs:12:6`; missing-profile-source-file.
- `money_frame_set_type_reproduces_missing_basic_message_dialog_helper` — `tests/mists_dialog_helpers.rs:47:5`; nil-call.

Four source-reader failures record OS code 2 / NotFound (currency TokenUI Cata source and nameplates TBC source); they do not exercise their intended downstream Lua assertions. Two failures record nil-call errors at honor/dialog fixtures; no runtime root-cause correction or native behavior claim inferred. Raw panic payloads withheld.

## Stream/privacy/acceptance boundary

Every stdout/stderr fully consumed privately. Safe per-stream byte/line/marker/privacy-candidate counts, hashes, test identifiers and static source locations retained in aggregate. Successful fixture summaries are not global-clean-startup claims. Installed-addon source contracts are not full native client startup or runtime acceptance. Retail 86f272 normal-startup receipt is not a comparable Mists fixture baseline; no transferable native TOC/addon seal recorded here. Entire inputs not sealed.

`runtime-aggregate.json` and `runtime-hashmanifest.json` bind all ordinary/profile receipts and full streams, source snapshots, selected compiler evidence and reports. Evidence integrity PASS does not soften acceptance FAIL. No reruns/builds/backend/delegation/operations; no full-suite/native parity or original-prefork acceptance.
