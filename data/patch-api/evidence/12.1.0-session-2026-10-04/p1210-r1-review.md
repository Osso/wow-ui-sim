# 12.1.0 round-1 independent source-read review

Revision reviewed: `102d7e9be` (master). Eleven commits: `05ca8c869`, `31ba71ee0`, `746602840`, `6a99cb20d`, `4aa79d193`, `e46c4aaaa`, `ba7f5b560`, `56df503da`, `a6e723d5a`, `a55abd62a`, `07b05bb81`.

Source-read only; no cargo, tests, builds, simulator, agents/model CLIs, or repository edits performed. Verdicts assess public-API assertions and stated bounded scope, not fresh execution or native parity. Reported green results are integrator evidence, not independently rerun. `CACHE/` = `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`.

## Row matrix

| row id | verdict | one-line reason with file:line |
|---|---|---|
| prose-undated-009 | bounded-coverage | Both intrinsic types construct with nondefault sizes, parents and visibility; reused real-template initializer adds region/anchor behavior; managed secrecy/pixels excluded. tests/patch_12_1_0_aura_creation.rs:13; tests/forbidden_aspect_creation.rs:117 |
| prose-2026-06-18-037 | bounded-coverage | Bounded customization/creation only: real icon/cooldown/count/border setup, not aura assignment or pixels. tests/forbidden_aspect_creation.rs:117; tests/patch_12_1_0_aura_creation.rs:18 |
| prose-2026-07-23-207 | bounded-coverage | Tainted addon closure constructs container under asserted combat lockdown and reads concrete frame state. tests/patch_12_1_0_aura_creation.rs:8-29 |
| prose-undated-011 | bounded-coverage | Real disk body executes once; missing/disabled loads show actual dialog, repeat error suppressed and old helper absent; no native parity claim. tests/patch_12_1_0_framexml_migrations.rs:37-70 |
| prose-undated-012 | bounded-coverage | Real cursor inside/outside plus unequal offset and hidden-frame checks exercise relocated helper; scaled/native offset semantics excluded. tests/patch_12_1_0_framexml_migrations.rs:13-33 |
| prose-2026-06-18-055 | bounded-coverage | Reused startup discovery/dependency and phased lifecycle tests assert Lua exports, exact file ordering, loaded/finished and one-time subsequent load; disabled path excluded. tests/load_order.rs:471,558,604,630 |
| prose-2026-06-18-056 | audit-pending | Disabled-startup behavior IS asserted in binary test, but no executed proof supplied for that target; C02 disabled full-load test is not startup. src/bin/wow_sim/addon_loading/tests.rs:190-237 |
| prose-2026-06-18-057 | partial-development-green | Reused synthetic bootstrap/dependency fixture proves mechanism, not actual UIParent-to-addon migration; cannot promote source migration claim. tests/load_order.rs:471-552; docs/specs/patch-12-1-0-bootstrap.md:8-10 |
| prose-2026-06-18-058 | bounded-coverage | Reused dispatch tests count concrete callbacks under all modes and reset/rearm; managed dirty-phase failure not credited. tests/on_update_modes.rs:7,35,82,125 |
| prose-2026-06-18-059 | bounded-coverage | Reused tests assert exact five values/default, invalid rejection and visible/hidden/one-shot dispatch, not constants alone. tests/on_update_modes.rs:6-80 |
| prose-2026-06-18-064 | bounded-coverage | Two disk addons assert private payload identity, shared live mutation on literal/runtime frames and isolation; invalid-path/security combinations excluded. tests/patch_12_1_0_xml.rs:20-53 |
| prose-2026-06-18-065 | bounded-coverage | Mixins block installs callable concrete methods with last-entry precedence on literal/runtime templates; no shape-only assertion. tests/patch_12_1_0_xml.rs:25-43 |
| prose-2026-06-18-066 | bounded-coverage | Local nested mixins read distinct private values across two addons without global leakage; exhaustive security excluded. tests/patch_12_1_0_xml.rs:17-53 |
| prose-2026-06-18-067 | bounded-coverage | Qualified global attribute and local block resolve nested methods, execute and observe private mutation; invalid lookup paths excluded. tests/patch_12_1_0_xml.rs:9,25-43 |
| prose-2026-06-23-083 | bounded-coverage | Five reused generic-aspect tests reject seven mutations in secure/tainted contexts and preserve listener/callback delivery with zero-mask controls; not intrinsic container coverage. tests/forbidden_aspect_creation.rs:155-351 |
| prose-2026-06-23-088 | audit-pending | Generic manually-applied aspect cannot prove intrinsic application; archived real-container test reports mask 0 and cached intrinsic XML declares no aspect. data/patch-api/evidence/12.1.0-session-2026-10-04/p1210-r1-result.md:60-96; CACHE/Blizzard_AuraContainer/Blizzard_AuraContainer.xml:5-20 |
| prose-2026-06-23-090 | bounded-coverage | Reused native_children_inherit_only_hierarchy_aspects_at_creation rejects SetPoint/SetParent acquisition and preserves zero baseline; other bits/nonempty tuples excluded. tests/forbidden_aspect_creation.rs:14-48 |
| prose-2026-07-07-119 | bounded-coverage | Real ManagedAuraContainer consumes three native aura fixtures, shows filtered frame counts and updates assignments; plain intrinsic/combat/removal-event lifecycle excluded. tests/patch_12_1_0_aura_groups.rs:11-59 |
| prose-2026-07-07-120 | bounded-coverage | Two live groups have independent HELPFUL versus HELPFUL\|PLAYER filters and cap state; actual shown counts 1/2 then 3/0. tests/patch_12_1_0_aura_groups.rs:24-57 |
| prose-2026-07-07-122 | bounded-coverage | Tainted AddAuraGroup with concrete keys/options creates retrievable owned AuraButtons, rejects duplicate/empty key and returns zero values. tests/patch_12_1_0_aura_groups.rs:24-41 |
| prose-2026-07-07-123 | bounded-coverage | Arbitrary colon-bearing keys support lookup/update; unknown lookup returns nil and duplicate/empty keys reject; exhaustive string validation excluded. tests/patch_12_1_0_aura_groups.rs:24-31,48-51 |
| prose-2026-07-07-125 | bounded-coverage | Empty options and nondefault maxFrameCount produce distinct real shown counts; does not claim every option, security or rendered output. tests/patch_12_1_0_aura_groups.rs:24-25,47-57 |
| prose-2026-07-07-126 | bounded-coverage | Three nonempty input auras exceed cap 1; changing cap to 3 changes actual shown frame count 1 to 3 rather than preallocation count. tests/patch_12_1_0_aura_groups.rs:11-19,47-57 |
| prose-2026-07-07-127 | bounded-coverage | Unsorted fixture IDs/icons 33/11/22 become actual x-ordered 11/22/33 then 33/22/11; only AuraInstanceIDOnly method and two directions proven. tests/patch_12_1_0_aura_group_options.rs:74-124 |
| prose-2026-07-07-128 | bounded-coverage | Positive allocated count, unique frame identity and every-frame size/icon/anchor readback prove one callback per initial button with addon taint; later allocation/lifetime excluded. tests/patch_12_1_0_aura_group_options.rs:44-70 |
| prose-2026-07-07-129 | bounded-coverage | Two actual addon XML templates apply 37x29 size and alpha 0.5 before callback; base icon API remains usable, calls nonzero; arbitrary field partitions excluded. tests/patch_12_1_0_aura_group_options.rs:6-33 |
| prose-2026-07-21-181 | bounded-coverage | Reused modern/legacy removers delete selected seeded live IDs with zero Lua results; actual cached deprecated chunk retains tainted legacy forwarding; playback/acquisition/alias chronology excluded. tests/private_aura_sound_removal.rs:202-248 |

