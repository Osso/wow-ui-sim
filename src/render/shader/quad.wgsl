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
    color_space: vec4<u32>,
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
const BLEND_MODULATE: u32 = 2u;
const BLEND_OPAQUE: u32 = 3u;
const BLEND_ALPHAKEY: u32 = 4u;
// Alpha-test reference for ALPHAKEY (texels below it are not drawn).
const ALPHAKEY_REF: f32 = 0.5;
const TEX_INDEX_GLYPH: i32 = 5;
const ATLAS_SIZE: f32 = 4096.0;
const MAX_ATLAS_UV: f32 = 4095.5 / 4096.0;
const DISPLAY_RGB_MAX_BYTE: f32 = 255.0;
const DISPLAY_RGB_ROUNDING_HALF: f32 = 0.5;

fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    let low = color / 12.92;
    let high = pow(max((color + 0.055) / 1.055, vec3<f32>(0.0)), vec3<f32>(2.4));
    return select(high, low, color <= vec3<f32>(0.04045));
}

fn quantize_display_rgb(color: vec3<f32>) -> vec3<f32> {
    // Keep UI float values aligned with the 8-bit display-byte screenshot.
    return floor(color * DISPLAY_RGB_MAX_BYTE + DISPLAY_RGB_ROUNDING_HALF)
        / DISPLAY_RGB_MAX_BYTE;
}

fn linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    let low = color * 12.92;
    let high = 1.055 * pow(max(color, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, color <= vec3<f32>(0.0031308));
}

fn modulate_texture(
    tex_color: vec4<f32>,
    tint: vec4<f32>,
    sample_rgb_is_linear: bool,
) -> vec4<f32> {
    if uniforms.color_space.x != 0u {
        var display_color = tex_color.rgb;
        if sample_rgb_is_linear {
            display_color = linear_to_srgb(display_color);
        }
        display_color = quantize_display_rgb(display_color * tint.rgb);
        return vec4<f32>(srgb_to_linear(display_color), tex_color.a * tint.a);
    }
    return tex_color * tint;
}

fn prepare_solid_color(color: vec4<f32>) -> vec4<f32> {
    if uniforms.color_space.x != 0u {
        return vec4<f32>(srgb_to_linear(quantize_display_rgb(color.rgb)), color.a);
    }
    return color;
}

fn clamp_cell_uv_for_mip(
    uv: vec2<f32>,
    cell_origin: vec2<f32>,
    cell_uv: f32,
    mip_level: f32,
) -> vec2<f32> {
    let half_texel = 0.5 * exp2(mip_level) / ATLAS_SIZE;
    let min_uv = cell_origin + half_texel;
    let max_uv = max(min_uv, cell_origin + cell_uv - half_texel);
    return clamp(uv, min_uv, max_uv);
}

