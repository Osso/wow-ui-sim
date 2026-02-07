# Glow / Blend Mode Support

## Current State

The glow system is structurally present in XML/Lua but visually broken — additive blending never reaches the GPU.

### What works

- **GlowEmitter lifecycle** — `GlowEmitterMixin` + `EffectFactory` create, pool, position and animate glow frames correctly
- **Animation** — alpha pulses between 0.5–1.0 via animation groups
- **Shader infrastructure** — `BlendMode::Additive` enum exists in `quad.rs`, vertex `flags` field carries the blend mode, fragment shader reads it
- **Button highlights** — hardcoded to `BlendMode::Additive` in `render.rs:195,203,211` (only place additive is actually used)

### What's broken

1. **`alphaMode` XML attribute ignored** — `TextureXml` (`src/xml/types.rs:540`) has no `alpha_mode` field; `alphaMode="ADD"` in GlowEmitter.xml is silently dropped during deserialization

2. **No blend mode on `Frame` struct** — `src/widget/frame.rs:104` has no `blend_mode` field; there's nowhere to store a texture's blend mode in the widget system

3. **`SetBlendMode()` is a no-op** — `src/lua_api/frame/methods/methods_texture.rs:96` ignores the argument; `GetBlendMode()` always returns `"BLEND"`

4. **`SetBorderBlendMode()` doesn't exist** — `GlowEmitter.lua` calls `self.NineSlice:SetBorderBlendMode("ADD")` but no Rust handler exists

5. **Single GPU pipeline** — `pipeline.rs:101` creates one pipeline with `wgpu::BlendState::ALPHA_BLENDING`; additive quads go through the same blend state as everything else

6. **Shader workaround** — `quad.wgsl:123-128` detects additive flag but can only boost alpha by 1.5x as a hack, since actual GPU blending is always alpha

### Data flow gap

```
XML alphaMode="ADD"  ──(dropped)──> TextureXml (no field)
                                         │
Lua SetBlendMode()   ──(no-op)───> Frame (no field)
                                         │
render.rs            ──(hardcoded)─> QuadBatch.push_*(BlendMode::Alpha)
                                         │
                                    QuadVertex.flags = 0 (alpha)
                                         │
pipeline.rs          ──(single)───> wgpu BlendState::ALPHA_BLENDING
```

## Implementation Plan

### Step 1: Add `blend_mode` to `Frame` struct

**File:** `src/widget/frame.rs`

Add field after `nine_slice_layout`:
```rust
pub blend_mode: BlendMode,
```

Add default `BlendMode::Alpha` to `frame_defaults!` macro.

Import `BlendMode` from `crate::render`.

### Step 2: Parse `alphaMode` from XML

**File:** `src/xml/types.rs`

Add to `TextureXml`:
```rust
#[serde(rename = "@alphaMode")]
pub alpha_mode: Option<String>,
```

### Step 3: Propagate XML `alphaMode` to Frame during loading

**File:** `src/loader/xml_texture.rs` (or wherever textures are loaded from XML into Frame)

When creating a texture Frame from `TextureXml`, map the `alpha_mode` string to `BlendMode`:
```rust
frame.blend_mode = match tex_xml.alpha_mode.as_deref() {
    Some("ADD") => BlendMode::Additive,
    _ => BlendMode::Alpha,
};
```

### Step 4: Implement `SetBlendMode` / `GetBlendMode`

**File:** `src/lua_api/frame/methods/methods_texture.rs:96`

Replace stub with real implementation:
```rust
methods.add_method("SetBlendMode", |_, this, mode: String| {
    let blend = match mode.as_str() {
        "ADD" => BlendMode::Additive,
        _ => BlendMode::Alpha,  // "BLEND", "DISABLE", "MOD", "ALPHAKEY"
    };
    this.state.borrow_mut().widgets.get_mut(this.id).blend_mode = blend;
    Ok(())
});

methods.add_method("GetBlendMode", |_, this, ()| {
    let state = this.state.borrow();
    let frame = state.widgets.get(this.id);
    Ok(match frame.blend_mode {
        BlendMode::Additive => "ADD",
        BlendMode::Alpha => "BLEND",
    })
});
```

Also update the pool_api stub at `src/lua_api/globals/pool_api.rs:46`.

