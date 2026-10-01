# Private warning-text anchor

`C_UnitAuras.SetPrivateWarningTextAnchor(parent, optionalAnchorBinding)` configures the separately registered private warning text frame, not the public parent frame. This bounded Retail 12.0.5 input/test contract replaces the current no-op as a future producer goal. Source: retained [12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt), profile-scoped cached Blizzard API documentation and actual RaidWarning/PrivateAurasUI consumers. [Widget system](../widget-system.md) describes frame parenting and geometry.

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
| `src/lua_api/workarounds/temporary/unit_auras_state.rs` | Current public warning-anchor no-op; unchanged in this slice. |
| `src/lua_api/workarounds/temporary/private_aura_state.rs` | Private setter retains actual frame in `_state.warningTextFrame`; unchanged. |
| `tests/common/blizzard_addon_harness.rs` | Existing real TOC dependency closure loader, reused without changes. |
| `tests/private_warning_text_anchor.rs` | Ten input-only behavioral fixtures in existing grouped `integration` target. |

## Tests asserting this spec

`tests/private_warning_text_anchor.rs` is discovered by the existing generated integration harness; no new Cargo target or Rust model is introduced.

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

Parent RED filter: `cargo test --test integration private_warning_text_anchor:: -- --nocapture`. Minimal historical 12.0.5 compilation can use `--no-default-features --features profile-retail,retail-12-0-5`; nine direct fixtures remain enabled, cached consumer fixture is deliberately excluded because current cached consumers require 12.1 aura/widget capabilities. No compiled RED or acceptance claim is made here. Existing `tests/unit_auras_private.rs` warning-frame identity control remains untouched.

## Known gaps (current cycle)

- [ ] Parent must compile fixtures and establish assertion-level RED before changing the producer. Builds/checks/test execution forbidden for this slice; formatting only.
- [ ] Implement public backing state/placement and private-registration connection; current public no-op cannot satisfy placement fixtures.
- [ ] Characterize native parenting, precise registration/replacement timing, nil behavior and input-table snapshot policy. These defaults are best-supported inferences, not native observations.
- [ ] Characterize native `AllowedWhenUntainted` acceptance/security. Conservative rejection is only simulator policy.
- [ ] Cached dependency fixture has not run: closure/template prerequisites were inspected, not runtime-proven. It reports existing closure-load errors separately and asserts registration/geometry directly; it does not silently skip missing frames. Successful lifecycle assertions would not establish error-free whole-addon loading.

## Out of scope

- Warning message/event production, private raid-boss callback delivery, text rendering and visibility enforcement: producer work not authorized in this slice.
- Private setter validation changes or removal of `_state.warningTextFrame` identity compatibility: existing contracts retained; this slice exercises valid private frames only.
- Other private-aura APIs, 12.0.0 gating/parity, all profiles, full startup/suite acceptance, native security parity and audit status promotion: bounded tests/spec only.
- Vendor changes, callback impersonation, public-parent geometry rewrites and new fallback behavior: prohibited.

## Evidence and confidence limits

Paths below resolve under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`; cache inspection on 2026-10-01 is source evidence, **not native runtime proof**.

1. `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:535–544`: required `parent: SimpleFrame`, nilable `anchor: AnchorBinding`, `SecretArguments = "AllowedWhenUntainted"`. `UISharedDocumentation.lua:18–27` declares all five binding fields nonnil, with `relativeTo: ScriptRegion`. No return declaration. Argument shape is sourced; target selection and nil action are not documented here.
2. `Blizzard_RaidWarning/RaidWarning.lua:354–363`: public anchor OnLoad calls the public setter with `parent = self` and binding `TOP/self/TOP/0/0`. `RaidWarning.xml:27–34` already anchors that same public frame to `RaidWarningFrame`. Applying its binding to itself would destroy that placement through a self-anchor, so that interpretation is rejected. `RaidWarningUtil.lua:51–63` dynamically repositions the public anchor.
3. `Blizzard_PrivateAurasUI/Blizzard_PrivateAurasUI.xml:55–61` creates distinct `RaidBossEmoteFramePrivate` inheriting `RaidWarningFrameTemplate`; `.lua:1332` registers `self` through `C_UnitAurasPrivate.SetPrivateWarningTextFrame`. The separate actual-frame registration supports, but does not native-verify, the inferred target policy. The existing private slot only stores identity today.
4. Real dependency inspection: `Blizzard_RaidWarning.toc` requires `Blizzard_FrameXMLUtil` and `Blizzard_EditMode`; its public anchor also inherits `PingTopLevelPassThroughAttributeTemplate` defined in `Blizzard_SharedXML/PingAttributes.xml:10`. `EditModeRaidWarningSystemTemplate` is defined in `Blizzard_EditMode/Shared/EditModeSystemTemplates.xml:393`. `Blizzard_PrivateAurasUI.toc` depends on SharedXMLGame, FrameXMLUtil, GameTooltip, RaidWarning and VisualAlerts. `DeadlyDebuffFrame` comes from `Blizzard_BuffFrame/BuffFrame.xml:139`, hence the additional real BuffFrame root for utility repositioning. No synthetic callbacks/templates are supplied.
5. Retained [register](../../data/patch-api/sources/12.0.5-register.json) exact IDs `prose-2026-03-31-168` and `global api-C_UnitAuras-SetPrivateWarningTextAnchor-405`: March 31 prose says restrictions were removed; consolidated delta is `- HasRestrictions`. This supports absence of the recent restriction, not undocumented validation, nil, rendering or native secrecy claims. Retained source line 168 distinguishes still-restricted `AddPrivateAuraAppliedSound` from this API.

Source correction received in `/tmp/patch-12.0.5-warning-anchor-source-correction.md` rejects public-parent SetPoint behavior. Its durable evidence and explicit unknowns are recorded above; the temporary note is not required to interpret this spec.