// The common tint/desaturation path works on straight RGB.
fn unpremultiply_sample(sample: vec4<f32>) -> vec4<f32> {
    if sample.a <= 0.0 {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(min(sample.rgb / sample.a, vec3<f32>(1.0)), sample.a);
}

fn sample_tier_mip(
    tex: texture_2d<f32>,
    uv: vec2<f32>,
    cell_origin: vec2<f32>,
    cell_uv: f32,
    lod: f32,
) -> vec4<f32> {
    let mip_uv = clamp_cell_uv_for_mip(uv, cell_origin, cell_uv, lod);
    return textureSampleLevel(tex, texture_sampler, mip_uv, lod);
}

fn sample_tier_trilinear(
    tex: texture_2d<f32>,
    uv: vec2<f32>,
    ddx: vec2<f32>,
    ddy: vec2<f32>,
    grid_size: f32,
    max_lod: f32,
) -> vec4<f32> {
    let rho = max(length(ddx * ATLAS_SIZE), length(ddy * ATLAS_SIZE));
    let lod = clamp(log2(max(rho, 1.0)), 0.0, max_lod);
    let cell_origin = floor(uv * grid_size) / grid_size;
    let cell_uv = 1.0 / grid_size;
    let mip_floor = floor(lod);
    var sample_floor = sample_tier_mip(tex, uv, cell_origin, cell_uv, mip_floor);
    let frac_lod = fract(lod);
    if frac_lod <= 0.001 {
        return select(unpremultiply_sample(sample_floor), sample_floor, mip_floor == 0.0);
    }
    if mip_floor == 0.0 {
        sample_floor = vec4<f32>(sample_floor.rgb * sample_floor.a, sample_floor.a);
    }
    let mip_ceil = min(mip_floor + 1.0, max_lod);
    let sample_ceil = sample_tier_mip(tex, uv, cell_origin, cell_uv, mip_ceil);
    return unpremultiply_sample(mix(sample_floor, sample_ceil, frac_lod));
}

fn sample_rgba_tier(
    tex_index: i32,
    uv: vec2<f32>,
    ddx: vec2<f32>,
    ddy: vec2<f32>,
) -> vec4<f32> {
    switch tex_index {
        case 0: { return sample_tier_trilinear(tier_64, uv, ddx, ddy, 64.0, 6.0); }
        case 1: { return sample_tier_trilinear(tier_128, uv, ddx, ddy, 32.0, 7.0); }
        case 2: { return sample_tier_trilinear(tier_256, uv, ddx, ddy, 16.0, 8.0); }
        case 3: { return sample_tier_trilinear(tier_512, uv, ddx, ddy, 8.0, 9.0); }
        default: { return sample_tier_trilinear(tier_2048, uv, ddx, ddy, 2.0, 11.0); }
    }
}

// Sample from the appropriate tier based on tex_index
// tex_index 0-4: tiered RGBA atlases, 5: glyph atlas, 6: BC1 atlas, 7: BC3 atlas
// UV coordinates are already transformed to the correct sub-region
fn sample_tiered_texture(
    tex_index: i32,
    tex_coords: vec2<f32>,
    ddx: vec2<f32>,
    ddy: vec2<f32>,
) -> vec4<f32> {
    let uv = clamp(tex_coords, vec2<f32>(0.0), vec2<f32>(MAX_ATLAS_UV));

    if tex_index >= 0 && tex_index < 5 {
        return sample_rgba_tier(tex_index, uv, ddx, ddy);
    }
    if tex_index == 6 {
        return textureSampleLevel(bc1_atlas, texture_sampler, uv, 0.0);
    }
    if tex_index == 7 {
        return textureSampleLevel(bc3_atlas, texture_sampler, uv, 0.0);
    }
    return textureSampleLevel(glyph_atlas, glyph_sampler, uv, 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Compute derivatives before any control-flow branch (WGSL uniformity requirement).
    let tex_ddx = dpdx(in.tex_coords);
    let tex_ddy = dpdy(in.tex_coords);
    let mask_ddx = dpdx(in.mask_tex_coords);
    let mask_ddy = dpdy(in.mask_tex_coords);

    var color: vec4<f32>;
    const FLAG_COOLDOWN_SWIPE: u32 = 0x200u;

    if (in.flags & FLAG_COOLDOWN_SWIPE) != 0u {
        if in.tex_index < 0 {
            color = prepare_solid_color(in.color);
        } else {
            // Cooldown swipes use mask_tex_coords as sample UVs because tex_coords.x
            // is reserved for the radial progress value.
            let tex_color =
                sample_tiered_texture(in.tex_index, in.mask_tex_coords, mask_ddx, mask_ddy);
            color = modulate_texture(
                tex_color,
                in.color,
                in.tex_index != TEX_INDEX_GLYPH,
            );
        }
    } else if in.tex_index < 0 {
        // Solid color or pending texture (-1 = solid, -2 = pending)
        color = prepare_solid_color(in.color);
    } else {
        // Textured quad - sample from the appropriate tier atlas
        let tex_color = sample_tiered_texture(in.tex_index, in.tex_coords, tex_ddx, tex_ddy);
        color = modulate_texture(
            tex_color,
            in.color,
            in.tex_index != TEX_INDEX_GLYPH,
        );
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

    // Mask coverage comes from alpha; opaque mask pixels may have black RGB.
    if in.mask_tex_index >= 0 {
        let mask_color =
            sample_tiered_texture(in.mask_tex_index, in.mask_tex_coords, mask_ddx, mask_ddy);
        color.a *= mask_color.a;
    }

    // Premultiplied alpha output: pipeline uses src + dst * (1 - src.a).
    // Standard alpha: premultiply color by alpha for correct blending.
    // Additive: premultiply then zero alpha so dst is fully preserved (src + dst).
    // MOD is drawn by the modulate pipeline (dst * src): the colour as is.
    if blend_mode == BLEND_MODULATE {
        return vec4f(color.rgb, 1.0);
    }
    // DISABLE: no blending, the colour replaces what is there, opaque.
    if blend_mode == BLEND_OPAQUE {
        return vec4f(color.rgb, 1.0);
    }
    // ALPHAKEY: alpha test, then opaque.
    if blend_mode == BLEND_ALPHAKEY {
        if color.a < ALPHAKEY_REF {
            discard;
        }
        return vec4f(color.rgb, 1.0);
    }
    color = vec4f(color.rgb * color.a, color.a);
    if blend_mode == BLEND_ADDITIVE {
        color.a = 0.0;
    }

    return color;
}