## Reused tests independently read (17)

| test and file:line | batch | observed assertion scope |
|---|---|---|
| `aura_button_icon_and_overlay_border_follow_real_initializer_order` — tests/forbidden_aspect_creation.rs:117 | C01 | Real CustomAuraButtonTemplate creates icon/cooldown/count/border regions, checks two-point anchor target and forbidden inheritance; no aura-data/pixel proof. |
| `bootstrap_owner_dependency_waits_for_full_load` — tests/load_order.rs:471 | C03 | Startup discovery loads real disk bootstrap only; Lua export/count and eager dependency order checked; public C_AddOns.LoadAddOn later creates visual dependency before owner. |
| `lod_bootstrap_lifecycle_publishes_once_before_full_load` — tests/load_order.rs:558 | C03 | Real annotated TOC exports concrete 42 once, records unfinished/full states and verifies subsequent full/repeated public loading. |
| `lod_bootstrap_lifecycle_preserves_mixed_stream_order` — tests/load_order.rs:604 | C03 | Exact mixed eager/bootstrap file sequence and public loaded/finished state; explicitly loader operations, not full startup wiring. |
| `lod_bootstrap_lifecycle_eager_files_keep_literal_toc_order` — tests/load_order.rs:630 | C03 | Exact eager annotated-file order and completion; no disabled scheduling claim. |
| `on_update_modes_publish_numeric_contract_and_default` — tests/on_update_modes.rs:7 | C04 | Exact five values/default plus Set/Get transitions and invalid-input rejection, not mere presence. |
| `on_update_modes_obey_ancestor_visibility_and_one_shots` — tests/on_update_modes.rs:35 | C04 | Exact callback counts across hidden/visible ancestor ticks; disabled/always/one-shot behavior asserted. |
| `on_update_modes_reset_before_handlers_and_preserve_rearming` — tests/on_update_modes.rs:82 | C04 | Callbacks/hooks observe reset before execution, rearm survives hooks, final exact two-call counts. |
| `on_update_modes_xml_names_select_numeric_modes` — tests/on_update_modes.rs:125 | C04 | Real XML hidden RunOnce frame executes once then resets disabled. |
| `event_registration_aspect_rejects_all_mutations_without_caller_exception` — tests/forbidden_aspect_creation.rs:155 | C06 | Seven mutations rejected across secure and stamped-tainted closures; positive zero-mask controls below guard against universal rejection. |
| `event_registration_aspect_preserves_existing_listener_delivery` — tests/forbidden_aspect_creation.rs:190 | C06 | Rejected clear/replacement leaves individual/unit/all delivery counts and unit filter unchanged on concrete events. |
| `event_registration_aspect_preserves_callbacks_when_replacement_is_rejected` — tests/forbidden_aspect_creation.rs:240 | C06 | Public FireEvent delivers original ordinary/unit callbacks; replacement count stays zero and original counts exactly one. |
| `event_registration_aspect_leaves_zero_mask_mutations_and_delivery_unchanged` — tests/forbidden_aspect_creation.rs:281 | C06 | Unrestricted registration/delivery/unregistration succeeds; later delivery counts unchanged. |
| `event_registration_aspect_leaves_zero_mask_callback_delivery_unchanged` — tests/forbidden_aspect_creation.rs:327 | C06 | Unrestricted callbacks fire once then stop after unregistration through public FireEvent. |
| `native_children_inherit_only_hierarchy_aspects_at_creation` — tests/forbidden_aspect_creation.rs:14 | C07 | Actual SetPoint/SetParent errors match acquisition boundary and leave zero anchors/foreign parent absent; only tested aspect subset. |
| `direct_modern_and_legacy_removers_share_real_state` — tests/private_aura_sound_removal.rs:202 | C10 | Modern RemoveAuraSound returns zero and changes {101,202} to {202}; legacy calls then empty same live ID set. |
| `actual_cached_deprecated_file_keeps_legacy_removal_durable_after_bootstrap` — tests/private_aura_sound_removal.rs:213 | C10 | Executes whole actual cached deprecated Lua; stamped legacy call preserves caller taint and removes selected ID, then empties remaining state. |

