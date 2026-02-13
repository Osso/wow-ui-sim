// WoW UI Quad Shader
//
// Renders textured/colored quads for UI elements.
// Supports solid colors and tiered 2D texture sampling.
// Textures are stored in 5 tier atlases: 64x64, 128x128, 256x256, 512x512, 2048x2048 cells.
// Each tier is a large 2D texture with textures packed in a grid.
// tex_index encodes tier: 0-4 for the 5 tiers.
// UV coordinates are pre-transformed to select the correct sub-region.

// Uniforms (group 0)
struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Tiered 2D texture atlases (group 1)
// Each tier is a large texture with multiple sub-textures packed in a grid
@group(1) @binding(0)
var tier_64: texture_2d<f32>;     // Atlas for 64x64 textures

@group(1) @binding(1)
var tier_128: texture_2d<f32>;    // Atlas for 128x128 textures

@group(1) @binding(2)
var tier_256: texture_2d<f32>;    // Atlas for 256x256 textures

@group(1) @binding(3)
var tier_512: texture_2d<f32>;    // Atlas for 512x512 textures

@group(1) @binding(4)
var tier_2048: texture_2d<f32>;   // Atlas for 2048x2048 textures

@group(1) @binding(5)
var texture_sampler: sampler;

@group(1) @binding(6)
var glyph_atlas: texture_2d<f32>; // Glyph atlas for text rendering

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) tex_index: i32,
    @location(4) flags: u32,
    @location(5) local_uv: vec2<f32>,
    @location(6) mask_tex_index: i32,
    @location(7) mask_tex_coords: vec2<f32>,
}

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // Use linear interpolation for 2D UI (no perspective correction needed)
    @location(0) @interpolate(linear) tex_coords: vec2<f32>,
    @location(1) @interpolate(linear) color: vec4<f32>,
    @location(2) @interpolate(flat) tex_index: i32,
    @location(3) @interpolate(flat) flags: u32,
    @location(4) @interpolate(linear) local_uv: vec2<f32>,
    @location(5) @interpolate(flat) mask_tex_index: i32,
    @location(6) @interpolate(linear) mask_tex_coords: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform position from screen coords to clip space
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);

    // Pass through to fragment shader
    out.tex_coords = in.tex_coords;
    out.color = in.color;
    out.tex_index = in.tex_index;
    out.flags = in.flags;
    out.local_uv = in.local_uv;
    out.mask_tex_index = in.mask_tex_index;
    out.mask_tex_coords = in.mask_tex_coords;

    return out;
}

// Blend mode constants
const BLEND_ALPHA: u32 = 0u;
const BLEND_ADDITIVE: u32 = 1u;

// Sample from the appropriate tier based on tex_index
// tex_index 0-4: tiered texture atlases, 5: glyph atlas
// UV coordinates are already transformed to the correct sub-region
fn sample_tiered_texture(tex_index: i32, tex_coords: vec2<f32>) -> vec4<f32> {
    // Clamp tex_coords to valid range
    let uv = clamp(tex_coords, vec2<f32>(0.0), vec2<f32>(0.9999));

    // Sample all textures unconditionally to avoid control flow issues
    let s0 = textureSampleLevel(tier_64, texture_sampler, uv, 0.0);
    let s1 = textureSampleLevel(tier_128, texture_sampler, uv, 0.0);
    let s2 = textureSampleLevel(tier_256, texture_sampler, uv, 0.0);
    let s3 = textureSampleLevel(tier_512, texture_sampler, uv, 0.0);
    let s4 = textureSampleLevel(tier_2048, texture_sampler, uv, 0.0);
    let sg = textureSampleLevel(glyph_atlas, texture_sampler, uv, 0.0);

    // Select result based on tier
    if tex_index == 0 {
        return s0;
    } else if tex_index == 1 {
        return s1;
    } else if tex_index == 2 {
        return s2;
    } else if tex_index == 3 {
        return s3;
    } else if tex_index == 4 {
        return s4;
    } else {
        return sg;
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32>;

    // Check if this is a textured or solid color quad
    if in.tex_index < 0 {
        // Solid color or pending texture (-1 = solid, -2 = pending)
        color = in.color;
    } else {
        // Textured quad - sample from the appropriate tier atlas
        let tex_color = sample_tiered_texture(in.tex_index, in.tex_coords);
        color = tex_color * in.color;
    }

    // Premultiplied alpha blending: pipeline uses src + dst * (1 - src.a).
    // Normal: output (rgb * a, a) → src.rgb*a + dst * (1-a) = correct alpha blend.
    // Additive: output (rgb * a, 0) → src.rgb*a + dst * 1 = correct additive.
    let blend_mode = in.flags & 0xFFu;
    if blend_mode == BLEND_ADDITIVE {
        color = vec4f(color.rgb * color.a, 0.0);
    } else {
        color = vec4f(color.rgb * color.a, color.a);
    }

    // Circle clip (for minimap) — uses local_uv which is preserved across atlas remapping
    // Scale both premultiplied RGB and alpha together.
    const FLAG_CIRCLE_CLIP: u32 = 0x100u;
    if (in.flags & FLAG_CIRCLE_CLIP) != 0u {
        let centered = in.local_uv * 2.0 - 1.0;
        let dist = length(centered);
        color *= 1.0 - smoothstep(0.96, 1.0, dist);
    }

    // Cooldown swipe — radial clock wipe from 12 o'clock clockwise.
    // tex_coords.x holds progress (0.0 = fully covered, 1.0 = fully revealed).
    // Pixels where the clock sweep has NOT yet passed are kept; passed pixels discarded.
    const FLAG_COOLDOWN_SWIPE: u32 = 0x200u;
    if (in.flags & FLAG_COOLDOWN_SWIPE) != 0u {
        let progress = in.tex_coords.x;
        // Convert progress to angle threshold (0 → 0, 1 → 2π)
        let threshold = progress * 6.2831853;
        // Compute pixel angle from center, 0 = top, clockwise
        let centered = in.local_uv * 2.0 - 1.0;
        let angle = atan2(centered.x, -centered.y);
        // Remap from [-π, π] to [0, 2π]
        var pixel_angle = angle;
        if pixel_angle < 0.0 {
            pixel_angle += 6.2831853;
        }
        // Keep pixels where pixel_angle >= threshold (not yet swept away)
        if pixel_angle < threshold {
            color = vec4f(0.0);
        }
    }

    // Desaturation — convert to greyscale using luminance weights.
    // Applied before masking so the mask alpha still works correctly.
    const FLAG_DESATURATE: u32 = 0x400u;
    if (in.flags & FLAG_DESATURATE) != 0u {
        // Unpremultiply, desaturate, re-premultiply.
        let a = color.a;
        if a > 0.001 {
            let rgb = color.rgb / a;
            let lum = dot(rgb, vec3f(0.2126, 0.7152, 0.0722));
            color = vec4f(vec3f(lum) * a, a);
        }
    }

    // Mask texture sampling — scale premultiplied output by mask alpha
    if in.mask_tex_index >= 0 {
        let mask_color = sample_tiered_texture(in.mask_tex_index, in.mask_tex_coords);
        color *= mask_color.a;
    }

    return color;
}
