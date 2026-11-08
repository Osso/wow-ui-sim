# Frame event removal

`Frame:UnregisterEvent(name)` must follow the explicitly selected simulator policy: remove existing registration state, rather than validate whether the name can be registered. This is model policy, not established native-client parity. Source lives in `src/lua_api/frame/methods/text_attribute_event/events.rs`; see the [event system](../event-system.md) for dispatch architecture.

## What it must do

- [x] On an ordinary frame, removing absent `PLAYER_EQUIPED_SPELLS_CHANGED` must not throw and must return exactly one `false` value, matching the selected cleanup policy for the cached `Base.lua:697` call.
- [x] Removing an absent synthetic unknown name must return exactly one `false` without installing that name or altering existing registrations; registered `PLAYER_LOGIN` must still deliver afterward.
- [x] Retail/PTR `RegisterEvent` and `RegisterUnitEvent` must retain strict existing registration gates: the synthetic unknown name must throw and remain unregistered, without disrupting known-event delivery.
- [x] Registered `PLAYER_LOGIN` must deliver before removal; its removal must return exactly one `true`, clear registration, and stop delivery. Repeated removal must return exactly one `false`.
- [ ] Preserve existing forbidden `EventRegistrations` aspect restrictions, callback API gates, unit-event filter behavior, and permissive Classic-profile registration policy. This scaffold does not establish their preservation.

## How it works

- [Event system](../event-system.md)
- [Frame unit-event filter contract](frame-unit-event-filter.md)
- [Forbidden event-input contract](forbidden-aspect-event-input.md)
- [Simulator-only event dispatch input](../admin-api/events.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/events.rs` — registration keeps catalog validation; removal queries/mutates existing listener state after the unchanged forbidden-aspect guard. Callback/unit-registration gates unchanged.
- `src/lua_api/globals/admin.rs` — registers existing `A_Admin.FireEvent`.
- `src/lua_api/globals/admin_events.rs` — existing synchronous modeled event-dispatch input used by the fixture, not a mock or native-client API.

## Tests asserting this spec

- `tests/fixtures/frame_unregister_unknown.lua` — reusable ordered behavioral fixture; first assertion checks the exact cached unregister call through `pcall`, followed by unknown-name return arity, strict registration rejection, and real registration/delivery/removal controls.
- `tests/frame_unregister_unknown.rs::unregister_unknown_preserves_known_event_delivery_and_strict_registration` — loads the same `include_str!` fixture in bare `WowLuaEnv`, enabled only for Retail/PTR.

## Current source checkpoint — 2026-10-13

At `621649e56`, deleting the shared `Op` enum leaves catalog validation on insertion only. Selected removal policy is state-based: absent/unknown names return false; `RegisterEvent`/`RegisterUnitEvent` remain strict. This is model policy, not native parity; catalog and guards remain unchanged in source; current bounded preservation proof below does not prove Classic policy.

Retained sealed `38ecbd8ee` direct shared-fixture execution: actual wait exit1 at the first unknown cleanup no-throw assertion; subsequent assertions unreached. Separate supplied Retail/PTR SpellDisplay producer results at `38ec`: each 6 model/query/feed/secret PASS and 3 cached-consumer FAIL at cleanup. Neither result proves post-fix behavior. [Current bounded gate](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-spell-display-2026-10-13/README.md#current-bounded-runtime-gate--2026-10-13) owns independent Retail removal/preservation PASS and independently accepted identical PTR execution; selected retention PASS is separate from runtime acceptance. [SpellDisplay owner](ui-widget-spell-display.md) retains its earlier checkpoint; [forbidden policy owner](forbidden-aspect-event-input.md) retains separate preservation scope.

## Known gaps (current cycle)

- [x] Execute the identical shared Lua fixture against sealed normal `38ecbd8ee` before the fix: actual wait exit1, first cached-name no-throw assertion fails; later controls unreached. JSON contains two error entries for the original error and ScriptErrors wrapper, not two independent failures. Executable receipt reused stat-only.
- [x] Accept independent bounded Retail/PTR post-fix scopes; both PASS.
- [ ] Fully check preservation: independent Retail/PTR aspect5/filter2/callback-lifecycle1 PASS; Classic policy unproved. Last requirement not fully checked.

At source revision `38ecbd8ee5dfae4315ca98001b7e794e324b1066`, the shared registration mutation validates the event name before either insertion or removal. The supplied prior actual Retail/PTR Spell-producer result is 6 PASS / 3 FAIL at cached `Base.lua:697` calling `UnregisterEvent('PLAYER_EQUIPED_SPELLS_CHANGED')`; that report is not execution evidence for this new fixture.

## Out of scope

Native removal/return parity, all-profile acceptance, changing registration catalog or callback gates, default shims and vendor changes. The private normal `--exec-lua` probe is an identical copy of the shared fixture, not a separate test contract.
