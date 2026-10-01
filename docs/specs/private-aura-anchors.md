# Public private-aura anchors

Public `C_UnitAuras.AddPrivateAuraAnchor` / `RemovePrivateAuraAnchor` must produce the anchor lifecycle consumed by `C_UnitAurasPrivate`, without manufacturing aura content. Current production owner is [`private_aura_state.rs`](../../src/lua_api/workarounds/temporary/private_aura_state.rs); only its private test helpers currently produce anchors. This checkpoint adds fixtures, not backing code. See [C API boundary](../../AGENTS.md#c-api-boundary) and [Lua API architecture](../lua-api.md).

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

## How it works

- [Lua API architecture](../lua-api.md)
- [Frame identity and data flow](../frame-data-flow.md)
- [Widget system](../widget-system.md)

## Implementation inventory

| Path | Role at input checkpoint |
| --- | --- |
| `tests/private_aura_anchors.rs` | New public-only behavioral fixtures; actual frame parents, no helper-produced anchors. |
| `tests/unit_auras_private.rs` | Existing helper and non-anchor controls; unchanged for initial RED. |
| `src/lua_api/workarounds/temporary/private_aura_state.rs` | Existing unconditional private bootstrap, anchor owners/helpers and mixed embedded test; unchanged. |
| `build.rs` / `Cargo.toml` | Existing grouped `integration` discovery with `autotests = false`; unchanged. |

### Later producer constraints (not implemented here)

User-required design: Rust C API record stores strings/scalars and frame IDs, **not unrooted `rilua::Val`**. `frame_ref` must retrieve the canonical rooted frame. Callback functions must be rooted in the Lua registry or existing namespace, not retained as unrooted values. Validate complete records through secret-aware, access-guarded reads before mutation; release the state borrow before dispatch. This is a producer handoff, not a claim about current architecture.

Replace exact old anchor ownership, not other aura models:

1. Retire `PrivateAuraState()` initialization/ownership of `state.anchors` and `state.nextAnchorID`.
2. Replace old `SetPrivateAuraAnchorAddedCallback`, `SetPrivateAuraAnchorRemovedCallback`, and `GetPrivateAuraAnchors` Lua publishers plus `_anchorAddedCallback` / `_anchorRemovedCallback` storage with rooted C API ownership.
3. Remove `_AddPrivateAuraAnchorForTest` and `_RemovePrivateAuraAnchorForTest` only after retained assertions migrate to public producers and pass. Do not leave parallel helper state or fallback producers.
4. Keep warning-frame/show-dispel/update callback owners, private aura lists/lookups and their controls. `CopyPrivateAuraValue` / `CopyPrivateAuraList` still serve those non-anchor paths; do not remove them just because anchor copying moves.

## Tests asserting this spec

`tests/private_aura_anchors.rs` is automatically included by `build.rs::discover_top_level_test_modules` / `should_include_in_integration_harness` in the existing `integration` binary. No standalone Cargo target is added.

| Fixture | Observable contract | Proof |
| --- | --- | --- |
| `ids_are_monotonic_and_environment_local` | Independent empty environments starting at 1, increasing IDs, no reuse | Written; unrun |
| `added_payload_and_listing_preserve_parent_identity_and_default_flags` | One callback argument, committed state, full required metadata/defaults, original frame, no aura content | Written; unrun |
| `nonempty_optional_bindings_publish_flattened_icon_dimensions` | Valid actual-frame bindings, true flags/container, scalar input isolation, optional border omission | Written; unrun |
| `listing_filters_units_and_isolates_callback_and_result_mutations` | Ordered all/unit lists, fresh records, callback/list mutation isolation | Written; unrun |
| `removal_has_no_results_and_reentrant_callbacks_observe_committed_state` | Zero results, exact IDs, missing-ID no-op, remove-before-callback, live-ID reentry | Written; unrun |
| `added_callback_can_remove_the_just_published_id` | Add-before-callback and meaningful removal reentry | Written; unrun |
| `callbacks_survive_gc_and_replacement_uses_only_latest_handlers` | Rooted closures after collection, latest added/removed handlers only | Written; unrun |
| `parent_identity_and_custom_fields_survive_gc_without_input_roots` | Canonical usable frame after collection | Written; unrun |
| `malformed_inputs_are_atomic_and_do_not_consume_ids` | Required/scalar validation, no callbacks/state/ID consumption | Written; unrun |
| `optional_icon_and_duration_bindings_validate_every_required_field_atomically` | Required nested fields and binding types, no partial registration | Written; unrun |
| `secret_outer_nested_and_scalar_inputs_reject_without_clearing_taint` | Conservative secret rejection at each consumed layer, atomicity/taint | Written; unrun |
| `secured_outer_and_nested_tables_respect_vm_access_guards` | Actual denied reads, nested guarded parse rejection, clean access | Written; unrun |
| `ordinary_public_add_and_remove_preserve_addon_taint` | No caller-taint laundering on ordinary lifecycle | Written; unrun |

### Retained assertion migration before helper removal

Old tests remain **unchanged** at this checkpoint. Parent must obtain actual public-method RED before a fresh producer starts; absence is source-observed, not a test result.

| Existing assertion owner | Assertions that must survive | Public destination / retained control |
| --- | --- | --- |
| `tests/unit_auras_private.rs::private_aura_anchor_callbacks_and_state_are_tracked` | Positive increasing IDs; added record ID/unit; all-list count and second ID; player-filter first ID; exact removed ID; one remaining second ID | New ID/payload/list/removal fixtures. Migrate helper calls to valid public args with actual parents; replace helper-boolean assertion with public zero-results assertion, keeping notification and post-removal content checks. |
| `private_aura_state.rs::tests::installs_private_aura_state_and_callbacks` anchor portion | Initial ID 1 and matching added ID; callback mutation does not alter unit; unit-filtered list; getter mutation isolated; matching removal callback | New public fixtures retain first-ID-1 in each fresh environment, progression/copy isolation/removal. This numbering contract is retained simulator policy, not native start-at-1 evidence. |
| Same embedded test non-anchor portion | Warning storage; available dispel notification/state; update source/spell and call count; all-private-aura mutation isolation; numeric/string private lookup mutation isolation | Keep as non-anchor embedded control after removing only anchor setup/assertions. Do not delete the mixed test wholesale. |
| `tests/unit_auras_private.rs` remaining tests | Warning-frame identity, available show-dispel state/callback, update callback/payload, configured aura reads and copy isolation | Leave unchanged; no anchor helper dependency to migrate. |

Old helper input extras `isBuff`, `maxAuras`, and `point` do not have retained value assertions in these tests and are not fabricated into the public args schema. Old helper success boolean is a helper-only contract, not the public remove return shape. Keep its equivalent state/notification proof, not an invented public boolean.

### Exact retained source evidence

[`12.0.5-register.json`](../../data/patch-api/sources/12.0.5-register.json) confirms these exact IDs and source-line references; none is edited or promoted:

| Exact source ID | Source line | Literal delta |
| --- | --- | --- |
| `global api-C_UnitAuras-AddPrivateAuraAnchor-359` | 359 | `- HasRestrictions` |
| `global api-C_UnitAuras-RemovePrivateAuraAnchor-401` | 401 | `- HasRestrictions` |
| `structures-AddPrivateAuraAnchorArgs-626` | 626 | `+ isContainer` |

The [retained source text](../../data/patch-api/sources/12.0.5-api-changes.txt) places method/structure names on preceding lines 358/400/625; register IDs refer to the delta lines, not the subject lines. Introductory retained text also describes restriction removal. This does not establish complete native 12.0.5 lifecycle/security semantics.

Local profile cache evidence, **12.1, not native 12.0.5**:

- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua`: Add lines 39–52, Remove 500–507. Add's documented numeric return is nullable; fixtures require a numeric ID only for accepted valid inputs. Both classify secret arguments `AllowedWhenUntainted`, neither lists `HasRestrictions`.
- Same generated directory: `UnitConstantsDocumentation.lua` lines 6–20 (`AddPrivateAuraAnchorArgs`), 32–40 (`PrivateAuraIconInfo`), 66–84 (`UnitPrivateAuraAnchorInfo`); `UISharedDocumentation.lua` lines 18–27 (`AnchorBinding`). Private payload exposes flattened dimensions and parent, not copied public nested bindings.
- `Blizzard_PrivateAurasUI/PrivateAuraInit.lua`: `AddPrivateAnchor(anchor)`, `RemovePrivateAnchor(anchorID)`, and zero-argument `GetPrivateAuraAnchors()` backfill show real consumption shapes. Existing simulator getter additionally accepts a unit filter; cache backfill alone does not prove a native filter parameter.
- `Blizzard_PrivateAurasUI/Blizzard_PrivateAurasUI.lua`: `GetBaseAuraSize` consumes flattened dimensions/border scale; `PrivateAuraUnitWatcher:AddAnchor` branches on `isContainer`; container initialization wraps `self.parent`. These ground metadata/identity assertions, not a full widget execution claim.

## Known gaps (current cycle)

- [ ] Parent executes targeted actual RED using existing grouped `integration` target, then assigns fresh producer; this session is forbidden from builds/checks/test runs/delegation.
- [ ] Producer implements C API lifecycle and migrates exact retained assertions before removing old owners/helpers; no backing code exists in this checkpoint.
- [ ] Producer/final verifier establishes GREEN, relevant old controls, formatting/compilation and normal startup after API registration changes under separately authorized scope. No runtime result is claimed here.
- [ ] Native untainted secret acceptance, private secure-only behavior and cross-profile parity remain unmodeled/unverified. Conservative rejection and existing private availability must not be reported as native security parity.

## Out of scope

- Full Blizzard widget/secure-environment execution: minimal prerequisites are not established. Fixtures prove dispatch payload/state, not `PrivateAuraUnitWatcher` or complete UI integration.
- Aura content, spell data, sounds, warning/update models, container layout/settings, and other aura structure changes: independent owners, not required for public anchor registration.
- Shared wiki/index/log, coverage registers, PLAN, source/register status edits, production code, builds/checks/tests, push/deploy and delegation: excluded from this input checkpoint.
