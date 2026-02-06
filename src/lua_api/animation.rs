//! Animation group state, handles, and tick logic.

use crate::lua_api::SimState;
use mlua::{Lua, MultiValue, RegistryKey, UserData, UserDataMethods, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Extract a numeric value from a Lua argument list at the given index.
fn extract_number(args: &[Value], index: usize) -> Option<f64> {
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
    fn total_time(&self) -> f64 {
        self.start_delay + self.duration + self.end_delay
    }

    /// Raw progress [0..1] based on elapsed time (within active duration, excluding delays).
    fn raw_progress(&self) -> f64 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        let active_elapsed = (self.elapsed - self.start_delay).clamp(0.0, self.duration);
        active_elapsed / self.duration
    }

    /// Smoothed progress [0..1] applying the easing function.
    fn smooth_progress(&self) -> f64 {
        compute_smoothed_progress(self.raw_progress(), self.smoothing)
    }

    /// Whether this animation has finished its total time.
    #[allow(dead_code)]
    fn is_finished(&self) -> bool {
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
    fn order_groups(&self) -> Vec<u32> {
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

/// Userdata handle for an AnimationGroup.
#[derive(Clone)]
pub struct AnimGroupHandle {
    pub group_id: u64,
    pub state: Rc<RefCell<SimState>>,
}

/// Userdata handle for an individual Animation.
#[derive(Clone)]
pub struct AnimHandle {
    pub group_id: u64,
    pub anim_index: usize,
    pub state: Rc<RefCell<SimState>>,
}

impl AnimGroupHandle {
    /// Register playback control methods: Play, Stop, Pause, Finish, Restart, PlaySynced.
    fn add_playback_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Play", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let reverse = args.first().and_then(|v| {
                if let Value::Boolean(b) = v { Some(*b) } else { None }
            }).unwrap_or(false);

            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                group.playing = true;
                group.paused = false;
                group.finished = false;
                group.reverse = reverse;
                group.elapsed = 0.0;
                for anim in &mut group.animations {
                    anim.elapsed = 0.0;
                }
            }
            Ok(())
        });

        methods.add_method("Stop", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                group.playing = false;
                group.paused = false;
                group.finished = true;
            }
            Ok(())
        });

        methods.add_method("Pause", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if group.playing {
                    group.paused = true;
                    group.playing = false;
                }
            }
            Ok(())
        });

        methods.add_method("Finish", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                group.playing = false;
                group.paused = false;
                group.finished = true;
                for anim in &mut group.animations {
                    anim.elapsed = anim.total_time();
                }
                group.elapsed = group.total_duration();
            }
            Ok(())
        });

        methods.add_method("Restart", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let reverse = args.first().and_then(|v| {
                if let Value::Boolean(b) = v { Some(*b) } else { None }
            }).unwrap_or(false);

            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                group.playing = true;
                group.paused = false;
                group.finished = false;
                group.reverse = reverse;
                group.elapsed = 0.0;
                for anim in &mut group.animations {
                    anim.elapsed = 0.0;
                }
            }
            Ok(())
        });

        methods.add_method("PlaySynced", |_, _this, _args: MultiValue| {
            Ok(())
        });
    }

    /// Register state query methods: IsPlaying, IsPaused, IsDone, IsPendingFinish, IsReverse.
    fn add_state_query_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("IsPlaying", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(false, |g| g.playing))
        });

        methods.add_method("IsPaused", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(false, |g| g.paused))
        });

        methods.add_method("IsDone", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(true, |g| g.finished))
        });

        methods.add_method("IsPendingFinish", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(false, |g| g.finished))
        });

        methods.add_method("IsReverse", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(false, |g| g.reverse))
        });
    }

    /// Register looping methods: SetLooping, GetLooping, GetLoopState.
    fn add_looping_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetLooping", |_, this, looping: Option<String>| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                group.looping = LoopType::from_str(looping.as_deref().unwrap_or("NONE"));
            }
            Ok(())
        });

        methods.add_method("GetLooping", |lua, this, ()| {
            let state = this.state.borrow();
            let s = state.animation_groups.get(&this.group_id)
                .map_or("NONE", |g| g.looping.as_str());
            Ok(Value::String(lua.create_string(s)?))
        });

        methods.add_method("GetLoopState", |lua, this, ()| {
            let state = this.state.borrow();
            let s = state.animation_groups.get(&this.group_id)
                .map_or("NONE", |g| g.looping.as_str());
            Ok(Value::String(lua.create_string(s)?))
        });
    }

    /// Register timing methods: GetDuration, GetElapsed, GetProgress, speed multiplier.
    fn add_timing_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("GetDuration", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(0.0, |g| g.total_duration()))
        });

        methods.add_method("GetElapsed", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(0.0, |g| g.elapsed))
        });

        methods.add_method("GetProgress", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(0.0, |g| {
                let dur = g.total_duration();
                if dur <= 0.0 { 0.0 } else { (g.elapsed / dur).clamp(0.0, 1.0) }
            }))
        });

        methods.add_method("SetAnimationSpeedMultiplier", |_, this, mult: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                group.speed_multiplier = mult;
            }
            Ok(())
        });

        methods.add_method("GetAnimationSpeedMultiplier", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(1.0, |g| g.speed_multiplier))
        });
    }

    /// Register alpha methods: SetToFinalAlpha, IsSetToFinalAlpha, GetToFinalAlpha, SetAlpha, GetAlpha.
    fn add_alpha_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetToFinalAlpha", |_, this, val: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                group.set_to_final_alpha = val;
            }
            Ok(())
        });

        methods.add_method("IsSetToFinalAlpha", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(false, |g| g.set_to_final_alpha))
        });

        methods.add_method("GetToFinalAlpha", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(false, |g| g.set_to_final_alpha))
        });

        methods.add_method("SetAlpha", |_, _this, _alpha: f64| {
            Ok(())
        });

        methods.add_method("GetAlpha", |_, _this, ()| {
            Ok(1.0_f64)
        });
    }

    /// Register script handler methods: SetScript, GetScript, HasScript, HookScript.
    fn add_script_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetScript", |lua, this, (event, handler): (String, Option<mlua::Function>)| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(old_key) = group.scripts.remove(&event) {
                    lua.remove_registry_value(old_key).ok();
                }
                if let Some(func) = handler {
                    let key = lua.create_registry_value(func)?;
                    group.scripts.insert(event, key);
                }
            }
            Ok(())
        });

        methods.add_method("GetScript", |lua, this, event: String| {
            let state = this.state.borrow();
            if let Some(group) = state.animation_groups.get(&this.group_id) {
                if let Some(key) = group.scripts.get(&event) {
                    if let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                        return Ok(Value::Function(func));
                    }
                }
            }
            Ok(Value::Nil)
        });

        methods.add_method("HasScript", |_, this, event: String| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .map_or(false, |g| g.scripts.contains_key(&event)))
        });

        methods.add_method("HookScript", |lua, this, (event, handler): (String, Option<mlua::Function>)| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(func) = handler {
                    if let Some(old_key) = group.scripts.remove(&event) {
                        lua.remove_registry_value(old_key).ok();
                    }
                    let key = lua.create_registry_value(func)?;
                    group.scripts.insert(event, key);
                }
            }
            Ok(())
        });
    }

    /// Register identity/hierarchy methods: GetName, GetParent.
    fn add_identity_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("GetName", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.name.clone()))
        });

        methods.add_method("GetParent", |lua, this, ()| {
            let state = this.state.borrow();
            if let Some(group) = state.animation_groups.get(&this.group_id) {
                let frame_key = format!("__frame_{}", group.owner_frame_id);
                let frame: Value = lua.globals().get(frame_key.as_str())?;
                return Ok(frame);
            }
            Ok(Value::Nil)
        });
    }

    /// Register animation management methods: GetAnimations, CreateAnimation, RemoveAnimations.
    fn add_animation_management_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("GetAnimations", |lua, this, ()| {
            let state = this.state.borrow();
            if let Some(group) = state.animation_groups.get(&this.group_id) {
                let mut values = Vec::new();
                for i in 0..group.animations.len() {
                    let handle = AnimHandle {
                        group_id: this.group_id,
                        anim_index: i,
                        state: Rc::clone(&this.state),
                    };
                    values.push(Value::UserData(lua.create_userdata(handle)?));
                }
                return Ok(MultiValue::from_vec(values));
            }
            Ok(MultiValue::new())
        });

        methods.add_method("CreateAnimation", |lua, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let anim_type_str = args.first().and_then(|v| {
                if let Value::String(s) = v { Some(s.to_string_lossy().to_string()) } else { None }
            });
            let anim_name = args.get(1).and_then(|v| {
                if let Value::String(s) = v { Some(s.to_string_lossy().to_string()) } else { None }
            });

            let anim_type = AnimationType::from_str(anim_type_str.as_deref().unwrap_or("Animation"));
            let mut anim = AnimState::new(anim_type);
            anim.name = anim_name;

            let anim_index;
            {
                let mut state = this.state.borrow_mut();
                if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                    anim_index = group.animations.len();
                    group.animations.push(anim);
                } else {
                    return Err(mlua::Error::runtime("Animation group not found"));
                }
            }

            let handle = AnimHandle {
                group_id: this.group_id,
                anim_index,
                state: Rc::clone(&this.state),
            };
            Ok(lua.create_userdata(handle)?)
        });

        methods.add_method("RemoveAnimations", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                group.animations.clear();
            }
            Ok(())
        });
    }
}

