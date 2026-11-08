# Saved Retail final runtime — PASS

Source `62f115d262613a848180143d6b30e2a0b3eb63b0`; production `e37f5426a1da8d721b0ab3ff0341877f59100c02` unchanged.
Epoch `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-tag-lifecycle-green-20261014T175706489371Z`; saved run `runtime-subtext-final-eleven-status-corrected`.
Artifact verification under `<private-agentconfig>/skills/verify/SKILL.md`. No builds, tests, reruns, delegation, canonical edits, payload reads/hashes. Read-only source/git/stat checks; existing receipts reused.

## Independent runtime count

**11/11 PASS; exit 0: 8 subtext including actual secrets + 3 unchanged lifecycle controls.**
Full stdout 4071 bytes; full stderr 7701 bytes, inspected without omission and retained in `proof.json`. Final stderr contains startup information only, no test panic/error. Admission allowed; launcher: `Finished with result: success`, `code=exited, status=0/SUCCESS`.
Outer summary: `11 passed; 0 failed; 0 ignored; 0 measured; 11055 filtered out`. Eleven nested one-case re-execution summaries each say `1 passed; 0 failed; 11065 filtered out`; **nonadditive**, not 22 tests. Unique child test names independently counted and matched archived functions.

| Exact case | Result |
|---|---|
| `ui_widget_manager_lifecycle::cached_ui_widget_container_filters_supplied_unit_updates` | PASS |
| `ui_widget_manager_lifecycle::cached_ui_widget_container_refreshes_removes_and_unregisters_supplied_widgets` | PASS |
| `ui_widget_manager_lifecycle::cached_ui_widget_icon_and_text_dynamic_states_toggle_acquired_child` | PASS |
| `ui_widget_text_with_subtext::consumer::text_with_subtext_cached_type25_normal_activation_update_hidden_shown_remove_unregister` | PASS |
| `ui_widget_text_with_subtext::feed::text_with_subtext_normal_feed_publishes_complete_state_before_synchronous_events` | PASS |
| `ui_widget_text_with_subtext::feed::text_with_subtext_wrong_feed_type_or_missing_set_changes_neither_state_nor_events` | PASS |
| `ui_widget_text_with_subtext::queries::text_with_subtext_exact_27_fields_fresh_snapshots_and_all_field_replacement` | PASS |
| `ui_widget_text_with_subtext::queries::text_with_subtext_required_selectors_reject_missing_and_invalid_arguments` | PASS |
| `ui_widget_text_with_subtext::queries::text_with_subtext_two_environments_keep_records_and_replacements_isolated` | PASS |
| `ui_widget_text_with_subtext::queries::text_with_subtext_unknown_returns_exactly_one_nil_control` | PASS |
| `ui_widget_text_with_subtext::security::text_with_subtext_actual_secret_selectors_allow_secure_deny_tainted_preserve_state` | PASS |

## Reached boundaries

Passing sequential consumer case reaches every phase, including those not reached in earlier runs. No sibling-control substitution for type25 lifecycle proof.

