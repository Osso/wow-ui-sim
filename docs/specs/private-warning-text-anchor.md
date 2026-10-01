# Private warning-text anchor

## Exact row405 restriction-removal acceptance — 2026-10-01

[Source-axis proof and limits](../wiki/investigations/patch-12-0-5-api-audit.md#private-aura-restriction-removals--bounded-independent-acceptance) owns independent330 acceptance of actual tainted, in-combat public placement with zero arity, retained taint and exact stored geometry. This does not close native secret acceptance, rendering, cached closure errors or separate prose168.

`C_UnitAuras.SetPrivateWarningTextAnchor(parent, optionalAnchorBinding)` configures the separately registered private warning text frame, not the public parent frame. This bounded implementation replaces the public no-op and temporary private registration owner. Parenting, timing, replacement and nil policies remain inferred, not native-verified. Source: retained [12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt), profile-scoped cached Blizzard API documentation and actual RaidWarning/PrivateAurasUI consumers. [Widget system](../widget-system.md) describes frame parenting and geometry.

## What it must do

### Public contract and lifecycle

- [ ] Accept a required actual SimpleFrame parent and optional AnchorBinding; return zero results. Binding fields are `point`, `relativeTo`, `relativePoint`, `offsetX`, `offsetY`; `relativeTo` is an actual ScriptRegion and need not equal the parent.
- [ ] **Inference:** latest valid public parent/binding determines the registered private text frame's parent and sole explicit anchor. Never apply that binding to the public parent itself.
- [ ] **Inference:** retain the public binding when it arrives before private registration; apply immediately when it arrives after registration. Private registration alone, without a public request, preserves existing geometry.
- [ ] **Inference:** replacement public requests supersede pending/applied placement; replacement private frames receive the retained request. Superseded private frames retain their last geometry and no longer follow updates.
- [ ] **Inference:** snapshot accepted binding values; subsequent mutation of the input table does not rewrite the pending request. Retain usable frame references across GC.
- [ ] **Inference; native unknown:** nil/omitted binding clears explicit private points while retaining the supplied parent, including pending requests and private-frame replacement. It does not clear or reanchor the public parent.
- [ ] Keep requests and private registrations isolated between Lua environments.
- [ ] Preserve each public parent's parent, size and existing anchor points throughout registration, replacement, nil and rejected requests.
- [ ] Do not impose a lockdown/combat or untainted-caller guard on valid nonsecret inputs; March 31 restriction removal is the exact patch delta.

### Conservative simulator validation policy

- [ ] Reject missing/nonframe/nonsimpleframe public parents, malformed binding structures, missing required fields, invalid points, nonregion `relativeTo`, and nonnumeric offsets without coercion or partial placement/request changes.
- [ ] Reject secret public parent/binding/fields from both secure and tainted callers without declassifying inputs or changing caller taint. This is intentionally conservative simulator policy, **not** proof of cached `AllowedWhenUntainted` parity.
- [ ] A rejected public call preserves both the applied private geometry and retained request used by later private registration/replacement.

### Actual cached consumer

- [ ] Load real RaidWarning and PrivateAurasUI TOC/XML/mixin dependencies through the existing closure harness. Actual public OnLoad retains a binding relative to itself; actual private OnLoad registers distinct `RaidBossEmoteFramePrivate`. Assert private `GetParent`/`GetPoint` state, not callback or source shape.
- [ ] Actual `RaidWarningUtil.UpdateCenterScreenAnchors` repositions the public anchor through real `DeadlyDebuffFrame` shown/hidden states. The private frame remains bound to that moving public anchor; the public anchor never becomes self-anchored.

## How it works

- [Widget system](../widget-system.md): frame ownership and visibility.
- [Anchor resolution](../anchor-resolution.md): stored anchor points and layout.
- [Addon loading pipeline](../addon-loading-pipeline.md): real dependency and XML/script loading.
- [Private aura anchors](private-aura-anchors.md): related, separate public anchor registration contract.

## Implementation inventory

| Path | Current role |
|---|---|
| `src/c_api/private_aura_anchors/warning.rs` | Typed per-environment request/private-frame state and sole providers; preserves `_state.warningTextFrame` identity. |
| `src/c_api/private_aura_anchors/input.rs` | Shared strict, secret/access-aware binding parser; exposed without duplicating parsing. |
| `src/lua_api/frame/methods/button_anchor_hierarchy/` | Reused native parent mutation exposed for C API placement; visibility dispatch retained after complete geometry update. |
| `src/lua_api/workarounds/temporary/{unit_auras_state,private_aura_state}.rs` | Obsolete warning providers removed; existing unit controls use actual frames/correct binding. |
| `tests/common/blizzard_addon_harness.rs` | Existing real TOC dependency closure loader, reused without changes. |
| `tests/private_warning_text_anchor.rs` | Ten original behavioral fixtures plus hierarchy/callback/override and secured-table controls in grouped `integration` target. |

## Tests asserting this spec

`tests/private_warning_text_anchor.rs` uses the existing generated integration harness; no new Cargo target. The following table preserves fixture-authoring status; saved compiled RED and current producer proof are recorded below.

| Fixture | Exact coverage | Proof level |
|---|---|---|
| `public_binding_before_private_registration_applies_to_private_frame` | Public-first retention, GC, zero results, real private parent/point; public unchanged | Authored, not compiled/RED |
| `public_binding_after_private_registration_accepts_tainted_caller_in_combat` | Private-first placement, actual combat and tainted caller, zero results, taint preserved | Authored, not compiled/RED |
| `latest_public_parent_and_binding_replace_previous_placement` | Latest request before/after registration; separate parent/relativeTo identities | Authored, not compiled/RED |
| `replacing_private_frame_reapplies_retained_binding_without_moving_old_frame` | Replacement registration and subsequent update, superseded frame unchanged | Authored, not compiled/RED |
| `nil_binding_clears_private_points_and_retains_parent_in_both_call_orders` | Explicit nil before registration; omitted binding after; replacement private frame | Authored, not compiled/RED |
| `warning_anchor_bindings_are_isolated_between_environments` | Pending request in one environment cannot affect another's private registration/request | Authored, not compiled/RED |
| `retained_binding_is_a_snapshot_and_never_reanchors_public_parent_to_itself` | Exact cached self-relative pattern, input mutation, GC, original public geometry | Authored, not compiled/RED |
| `malformed_public_inputs_preserve_applied_and_pending_binding_atomically` | Parent/outer/required-field/type errors before/after registration; replacement sees unchanged request | Authored, not compiled/RED |
| `secret_public_inputs_preserve_binding_and_caller_taint_atomically` | Actual VM secret wrappers, both caller contexts, pending/applied/replacement atomicity | Authored, not compiled/RED |
| `cached_raid_warning_and_private_auras_lifecycle_places_distinct_private_frame` | Actual cached OnLoads and public utility repositioning; hidden private frame located through existing registration slot | Authored, not compiled/RED; requires `retail-12-1-0` cache capability |

Parent filter: `cargo test --test integration private_warning_text_anchor:: -- --nocapture`. Historical 12.0.5 compilation can use `--no-default-features --features profile-retail,retail-12-0-5`; cached lifecycle is excluded without 12.1 aura/widget capability. Existing `tests/unit_auras_private.rs` identity control remains untouched.

### Batch33 proof ledger — 2026-10-01

- Saved parent revision `303de9af7`: `cargo test --test integration --no-run --message-format=json`, exit **0**, 152.67s; artifact SHA-256 `6a9cf4abf3aed2bbfcd7648c822cd082e3095b46575dc33e5a938b8ffa681b33`. Evidence: `/tmp/patch-12.0.5-batch33-red-build-result.json`.
- Same artifact: `timeout 90 target/debug/deps/integration-a11e89d240f9bd0c private_warning_text_anchor:: --nocapture --test-threads=1`, exit **101**, **0 PASS / 10 FAIL**. Nine direct failures establish no-op/validation RED; cached assertion also fails, with separate `PingSystemTutorial` string.find closure errors preserved. Evidence: `/tmp/patch-12.0.5-batch33-red-run.{json,log}`.
- Producer now authored: native hierarchy primitive preserves alpha/scale/strata, child lists, dirty/hit-grid state; anchor replacement maintains dependency edges. Ordinary parent visibility callbacks run after full placement and retained state commit, without Lua override dispatch. Callback errors do not roll back committed placement, matching native primitive ordering rather than hiding errors.
- `placement_preserves_native_hierarchy_visibility_and_bypasses_method_overrides` and `tainted_access_to_secured_binding_is_rejected_before_placement_or_retention`: additional authored controls, **not compiled or run**. Cached fixture gains diagnostic assertion messages only; unrelated closure errors remain logged.
- Parent owns producer compilation/GREEN. No producer build, test, check, readability, broad-suite or startup acceptance performed in this implementation slice; earlier RED does not prove changed producer passes.

### Saved batch33 parent GREEN — 2026-10-01

Producer `dae082322085ed3c28dbef9302d0cc7996c79989`; bounded independent acceptance recorded below, no row promotion. Saved parent artifacts inspected only; no reruns.

| Saved parent scope | Result | Evidence |
|---|---|---|
| Default integration compilation | Exit 0, 335.99s | `/tmp/patch-12.0.5-batch33-green-build-result.json` |
| `private_warning_text_anchor::` | 12 PASS, exit 0 | `batch33-green-runs.json`, `batch33-green-run-0.log` |
| `private_aura_anchors::` | 18 PASS, exit 0 | Same run manifest, `batch33-green-run-1.log` |
| `unit_auras_private::` | 4 PASS / 1 FAIL, exit 101 | Same run manifest, `batch33-green-run-2.log` |
| Normal no-addons/no-saved-vars startup | Exit 0, stdout `[]`, 5.51s | `/tmp/patch-12.0.5-batch33-green-startup-run.json`, `batch33-green-startup.json` |

Run/log basenames above resolve under `/tmp/patch-12.0.5-`. Integration SHA-256 `c4fa7b32aaa1e28240ea14a8a62cc94093bd4b14a119acded3a8c787a2abdea7`; startup binary SHA-256 `f03b5d116732798cc3ac5eb12924b43879d1a19dddf8dc781f0b05d0f6cba09f`. The twelve warning fixtures include the ten historical authored fixtures above plus hierarchy/override and secured-table controls; their historical pre-compilation labels are superseded by this parent proof, with bounded independent acceptance below.

The failed control is `native_unit_event_dispatch_respects_unit_filter`: specialization-cast payload/unit-filter assertion fails. Retained ancestor `bff26b9c1` build/run manifests and log establish the same failure predates warning producer (4 PASS / 1 FAIL, unchanged failing fixture); root cause and clean immediate-parent reproduction remain unresolved. No broad GREEN claim. Cached warning lifecycle assertions PASS **while** separate `PingSystemTutorial` string.find closure-load errors remain logged. Normal startup `[]` does not establish error-free cached closure loading or resolve those errors.

Parenting, ordering, replacement, nil, snapshot and callback-error policies remain inferred simulator behavior; conservative secret rejection is not native `AllowedWhenUntainted` parity. Exact rows `prose-2026-03-31-168` and `global api-C_UnitAuras-SetPrivateWarningTextAnchor-405` remain pending. All 362 source IDs/hash and **264 pending / 84 bounded / 14 partial** totals unchanged.

### Independent bounded acceptance — 2026-10-01

Independent report `/tmp/patch-12.0.5-warning-placement-independent-proof.md` accepts bounded saved producer `dae082322` proof: **12 warning + 18 anchor + 4 private-unit PASS / 1 historically established specialization FAIL** (34 PASS / 1 FAIL). Fresh default fmt/check exit **0** at producer; normal saved startup exit **0**, `[]`. Cached `PingSystemTutorial` string.find closure errors persist; broad controls and clean cached closure remain **NOT GREEN**. Two function-length suggestions (`validate_placement`, `apply_placement`) deferred as nonbehavioral blockers, not zero readability findings. Parenting/order/nil/snapshot/security policies remain inferred or unknown; no native, full-row/page or all-profile acceptance. **264 pending / 84 bounded / 14 partial = 362**, source IDs/hash unchanged.

Gate manifests `/tmp/patch-12.0.5-warning-placement-independent-{fmt,check}.json` record start=end `dae082322`, fmt 19.476s/check 32.385s. Artifact/source hash audits in the same prefix corroborate saved manifest provenance, not hermetically archived environment inputs. Later docs and unrelated loot fixtures receive no runtime credit. No commands rerun for this reconciliation. Exact warning rows `prose-2026-03-31-168` and `global api-C_UnitAuras-SetPrivateWarningTextAnchor-405` remain pending.

### Availability and implementation limits

Both replaced providers were unconditional. Cached Mists and Forever consumers also reference warning placement; preserve that existing availability without a new retail epoch gate. No all-profile behavior proof. Public parents must be SimpleFrames; private registration accepts actual frames and preserves exact Lua identity. Invalid private values are rejected rather than retained as fake layout targets. No chat/combat/untainted-caller guard is introduced. Table-access and forbidden-aspect inheritance checks remain active; secret rejection is conservative, not `AllowedWhenUntainted` parity.

## Known gaps (current cycle)

- [x] Parent compiled producer and saved 12 warning PASS; original compiled RED retained above.
- [x] Independent bounded placement acceptance; saved 34 PASS / 1 historically established specialization FAIL retained, not broad GREEN.
- [ ] Deferred function-length suggestions for `validate_placement` and `apply_placement`; no demonstrated behavioral blocker.
- [ ] Characterize native parenting, precise registration/replacement timing, nil behavior and input-table snapshot policy. These defaults are best-supported inferences, not native observations.
- [ ] Characterize native `AllowedWhenUntainted` acceptance/security. Conservative rejection is only simulator policy.
- [ ] Resolve separate cached `PingSystemTutorial` closure-load errors; cached placement assertions now PASS, not error-free whole-addon loading.

## Out of scope

- Warning message/event production, private raid-boss callback delivery, text rendering and visibility enforcement: producer work not authorized in this slice.
- Native private-setter invalid/nil input semantics and removal of `_state.warningTextFrame` identity compatibility: unsupported; valid frame registration remains the bounded contract.
- Other private-aura APIs, 12.0.0 gating/parity, all profiles, full startup/suite acceptance, native security parity and audit status promotion: bounded placement only.
- Vendor changes, callback impersonation, public-parent geometry rewrites and new fallback behavior: prohibited.

## Evidence and confidence limits

Paths below resolve under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`; cache inspection on 2026-10-01 is source evidence, **not native runtime proof**.

1. `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:535–544`: required `parent: SimpleFrame`, nilable `anchor: AnchorBinding`, `SecretArguments = "AllowedWhenUntainted"`. `UISharedDocumentation.lua:18–27` declares all five binding fields nonnil, with `relativeTo: ScriptRegion`. No return declaration. Argument shape is sourced; target selection and nil action are not documented here.
2. `Blizzard_RaidWarning/RaidWarning.lua:354–363`: public anchor OnLoad calls the public setter with `parent = self` and binding `TOP/self/TOP/0/0`. `RaidWarning.xml:27–34` already anchors that same public frame to `RaidWarningFrame`. Applying its binding to itself would destroy that placement through a self-anchor, so that interpretation is rejected. `RaidWarningUtil.lua:51–63` dynamically repositions the public anchor.
3. `Blizzard_PrivateAurasUI/Blizzard_PrivateAurasUI.xml:55–61` creates distinct `RaidBossEmoteFramePrivate` inheriting `RaidWarningFrameTemplate`; `.lua:1332` registers `self` through `C_UnitAurasPrivate.SetPrivateWarningTextFrame`. The separate actual-frame registration supports, but does not native-verify, the inferred target policy. The existing private slot only stores identity today.
4. Real dependency inspection: `Blizzard_RaidWarning.toc` requires `Blizzard_FrameXMLUtil` and `Blizzard_EditMode`; its public anchor also inherits `PingTopLevelPassThroughAttributeTemplate` defined in `Blizzard_SharedXML/PingAttributes.xml:10`. `EditModeRaidWarningSystemTemplate` is defined in `Blizzard_EditMode/Shared/EditModeSystemTemplates.xml:393`. `Blizzard_PrivateAurasUI.toc` depends on SharedXMLGame, FrameXMLUtil, GameTooltip, RaidWarning and VisualAlerts. `DeadlyDebuffFrame` comes from `Blizzard_BuffFrame/BuffFrame.xml:139`, hence the additional real BuffFrame root for utility repositioning. No synthetic callbacks/templates are supplied.
5. Retained [register](../../data/patch-api/sources/12.0.5-register.json) exact IDs `prose-2026-03-31-168` and `global api-C_UnitAuras-SetPrivateWarningTextAnchor-405`: March 31 prose says restrictions were removed; consolidated delta is `- HasRestrictions`. This supports absence of the recent restriction, not undocumented validation, nil, rendering or native secrecy claims. Retained source line 168 distinguishes still-restricted `AddPrivateAuraAppliedSound` from this API.

Source correction received in `/tmp/patch-12.0.5-warning-anchor-source-correction.md` rejects public-parent SetPoint behavior. Its durable evidence and explicit unknowns are recorded above; the temporary note is not required to interpret this spec.
