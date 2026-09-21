# Script-object OnUpdate modes

The shared `on-update-modes` capability exposes the numeric frame update-mode contract used by Retail 12.1+ and Forever. The enum producer lives in `src/c_api/on_update_modes.rs`; frame methods and the existing update dispatcher share that representation.

## What it must do

- [ ] Publish exactly `Disabled=0`, `RunWhenVisible=1`, `RunWhenVisibleOnce=2`, `RunOnce=3`, `RunAlways=4`, with metadata min 0, max 4, count 5; do not invent `ScriptObjectOnUpdateMode`.
- [ ] New frames default to `RunWhenVisible`; setters/getters use numeric enum values, not compatibility strings.
- [ ] Visible modes respect ancestor visibility; hidden visible-once frames retain their armed state. `RunOnce` and `RunAlways` do not require visibility; `Disabled` never dispatches.
- [ ] Reset one-shot mode to `Disabled` before callbacks. Rearming inside a handler survives the dispatch and executes on the next eligible tick, including across script hooks.
- [ ] Translate authored XML `onUpdateMode` names into the same numeric state.
- [ ] Actual ManagedAuraContainer dirty scheduling reaches its inherited `ProcessDirtyFlags` through the normal dispatcher, once when visible, then remains disabled until dirtied again.

## How it works

- [Event system](../event-system.md)
- [Ellesmere investigation](../wiki/investigations/ellesmereui-forever.md)

## Implementation inventory

- `Cargo.toml` — shared capability selected by Retail 12.1+ and Forever, without promoting Forever to an entire Retail epoch.
- `src/c_api/on_update_modes.rs` — numeric enum, metadata and shared mode interpretation.
- `src/lua_api/env_init/enums.rs` — initial publication.
- `src/lua_api/frame/methods/text_attribute_event/mod.rs` — frame method registration and storage.
- `src/lua_api/script_helpers/event_dispatch.rs` — visibility and pre-callback one-shot reset.
- `src/loader/xml_frame/setup.rs` — XML-name conversion.
- `src/ptr/compat_bootstrap.rs` — restoration uses the shared producer rather than Lua string defaults.

## Tests asserting this spec

`tests/on_update_modes.rs` in the existing grouped `integration` target.

## Evidence

Pinned Forever 1.60.1.69913 `SimpleFrameScriptObjectConstantsDocumentation.lua` documents five numeric values and explicitly says one-shot modes reset before running. `SimpleFrameAPIDocumentation.lua` declares the getter and setter. `Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua` arms visible-once from `OnDirtyChanged` and calls `ProcessDirtyFlags` in `OnUpdate`; `Blizzard_SharedXML/MixinUtil.lua` supplies the dirty-phase machinery.

## Known gaps (current cycle)

- [ ] Development RED/GREEN and parent-owned final verification pending.

## Out of scope

No new Retail epoch, aura subsystem rewrite, vendor/addon edits, compatibility string inputs, undocumented enum aliases, or additional secret-argument enforcement is introduced. Existing XML and other-profile capability boundaries are retained.
