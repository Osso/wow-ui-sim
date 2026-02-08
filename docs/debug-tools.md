# Debug Tools

Three tools for inspecting the frame hierarchy at runtime: the **Inspector Panel** (GUI), **dump-tree** (CLI), and **debug visualization** (overlay).

---

## Inspector Panel

**File:** `src/iced_app/view.rs:340-527`, `src/iced_app/update.rs:152-159, 600-636`

Middle-click any frame in the GUI to open a floating inspector panel showing its properties.

### How It Works

1. **Hit test** (`view.rs:530-542`): Middle-click triggers `hit_test()` which searches the cached hittable list in reverse strata/level order (topmost frame first)
2. **Populate** (`update.rs:601-614`): Reads the frame's properties from the Rust `WidgetRegistry`
3. **Display**: Panel appears at click position + 10px offset

### Properties Shown

| Field | Source | Editable |
|-------|--------|----------|
| Name | `frame.name` (or `(anon)`) | No |
| Widget type | `frame.widget_type` | No |
| ID | Widget registry key | No |
| Pos | Computed via `compute_frame_rect()` | No |
| W / H | `frame.width` / `frame.height` (stored) | Yes |
| Alpha | `frame.alpha` | Yes |
| Level | `frame.frame_level` | Yes |
| Visible | `frame.visible` | Yes (checkbox) |
| Mouse | `frame.mouse_enabled` | Yes (checkbox) |
| Anchors | `frame.anchors` list | No |

### Known Limitation: W/H Shows Stored Dimensions

The W and H fields show `frame.width` and `frame.height` — the **stored explicit dimensions**, not the computed layout size. For frames with two-point anchoring (e.g., TopLeft + BottomRight to parent), the stored dimensions are 0 because actual size comes from anchor resolution at render time. The `Pos` field does show computed coordinates via `compute_frame_rect()`.

### Apply Button

Clicking **Apply** writes editable fields back to the `WidgetRegistry` (`update.rs:617-636`) and invalidates the layout cache, causing an immediate re-render.

### Hit Test Details

**File:** `src/iced_app/view.rs:20-78`

The hittable list is lazily built and cached. It includes frames that are:
- Visible (`frame.visible == true`)
- Mouse-enabled (`frame.mouse_enabled == true`)
- Ancestor-visible (all parents up the chain are visible)
- Not in the exclusion list: `UIParent`, `Minimap`, `WorldFrame`, `DEFAULT_CHAT_FRAME`, `ChatFrame1`, `EventToastManagerFrame`, `EditModeManagerFrame`

Frames are sorted by strata then level. The hit test iterates in reverse (highest first), returning the topmost frame under the cursor.

---

## Dump Tree

Two variants with different output formats and data sources.

### Connected: `wow-cli dump-tree`

**Files:** `src/bin/wow_cli/main.rs:44-52, 212-221`, `src/iced_app/tree_dump.rs:44-79`

Connects to a running `wow-sim` via Unix socket IPC. Shows **computed layout positions** from the live renderer.

```bash
wow-cli dump-tree                      # All frames
wow-cli dump-tree --filter Button      # Filter by name substring
wow-cli dump-tree --visible-only       # Visible frames only
```

**Output format:**
```
+- UIParent (Frame) @ (0,0) 814x792
      [anchor] TOPLEFT -> $parent:TOPLEFT offset(0,0) -> (0,0)
      [anchor] BOTTOMRIGHT -> $parent:BOTTOMRIGHT offset(0,0) -> (814,792)
   +- SettingsPanel (Frame) @ (-53,34) 920x724 [hidden]
   |     [anchor] CENTER -> $parent:CENTER offset(0,0) -> (407,396)
   |  +- __tpl_107 (Frame) @ (-46,52) 910x703 [stored=0x0]
   |  |     [anchor] TOPLEFT -> $parent:TOPLEFT offset(7,-18) -> (-46,52)
   |  |     [anchor] BOTTOMRIGHT -> $parent:BOTTOMRIGHT offset(-3,3) -> (864,755)
```

