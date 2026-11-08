# Saved PTR final runtime — PASS, bounded

Source `62f115d262613a848180143d6b30e2a0b3eb63b0`; production delta `e37f5426a`.
Epoch: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-ptr-final-20261014T181140348741Z`.

Saved-artifact verification under `<private-agentconfig>/skills/verify/SKILL.md`. No builds, tests, reruns, delegation, canonical edits, executable reads/hashes or executable copies. Both full runtime stdout/stderr and argv/admission/result/launcher files inspected; exact content retained in `proof.json`.

## Independent case count and exit proof

`runtime-subtext-ptr-final-eleven`: **11 unique cases PASS, 0 FAIL**, comprising **8 subtext (including actual-secret selectors) + 3 lifecycle controls**. Names independently extracted from child `test … ok` records and matched one-to-one to archived test functions.

Main summary: `ok. 11 passed; 0 failed; 0 ignored; 0 measured; 10232 filtered out; finished in 11.95s`.
Eleven nested re-execution summaries each report `1 passed; 0 failed; 10242 filtered out`; **nonadditive: not 22 tests**. Result launcher exit 0 and launcher `code=exited, status=0/SUCCESS` agree. Saved service runtime 12.026s, CPU 11.367s, memory peak 1.4G, swap 0B. Integration stderr is 132 informational startup lines, no warning/error diagnostic.

- PASS `ui_widget_manager_lifecycle::cached_ui_widget_container_filters_supplied_unit_updates`
- PASS `ui_widget_manager_lifecycle::cached_ui_widget_container_refreshes_removes_and_unregisters_supplied_widgets`
- PASS `ui_widget_manager_lifecycle::cached_ui_widget_icon_and_text_dynamic_states_toggle_acquired_child`
- PASS `ui_widget_text_with_subtext::consumer::text_with_subtext_cached_type25_normal_activation_update_hidden_shown_remove_unregister`
- PASS `ui_widget_text_with_subtext::feed::text_with_subtext_normal_feed_publishes_complete_state_before_synchronous_events`
- PASS `ui_widget_text_with_subtext::feed::text_with_subtext_wrong_feed_type_or_missing_set_changes_neither_state_nor_events`
- PASS `ui_widget_text_with_subtext::queries::text_with_subtext_exact_27_fields_fresh_snapshots_and_all_field_replacement`
- PASS `ui_widget_text_with_subtext::queries::text_with_subtext_required_selectors_reject_missing_and_invalid_arguments`
- PASS `ui_widget_text_with_subtext::queries::text_with_subtext_two_environments_keep_records_and_replacements_isolated`
- PASS `ui_widget_text_with_subtext::queries::text_with_subtext_unknown_returns_exactly_one_nil_control`
- PASS `ui_widget_text_with_subtext::security::text_with_subtext_actual_secret_selectors_allow_secure_deny_tainted_preserve_state`

## Reached behavior matrix

All references below resolve inside the epoch's archived `source/`, not current canonical source. Passing sequential type25 consumer case proves its later phases were reached; sibling controls are not substituted for those phases.

| Boundary | Concrete saved/source evidence | Proof |
|---|---|---|
| Complete populated projection | `widget_subtext/queries.rs`, `fixtures.rs`, `widget_two_records/helpers.rs`: exactly one table with exactly 27 typed fields/values; fresh tables; caller mutation isolation; all 27 replacement values differ; retained old snapshots | PASS |
| Required selectors and isolation | Queries: missing/nil/string/boolean/table/function/thread rejected; numeric edge controls yield one nil; populated fractional key accepted; two independent environments and crossed replacements | PASS |
| Normal producer ordering | `widget_subtext/feed.rs`: public event subscriber observes complete set/membership/record before synchronous update; replacement and removal; wrong type/missing set and absent removal preserve maps, emit no event | PASS |
| Actual cached activation | `consumer.rs:10-54`, `register.lua:1-25`: actual type25 provider registered, real XML container subscribed, empty set activated via host feed; acquired frame type/set/ID checked | PASS |
| Both texts/style/geometry | `consumer_assertions.lua:2-67`: initial heading/secondary and updated heading/secondary visible; font path/size/flags; white/green colors swap; alignments swap; measured heights/widths; parent TOPLEFT anchors and offsets; widths 180.5→206.25; spacing 3.5→7.25; rounded total height | PASS |
| In-place identity and acquisition tag | `consumer.rs:56-75`: same frame; getter tag becomes `updated-subtext-tag` while frame retains `subtext-contract-731`; `GatherWidgetsByWidgetTag` finds old=1/new=0 | PASS |
| Hide/reacquire | `consumer.rs:78-107`: Hidden releases/hides frame but retains membership and getter; Shown reacquires tag `updated-subtext-tag`; new-tag gathering finds 1; old snapshot intact | PASS |
| Remove/live unregister | `consumer.rs:110-149`: membership removal empties/hides container child, retains record; resupply live frame then unregister clears set/manager/events/map/pool/timer, preserves supplied membership/getter | PASS |
| Actual secret/tainted denial | `security.rs:7-33`, shared helper: actual rilua host-secret NUMBER wrappers, secure present/absent lookup; untainted public snapshot; tainted present AND absent denial before lookup; wrapper and cumulative taint preserved; public tainted query permitted; secure recovery; final Rust state maps unchanged | PASS |
| Three unchanged controls | `ui_widget_manager_lifecycle.rs`: unit-filtered updates, refresh/remove/unregister pools/events/timer, dynamic child visibility | PASS, bounded fixture semantics |

27-field replacement with nondefault texture kits, hasTimer, animations, model layer and negative width is **projection-only**, not proof every native field effect is activated. Consumer uses empty texture kits, no timer/animations/model scene. Unit control injects `attachedUnit`; not native registration proof. Dynamic icon control checks visibility, not flash timing.

## Archived wiring and actual PTR cache selection

[EXIST] PASS — archived implementation, fixtures/assertions, launch controls and both runtime output streams present.
[SUBSTANTIVE] PASS — explicit complete record/map, populated projection and concrete host-feed/cached-consumer assertions, not callable-placeholder proof.
[WIRED] PASS — `src/lua_api/globals/missing_surface.rs:258` calls manager query registration; `src/c_api/ui_widget_manager/queries.rs:25` registers subtext getter; typed feed inserts supplied record (`feed.rs:188-190`). `tests/integration.rs` invokes generated harness; `crates/test-harness/build.rs` discovers top-level test modules, and subtext module selects consumer/feed/query case files plus actual-secret file under `retail-12-1-0`.
[ANTI-PATTERN] PASS — scoped getter/record/feed/assertions are substantive and do not substitute providers, templates, handlers or font APIs. No vendor behavior patch inferred or required.

Both reused authenticated artifacts carry actual Cargo `client-ptr`, **not client-retail**, with `retail-12-1-0`, `retail-12-1-5` and forbidden-aspect features. Retail-named epoch features are API features, not evidence of selecting the Retail client profile.

Consumer obtains `paths::default_blizzard_ui_addons_path` (`paths.rs:73-81`), which resolves profile cache (`blizzard_ui_sync.rs:41-63`), with `ClientProfile::Ptr` active and `cache_subdir() == "ptr"`. Expected standard-home path: `<private-cache>/wow-ui-sim/blizzard-ui/ptr/AddOns`. Completeness, pinned PTR identity and required-file checks gate cache selection. Recorded argv mounts archived source read-only, keeps HOME `<private-home>`, and supplies no alternate cache argument. No legacy repo-local Blizzard tree is selected. Actual consumer uses dependency closure roots SharedXMLGame, Fonts_Shared, FrameEffects, UIWidgets, with empty overrides; passing acquired-template assertions demonstrate loaded provider/template execution. This is source/feature/runtime-path evidence, not sealed cache-byte authentication or a printed absolute-path trace.

Current PTR vendor inspection corroborates tag behavior: `Blizzard_UIWidgetManager.lua:438-446` **CreateWidget** assigns acquisition tag; ProcessWidget uses CreateWidget for a new frame then ordinary Setup (`:526`); `GatherWidgetsByWidgetTag` compares stored tags (`:364-369`). Type25 Setup updates both lines, width, spacing and layout, not tag. Cached XML uses `UIWidgetBaseTemplate, VerticalLayoutFrame`, with Text/SubText layout indices 1/2. Current cache inspection is explanatory only; no historical cache/library/settings authentication claimed.

## Exact valid-hash-once receipt reuse

Reused `<private-state>/verification/widget-subtext-ptr-auth-current/proof.json` and epoch's flat `receipt-integration.json` / `receipt-wow-sim.json`; each exactly matches prior authenticated receipt, status **PASS**, integer `hash_reads=1`, `sha256_matches_seal=true`. No second hash stream or executable payload read. Each receipt path equals its seal's sealed_path and runtime argv executable; source/profile/features agree. Receipt before/after stats equal runtime reused stats and current metadata-only stat; mode remains 0700.

- integration SHA-256: `8a581aaa25055c8f763ec098e6f6408ab95fe8739ca0812dd3f51aab15cc90fc`; exact path `<private-state>/verification/widget-discrete-progress-current/widget-subtext-ptr-final-20261014T181140348741Z/artifacts/deps/integration-020202567c2bd5a4`.
- wow-sim SHA-256: `d8d18c0ac250022117810d2a67c4e155c1c98434719d762b8a2d5f6c5a13467c`; exact path `<private-state>/verification/widget-discrete-progress-current/widget-subtext-ptr-final-20261014T181140348741Z/artifacts/wow-sim`.

Launch argv: systemd-run user/wait, network-unshared bwrap, read-only archived source plus target bindings, timeout TERM 86s/kill-after 2s, exact sealed artifact. Integration filters subtext + lifecycle, nocapture, one test thread. Admissions allowed=true: integration max 2GiB/high 1536MiB/CPU 200%; normal max 4GiB/high 3GiB/CPU 200%. Recorded resultpending labels are pre-run scope text, not terminal results; complete stdout and launcher establish terminal success. Saved metadata binds source; no independent source/dependency/archive cryptographic provenance claim.

## Full normal PTR diagnostics — PASS, limited

`runtime-normal-ptr-post-subtext`: stdout exactly `[]\n` (3 bytes), JSON empty array; result launcher exit 0; process status 0/SUCCESS. Saved runtime 8.916s, CPU 7.847s, peak 1.9G/swap 0B. **219 stderr lines: 214 information + 5 final-summary, 0 warning diagnostics, 0 error diagnostics.** Full classification and exact lines preserved in proof JSON. Summary heading containing “error” is not an error diagnostic.

Final summary: `Status: CLEAN`; `Lua errors: 0 unique, 0 occurrence(s)`; attributed owners 0; unattributed Lua errors 0. CLEAN is this observed Lua-error summary, **not global clean startup**.

Information includes resource/startup timing, **283 Blizzard addons**, 2481 frames, **1568/1568 bytecode cache hits**, applied simulator post-load workarounds and GC restart. `WOW_SIM_CASC=0`, `WOW_SIM_NO_SOUND=1`, `--no-addons --no-saved-vars`: CASC/sound/third-party addons/ordinary SavedVariables disabled. Font fallback loaded (`casc=false`). **Read-only WTF EditMode cache still used**, <WTF-account>, <WTF-character>/<WTF-character>. NoSavedVars does not imply no WTF state. Warm cache, fallback fonts and disabled assets limit this observation; not rendering/assets/sound/all-addon/native/global-startup proof.

## Retail and schema failures remain separate

No Retail runtime observation inherited for PTR. Preserve original reports/records unchanged:

| Historical Retail evidence | Outcome | Boundary |
|---|---|---|
| `widget-subtext-populated-red-current` source0a3 | 1 PASS / 7 FAIL, exit101 | Six populated lookup failures + selector validation failure; consumer fails before cached setup |
| `widget-subtext-retail-partial-runtime-current` productione37 | 10 PASS / 1 FAIL, exit101 | Initial activation reached; in-place update fails; later hide/reacquire/remove/unregister not reached |
| Diagnostic sourcee948, retained by `widget-subtext-retail-final-runtime-current` | 0 PASS / 1 FAIL, exit101 | `updated tag/layout` assertion, not a PTR failure |
| Retail receipt-schema prelaunch failure, retained by same report | No admission/launch/test result | original status `authenticated` rejected by helper requiring PASS; empty runtime directory and explicit correction preserved; not eleven failed tests |

Retail final source62 success is independently reported elsewhere and is not additive or substituted here. Historical child exit101 is assertion failure propagation, not evidence of elapsed timeout. Default source-check/format and compile warnings remain their own gates, not normal-runtime diagnostics.

**OVERALL PASS — saved bounded PTR eleven-case behavior + this exact normal PTR Lua-error observation only. NOT native parity, all-native semantics, other-profile proof, parent completion, current HEAD or full-suite GREEN.**
