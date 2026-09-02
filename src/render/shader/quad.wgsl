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
    // x = brightness gamma divisor (1.5 lifts dark UI, 1.0 = identity)
    params: vec4<f32>,
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

@group(1) @binding(7)
var bc1_atlas: texture_2d<f32>;   // BC1 (DXT1) compressed textures

@group(1) @binding(8)
var bc3_atlas: texture_2d<f32>;   // BC3 (DXT3/DXT5) compressed textures

@group(1) @binding(9)
var glyph_sampler: sampler;

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
// tex_index 0-4: tiered RGBA atlases, 5: glyph atlas, 6: BC1 atlas, 7: BC3 atlas
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
    let sg = textureSampleLevel(glyph_atlas, glyph_sampler, uv, 0.0);
    let sbc1 = textureSampleLevel(bc1_atlas, texture_sampler, uv, 0.0);
    let sbc3 = textureSampleLevel(bc3_atlas, texture_sampler, uv, 0.0);

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
    } else if tex_index == 5 {
        return sg;
    } else if tex_index == 6 {
        return sbc1;
    } else if tex_index == 7 {
        return sbc3;
    } else {
        return sg;
    }
}

// sRGB electro-optical transfer: the vertex colour arrives as the sRGB value
// Lua set (SetColorTexture, SetVertexColor, font colours), while atlas samples
// are decoded to linear by their sRGB texture format and the render target
// re-encodes on write. Without this the two paths disagree: a texel round-trips
// unchanged, but a solid 0.5 lands at 0.735 (188/255) — one encode too bright.
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + vec3f(0.055)) / 1.055, vec3f(2.4));
    return select(high, low, c <= vec3f(0.04045));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32>;
    const FLAG_COOLDOWN_SWIPE: u32 = 0x200u;
    let vertex_color = vec4f(srgb_to_linear(in.color.rgb), in.color.a);

    if (in.flags & FLAG_COOLDOWN_SWIPE) != 0u {
        if in.tex_index < 0 {
            color = vertex_color;
        } else {
            // Cooldown swipes use mask_tex_coords as sample UVs because tex_coords.x
            // is reserved for the radial progress value.
            let tex_color = sample_tiered_texture(in.tex_index, in.mask_tex_coords);
            color = tex_color * vertex_color;
        }
    } else if in.tex_index < 0 {
        // Solid color or pending texture (-1 = solid, -2 = pending)
        color = vertex_color;
    } else {
        // Textured quad - sample from the appropriate tier atlas
        let tex_color = sample_tiered_texture(in.tex_index, in.tex_coords);
        color = tex_color * vertex_color;
    }

    let blend_mode = in.flags & 0xFFu;

    // Circle clip (for minimap) — uses local_uv which is preserved across atlas remapping
    const FLAG_CIRCLE_CLIP: u32 = 0x100u;
    if (in.flags & FLAG_CIRCLE_CLIP) != 0u {
        let centered = in.local_uv * 2.0 - 1.0;
        let dist = length(centered);
        color.a *= 1.0 - smoothstep(0.96, 1.0, dist);
    }

    // Cooldown swipe — radial clock wipe from 12 o'clock clockwise.
    // tex_coords.x holds progress (0.0 = fully covered, 1.0 = fully revealed).
    // Pixels where the clock sweep has NOT yet passed are kept; passed pixels discarded.
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
            color.a = 0.0;
        }
    }

    // Desaturation — convert to greyscale using luminance weights.
    // Applied before masking so the mask alpha still works correctly.
    const FLAG_DESATURATE: u32 = 0x400u;
    if (in.flags & FLAG_DESATURATE) != 0u {
        let lum = dot(color.rgb, vec3f(0.2126, 0.7152, 0.0722));
        color = vec4f(vec3f(lum), color.a);
    }

    // Mask texture sampling. Some WoW masks encode coverage in alpha even
    // when visible regions are black; others encode coverage as white-on-black
    // RGB with opaque alpha.
    if in.mask_tex_index >= 0 {
        let mask_color = sample_tiered_texture(in.mask_tex_index, in.mask_tex_coords);
        const FLAG_MASK_ALPHA_COVERAGE: u32 = 0x800u;
        if (in.flags & FLAG_MASK_ALPHA_COVERAGE) != 0u {
            color.a *= mask_color.a;
        } else {
            let mask_intensity = max(mask_color.r, max(mask_color.g, mask_color.b));
            color.a *= mask_color.a * mask_intensity;
        }
    }

    // Brightness boost — the simulator has no 3D game world behind the UI,
    // so dark/semi-transparent textures (borders, backgrounds) are harder to
    // see against the black clear color. Additive overlays already brighten the
    // destination; boosting them also makes near-transparent atlas edge pixels
    // visible as stripes.
    if blend_mode != BLEND_ADDITIVE {
        color = vec4f(pow(color.rgb, vec3f(1.0 / uniforms.params.x)), color.a);
    }

    // Premultiplied alpha output: pipeline uses src + dst * (1 - src.a).
    // Standard alpha: premultiply color by alpha for correct blending.
    // Additive: premultiply then zero alpha so dst is fully preserved (src + dst).
    color = vec4f(color.rgb * color.a, color.a);
    if blend_mode == BLEND_ADDITIVE {
        color.a = 0.0;
    }

    return color;
}
