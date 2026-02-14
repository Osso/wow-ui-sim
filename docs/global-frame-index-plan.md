# Plan: `__index` on `_G` for Lazy Frame Lookup

## Problem

Opening the talent panel is slow because it creates hundreds of frames, each
requiring Lua object creation and global registration. The cost per frame:

1. `lua.create_userdata(FrameHandle)` — mlua allocation + metatable setup
2. `lua.globals().set(name, ud)` — insert into `_G` table
3. `lua.globals().set("__frame_{id}", ud)` — insert internal event dispatch ref
4. Template child codegen — `_G["childName"] = child` via `lua.load().exec()`

For frames that are never accessed from Lua (off-screen talent nodes, hidden
children), this is wasted work.

## Goal

Create frames entirely in Rust and only materialize Lua FrameHandle userdata
when Lua code actually accesses the frame. The `__index` metamethod on `_G`
is the mechanism that makes this possible.

## Design

### Phase 1: `__index` on `_G` (lazy lookup)

Set a metatable on `lua.globals()` with an `__index` function that checks
`widgets.get_id_by_name(key)`. On hit, create a FrameHandle and cache it
via `rawset(_G, key, handle)` so subsequent accesses are direct table hits.

```rust
let mt = lua.create_table()?;
let state_rc = Rc::clone(&state);
mt.set("__index", lua.create_function(move |lua, (table, key): (Table, String)| {
    // Skip internal keys
    if key.starts_with("__") {
        return Ok(Value::Nil);
    }
    let state = state_rc.borrow();
    if let Some(id) = state.widgets.get_id_by_name(&key) {
        drop(state);
        let handle = FrameHandle { id, state: Rc::clone(&state_rc) };
        let ud = lua.create_userdata(handle)?;
        // Cache so next access is a direct table hit
        table.raw_set(key, ud.clone())?;
        Ok(Value::UserData(ud))
    } else {
        Ok(Value::Nil)
    }
})?;
lua.globals().set_metatable(Some(mt));
```

#### `__frame_{id}` refs

The `__index` fallback handles name→id lookups. The `__frame_{id}` pattern
is keyed by ID, not name. Two options:

**Option A**: Keep setting `__frame_{id}` eagerly. These are internal-only
keys and don't require name lookup. Cost is one `globals.set()` per frame.

**Option B**: Also handle `__frame_` prefix in `__index`:
```rust
if let Some(id_str) = key.strip_prefix("__frame_") {
    if let Ok(id) = id_str.parse::<u64>() {
        if state.widgets.get(id).is_some() {
            // create handle and cache
        }
    }
}
```

Option B eliminates all eager `globals.set()` calls. Prefer B.

#### What can be removed

Once `__index` handles both named frames and `__frame_{id}` refs:

- `create_frame.rs:243` — `globals.set(name, ud)` (named global)
- `create_frame.rs:247` — `globals.set("__frame_{id}", ud)` (internal ref)
- `global_frames.rs:69` — `globals.set(name, ud)` (pre-registered frames)
- `global_frames.rs:72` — `globals.set("__frame_{id}", ud)` (pre-registered)
- `template/mod.rs:465` — `_G["name"] = child` codegen in templates
- `template/elements.rs:80,307,429,488,587` — `_G["name"] = tex/fs/bar/thumb`

#### Behavior to preserve

- `pairs(_G)` / iteration: Lua code that iterates `_G` to find frames won't
  see uncached frames. This is acceptable — WoW addons access frames by name,
  not by iterating `_G`. The only known iterator is in the test
  `test_create_frame_unnamed_not_in_globals`, which checks that unnamed frames
  are NOT visible in iteration (passes either way).

- `rawget(_G, name)`: Returns nil for uncached frames. Rare in addon code.
  Not expected to be an issue.

- `CreateFrame` return value: Still returns a FrameHandle userdata — the
  caller has the reference. The `__index` fallback is for subsequent lookups
  by other code.

#### Button child globals

`ButtonNameNormalTexture` etc. are unnamed child frames with no entry in
`widgets.names`. The `__index` lookup wouldn't find them. Options:

**Option A**: Register button children with names in the widget registry
(e.g. `names.insert("BtnNormalTexture", id)`). Then `__index` finds them.

**Option B**: Keep eager `globals.set()` for button children only.

Prefer A — it's consistent and lets us remove all eager sets.

### Phase 2: Skip FrameHandle creation in CreateFrame

Once `_G` is handled by `__index`, `CreateFrame` no longer needs to create
a FrameHandle at all for frames that won't be immediately used. However,
`CreateFrame` returns the handle to the caller, so:

- `CreateFrame` still creates and returns a FrameHandle (caller expects it)
- But template child creation (`build_create_child_code`) can skip the
  Lua `CreateFrame` call entirely and use `register_new_frame` directly

This is Phase 2: move `create_child_frame_from_template` to pure Rust.

### Phase 3: Pure Rust template child creation

Replace `build_create_child_code` (which generates a Lua string calling
`CreateFrame`) with direct Rust calls:

```rust
fn create_child_frame_rust(state, parent_id, template) -> u64 {
    let frame_id = register_new_frame(state, widget_type, name, parent_id);
    direct::set_size(state, frame_id, template);
    direct::set_anchors(state, frame_id, template, name);
    direct::set_hidden(state, frame_id, template);
    // parentKey → parent.children_keys.insert(key, frame_id)
    frame_id
}
```

This eliminates `lua.load().exec()` for each child frame. Combined with
Phase 1, the frame exists only in Rust until Lua code actually accesses it.

Mixin, KeyValues, and Scripts still require Lua execution — but they can be
batched into fewer `lua.load()` calls (one chunk for all children instead of
one per child).

## Test coverage

Tests in `src/loader/tests/global_frame_access.rs` cover current behavior:

| Test | What it verifies |
|------|-----------------|
| `test_create_frame_named_sets_global` | `_G["name"]` and bare `Name` both resolve |
| `test_create_frame_unnamed_not_in_globals` | Unnamed frames don't pollute `_G` |
| `test_create_frame_returns_functional_handle` | Returned handle supports methods |
| `test_global_overwritten_by_recreate` | Re-creating updates `_G` to new frame |
| `test_xml_named_frame_in_global` | XML-loaded frames are in `_G` |
| `test_xml_child_texture_in_global` | Named child textures are globals |
| `test_button_child_globals` | `ButtonNameNormalTexture` etc. exist |
| `test_preexisting_global_frames` | UIParent, WorldFrame, Minimap exist at startup |
| `test_global_nil_for_nonexistent_frame` | Missing names return nil |

All tests should pass unchanged after the refactor.

## Risks

- **`pairs(_G)` iteration**: Any code that iterates `_G` to discover frames
  won't see lazily-created ones. Mitigated: no known WoW addon does this.

- **Performance of failed lookups**: Every undefined global access hits the
  Rust HashMap. Mitigated: HashMap lookup is O(1), and `rawset` caching means
  each name is looked up at most once.

- **Ordering**: Some OnLoad handlers expect other frames to already be in `_G`.
  With lazy lookup, they still will be — `__index` resolves them on access.
  The frame just needs to exist in `widgets.names`, which happens at
  `register()` time (before any Lua runs).
