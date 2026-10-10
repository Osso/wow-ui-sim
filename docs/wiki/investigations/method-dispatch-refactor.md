# Method Dispatch Refactor

Refactor of frame method dispatch to fix runtime pollution (wrong-type methods exposed) and align `__index` lookup order with real WoW behavior.

## Current State (Fixed)

Verified by source inspection on 2026-10-09: `env_init/frames.rs` registers method groups on a base frame metatable, then builds a shallow shared `__index` without `__*` keys. `methods/frame_metatable.rs` clones that index into cached per-widget metatables, excluding explicitly named scroll/message methods and `SetStatusBarAtlas` for StatusBar. WorldFrame uses the base metatable. No `is_method_allowed` registry or positive per-type allowlist exists.

`methods.rs::attach_frame_metatable` attaches the selected metatable to each Lua frame table. Its filtered `__index` therefore participates in ordinary lookup, not just introspection. Shared registration does not prove all-type dispatch. These inspected files do not support the former introspection-only filter claim; no invocation tests or native parity proof were run for this correction.

The reported historical **491-row delta across 31 names** and the **11,985-row baseline file total** measure different scopes, not contradictory totals. The historical delta was not independently reproduced here. Neither count alone justifies snapshot replacement, a native allowlist, or a new publication contract.

## Current Source Representation

- Shared registration/index construction: `src/lua_api/env_init/frames.rs`
- Named per-widget exclusions/index cloning: `src/lua_api/methods/frame_metatable.rs`
- Frame-table metatable attachment: `src/lua_api/methods.rs`

## Historical Target Architecture

```
__index(ud, key):
  1. mixin_overrides[frame_id][key]    — Mixin() shadows
  2. children_keys[key]                — child frame lookup
  3. custom_fields[frame_id][key]      — script handlers, properties
  4. rust_dispatch(widget_type, key)   — direct Rust match, no Lua table
```

Step 4 resolves methods directly from widget type + name via a `HashMap<(WidgetType, &str), mlua::Function>`. No intermediate Lua table, no shared-method fallback.

## Historical Proposed Work (Not Current Requirements)

- Trim `diff_methods_extra.txt` (types still exposing too many methods in discovery)
- Add diff-driven coverage so unintentional method-surface changes fail tests
- Optionally collapse runtime dispatch and metatable exposure onto one direct Rust lookup

## Sources

- [frame_metatable.rs](../../../src/lua_api/methods/frame_metatable.rs) — named exclusions and cached metatable clones
- [frames.rs](../../../src/lua_api/env_init/frames.rs) — shared registration and method-index construction
- [methods.rs](../../../src/lua_api/methods.rs) — attaches the selected metatable to frame tables
- [method-dispatch-refactor.md](../../method-dispatch-refactor.md) — historical design

## See Also

- [[editmode-layout]] — regression caused by the Lua-fields-first `__index` change
- [[global-frame-index]] — related `__index` on `_G` for lazy frame lookup
