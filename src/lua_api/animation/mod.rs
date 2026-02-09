//! Animation group state, handles, and tick logic.

mod anim_handle;
mod group_handle;
mod tick;

use mlua::{RegistryKey, Value};
use std::collections::HashMap;

pub use anim_handle::AnimHandle;
pub use group_handle::AnimGroupHandle;
pub use tick::tick_animation_groups;

/// Extract a numeric value from a Lua argument list at the given index.
pub(crate) fn extract_number(args: &[Value], index: usize) -> Option<f64> {
    args.get(index).and_then(|v| match v {
        Value::Number(n) => Some(*n),
        Value::Integer(n) => Some(*n as f64),
        _ => None,
    })
}

/// Animation type (Alpha, Translation, etc.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationType {
    Alpha,
    Translation,
    Scale,
    Rotation,
    LineTranslation,
    LineScale,
    Path,
    FlipBook,
    VertexColor,
    TextureCoordTranslation,
    Animation, // generic
}

impl AnimationType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ALPHA" => Self::Alpha,
            "TRANSLATION" => Self::Translation,
            "SCALE" => Self::Scale,
            "ROTATION" => Self::Rotation,
            "LINETRANSLATION" => Self::LineTranslation,
            "LINESCALE" => Self::LineScale,
            "PATH" => Self::Path,
            "FLIPBOOK" => Self::FlipBook,
            "VERTEXCOLOR" => Self::VertexColor,
            "TEXTURECOORDTRANSLATION" => Self::TextureCoordTranslation,
            _ => Self::Animation,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Alpha => "Alpha",
            Self::Translation => "Translation",
            Self::Scale => "Scale",
            Self::Rotation => "Rotation",
            Self::LineTranslation => "LineTranslation",
            Self::LineScale => "LineScale",
            Self::Path => "Path",
            Self::FlipBook => "FlipBook",
            Self::VertexColor => "VertexColor",
            Self::TextureCoordTranslation => "TextureCoordTranslation",
            Self::Animation => "Animation",
        }
    }
}

/// Smoothing (easing) type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Smoothing {
    None,
    In,
    Out,
    InOut,
}

impl Smoothing {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "IN" => Self::In,
            "OUT" => Self::Out,
            "IN_OUT" | "INOUT" => Self::InOut,
            _ => Self::None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::In => "IN",
            Self::Out => "OUT",
            Self::InOut => "IN_OUT",
        }
    }
}

/// Loop type for animation groups.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopType {
    None,
    Repeat,
    Bounce,
}

impl LoopType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "REPEAT" => Self::Repeat,
            "BOUNCE" => Self::Bounce,
            _ => Self::None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Repeat => "REPEAT",
            Self::Bounce => "BOUNCE",
        }
    }
}

/// State for a single animation within a group.
pub struct AnimState {
    pub anim_type: AnimationType,
    pub name: Option<String>,
    pub order: u32,
    pub duration: f64,
    pub start_delay: f64,
    pub end_delay: f64,
    pub smoothing: Smoothing,
    /// childKey: target a child region by parentKey instead of the owner frame.
    pub child_key: Option<String>,
    // Alpha
    pub from_alpha: f64,
    pub to_alpha: f64,
    // Translation
    pub offset_x: f64,
    pub offset_y: f64,
    // Scale
    pub scale_x: f64,
    pub scale_y: f64,
    pub from_scale_x: f64,
    pub from_scale_y: f64,
    pub to_scale_x: f64,
    pub to_scale_y: f64,
    // Rotation
    pub degrees: f64,
    // Runtime
    pub elapsed: f64,
    /// Script handlers (OnPlay, OnFinished, OnStop, etc.)
    pub scripts: HashMap<String, RegistryKey>,
}

impl AnimState {
    pub fn new(anim_type: AnimationType) -> Self {
        Self {
            anim_type,
            name: None,
            order: 1,
            duration: 0.0,
            start_delay: 0.0,
            end_delay: 0.0,
            smoothing: Smoothing::None,
            child_key: None,
            from_alpha: 0.0,
            to_alpha: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            from_scale_x: 1.0,
            from_scale_y: 1.0,
            to_scale_x: 1.0,
            to_scale_y: 1.0,
            degrees: 0.0,
            elapsed: 0.0,
            scripts: HashMap::new(),
        }
    }

    /// Total time for this animation (start_delay + duration + end_delay).
    pub(crate) fn total_time(&self) -> f64 {
        self.start_delay + self.duration + self.end_delay
    }

    /// Raw progress [0..1] based on elapsed time (within active duration, excluding delays).
    pub(crate) fn raw_progress(&self) -> f64 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        let active_elapsed = (self.elapsed - self.start_delay).clamp(0.0, self.duration);
        active_elapsed / self.duration
    }

    /// Smoothed progress [0..1] applying the easing function.
    pub(crate) fn smooth_progress(&self) -> f64 {
        compute_smoothed_progress(self.raw_progress(), self.smoothing)
    }

    /// Whether this animation is past its start delay and actively animating.
    pub(crate) fn is_active(&self) -> bool {
        self.elapsed >= self.start_delay
    }

    /// Whether this animation has finished its total time.
    #[allow(dead_code)]
    pub(crate) fn is_finished(&self) -> bool {
        self.elapsed >= self.total_time()
    }
}

/// State for an animation group.
pub struct AnimGroupState {
    pub owner_frame_id: u64,
    pub name: Option<String>,
    pub playing: bool,
    pub paused: bool,
    pub finished: bool,
    pub reverse: bool,
    pub elapsed: f64,
    pub looping: LoopType,
    pub speed_multiplier: f64,
    pub set_to_final_alpha: bool,
    pub animations: Vec<AnimState>,
    /// Script handlers (OnPlay, OnFinished, OnStop, OnLoop, OnUpdate)
    pub scripts: HashMap<String, RegistryKey>,
    /// Pre-animation alpha saved on Play(), keyed by resolved frame ID.
    /// Restored on Stop/Finish when `set_to_final_alpha` is false.
    pub saved_alphas: HashMap<u64, f32>,
}

impl AnimGroupState {
    pub fn new(owner_frame_id: u64) -> Self {
        Self {
            owner_frame_id,
            name: None,
            playing: false,
            paused: false,
            finished: false,
            reverse: false,
            elapsed: 0.0,
            looping: LoopType::None,
            speed_multiplier: 1.0,
            set_to_final_alpha: false,
            animations: Vec::new(),
            scripts: HashMap::new(),
            saved_alphas: HashMap::new(),
        }
    }

    /// Compute total duration of the group (max over order-groups sequentially).
    pub fn total_duration(&self) -> f64 {
        let max_order = self.animations.iter().map(|a| a.order).max().unwrap_or(0);
        let mut total = 0.0;
        for order in 1..=max_order {
            let group_dur = self
                .animations
                .iter()
                .filter(|a| a.order == order)
                .map(|a| a.total_time())
                .fold(0.0_f64, f64::max);
            total += group_dur;
        }
        total
    }

    /// Get all unique order values sorted.
    pub(crate) fn order_groups(&self) -> Vec<u32> {
        let mut orders: Vec<u32> = self.animations.iter().map(|a| a.order).collect();
        orders.sort();
        orders.dedup();
        orders
    }
}

/// Compute smoothed progress using an easing function.
pub fn compute_smoothed_progress(t: f64, smoothing: Smoothing) -> f64 {
    let t = t.clamp(0.0, 1.0);
    match smoothing {
        Smoothing::None => t,
        Smoothing::In => t * t,
        Smoothing::Out => 1.0 - (1.0 - t) * (1.0 - t),
        Smoothing::InOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
    }
}
