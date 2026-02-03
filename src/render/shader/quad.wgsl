// WoW UI Quad Shader
//
// Renders textured/colored quads for UI elements.
// Supports solid colors and tiered texture array sampling.
// Textures are stored in 4 tiers: 64x64, 128x128, 256x256, 512x512.
// tex_index encodes tier and layer: tier * 1000 + layer.

// Uniforms (group 0)
struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Tiered texture arrays (group 1)
@group(1) @binding(0)
var tier_64: texture_2d_array<f32>;    // 64x64 textures

@group(1) @binding(1)
var tier_128: texture_2d_array<f32>;   // 128x128 textures

@group(1) @binding(2)
var tier_256: texture_2d_array<f32>;   // 256x256 textures

@group(1) @binding(3)
var tier_512: texture_2d_array<f32>;   // 512x512 textures

@group(1) @binding(4)
var texture_sampler: sampler;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) tex_index: i32,
    @location(4) flags: u32,
}

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) tex_index: i32,
    @location(3) @interpolate(flat) flags: u32,
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

    return out;
}

// Blend mode constants
const BLEND_ALPHA: u32 = 0u;
const BLEND_ADDITIVE: u32 = 1u;

// Sample from the appropriate tier based on encoded tex_index
fn sample_tiered_texture(tex_index: i32, tex_coords: vec2<f32>) -> vec4<f32> {
    // Decode tier and layer from tex_index (tier * 1000 + layer)
    let tier = tex_index / 1000;
    let layer = tex_index % 1000;

    // Sample from the appropriate tier
    switch tier {
        case 0: {
            return textureSample(tier_64, texture_sampler, tex_coords, layer);
        }
        case 1: {
            return textureSample(tier_128, texture_sampler, tex_coords, layer);
        }
        case 2: {
            return textureSample(tier_256, texture_sampler, tex_coords, layer);
        }
        case 3: {
            return textureSample(tier_512, texture_sampler, tex_coords, layer);
        }
        default: {
            // Invalid tier, return magenta for debugging
            return vec4<f32>(1.0, 0.0, 1.0, 1.0);
        }
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
        // Sample from tiered texture array
        let tex_color = sample_tiered_texture(in.tex_index, in.tex_coords);
        // Multiply by vertex color for tinting
        color = tex_color * in.color;
    }

    // Apply blend mode adjustments
    // Note: Actual blending is handled by the pipeline blend state,
    // but we can adjust the output color for additive effects
    let blend_mode = in.flags & 0xFFu;
    if blend_mode == BLEND_ADDITIVE {
        // For additive blending, we want the color to add to the background
        // The pipeline should be set to additive blend for these quads
        // For now, just boost the alpha slightly for visibility
        color.a = min(color.a * 1.5, 1.0);
    }

    return color;
}
