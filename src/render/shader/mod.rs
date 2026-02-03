//! GPU-accelerated shader rendering for WoW UI frames.
//!
//! This module provides wgpu-based rendering using iced's `shader::Primitive` trait,
//! replacing the CPU-bound canvas rendering with GPU-accelerated quad batching.

mod atlas;
mod pipeline;
mod primitive;
mod program;
mod quad;

pub use atlas::{GpuTextureAtlas, TextureEntry};
pub use pipeline::WowUiPipeline;
pub use primitive::{GpuTextureData, WowUiPrimitive};
pub use program::WowUiProgram;
pub use quad::{BlendMode, NineSliceTextures, QuadBatch, QuadVertex, TextureRequest};