impl UserData for AnimGroupHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_playback_methods(methods);
        Self::add_state_query_methods(methods);
        Self::add_looping_methods(methods);
        Self::add_timing_methods(methods);
        Self::add_alpha_methods(methods);
        Self::add_script_methods(methods);
        Self::add_identity_methods(methods);
        Self::add_animation_management_methods(methods);
    }
}

impl AnimHandle {
    /// Register timing methods: SetDuration, GetDuration, delays, and order.
    fn add_timing_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetDuration", |_, this, dur: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.duration = dur;
                }
            }
            Ok(())
        });

        methods.add_method("GetDuration", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(0.0, |a| a.duration))
        });

        // Delays
        methods.add_method("SetStartDelay", |_, this, delay: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.start_delay = delay;
                }
            }
            Ok(())
        });

        methods.add_method("GetStartDelay", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(0.0, |a| a.start_delay))
        });

        methods.add_method("SetEndDelay", |_, this, delay: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.end_delay = delay;
                }
            }
            Ok(())
        });

        methods.add_method("GetEndDelay", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(0.0, |a| a.end_delay))
        });

        // Order
        methods.add_method("SetOrder", |_, this, order: u32| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.order = order;
                }
            }
            Ok(())
        });

        methods.add_method("GetOrder", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(1_u32, |a| a.order))
        });
    }

    /// Register all property methods by delegating to sub-helpers.
    fn add_property_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_smoothing_methods(methods);
        Self::add_alpha_property_methods(methods);
        Self::add_translation_methods(methods);
        Self::add_scale_methods(methods);
        Self::add_rotation_methods(methods);
    }

    /// Register smoothing methods: SetSmoothing, GetSmoothing.
    fn add_smoothing_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetSmoothing", |_, this, smooth: String| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.smoothing = Smoothing::from_str(&smooth);
                }
            }
            Ok(())
        });

        methods.add_method("GetSmoothing", |lua, this, ()| {
            let state = this.state.borrow();
            let s = state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or("NONE", |a| a.smoothing.as_str());
            Ok(Value::String(lua.create_string(s)?))
        });
    }

    /// Register alpha property methods: SetFromAlpha, GetFromAlpha, SetToAlpha, GetToAlpha.
    fn add_alpha_property_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetFromAlpha", |_, this, alpha: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.from_alpha = alpha;
                }
            }
            Ok(())
        });

        methods.add_method("GetFromAlpha", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(0.0, |a| a.from_alpha))
        });

        methods.add_method("SetToAlpha", |_, this, alpha: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.to_alpha = alpha;
                }
            }
            Ok(())
        });

        methods.add_method("GetToAlpha", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(1.0, |a| a.to_alpha))
        });
    }

    /// Register translation methods: SetOffset, SetChange.
    fn add_translation_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetOffset", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let x = extract_number(&args, 0).unwrap_or(0.0);
            let y = extract_number(&args, 1).unwrap_or(0.0);
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.offset_x = x;
                    anim.offset_y = y;
                }
            }
            Ok(())
        });

        methods.add_method("SetChange", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let val = extract_number(&args, 0).unwrap_or(0.0);
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    if anim.anim_type == AnimationType::Alpha {
                        anim.to_alpha = anim.from_alpha + val;
                    }
                }
            }
            Ok(())
        });
    }

    /// Register scale methods: SetScale, SetScaleFrom, SetScaleTo.
    fn add_scale_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetScale", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let x = extract_number(&args, 0).unwrap_or(1.0);
            let y = extract_number(&args, 1).unwrap_or(1.0);
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.scale_x = x;
                    anim.scale_y = y;
                }
            }
            Ok(())
        });

        methods.add_method("SetScaleFrom", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let x = extract_number(&args, 0).unwrap_or(1.0);
            let y = extract_number(&args, 1).unwrap_or(1.0);
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.from_scale_x = x;
                    anim.from_scale_y = y;
                }
            }
            Ok(())
        });

        methods.add_method("SetScaleTo", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let x = extract_number(&args, 0).unwrap_or(1.0);
            let y = extract_number(&args, 1).unwrap_or(1.0);
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.to_scale_x = x;
                    anim.to_scale_y = y;
                }
            }
            Ok(())
        });
    }

    /// Register rotation methods: SetDegrees, SetOrigin.
    fn add_rotation_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetDegrees", |_, this, degrees: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.degrees = degrees;
                }
            }
            Ok(())
        });

        methods.add_method("SetOrigin", |_, _this, _args: MultiValue| {
            Ok(()) // Store only, no visual effect
        });
    }

    /// Register playback control stubs and state queries.
    fn add_playback_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Play", |_, _this, ()| Ok(()));
        methods.add_method("Stop", |_, _this, ()| Ok(()));
        methods.add_method("Pause", |_, _this, ()| Ok(()));
        methods.add_method("Restart", |_, _this, ()| Ok(()));
        methods.add_method("Finish", |_, _this, ()| Ok(()));

        methods.add_method("IsPlaying", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(false, |g| g.playing))
        });

        methods.add_method("IsPaused", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(false, |g| g.paused))
        });

        methods.add_method("IsDone", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(true, |g| g.finished))
        });

        methods.add_method("IsStopped", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).map_or(true, |g| !g.playing && !g.paused))
        });

        methods.add_method("IsDelaying", |_, _this, ()| Ok(false));
    }

    /// Register progress query methods: GetProgress, GetSmoothProgress, GetElapsed.
    fn add_progress_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("GetProgress", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(0.0, |a| a.raw_progress()))
        });

        methods.add_method("GetSmoothProgress", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(0.0, |a| a.smooth_progress()))
        });

        methods.add_method("GetElapsed", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(0.0, |a| a.elapsed))
        });
    }

    /// Register parent, name, and target accessor methods.
    fn add_accessor_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("GetParent", |lua, this, ()| {
            let handle = AnimGroupHandle {
                group_id: this.group_id,
                state: Rc::clone(&this.state),
            };
            Ok(lua.create_userdata(handle)?)
        });

        methods.add_method("GetRegionParent", |lua, this, ()| {
            let state = this.state.borrow();
            if let Some(group) = state.animation_groups.get(&this.group_id) {
                let frame_key = format!("__frame_{}", group.owner_frame_id);
                let frame: Value = lua.globals().get(frame_key.as_str())?;
                return Ok(frame);
            }
            Ok(Value::Nil)
        });

        // Name
        methods.add_method("GetName", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .and_then(|a| a.name.clone()))
        });

        // Target methods (stubs — no visual effect)
        methods.add_method("GetTarget", |_, _this, ()| Ok(Value::Nil));
        methods.add_method("SetTarget", |_, _this, _target: Value| Ok(()));
        methods.add_method("SetChildKey", |_, _this, _key: String| Ok(()));
        methods.add_method("SetTargetKey", |_, _this, _key: String| Ok(()));
        methods.add_method("SetTargetName", |_, _this, _name: String| Ok(()));
        methods.add_method("SetTargetParent", |_, _this, ()| Ok(()));
    }

    /// Register script handler methods: SetScript, GetScript, HasScript, HookScript.
    fn add_script_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetScript", |lua, this, (event, handler): (String, Option<mlua::Function>)| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    if let Some(old_key) = anim.scripts.remove(&event) {
                        lua.remove_registry_value(old_key).ok();
                    }
                    if let Some(func) = handler {
                        let key = lua.create_registry_value(func)?;
                        anim.scripts.insert(event, key);
                    }
                }
            }
            Ok(())
        });

        methods.add_method("GetScript", |lua, this, event: String| {
            let state = this.state.borrow();
            if let Some(group) = state.animation_groups.get(&this.group_id) {
                if let Some(anim) = group.animations.get(this.anim_index) {
                    if let Some(key) = anim.scripts.get(&event) {
                        if let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                            return Ok(Value::Function(func));
                        }
                    }
                }
            }
            Ok(Value::Nil)
        });

        methods.add_method("HasScript", |_, this, event: String| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(false, |a| a.scripts.contains_key(&event)))
        });

        methods.add_method("HookScript", |lua, this, (event, handler): (String, Option<mlua::Function>)| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id) {
                if let Some(anim) = group.animations.get_mut(this.anim_index) {
                    if let Some(func) = handler {
                        if let Some(old_key) = anim.scripts.remove(&event) {
                            lua.remove_registry_value(old_key).ok();
                        }
                        let key = lua.create_registry_value(func)?;
                        anim.scripts.insert(event, key);
                    }
                }
            }
            Ok(())
        });
    }
}