**Output details:**
- Tree connectors: `+-` for each frame, `|` for ancestor continuation
- Position and size are **computed** from `compute_frame_rect()` (anchor-resolved)
- `[stored=WxH]` appears when stored `frame.width`/`frame.height` differ from computed size (`tree_dump.rs:212-218`)
- `[hidden]` for invisible frames
- `[anchor]` lines show each anchor's source point, target frame/point, offsets, and resolved absolute position
- `[texture]` line for frames with a texture path set
- Anonymous frames with text content show the text as display name: `"Cancel"` instead of `(anon)`

**Name resolution** (`tree_dump.rs:194-209`):
- Named frames show their global name
- `__anon_*`, `__fs_*`, `__tex_*` prefixed frames are treated as anonymous
- Anonymous frames with `frame.text` show the quoted text (truncated to 20 chars)
- Truly anonymous frames show `(anon)`

**Filter** matches frame names only (case-insensitive substring). If a frame doesn't match but has descendants that do, it still appears to preserve hierarchy.

### Standalone: `wow-sim dump-tree`

**Files:** `src/main.rs:48-58, 178-184`, `src/dump.rs`

Loads the full UI (Blizzard + addons) without starting the GUI, then dumps.

```bash
wow-sim dump-tree                                     # Full load
wow-sim --no-addons --no-saved-vars dump-tree         # Fast: skip addons
wow-sim dump-tree --filter ScrollBar                  # Filter by name
wow-sim dump-tree --visible-only                      # Visible only
wow-sim dump-tree --delay 500                         # Wait 500ms after startup events
```

**Output format (different from connected):**
```
=== Anchor Diagnostic ===
Anchored: 5234, Unanchored: 1892
Top unanchored keys: [("NormalTexture", 312), ("PushedTexture", 287), ...]

=== Frame Tree ===
SettingsPanel [Frame] (920x550) visible keys=[TitleContainer, ...]
  .TitleContainer [Frame] (920x18) visible
    .Title [FontString] (0x14) visible text="Options" font="GameFontNormal" size=0
```

**Key differences from connected mode:**
- Prints anchor diagnostic summary first (anchored/unanchored counts, top unanchored parent keys)
- Shows **stored** `frame.width`/`frame.height`, not computed layout
- Uses parentKey names with `.` prefix for anonymous children (e.g., `.NormalTexture`)
- Shows `children_keys` list, font info for FontStrings, and text content
- No anchor detail lines per frame
- No tree connector graphics (uses indentation only)

---

## Debug Visualization

**File:** `src/iced_app/app.rs:37-41, 318-331`

Overlay rendering for debugging layout. Currently stored as flags but marked `TODO: Re-implement as shader quads`.

### Activation

**CLI flags** (on `wow-sim`):
```bash
wow-sim --debug-elements    # Both borders + anchor points
wow-sim --debug-borders     # Red borders around all elements
wow-sim --debug-anchors     # Green dots at anchor points
```

**Environment variables** (override CLI):
```bash
WOW_SIM_DEBUG_ELEMENTS=1    # Both borders + anchor points
WOW_SIM_DEBUG_BORDERS=1     # Red borders only
WOW_SIM_DEBUG_ANCHORS=1     # Green anchor dots only
```

---

## Architecture

All three tools read from the same Rust `WidgetRegistry` (`state.widgets`). No data comes from Lua — the Lua API methods (e.g., `SetPoint`, `SetSize`) write to the registry, and debug tools read from it.

```
Lua API calls ──> WidgetRegistry <──┬── Inspector Panel (live, editable)
                                    ├── dump-tree connected (live, computed layout)
                                    ├── dump-tree standalone (one-shot, stored sizes)
                                    └── Debug overlay (live, shader quads)
```

### Layout Resolution

**File:** `src/iced_app/layout.rs`

The connected dump-tree and inspector use `compute_frame_rect()` which resolves anchors dynamically:
1. Look up the parent's rect (recursive)
2. For each anchor, resolve the relative frame's position
3. Compute edges from anchor points + offsets
4. Derive width/height from opposite edges, or fall back to stored `frame.width`/`frame.height`

The standalone dump-tree uses stored `frame.width`/`frame.height` directly without anchor resolution.
