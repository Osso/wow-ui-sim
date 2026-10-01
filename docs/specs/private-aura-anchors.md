# Public private-aura anchors

Public `C_UnitAuras.AddPrivateAuraAnchor` / `RemovePrivateAuraAnchor` must produce the anchor lifecycle consumed by `C_UnitAurasPrivate`, without manufacturing aura content. Registration now lives in [`private_aura_anchors.rs`](../../src/c_api/private_aura_anchors.rs). The former private helper owner is removed; unrelated temporary private aura data/update/warning/dispel state remains. Independent inspection accepts bounded saved lifecycle and cached-consumer PASS; native/security/profile completion remains unproven. See [C API boundary](../../AGENTS.md#c-api-boundary) and [Lua API architecture](../lua-api.md).

## What it must do

### Public contract and payload

- [ ] Accept one `AddPrivateAuraAnchorArgs` record with required `unitToken` string, `auraIndex` number and actual `CreateFrame` parent; return a positive numeric ID for valid fixture inputs, monotonically allocated per environment, without reuse after removal, starting at 1 in each fresh environment. Allocation policy is retained simulator behavior, not native-verified numbering.
- [ ] Default `showCooldownFrame`, `showCooldownEdge`, `showCountdownNumbers`, `showDispelIcon`, and `isContainer` to false; preserve explicit true values independently of private aura content.
- [ ] Accept optional `iconInfo` only when its required `iconAnchor`, `iconWidth`, and `iconHeight` parse. `borderScale` is optional. A nonempty `AnchorBinding` requires `point`, actual frame `relativeTo`, `relativePoint`, numeric `offsetX` and `offsetY`; parse optional `durationAnchor` by the same contract. Fixtures cover frame receivers, not every ScriptRegion subtype.
- [ ] Publish a single anchor record to the added callback and fresh private getter records with `anchorID`, `unitToken`, `auraIndex`, identical canonical `parent`, display flags, `isContainer`, and flattened optional `iconWidth`, `iconHeight`, `borderScale`. Parent must remain a usable original frame, never a recursively copied table.
- [ ] Do not invent `spellID`, `spellId`, `auraInstanceID`, private aura lists, or lookup content when registering anchors.

### Lifecycle, snapshots and ownership

- [ ] Preserve `SetPrivateAuraAnchorAddedCallback(callback)` / `SetPrivateAuraAnchorRemovedCallback(callback)` replacement semantics. Added callback receives one record; removed callback receives one numeric ID. Registered closures survive forced GC.
- [ ] Commit state before callback dispatch. Added callback may remove the just-added ID; removed callback may query state, remove its own now-missing ID, and remove another live ID without duplicate dispatch or borrow failure. Synchronous ordering is simulator policy inferred from the existing helper.
- [ ] Public removal returns **zero Lua values**, unlike the old helper's boolean. Unknown numeric IDs are inferred no-ops with no notification; absent required argument remains invalid.
- [ ] Preserve ordered all-anchor listing and existing optional unit-token filtering. Return fresh list/record snapshots; mutation of input, callback payload, or getter results must not change stored scalar metadata. Frame identity is deliberately shared. Optional unit filtering is retained simulator behavior, not a native signature claim.
- [ ] Retain usable parent identity/custom fields through GC after dropping fixture input roots; retrieving a record must recover the canonical rooted frame.

### Validation and security

- [ ] Reject malformed required fields, invalid display-flag types, malformed optional structures, missing binding fields, invalid frame points/receivers, and nonnumeric dimensions/offsets with contextual errors. Validate everything before storing state, consuming an ID, or dispatching callbacks.
- [ ] Conservatively reject secret outer args, nested `iconInfo`/`iconAnchor`/`durationAnchor`, secret scalar fields and parent/binding frame receivers, and secret removal IDs without unwrapping or clearing caller taint. Fixtures exercise clean and addon-tainted callers.
- [ ] Honor actual VM table-access guards for outer and nested structures. Prove an addon cannot read each guarded fixture before testing API rejection; an untainted caller can still register the same guarded ordinary structure. Guard failure must not consume IDs or mutate anchor state.
- [ ] Ordinary public add/remove must preserve addon caller taint. Cache classifies both public methods `SecretArguments = "AllowedWhenUntainted"`; conservative rejection does **not** implement native acceptance of secret values on an untainted stack.

### Registration scope

- [ ] Preserve previous unconditional private namespace availability. Recommend unconditional public publication in the existing namespace, consistent with that owner: no new retail-only cfg walls absent actual profile-contract evidence. Retail-only publication is not an established requirement; historical/all-profile native parity remains unverified.

### Cached retail 12.1 consumer regression

- [ ] Under `retail-12-1-0` only, accept exact `UNIT_AURA_BLOCK_LIST_CLEARED` registration through `RegisterEvent` and `RegisterUnitEvent`; continue rejecting an unknown near-match. `RegisterAllEvents` must observe explicitly dispatched instances.
- [ ] Explicit `env.fire_event_with_args` delivery carries exactly one unit payload, reaches all-event receivers for player and target, and reaches a player-filtered receiver only for player. Assert counts immediately after each dispatch; this proves simulator dispatch, not a native producer.
- [ ] Load the actual cached `Blizzard_PrivateAurasUI` dependency closure and root TOC, including its final `PrivateAuraInit.lua`. Preserve its actual added/removed callbacks and vendor functions. Public container Add/Remove, same-parent player re-add, and player→target transition must produce no new Lua errors, retain public anchor metadata/original parent identity, and install/release the real container settings handler. Exercise that handler through `update-settings` without replacing it.
- [ ] Record closure-load errors separately before public lifecycle operations; assert each operation's new callback/handler errors before state assertions, preserving the observed unknown-event failure boundary.

## How it works

- [Lua API architecture](../lua-api.md)
- [Frame identity and data flow](../frame-data-flow.md)
- [Widget system](../widget-system.md)

## Implementation inventory

| Path | Role |
| --- | --- |
| `src/c_api/private_aura_anchors.rs` | Empty per-environment records, monotonic IDs, callbacks and fresh flattened DTOs. |
| `src/c_api/private_aura_anchors/input.rs` | Complete secret-aware and VM-guarded public/nested input validation. |
| `src/c_api/mod.rs`, `src/lua_api/globals/register.rs`, `src/lua_api/state.rs`, `src/lua_api/state/sim_state.rs` | Unconditional module/publication and per-environment model wiring, separate from gated aura enumeration. |
| `tests/private_aura_anchors.rs` | Public lifecycle fixtures; explicit delivery counts repair vacuous reentry PASS; flattened DTO exclusions. |
| `tests/unit_auras_private.rs` | Retained anchor assertions migrated to public producers; non-anchor controls unchanged. |
| `src/lua_api/workarounds/temporary/private_aura_state.rs` | Old anchor state/counters/setters/getter/callback slots/helpers removed; embedded anchor assertions migrated, unrelated controls retained. |
| `build.rs` / `Cargo.toml` | Existing grouped `integration` discovery with `autotests = false`; unchanged. |

### Ownership and publication boundary

Public names already existed through `runtime_surface_bootstrap.lua`'s `__wow_namespace_mt.__index`, which logs missing symbols and caches functions returning nil. Those are not lifecycle implementations. Static namespace stubs and the public aura registrar have no explicit anchor publishers. The old temporary bootstrap independently owned private anchors, counter, callback slots, getter and test-only producers. Concrete C API publication now runs after `auras::register_all` and before runtime namespace metatable installation; existing concrete slots bypass lazy stubs. Both public methods and all three private methods are unconditional; no retail/aura-enumeration cfg is introduced.

Records hold strings/scalars and canonical native frame IDs only. Parent and binding receivers use native backing identity, not overridable dispatch tokens. Existing `frame_ref` caches canonical tables in the rooted registry, pins frame tables without skipping traversal, and shares those tables with per-frame custom fields. No new frame deep copy or Rust-held permanent `Val` root is added. Callback replacement uses named Lua registry roots with GC barriers. Full parsing precedes mutation; model borrows end before synchronous callbacks. Bindings are captured for future `AnchorPrivateAura`, but registration does not apply layout/rendering and DTOs expose no nested input structures.

`CopyPrivateAuraValue` / `CopyPrivateAuraList` still serve unrelated private aura paths. There is no parallel anchor helper state or fallback producer.

## Tests asserting this spec

`tests/private_aura_anchors.rs` is automatically included by `build.rs::discover_top_level_test_modules` / `should_include_in_integration_harness` in the existing `integration` binary. No standalone Cargo target is added.

### Batch29 proof ledger — 2026-10-01

Inputs: `39dd9cce6` and import fix `7926dfdfe`. Parent artifacts `/tmp/patch-12.0.5-batch29-red-fixed-{revision.txt,build-result.json,runs.json,run-0.log}` bind actual compiled RED to `7926dfdfe19b2ee236f14f8084153b604dc1c4d5`. Command: `timeout 90 target/debug/deps/integration-a11e89d240f9bd0c private_aura_anchors:: --nocapture --test-threads=1`; binary SHA256 `c96aa74ceffe25c1461c5a97b73a35e322ad8babecc1d2a0175c0e6300b240c8`, exit 101, **1 PASS / 12 FAIL**. Fixture function assertions passed; failures reached behavioral assertions. Initial exit-101 build missing `LuaApiMut` is not behavior proof.

The lone PASS, `added_callback_can_remove_the_just_published_id`, was vacuous: nil-returning public stubs never delivered callbacks, `removed == id` compared nils and the empty-list check passed. The producer adds numeric ID and exact added/removed delivery counts plus final zero-record count. **No standalone RED exists for these added assertions**, nor for new nested-output exclusion assertions. Prior twelve failures remain historical missing-behavior proof; they do not validate current producer code. All table entries below describe producer proof, not that historical RED.

Parent `run-1.log` separately records seven `aura_table_shape::` controls PASS at the input revision; those are not current anchor GREEN. Saved parent GREEN covers thirteen fixtures below and one migrated integration control; independent acceptance and separate embedded-control execution remain unclaimed.

| Fixture | Observable contract | Producer proof |
| --- | --- | --- |
| `ids_are_monotonic_and_environment_local` | Independent empty environments starting at 1, increasing IDs, no reuse | Saved parent PASS; see reconciled proof |
| `added_payload_and_listing_preserve_parent_identity_and_default_flags` | One callback argument, committed state, full required metadata/defaults, original frame, no aura content | Saved parent PASS; see reconciled proof |
| `nonempty_optional_bindings_publish_flattened_icon_dimensions` | Valid actual-frame bindings, true flags/container, scalar input isolation, optional border omission | Saved parent PASS; see reconciled proof |
| `listing_filters_units_and_isolates_callback_and_result_mutations` | Ordered all/unit lists, fresh records, callback/list mutation isolation | Saved parent PASS; see reconciled proof |
| `removal_has_no_results_and_reentrant_callbacks_observe_committed_state` | Zero results, exact IDs, missing-ID no-op, remove-before-callback, live-ID reentry | Saved parent PASS; see reconciled proof |
| `added_callback_can_remove_the_just_published_id` | Add-before-callback and meaningful removal reentry | Saved parent PASS; see reconciled proof |
| `callbacks_survive_gc_and_replacement_uses_only_latest_handlers` | Rooted closures after collection, latest added/removed handlers only | Saved parent PASS; see reconciled proof |
| `parent_identity_and_custom_fields_survive_gc_without_input_roots` | Canonical usable frame after collection | Saved parent PASS; see reconciled proof |
| `malformed_inputs_are_atomic_and_do_not_consume_ids` | Required/scalar validation, no callbacks/state/ID consumption | Saved parent PASS; see reconciled proof |
| `optional_icon_and_duration_bindings_validate_every_required_field_atomically` | Required nested fields and binding types, no partial registration | Saved parent PASS; see reconciled proof |
| `secret_outer_nested_and_scalar_inputs_reject_without_clearing_taint` | Conservative secret rejection at each consumed layer, atomicity/taint | Saved parent PASS; see reconciled proof |
| `secured_outer_and_nested_tables_respect_vm_access_guards` | Actual denied reads, nested guarded parse rejection, clean access | Saved parent PASS; see reconciled proof |
| `ordinary_public_add_and_remove_preserve_addon_taint` | No caller-taint laundering on ordinary lifecycle | Saved parent PASS; see reconciled proof |

### Reconciled batch29 bounded proof — 2026-10-01

| Revision / saved artifacts | Result | Boundary |
| --- | --- | --- |
| Producer `61cd50cd8`; `/tmp/patch-12.0.5-batch29-green-*` | Thirteen anchors + one migrated integration PASS; startup 21 unique errors | Producer fixtures alone did not close cached callback integration. |
| Tests `f589f1107` + `c9e1696d7`; `/tmp/patch-12.0.5-batch29-consumer-red-fixed-run.{json,log}` | Three cached regressions FAIL at unknown exact event and container fixture nil `Symbol` | Actual RED, not three independent root causes. |
| Exact event correction `2b24386c5`; `/tmp/patch-12.0.5-batch29-consumer-green-runs.json`, `green-run-{0,1}.log` | Fifteen of sixteen anchors PASS; migrated integration PASS; only fixture nil `Symbol` remains | Thirteen producer fixtures and two event fixtures PASS. |
| Same revision; `/tmp/patch-12.0.5-batch29-consumer-green-startup{.json,-run.json,.log}` | Startup exit 0, `[]`, 5.53s | All prior 21 messages disappear after exact event correction; earlier handler message is not an independently established root cause. |
| Fixture `053f6c860`; `/tmp/patch-12.0.5-batch29-consumer-fixture-green-run.{json,log}` | Cached container lifecycle one PASS, exit 0, 1.84s | Separate targeted run, not a combined sixteen-test execution or startup rerun. |

Cached **12.1.0.69933**, not native 12.0.5, `UnitAuraDocumentation.lua:612–619` declares exact `UNIT_AURA_BLOCK_LIST_CLEARED`, synchronous metadata and one `unitTarget` payload. Correction `2b24386c5` adds the exact event to the retail-12.1 registry. Saved event tests prove both registration methods, unknown near-match rejection, all-event delivery and player filtering with immediate counts and exactly one payload. Explicit `env.fire_event_with_args` dispatch is **not native event production or historical 12.0.5 availability evidence**.

Fixture `053f6c860` loads the actual cached `Blizzard_BuffFrame` root to supply inherited `AuraButtonArtTemplate.Symbol`, without vendor overrides or a synthesized Symbol child. Actual `Blizzard_PrivateAurasUI` dependency closure/root TOC, including final `PrivateAuraInit.lua`, installs the real callbacks. Targeted PASS covers Add/Remove, same-parent player re-add, player→target transition, metadata/original parent identity, settings-handler installation, `update-settings`, teardown and no new phase errors. This repairs a fixture prerequisite, not production anchor behavior.

Metadata binds `2b24386c59e908f154b8b8c64efbc1afd6f5ebe3` integration binary SHA256 `b84c5a3ef8dc2ff839a5b3faffa2c79f0b68d0f50b891a9798ee94450b9f7cf9` and startup binary `eda8a709b6576882d80197597a7344ef6dae623a2dedf21179942695d2a849b7`. Fixture run binds `053f6c860ec553f83b0c959d9adafc881753a87d`, binary `91b46b705ffbe70cc38444a72f83b7120c7b26a344ee4f06f3853dbe789a4f8f`. Saved commands use `timeout 90`, grouped integration filters with `--nocapture --test-threads=1`; startup uses `wow-sim --no-addons --no-saved-vars lua-errors`.

Independent report `/tmp/patch-12.0.5-private-anchor-independent-proof.md` now gives **bounded PASS** after inspecting saved runtime artifacts, not rerunning behavior. Fresh default `cargo fmt --check` / `cargo check` each exit 0 at clean `053f6c860`; check has zero warnings/errors. Original sixteen distinct anchor cases have fifteen saved passes at `2b24386c5` plus one at `053f6c860`, not a single sixteen-test run. One migrated integration case and seven shape controls are separately reusable; embedded control execution remains unclaimed.

Callback-error tests authored at `f69497fc6` were actually compiled/run at **`91e8450213ebcad0515a6e876febd9912ed590d8`**: `callback_error_retains` **2 PASS**, exit 0, integration SHA256 `f373efb8bcfbe9990b72a53220902c9cefa5783af999b33b7efa8a228aaf3537`. Entire anchor-test file hashes match at both revisions. Added failure retains insertion/consumed ID without returning a success ID; removed failure retains deletion without duplicate retry notification; replacement callbacks and subsequent dispatch recover. This characterizes inferred simulator policy, **not native error/rollback semantics**, with no standalone RED. Separate pinned-file `rustfmt --check --edition 2024` exits 0; it does not extend whole-repo fmt to later revisions. Production/check/startup scope is unchanged. Incoming chat inputs **`f777027be` are explicitly excluded** from all these gates.

Only literal structure rows **626/675/676** gain bounded coverage: false/default and explicit true input/output `isContainer`, and canonical usable output `parent` in added callback/list snapshots, including GC identity. Exact assertions are `added_payload_and_listing_preserve_parent_identity_and_default_flags`, `nonempty_optional_bindings_publish_flattened_icon_dimensions`, and `parent_identity_and_custom_fields_survive_gc_without_input_roots`; cached container lifecycle is a further bounded control. Restriction rows359/401 remain unresolved. Accounting: **265 audit-pending / 83 bounded-coverage / 14 partial-development-green = 362**. IDs, retained text SHA256 and unrelated rows unchanged; audit **IN PROGRESS**, broad suite **NOT GREEN**. No fresh builds/tests/checks or delegation in this docs reconciliation.

### Retained assertion migration before helper removal

Both old assertion owners now call public producers with actual frame parents and required aura indices. Notification, ID, filter and snapshot assertions remain; helper boolean success becomes public zero-results plus state/notification proof. Migrated integration has saved PASS; separate embedded execution remains unclaimed. Public function absence was an incorrect initial source inference: the lazy namespace publisher exists.

| Existing assertion owner | Assertions that must survive | Public destination / retained control |
| --- | --- | --- |
| `tests/unit_auras_private.rs::private_aura_anchor_callbacks_and_state_are_tracked` | Positive increasing IDs; added record ID/unit; all-list count and second ID; player-filter first ID; exact removed ID; one remaining second ID | New ID/payload/list/removal fixtures. Migrate helper calls to valid public args with actual parents; replace helper-boolean assertion with public zero-results assertion, keeping notification and post-removal content checks. |
| `private_aura_state.rs::tests::installs_private_aura_state_and_callbacks` anchor portion | Initial ID 1 and matching added ID; callback mutation does not alter unit; unit-filtered list; getter mutation isolated; matching removal callback | New public fixtures retain first-ID-1 in each fresh environment, progression/copy isolation/removal. This numbering contract is retained simulator policy, not native start-at-1 evidence. |
| Same embedded test non-anchor portion | Warning storage; available dispel notification/state; update source/spell and call count; all-private-aura mutation isolation; numeric/string private lookup mutation isolation | Preserved unchanged alongside migrated public anchor setup/assertions. |
| `tests/unit_auras_private.rs` remaining tests | Warning-frame identity, available show-dispel state/callback, update callback/payload, configured aura reads and copy isolation | Leave unchanged; no anchor helper dependency to migrate. |

Old helper input extras `isBuff`, `maxAuras`, and `point` do not have retained value assertions in these tests and are not fabricated into the public args schema. Old helper success boolean is a helper-only contract, not the public remove return shape. Keep its equivalent state/notification proof, not an invented public boolean.

### Exact retained source evidence

[`12.0.5-register.json`](../../data/patch-api/sources/12.0.5-register.json) confirms these exact IDs and source-line references; source inventory is unchanged. Only the three structure rows gain bounded status in page coverage:

| Exact source ID | Source line | Literal delta |
| --- | --- | --- |
| `global api-C_UnitAuras-AddPrivateAuraAnchor-359` | 359 | `- HasRestrictions` |
| `global api-C_UnitAuras-RemovePrivateAuraAnchor-401` | 401 | `- HasRestrictions` |
| `structures-AddPrivateAuraAnchorArgs-626` | 626 | `+ isContainer` |
| `structures-UnitPrivateAuraAnchorInfo-675` | 675 | `+ isContainer` |
| `structures-UnitPrivateAuraAnchorInfo-676` | 676 | `+ parent` |

The [retained source text](../../data/patch-api/sources/12.0.5-api-changes.txt) places method/structure names on preceding lines 358/400/625/674; register IDs refer to the delta lines, not the subject lines. Introductory retained text also describes restriction removal. This does not establish complete native 12.0.5 lifecycle/security semantics.

Local profile cache evidence, **12.1, not native 12.0.5**:

- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua`: Add lines 39–52, Remove 500–507. Add's documented numeric return is nullable; fixtures require a numeric ID only for accepted valid inputs. Both classify secret arguments `AllowedWhenUntainted`, neither lists `HasRestrictions`.
- Same generated directory: `UnitConstantsDocumentation.lua` lines 6–20 (`AddPrivateAuraAnchorArgs`), 32–40 (`PrivateAuraIconInfo`), 66–84 (`UnitPrivateAuraAnchorInfo`); `UISharedDocumentation.lua` lines 18–27 (`AnchorBinding`). Private payload exposes flattened dimensions and parent, not copied public nested bindings.
- `Blizzard_PrivateAurasUI/PrivateAuraInit.lua`: `AddPrivateAnchor(anchor)`, `RemovePrivateAnchor(anchorID)`, and zero-argument `GetPrivateAuraAnchors()` backfill show real consumption shapes. Existing simulator getter additionally accepts a unit filter; cache backfill alone does not prove a native filter parameter.
- `Blizzard_PrivateAurasUI/Blizzard_PrivateAurasUI.lua`: `GetBaseAuraSize` consumes flattened dimensions/border scale; `PrivateAuraUnitWatcher:AddAnchor` branches on `isContainer`; container initialization wraps `self.parent`. These ground metadata/identity assertions, not a full widget execution claim.

## Known gaps (current cycle)

- [ ] Bounded independent PASS inspects saved parent behavior/startup, not independent runtime execution. Embedded controls and all-profile execution remain unestablished.
- [ ] Strengthened reentry counts and nested-output exclusions have saved GREEN but no standalone RED. Callback errors have bounded simulator characterization only; native rollback policy and frame destruction remain unproven.
- [ ] Rendering/anchor application remains absent despite retained parsed binding data. Full frame/ScriptRegion and all-profile native parity remain unverified.
- [ ] Native `AllowedWhenUntainted` secret acceptance, private secure-only behavior and cross-profile parity remain unmodeled/unverified. Conservative rejection and existing private availability must not be reported as native security parity. No combat lockout is introduced: retained source removes restrictions; that is not full native security proof.

## Out of scope

- Full Blizzard widget/secure-environment acceptance: new cached-consumer regression covers only synchronous container callback lifecycle with explicit inputs and independently inspected saved targeted PASS, not full-page acceptance. Timers, aura content/update rendering, complete `PrivateAuraUnitWatcher` behavior, and full startup/native acceptance remain excluded; bounded saved startup is recorded above.
- Aura content, spell data, sounds, warning/update models, container layout/settings, and other aura structure changes: independent owners, not required for public anchor registration.
- Only structure field rows626/675/676 may gain bounded coverage. Other source-status promotion, PLAN, chat files, code/tests edits, fresh builds/checks/tests/startup, push/deploy and delegation are excluded from this docs followup.