### Step 5: Use `frame.blend_mode` in rendering

**File:** `src/iced_app/render.rs`

In `build_texture_quads`, replace hardcoded `BlendMode::Alpha` with `f.blend_mode`:
```rust
// Before
batch.push_textured_path(bounds, tex_path, [1.0, 1.0, 1.0, f.alpha], BlendMode::Alpha);

// After
batch.push_textured_path(bounds, tex_path, [1.0, 1.0, 1.0, f.alpha], f.blend_mode);
```

Apply to all `push_textured_path*` calls in `build_texture_quads` and `emit_tiled_texture`.

### Step 6: Two-pipeline rendering for additive blend

**File:** `src/render/shader/pipeline.rs`

The core problem: wgpu blend state is per-pipeline, not per-draw-call. Quads with different blend modes can't use the same pipeline.

**Option A: Two pipelines, two passes (recommended)**

Add a second pipeline with additive blend state:
```rust
wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::One,  // key difference: One instead of OneMinusSrcAlpha
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    },
}
```

Split `QuadBatch` into two sub-batches (alpha and additive) during prepare, or maintain two `QuadBatch` instances. Render alpha batch first, then additive batch with the second pipeline.

Store both pipelines in `WowUiPipeline`:
```rust
pub struct WowUiPipeline {
    pipeline_alpha: wgpu::RenderPipeline,
    pipeline_additive: wgpu::RenderPipeline,
    // ...
}
```

In `render()`, do two draw calls in the same render pass:
```rust
// Draw alpha-blended quads
render_pass.set_pipeline(&self.pipeline_alpha);
render_pass.draw_indexed(0..self.alpha_index_count, 0, 0..1);

// Draw additive quads
render_pass.set_pipeline(&self.pipeline_additive);
render_pass.draw_indexed(0..self.additive_index_count, 0, 0..1);
```

**Option B: Shader-only approximation (simpler, less correct)**

Keep single pipeline. In the shader, for additive quads, pre-multiply color by alpha and set alpha to 0 — this makes standard alpha blending approximate additive:
```wgsl
if blend_mode == BLEND_ADDITIVE {
    color = vec4(color.rgb * color.a, 0.0);
}
```

With `ALPHA_BLENDING`: `output = src * src.a + dst * (1 - src.a)` becomes `output = (rgb * a) * 0 + dst * (1 - 0) = dst` — that's wrong. This doesn't work. **Option A is required.**

### Step 7: Remove shader workaround

**File:** `src/render/shader/quad.wgsl:119-128`

Remove the alpha boost hack once real additive blending works:
```wgsl
// Remove this block:
if blend_mode == BLEND_ADDITIVE {
    color.a = min(color.a * 1.5, 1.0);
}
```

### Step 8: Stub `SetBorderBlendMode`

**File:** `src/lua_api/frame/methods/` (wherever NineSlice/frame methods live)

Add a no-op for now — the NineSlice system would need per-piece blend modes for full support, which is a larger change:
```rust
methods.add_method("SetBorderBlendMode", |_, _this, _mode: String| Ok(()));
```

## Build sequence

1. `Frame.blend_mode` field + default (step 1)
2. XML parsing (steps 2-3) — glow textures get `Additive` from XML
3. Lua API (step 4) — `SetBlendMode` sets `Additive` at runtime
4. Render propagation (step 5) — `build_texture_quads` reads `frame.blend_mode`
5. Two-pipeline split (step 6) — GPU actually does additive blending
6. Clean up shader hack (step 7) + stub `SetBorderBlendMode` (step 8)

Steps 1-4 can be done without visual change (additive flag flows through but hits same pipeline). Step 5-6 is where the visual fix happens.

## WoW blend modes for reference

| Mode | Description | GPU blend |
|------|-------------|-----------|
| `BLEND` | Standard alpha | `src * src.a + dst * (1 - src.a)` |
| `ADD` | Additive (glow) | `src * src.a + dst` |
| `MOD` | Modulative | `src * dst` |
| `DISABLE` | No blending | `src` (opaque) |
| `ALPHAKEY` | Alpha test | Discard if alpha < threshold |

Only `ADD` is needed now. `MOD`/`DISABLE`/`ALPHAKEY` can be added later with additional pipeline variants if needed.
