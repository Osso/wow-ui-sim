//! Minimal animation state used by the current rilua-side create methods.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Animation,
}

impl AnimationType {
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
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
            Self::TextureCoordTranslation | Self::Animation => "Animation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopType {
    None,
    Repeat,
    Bounce,
}

impl LoopType {
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "REPEAT" => Self::Repeat,
            "BOUNCE" => Self::Bounce,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnimState {
    pub anim_type: AnimationType,
    pub name: Option<String>,
    pub child_key: Option<String>,
    pub order: u32,
    pub duration: f64,
    pub start_delay: f64,
    pub end_delay: f64,
    pub elapsed: f64,
    pub from_alpha: f64,
    pub to_alpha: f64,
    pub smoothing: String,
    pub flipbook_rows: u32,
    pub flipbook_columns: u32,
    pub flipbook_frames: u32,
    pub flipbook_frame_width: f64,
    pub flipbook_frame_height: f64,
    pub scripts: HashMap<String, ()>,
}

impl AnimState {
    pub fn new(anim_type: AnimationType) -> Self {
        Self {
            anim_type,
            name: None,
            child_key: None,
            order: 1,
            duration: 0.0,
            start_delay: 0.0,
            end_delay: 0.0,
            elapsed: 0.0,
            from_alpha: 0.0,
            to_alpha: 1.0,
            smoothing: "NONE".to_string(),
            flipbook_rows: 0,
            flipbook_columns: 0,
            flipbook_frames: 0,
            flipbook_frame_width: 0.0,
            flipbook_frame_height: 0.0,
            scripts: HashMap::new(),
        }
    }

    pub fn total_time(&self) -> f64 {
        self.start_delay + self.duration + self.end_delay
    }
}

#[derive(Debug, Clone)]
pub struct AnimGroupState {
    pub owner_frame_id: u64,
    pub frame_id: Option<u64>,
    pub name: Option<String>,
    pub playing: bool,
    pub paused: bool,
    pub done: bool,
    pub pending_finish: bool,
    pub reverse: bool,
    pub elapsed: f64,
    pub looping: LoopType,
    pub speed_multiplier: f64,
    pub set_to_final_alpha: bool,
    pub animations: Vec<AnimState>,
    pub scripts: HashMap<String, ()>,
    pub saved_alphas: HashMap<u64, f32>,
}

impl AnimGroupState {
    pub fn new(owner_frame_id: u64) -> Self {
        Self {
            owner_frame_id,
            frame_id: None,
            name: None,
            playing: false,
            paused: false,
            done: false,
            pending_finish: false,
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

    pub fn has_visual_effects(&self) -> bool {
        !self.animations.is_empty()
    }

    /// True when every animation is a plain `Animation`, which changes
    /// nothing on screen; addons use such groups as OnLoop/OnFinished timers.
    pub fn is_timer_only(&self) -> bool {
        self.animations
            .iter()
            .all(|animation| animation.anim_type == AnimationType::Animation)
    }

    /// Length of one pass: animations sharing an order run in parallel,
    /// orders run in sequence.
    pub fn total_duration(&self) -> f64 {
        let mut duration_by_order = std::collections::BTreeMap::<u32, f64>::new();
        for animation in &self.animations {
            let total_time = animation.total_time();
            duration_by_order
                .entry(animation.order)
                .and_modify(|current| *current = current.max(total_time))
                .or_insert(total_time);
        }
        duration_by_order.into_values().sum()
    }

    /// Real time until playback reaches the next loop or finish boundary,
    /// where OnLoop/OnFinished fire. `None` when playback is frozen.
    pub fn time_to_next_boundary(&self) -> Option<std::time::Duration> {
        if self.pending_finish {
            return Some(std::time::Duration::ZERO);
        }
        if self.speed_multiplier <= 0.0 {
            return None;
        }
        let remaining = if self.reverse {
            self.elapsed
        } else {
            self.total_duration() - self.elapsed
        };
        let real_seconds = remaining.max(0.0) / self.speed_multiplier;
        Some(std::time::Duration::from_secs_f64(real_seconds))
    }
}