impl UserData for AnimHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_timing_methods(methods);
        Self::add_property_methods(methods);
        Self::add_playback_methods(methods);
        Self::add_progress_methods(methods);
        Self::add_accessor_methods(methods);
        Self::add_script_methods(methods);
    }
}

/// Advance all playing animation groups by `delta` seconds.
/// Applies alpha animations to frame opacity and fires script callbacks.
pub fn tick_animation_groups(state_rc: &Rc<RefCell<SimState>>, lua: &Lua, delta: f64) -> mlua::Result<()> {
    let playing_ids: Vec<u64> = {
        let state = state_rc.borrow();
        state.animation_groups.iter()
            .filter(|(_, g)| g.playing && !g.paused)
            .map(|(id, _)| *id)
            .collect()
    };

    for group_id in playing_ids {
        let (finished, scripts_to_fire) = advance_group(state_rc, &group_id, lua, delta);
        fire_animation_scripts(state_rc, lua, group_id, finished, &scripts_to_fire, delta)?;
    }

    Ok(())
}

/// Advance a single animation group: update elapsed times, compute alpha, handle looping.
/// Returns (group_finished, scripts_to_fire).
fn advance_group(
    state_rc: &Rc<RefCell<SimState>>,
    group_id: &u64,
    lua: &Lua,
    delta: f64,
) -> (bool, Vec<mlua::Function>) {
    let mut state = state_rc.borrow_mut();

    let (group_finished, alpha_to_apply, scripts_to_fire) = {
        let Some(group) = state.animation_groups.get_mut(group_id) else {
            return (false, Vec::new());
        };

        let effective_delta = delta * group.speed_multiplier;
        group.elapsed += effective_delta;

        let alpha_to_apply = advance_order_groups(group);
        let total_dur = group.total_duration();
        let group_finished = group.elapsed >= total_dur;

        let scripts_to_fire = if group_finished {
            handle_group_finish(group, total_dur, lua)
        } else {
            Vec::new()
        };

        (group_finished, alpha_to_apply, scripts_to_fire)
    };

    if let Some((frame_id, alpha)) = alpha_to_apply {
        if let Some(frame) = state.widgets.get_mut(frame_id) {
            frame.alpha = alpha;
        }
    }

    (group_finished, scripts_to_fire)
}