## Findings

1. **Keep L056 and L088 audit-pending.** L056 has a substantive binary startup test (`src/bin/wow_sim/addon_loading/tests.rs:190-237`), including disabled bootstrap producing only `A,C`, but no execution evidence in this round. L088 is a different intrinsic-contract gap: manually adding EventRegistrations in generic tests cannot prove it is present on AuraContainer. Cached intrinsic XML (`CACHE/Blizzard_AuraContainer/Blizzard_AuraContainer.xml:5-20`) lacks the declaration; archived failure reports mask 0 (`p1210-r1-result.md:56-96`). Source-read confirms pending status; no fresh runtime reproduction claimed. Modelable recommendation is reasonable, not a producer fix performed here.
2. **Keep L057 partial-development-green.** Synthetic bootstrap lifecycle/dependency fixtures are substantive but do not assert that real UIParent loading responsibilities moved into real addons (`tests/load_order.rs:471-552`). Integrator states that gap honestly; mechanism credit is not migration acceptance.
3. **C04 managed dirty-phase test remains failed/uncredited.** Test replaces phases with only flag 1, then shows real managed container (`tests/on_update_modes.rs:152-196`); integrator reports residual mask 18. Four separate native-mode behavioral tests support bounded L058/L059, not passing managed integration or entire suite. Merge risk: already-known test failure remains; these proof-only commits do not change its producer or fixture.
4. **No vacuous committed new test or weakened committed assertion found.** Positive frame/callback counts exclude empty-loop success (C08/C09); sorting checks three nonempty icons and strict x order. C02 disabled fixture registration corrects inventory precondition while keeping disabled-load assertions (`tests/patch_12_1_0_framexml_migrations.rs:64-70`). C09 archived failed public-KeyValue probe was replaced by independent observable template size/alpha proof, not a passing version of that original expectation (`tests/patch_12_1_0_aura_group_options.rs:11-33`; result file:139-180). Arbitrary-field ownership remains explicitly unproved.
5. **Bounded scope must remain literal.** C01 has construction/region setup, not managed aura assignment/pixels/security. C08 uses ManagedAuraContainer + CustomAuraContainerTemplate, and observer helpers unwrap secrets under untainted test access; it proves count/filter/cap behavior, not addon secret observability. C09 tests only AuraInstanceIDOnly sorting and initial allocation callbacks. C10 public Lua deletion is backed by direct host live-ID readback, not public acquisition/playback or historical intermediate alias removal. No producer changes in eleven reviewed commits; primary merge risk is overstating ledger coverage, not altered runtime behavior.

