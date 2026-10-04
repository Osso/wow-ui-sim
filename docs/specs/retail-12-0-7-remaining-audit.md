# Retail 12.0.7 remaining audit rows

Eighteen source rows in the [12.0.7 retained excerpt](../../data/patch-api/sources/12.0.7-api-changes.txt) cover duration helper objects, race discovery, model-unit identity, club events, timeline notification, mouse simulation and debugger secrecy. Round 10 integrates only rows 137 and 145 against master `55eb92602a379496a6de9d4fa4d90215c97c2f7d`; proof remains bounded by the feature set and inferred policies below. Later cached declarations are contract context, not authenticated build-68182 execution evidence. One row is modeled, one has an existing later producer needing proof, sixteen require missing authority/capability before implementation.

| Source suffix | Decision | Bounded contract |
|---|---|---|
| 111–114 | c — blocked | Exact DurationTextFormattingOptions declaration and creation route absent |
| 115–118 | c — blocked | Exact DurationTextRawValue declaration and creation route absent |
| 048 | c — blocked | Race getter signature and populated result schema absent |
| 137 | a — author | Non-rendering actor binding GUID getter; inferred policies below |
| 158–162 | c — blocked | ClubMemberOpaqueId representation and identity lifecycle absent |
| 145 | b — proving tests only | Existing 12.1.5 timeline notification; no strict-12.0.7 proof |
| prose 013 | c — blocked | Limited-input action/budget authority and focus predicates missing |
| prose 014 | c — blocked | Per-call-frame accessed-secret history and historical lifetime missing |

## What it must do

### Model-unit GUID — row 137

- [ ] Register `GetModelUnitGUID` only with feature `retail-12-0-7`; older builds retain absence. No renderer or vendor Lua changes.
- [ ] Read existing per-actor Rust `PlayerModelState.last_unit`, not a Lua field, Lua `UnitGUID` or replaceable `GetObjectType` method.
- [ ] INFERRED: resolve the bound token's current host identity on every call; native bind-time snapshot versus live token semantics are not established.
- [ ] INFERRED: an unbound actor or missing identity returns exactly one empty string. Cached declaration says nonnil WOWGUID but does not specify that default. No cached Lua consumer of this getter was located, so no observed consumer conflicts with this empty default; this is not native default evidence.
- [ ] Require a genuinely native-backed simulator actor and check its Rust `object_type_name`. Copying an actor identity token into a plain table does not authorize access.
- [ ] Return a public GUID even when the corresponding UnitGUID is conditionally secret. The source removes `ret1.ConditionalSecret`; do not substitute suppression or wrapped constants for that delta.
- [ ] INFERRED NotAllowed: cached getter omits input policy. Reject secret receiver and every extra for secure and tainted callers before any receiver validation. Public extras are ignored; no caller taint changes.
- [ ] Host identity changes and actor binding changes are observed live; actors and environments remain independent.

### Existing timeline color notification — row 145

- [ ] Prove existing Rust timer/layout transition emits exactly one public eventID when remaining time first reaches five seconds, with state visible during the listener callback.
- [ ] Prove empty timeline silence, no duplicate notification on later unchanged ticks, cancellation suppression and environment isolation through public Lua listeners.
- [ ] Prove a tainted listener receives the public payload without changing its taint while querying committed state.
- [ ] Keep proof explicitly scoped to `retail-12-1-5`, which transitively includes `retail-12-0-7`. Do not claim strict-12.0.7 availability or backport the later timeline lifecycle.

### Blocked rows

- [ ] Rows 111–118: obtain historical receiver/factory reachability, full method declarations, duration-type domain, defaults, units and secret annotations before authoring helpers. Current `DurationTextBindingFormatOptions` structure is not the historical `DurationTextFormattingOptions` script object.
- [ ] Row 048: obtain historical arguments, return arity, nilability, ordered populated race schema and secret policy before replacing the empty-table probe. A per-POI map without a known value schema is not a model.
- [ ] Rows 158–162: obtain opaque member-ID representation, equality/lifetime and native event tuples before replacing numeric guild-index identities. Do not infer string, userdata or numeric alias from the type name alone.
- [ ] Prose 013: obtain historical budget/action/focus rules and genuine host gamepad-action context before implementing four SimulateMouse APIs. No permanently-true context, generic event injection or taint-as-permission surrogate.
- [ ] Prose 014: obtain a VM-backed current/caller accessed-secret history with documented lifetime/reset semantics. Stack taint and currently secret locals are not access history; wrapping ordinary debug text based on either would not satisfy the source.

## How it works

- [Lua API](../lua-api.md)
- [Widget system](../widget-system.md)
- [Model assignment guard](model-unit-identity-guard.md)
- [Timeline lifecycle](encounter-timeline-script-core.md)

## Implementation inventory

- `src/widget/frame_types.rs` — existing optional actor unit token; no new state or parallel identity provider.
- `src/lua_api/frame/methods/widgets/model/model_unit_guid.rs` — proposed gated Rust getter, native receiver validation and secret rejection.
- `src/lua_api/frame/methods/widgets/model.rs` — proposed feature-gated module and method registration.
- `src/lua_api/globals/unit_misc.rs` — existing host-state GUID lookup reused without Lua calls.
- `src/c_api/c_encounter_timeline/notifications.rs`, `layout.rs` — existing later-epoch color notification and five-second transition.
- `src/loader/tests/wow_api_globals/startup_globals.rs` — existing absence assertion replaced by unbound value/arity assertion.

## Tests asserting this spec

- `tests/p1207_remaining_model_unit_guid.rs` — six authored cases: default/arity, live host mutation and Lua replacement immunity, binding/actor/environment independence, ConditionalSecret removal, native handle validation and all-argument secret denial for both caller contexts.
- `tests/p1207_remaining_timeline_color_event.rs` — four authored later-epoch cases: empty timeline, threshold/exact payload/no duplicate, environment isolation and cancellation; callback checks public payload, live state and taint preservation.
- Existing startup widget-compatibility fixture changes with the new getter. Duration factory-absence assertions remain unchanged while their rows are blocked.

## Known gaps (current cycle)

- [ ] Integration verification is limited to default-Retail targeted tests and compiler/format checks. The four timeline tests require `retail-12-1-5`, disabled by default; they remain authored but unrun. No alternate profiles, startup CLI or native execution are included.
- [ ] Row 137's default, token-versus-snapshot choice and input policy are INFERRED, not native conformance. ClearModel/rebinding lifecycle parity is not established by this getter proposal.
- [ ] Row 145's strict-12.0.7 lifecycle is missing. Existing 12.1.5 notifications do not supply that proof; color-setter/alpha/native UniqueEvent coalescing are not covered by these tests.
- [ ] Sixteen blocked rows retain audit-pending with exact missing-evidence notes in the authoring handoff. No tests should falsely close them by asserting absence, wrapped constants or name registration.

## Out of scope

- 3D rendering, geometry, camera, lighting, mesh or animation implementation: intentionally unsupported domain; row 137 is only non-rendering identity.
- Invented duration factories, race DTOs or opaque club-ID representations.
- Backporting the 12.1.5 timeline or creating a parallel 12.0.7 provider.
- Native/historical parity, automatic content discovery, service integration and page-accounting promotion without integrator proof.