| Boundary | Archived evidence | Result |
|---|---|---|
| exact 27 fields, types, values, one result; fresh tables, caller mutation isolation and all-field replacement with retained snapshot | `tests/widget_subtext/queries.rs:40-66`; `tests/widget_two_records/helpers.rs:14-101`; `tests/widget_subtext/fixtures.rs:11-119` | PASS |
| missing/nil/string/boolean/table/function/thread rejected; numeric edge selectors return one nil; populated fractional selector accepted | `tests/widget_subtext/queries.rs:28-37`; `tests/widget_two_records/helpers.rs:103-145` | PASS |
| unknown IDs one nil; two environments absent/independent records and replacements | `tests/widget_subtext/queries.rs:9-25`; `tests/widget_subtext/queries.rs:68-115` | PASS |
| real host secret-number wrappers: secure present/absent; public snapshot fields; tainted denial before lookup, wrappers/state preserved, public tainted query and secure recovery | `tests/widget_subtext/security.rs:7-33`; `tests/widget_two_records/helpers.rs:147-232` | PASS |
| normal feed commits membership and visualization before synchronous event; replacement; removal retains record; wrong type/missing set/no-op removal preserve state and emit no events | `tests/widget_subtext/feed.rs:9-131`; `src/c_api/ui_widget_manager/feed.rs:69-115,188-190` | PASS |
| real cached provider type25/container registration; initial two lines, fonts/colors/alignment/state, dimensions/anchors/spacing, tooltip/scale/order/tag | `tests/widget_subtext/consumer.rs:10-50`; `tests/widget_subtext/register.lua:1-26`; `tests/widget_subtext/consumer_assertions.lua:2-52` | PASS |
| in-place same frame updates both lines/style/geometry; getter replacement tag but acquired frame tag retained; GatherWidgetsByWidgetTag old=1/new=0 | `tests/widget_subtext/consumer.rs:56-75`; `tests/widget_subtext/consumer_assertions.lua:54-67` | PASS |
| Hidden releases/hides frame while retaining membership/getter; Shown reacquires updated tag and GatherWidgetsByWidgetTag new=1; old snapshot intact | `tests/widget_subtext/consumer.rs:78-107` | PASS |
| membership removal empties container, hides frame, retains getter record/snapshot | `tests/widget_subtext/consumer.rs:110-124` | PASS |
| resupply live frame then unregister clears registered set/manager/events/map/pool/timer while retaining supplied membership/getter | `tests/widget_subtext/consumer.rs:126-149`; `tests/widget_two_records/helpers.rs:234-251` | PASS |
| unchanged lifecycle controls: unit filtering; icon dynamic states; refresh/remove/unregister active widget/row pools, subscriptions and timer | `tests/ui_widget_manager_lifecycle.rs:210-572` | PASS |

All references above are relative to `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-tag-lifecycle-green-20261014T175706489371Z/source`. Exact archived source text included in `proof.json`; inspected eleven key implementation/assertion files match committed `62f115d262613a848180143d6b30e2a0b3eb63b0` text without hashing. Supplied/replacement fixtures enumerate exactly 27 matching keys; all 27 values differ. Negative width/texture kits/animations in replacement are **projection-only** proof, not asset/animation activation proof.

## Cached acquisition-tag contract

Actual cache: `<private-cache>/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_UIWidgets`.
Literal source method is **`UIWidgetContainerMixin:CreateWidget`**, not a method named `Initialize`. It assigns `widgetFrame.widgetTag = widgetInfo.widgetTag` at `Blizzard_UIWidgetManager.lua:446`, during pool acquisition/creation (`438-446`). `ProcessWidget` calls it only for a newly acquired widget (`509-516`), then calls ordinary `Setup` (`527-528`). Type25 `Setup` (`Blizzard_UIWidgetTemplateTextWithSubtext.lua:12-33`) updates both lines/width/spacing/layout, not `widgetTag`. `GatherWidgetsByWidgetTag` compares cached frame tags (`364-369`).

Thus ordinary update keeps `subtext-contract-731` on the same frame while getter projects `updated-subtext-tag`; old-tag gathering finds one frame, replacement-tag gathering zero. Hide releases it; Shown reacquires with `updated-subtext-tag`, and gathering finds that frame. Updated assertions preserve this vendor behavior. No vendor or production/model patch between e37 and source62. Current cache source inspection is explanatory evidence, not sealed cache authentication.

## Receipt/status correction — PASS, bounded

Original `receipt-integration.json` retained with `status: authenticated`. Runtime helper required `PASS`: initial `runtime-subtext-final-eleven` directory remains empty, with no admission/argv/result/stdout launch artifacts. `status-schema-correction.json` records assertion failure **before admission/launch**, not an eleven-case failed run.