## Artifact verification (verify skill Mode B, source-read adaptation)

- **[EXIST] PASS:** all 15 changed artifacts read: ten specs, five added Rust integration files (eight new test functions; 345 Rust lines). Eleven diffs contain tests/specs only. Corrected C01 row date and producer inventory paths exist; ledger has exactly the stated 25 partial rows and two pending rows.
- **[SUBSTANTIVE] PASS:** concrete disk Lua/XML addons, nondefault cursor/frame/aura fixtures, actual Lua API effects/rejections and state readbacks; no placeholder test bodies. Reused tests independently read and named above. No new test substitutes source-substring/implementation-shape checking for behavior.
- **[WIRED] PASS (source):** `tests/integration.rs:1` includes generated harness; `build.rs:54-90,463-499` discovers top-level Rust test modules and excludes none of the five new files. Retail feature guards select these tests. Binary disabled-startup test is outside this integration harness; no runtime execution verified.
- **[ANTI-PATTERN] PASS (source):** all 15 changed files contain zero TODO/FIXME/HACK/XXX markers; no empty catch/silent exception, warning suppression or commented-out test implementation in added Rust. Known failed probes are documented as gaps, not disguised as green tests.

## Overall verdict

**ACCEPT WITH QUALIFICATIONS.** Accept proof-only commits and bounded recommendations for 24 rows; retain one partial-development-green (L057) and two audit-pending (L056/L088). Do not label all 27 rows complete or entire integration suite passing. This review supplies source-read acceptance only; no independent fresh runtime/build result.