/// Advance animations within each order group, returning computed alpha if any.
fn advance_order_groups(group: &mut AnimGroupState) -> Option<(u64, f32)> {
    let owner_frame_id = group.owner_frame_id;
    let orders = group.order_groups();
    let mut order_start = 0.0;
    let mut alpha_to_apply: Option<(u64, f32)> = None;

    for &order in &orders {
        let order_dur = group.animations.iter()
            .filter(|a| a.order == order)
            .map(|a| a.total_time())
            .fold(0.0_f64, f64::max);

        let time_in_group = (group.elapsed - order_start).clamp(0.0, order_dur);

        for anim in group.animations.iter_mut().filter(|a| a.order == order) {
            anim.elapsed = time_in_group.min(anim.total_time());
        }

        for anim in group.animations.iter().filter(|a| a.order == order) {
            if anim.anim_type == AnimationType::Alpha {
                let progress = if group.reverse {
                    1.0 - anim.smooth_progress()
                } else {
                    anim.smooth_progress()
                };
                let alpha = anim.from_alpha + (anim.to_alpha - anim.from_alpha) * progress;
                alpha_to_apply = Some((owner_frame_id, alpha as f32));
            }
        }

        order_start += order_dur;
    }

    alpha_to_apply
}

/// Handle a finished animation group: update state and collect scripts to fire.
fn handle_group_finish(
    group: &mut AnimGroupState,
    total_dur: f64,
    lua: &Lua,
) -> Vec<mlua::Function> {
    let mut scripts = Vec::new();

    match group.looping {
        LoopType::None => {
            group.playing = false;
            group.finished = true;
            if let Some(key) = group.scripts.get("OnFinished") {
                if let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                    scripts.push(func);
                }
            }
        }
        LoopType::Repeat => {
            group.elapsed -= total_dur;
            for anim in &mut group.animations {
                anim.elapsed = 0.0;
            }
            if let Some(key) = group.scripts.get("OnLoop") {
                if let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                    scripts.push(func);
                }
            }
        }
        LoopType::Bounce => {
            group.elapsed -= total_dur;
            group.reverse = !group.reverse;
            for anim in &mut group.animations {
                anim.elapsed = 0.0;
            }
            if let Some(key) = group.scripts.get("OnLoop") {
                if let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                    scripts.push(func);
                }
            }
        }
    }

    scripts
}

