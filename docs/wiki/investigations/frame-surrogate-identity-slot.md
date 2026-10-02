# Frame Surrogate Identity Slot

Frame method dispatch now models WoW's frame identity slot so Restricted Environment-style surrogate tables carry identity through `self[0]` instead of the simulator's older `self[1]` unwrap convention.

## Content

wow-ui-sim frames are rilua tables with VM-level backing metadata that points to the Rust `widget::Frame` in `WidgetRegistry.widgets`. Frame methods are shared through one frame metatable, and receiver resolution uses the table backing to recover the frame id.

`methods::frame_ref` returns cached `Val::Table` backing or creates/caches that table; `create_frame_table` stores separate `FrameIdentity` userdata at numeric slot0. A fixture requiring the frame itself to be `Val::Userdata` fails before reaching its intended API boundary. [Row387 corrected RED](../../specs/cooldown-aura-spell-identifiers.md#corrected-compiled-red-and-producer-boundary--2026-10-02) instead checks `Frame:GetObjectType()`, keeps the actual frame globally rooted, and wraps that value; these are simulator representation facts, not a new native-client claim.

Blizzard's Restricted Environment creates frame-handle surrogates with the real frame's identity token in slot `0` and the original frame in slot `1`. The simulator previously left `frame[0]` nil and let universal dispatch recover surrogate identity from `self[1]`. That made Restricted Environment probes work, but it hardcoded one Blizzard surrogate layout into every frame method call.

The simulator now seeds each frame table's numeric slot `0` with a `FrameIdentity` userdata token. Method receiver resolution reads `self[0]` before the table's direct backing, so a surrogate shaped like `{ [0] = frame[0] }` can call shared frame methods such as `GetName()` and `IsProtected()` without carrying the original frame in slot `1`. A `[1]`-only surrogate no longer dispatches; code that wants frame identity must copy the identity token.

Real-client probe on retail `12.0.5.67823` confirmed slot `0` is the dispatch identity: after `a[0] = b[0]`, `a:IsProtected()` returned `b`'s protected state and `a:GetName()` returned `b`'s name. On the real client `frame[0]` is a userdata token; wow-ui-sim now matches that visible type while keeping the frame itself as a table.

`DevTools_Dump(frame)` depends on two related pieces: `pairs(frame)` must enumerate the table's raw `[0]` and array slots, and `dumpobject(frame[0])` must exist and return nil so Blizzard's Dump.lua treats the identity token as opaque userdata. With those in place, the dump shape matches retail: `[1]="foo"` and `[0]=<userdata>` instead of expanding the identity token as a table.

Lua-visible behavior changes from `frame[0] == nil` to `type(frame[0]) == "userdata"`, so regressions should first check code that inspected slot `0` directly.

`extract_frame_id(state, val)` is dispatch-aware: it prefers `val[0]` and only falls back to native table backing when the identity slot is absent. Code that needs the original Lua table's frame id must use `native_frame_id_from_val` explicitly.

Duplicate named `CreateFrame` calls must produce a fresh identity token and must not migrate Lua table fields from the old global binding. Retail `12.0.5.67823` and wowless both agreed on the important behavior: recreating `CreateFrame("Frame", "SameName")` creates a distinct Lua object, leaves the original object's custom fields in place, gives the replacement a different `[0]`, and leaves the replacement's old custom hash/array fields nil. The simulator previously copied the old global frame table into the replacement, including stale `[0]`, which made startup handlers dispatch through the retired widget and produced `pairs`/`tinsert` errors.

Regression coverage lives in `security_api::test_frame_identity_slots_dispatch_surrogate_methods`, `frame_table_iteration::{frame_pairs_enumerates_identity_and_array_slots, frame_identity_userdata_is_opaque_to_dumpobject, frame_arguments_resolve_through_identity_slot}`, and `globals_legacy::duplicate_named_frame_gets_fresh_identity_and_fields`.

## Sources

- [methods.rs](../../../src/lua_api/methods.rs) - frame receiver and surrogate backing resolution
- [table_builder.rs](../../../src/lua_bridge/table_builder.rs) - backed frame table and identity-token creation
- [security_api.rs](../../../tests/security_api.rs) - regression coverage for `[0]` surrogate dispatch
- [frame_table_iteration.rs](../../../tests/frame_table_iteration.rs) - regression coverage for frame pair iteration, userdata identity, and `dumpobject`
- [globals_legacy.rs](../../../tests/globals_legacy.rs) - regression coverage for duplicate named `CreateFrame` fresh identity and no field migration
- [helpers_shared.rs](../../../src/lua_api/globals/create_frame/helpers_shared.rs) - global name registration for runtime `CreateFrame`
- [FrameIdentityProbe](../../../docs/addons/FrameIdentityProbe) - real-client SavedVariables probe for slot `0` reassignment behavior
- [[taint-system]] - Restricted Environment and secure-handler context

## See Also

- [[taint-system]] - protected-frame and Restricted Environment behavior
- [[lua-api]] - frame method dispatch and Lua API surface
- [[frame-data-flow]] - Lua/Rust frame identity and data flow
