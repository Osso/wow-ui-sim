# Button Text Rendering

How button text is rendered, why three-slice button text appears behind
the button background, and how to fix it.

## Button Widget Structure

Every Button created via `CreateFrame("Button", ...)` gets default children
in `create_button_defaults()` (`src/lua_api/globals/create_frame.rs:205-251`):

| Child (parentKey) | WidgetType | DrawLayer | Notes |
|-------------------|------------|-----------|-------|
| `NormalTexture`   | Texture    | Artwork (default) | Shown in normal state |
| `PushedTexture`   | Texture    | Artwork (default) | Shown when pressed |
| `HighlightTexture`| Texture    | Highlight | Shown on hover, additive blend |
| `DisabledTexture` | Texture    | Artwork (default) | Shown when disabled |
| `Text`            | FontString | Overlay   | Button label |

The `Text` FontString has a single CENTER anchor to its parent button and
no explicit width or height.

## Rendering Sort Order

`sorted_visible_frames()` in `src/iced_app/render.rs:390-407` sorts all
widgets for rendering:

```
1. frame_strata  (Background < Low < Medium < ... < Tooltip)
2. frame_level   (integer, higher = on top)
3. region vs non-region:
   - Non-regions (Frame, Button) render FIRST
   - Regions (Texture, FontString) render AFTER
   - Among regions: sorted by draw_layer, then draw_sub_layer
4. widget ID     (tiebreaker)
```

Draw layer order: `Background(1) < Border(2) < Artwork(3) < Overlay(4) < Highlight(5)`

## Where Button Text Is Emitted

Button text is emitted in **two places**:

### 1. Button's own `emit_frame_quads` (render.rs:462-467)

```rust
WidgetType::Button => {
    build_button_quads(batch, bounds, f, pressed, hovered);
    // Text emitted here, using the button's own `f.text` field:
    emit_widget_text_quads(batch, fs, ga, f, txt, bounds, Center, Center, ...);
}
```

This renders as part of the **Button frame** (a non-region), which sorts
**before** all child regions.

### 2. Child `Text` FontString's own `emit_frame_quads` (render.rs:474-478)

```rust
WidgetType::FontString => {
    emit_widget_text_quads(batch, fs, ga, f, txt, bounds, ...);
}
```

This renders as a **region** at Overlay layer, sorting after all Artwork
and Background layer textures.

`SetText()` sets the text string on **both** the button and the child
FontString (`src/lua_api/frame/methods/methods_text/mod.rs:57-85`).

## The Three-Slice Button Problem

Three-slice buttons (ThreeSliceButtonTemplate) have three child Textures
defined in XML at `BACKGROUND` layer (`ThreeSliceButtonTemplate.xml:22`):

| Child (parentKey) | DrawLayer | Purpose |
|-------------------|-----------|---------|
| `Left`            | Background | Left cap |
| `Right`           | Background | Right cap |
| `Center`          | Background | Tiling center fill |

These textures have their atlas set by `ThreeSliceButtonMixin:InitButton()`
at OnLoad time and are the button's visible background.

### Rendering Order (Current)

```
Step 1: Button (non-region) renders first
  ├── build_button_quads → nothing (no NormalTexture on three-slice buttons)
  └── emit text quads    → TEXT RENDERED HERE ← rendered early

Step 2: Child regions sorted by draw_layer:
  ├── Left   (Background=1)  → COVERS THE TEXT FROM STEP 1
  ├── Right  (Background=1)  → COVERS THE TEXT
  ├── Center (Background=1)  → COVERS THE TEXT
  ├── NormalTexture  (Artwork=3) → empty, no atlas set
  ├── PushedTexture  (Artwork=3) → empty
  ├── DisabledTexture(Artwork=3) → empty
  ├── Text FontString (Overlay=4) → SHOULD re-render text on top...
  └── HighlightTexture(Highlight=5) → hover overlay
```

The child `Text` FontString at Overlay layer **should** render on top of
the Background-layer three-slice textures. But it doesn't render anything
useful because it has **zero width**.

### Why the Child Text FontString Has Zero Width

`create_button_defaults()` creates the `Text` FontString with only a
single CENTER anchor and no explicit size:

```rust
text_fs.anchors.push(Anchor {
    point: Center,
    relative_to_id: Some(frame_id),
    relative_point: Center,
    x_offset: 0.0, y_offset: 0.0,
});
```

`resolve_single_anchor()` in `layout.rs:119-149` computes the rect using
`frame.width * scale` — but the FontString's width is 0, so the computed
rect has zero width. Text can't render in a zero-width bounding box.

### Result

- Step 1 text: rendered, then covered by three-slice textures
- Step 2 child FontString: has the text string but 0-width rect, renders nothing
- **Net effect: text is invisible (behind the button background)**

## Fix

Give the child `Text` FontString fill-parent anchors instead of a single
CENTER point. This makes it inherit the button's width via two-point
anchor resolution (`resolve_multi_anchor_edges`).

In `create_button_defaults()`, replace:

```rust
text_fs.anchors.push(Anchor {
    point: Center,
    relative_to_id: Some(frame_id),
    relative_point: Center,
    ...
});
```

With fill-parent anchors (equivalent to `SetAllPoints`):

```rust
add_fill_parent_anchors(&mut text_fs, frame_id);
```

This gives the FontString TOPLEFT→TOPLEFT and BOTTOMRIGHT→BOTTOMRIGHT
anchors, so `resolve_multi_anchor_edges()` computes proper width and
height from the parent button's bounds.

With proper width, the child FontString renders text at Overlay layer (4),
which sorts after the three-slice Background (1) textures.

## Affected Button Types

Any button using child Texture regions for its background instead of the
button's own `normal_texture` field:

| Template | Textures | Used By |
|----------|----------|---------|
| ThreeSliceButtonTemplate | Left/Right/Center at BACKGROUND | Game menu, AddOn list |
| BigRedThreeSliceButtonTemplate | inherits ThreeSliceButton | Game menu buttons |
| SharedButtonSmallTemplate | inherits BigRedThreeSlice | Enable All, Disable All |
| MinimalScrollBar | Back/Forward with custom parentKey | Scroll bars |

Standard buttons that use `<NormalTexture>` XML (which sets the button's
`normal_texture` field rendered by `build_button_quads`) are **not affected**
— their text from step 1 renders on top of the button's own texture, and
no child textures cover it.