/// Fire collected animation scripts and OnUpdate callback for a group.
fn fire_animation_scripts(
    state_rc: &Rc<RefCell<SimState>>,
    lua: &Lua,
    group_id: u64,
    finished: bool,
    scripts_to_fire: &[mlua::Function],
    delta: f64,
) -> mlua::Result<()> {
    for func in scripts_to_fire {
        let handle = AnimGroupHandle {
            group_id,
            state: Rc::clone(state_rc),
        };
        let ud = lua.create_userdata(handle)?;
        if let Err(e) = func.call::<()>(ud) {
            eprintln!("Animation script error: {}", e);
        }
    }

    let on_update_func = {
        let state = state_rc.borrow();
        state.animation_groups.get(&group_id)
            .and_then(|g| {
                if g.playing || !finished {
                    g.scripts.get("OnUpdate")
                        .and_then(|key| lua.registry_value::<mlua::Function>(key).ok())
                } else {
                    None
                }
            })
    };

    if let Some(func) = on_update_func {
        let handle = AnimGroupHandle {
            group_id,
            state: Rc::clone(state_rc),
        };
        let ud = lua.create_userdata(handle)?;
        if let Err(e) = func.call::<()>((ud, delta)) {
            eprintln!("Animation OnUpdate error: {}", e);
        }
    }

    Ok(())
}