Explicit `seal-integration-ready.json` equals original seal. `receipt-integration-ready.json` preserves every original field except status, adding provenance/scope. No additional hash/payload read. Recorded digest `e17197dfa8c34c4c23ac56e168381103d0ad78fafc575858a359a3c5ff73e503` agrees across both seals/receipts. Exact path `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-tag-lifecycle-green-20261014T175706489371Z/artifacts/deps/integration-9f07e4a55f2bbbf7` agrees with runtime argv. Source/profile/features agree; stat-before/stat-after/runtime reused/current metadata agree: dev 66306, inode 147498095, size 472582088, mtime_ns 1792000676322993451, ctime_ns 1792000937265741453, mode 0o100700.

Argv uses `systemd-run --user --wait`, bounded timeout 86s, network-unshared bwrap read-only archived-source bind, exact sealed executable, two intended filters, `--nocapture --test-threads=1`. Admission allows 2GiB max / 1536MiB high / 200% CPU; result and launcher agree. Runtime receipt authentication is inherited, not freshly rehashed. This does not authenticate source/cache/libraries/dependencies.

## Earlier failures retained

| Run | Saved outcome | Boundary |
|---|---|---|
| Pre-implementation RED `0a3a5ac207e0b7698f80e61e548907346c80880c` | 1 PASS / 7 FAIL, exit 101 | Unknown-nil control passes; populated getter/required selector/secret cases fail. Consumer fails before cached setup: `supplied record must be a table`. |
| First producer e37 | 10 PASS / 1 FAIL, exit 101 | Initial activation passes; in-place update fails `Lua error: assertion failed!`; later lifecycle not reached. |
| Diagnostic `e948624b0b3691b626c49369f5f93b7d2c2c1ae6` | 0 PASS / 1 FAIL, exit 101 | `Lua error: updated tag/layout`; pinpoints erroneous update tag/layout expectation, not exact individual value. |
| Final source62 | 11 PASS / 0 FAIL, exit 0 | Acquisition-tag expectation corrected; lifecycle phases now reached. |

Historical full logs/results/argv/admission retained in `proof.json`; original files unchanged. Diagnostic and first-producer counts remain failures, not overwritten by final GREEN. Child exit 101 propagated by `timeout_reexec.rs:184:9` does not indicate elapsed timeout.

## Reused normal Retail observation — PASS, limited

Accepted `<private-state>/verification/widget-subtext-retail-partial-runtime-current/proof.json` observation reused **only after independently observing zero production diff** from e37 to source62: `git diff --name-status` over `src`, Cargo manifest/lock, build script, Interface and profile manifests exits 0 with empty output. Complete revision diff contains test assertions and docs/evidence only. Three lifecycle control source unchanged.

Saved e37 normal stdout `[]\n`, JSON `[]`, launcher exit 0; complete stderr 219 lines: **219 informational/summary, 0 warning diagnostics, 0 error diagnostics**. `Status: CLEAN`; `Lua errors: 0 unique, 0 occurrence(s)`; attributed owners 0, unattributed 0. Summary heading containing “error” is not a diagnostic.

CASC/sound/third-party addons/SavedVariables disabled. Font fallback loaded; read-only WTF EditMode cache still used; 291 Blizzard addons, 1569/1569 bytecode-cache hits, post-load workarounds and GC restart observed. No fresh normal binary or launch, no source62 normal-binary authentication, current HEAD/global startup, all-addon/rendering, PTR/native or parent-goal claim.

## Artifact audit / delivery

[EXIST] PASS — runtime, controls and archived assertions present.
[SUBSTANTIVE] PASS — complete projection, host feed, concrete behavioral fixtures.
[WIRED] PASS — `queries.rs:25` registers getter; typed host feed and actual cached provider/container execute.
[ANTI-PATTERN] PASS — inspected changed consumer files/getter have zero TODO/FIXME/HACK/XXX and no placeholder or vendor/provider replacement bodies.

`report.md` and `proof.json`: directory 0700, files 0600; file/directory fsync and exact-byte readback performed. **OVERALL PASS — saved Retail eleven-case scope only.**
