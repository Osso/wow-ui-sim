//! AnimHandle userdata methods.

use crate::lua_api::SimState;
use mlua::{MultiValue, UserData, UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;

use super::{extract_number, AnimGroupHandle, AnimationType, Smoothing};

/// Userdata handle for an individual Animation.
#[derive(Clone)]
pub struct AnimHandle {
    pub group_id: u64,
    pub anim_index: usize,
    pub state: Rc<RefCell<SimState>>,
}

impl AnimHandle {
    /// Register duration methods: SetDuration, GetDuration.
    fn add_duration_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetDuration", |_, this, dur: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.duration = dur;
                }
            Ok(())
        });

        methods.add_method("GetDuration", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(0.0, |a| a.duration))
        });
    }

    /// Register delay and order methods.
    fn add_delay_and_order_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetStartDelay", |_, this, delay: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.start_delay = delay;
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
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.end_delay = delay;
                }
            Ok(())
        });

        methods.add_method("GetEndDelay", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .map_or(0.0, |a| a.end_delay))
        });

        methods.add_method("SetOrder", |_, this, order: u32| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.order = order;
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
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.smoothing = Smoothing::from_str(&smooth);
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

    /// Register alpha property methods.
    fn add_alpha_property_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetFromAlpha", |_, this, alpha: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.from_alpha = alpha;
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
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.to_alpha = alpha;
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
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.offset_x = x;
                    anim.offset_y = y;
                }
            Ok(())
        });

        methods.add_method("SetChange", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let val = extract_number(&args, 0).unwrap_or(0.0);
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index)
                    && anim.anim_type == AnimationType::Alpha {
                        anim.to_alpha = anim.from_alpha + val;
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
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.scale_x = x;
                    anim.scale_y = y;
                }
            Ok(())
        });

        methods.add_method("SetScaleFrom", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let x = extract_number(&args, 0).unwrap_or(1.0);
            let y = extract_number(&args, 1).unwrap_or(1.0);
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.from_scale_x = x;
                    anim.from_scale_y = y;
                }
            Ok(())
        });

        methods.add_method("SetScaleTo", |_, this, args: MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();
            let x = extract_number(&args, 0).unwrap_or(1.0);
            let y = extract_number(&args, 1).unwrap_or(1.0);
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.to_scale_x = x;
                    anim.to_scale_y = y;
                }
            Ok(())
        });
    }

    /// Register rotation methods: SetDegrees, SetOrigin.
    fn add_rotation_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetDegrees", |_, this, degrees: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    anim.degrees = degrees;
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
            Ok(state.animation_groups.get(&this.group_id).is_some_and(|g| g.playing))
        });

        methods.add_method("IsPaused", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).is_some_and(|g| g.paused))
        });

        methods.add_method("IsDone", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).is_none_or(|g| g.finished))
        });

        methods.add_method("IsStopped", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id).is_none_or(|g| !g.playing && !g.paused))
        });

        methods.add_method("IsDelaying", |_, _this, ()| Ok(false));
    }

    /// Register progress query methods.
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
            lua.create_userdata(handle)
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

        methods.add_method("GetName", |_, this, ()| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .and_then(|a| a.name.clone()))
        });

        methods.add_method("GetTarget", |_, _this, ()| Ok(Value::Nil));
        methods.add_method("SetTarget", |_, _this, _target: Value| Ok(()));
        methods.add_method("SetChildKey", |_, _this, _key: String| Ok(()));
        methods.add_method("SetTargetKey", |_, _this, _key: String| Ok(()));
        methods.add_method("SetTargetName", |_, _this, _name: String| Ok(()));
        methods.add_method("SetTargetParent", |_, _this, ()| Ok(()));
    }

    /// Register script handler methods.
    fn add_script_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetScript", |lua, this, (event, handler): (String, Option<mlua::Function>)| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index) {
                    if let Some(old_key) = anim.scripts.remove(&event) {
                        lua.remove_registry_value(old_key).ok();
                    }
                    if let Some(func) = handler {
                        let key = lua.create_registry_value(func)?;
                        anim.scripts.insert(event, key);
                    }
                }
            Ok(())
        });

        methods.add_method("GetScript", |lua, this, event: String| {
            let state = this.state.borrow();
            if let Some(group) = state.animation_groups.get(&this.group_id)
                && let Some(anim) = group.animations.get(this.anim_index)
                    && let Some(key) = anim.scripts.get(&event)
                        && let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                            return Ok(Value::Function(func));
                        }
            Ok(Value::Nil)
        });

        methods.add_method("HasScript", |_, this, event: String| {
            let state = this.state.borrow();
            Ok(state.animation_groups.get(&this.group_id)
                .and_then(|g| g.animations.get(this.anim_index))
                .is_some_and(|a| a.scripts.contains_key(&event)))
        });

        methods.add_method("HookScript", |lua, this, (event, handler): (String, Option<mlua::Function>)| {
            let mut state = this.state.borrow_mut();
            if let Some(group) = state.animation_groups.get_mut(&this.group_id)
                && let Some(anim) = group.animations.get_mut(this.anim_index)
                    && let Some(func) = handler {
                        if let Some(old_key) = anim.scripts.remove(&event) {
                            lua.remove_registry_value(old_key).ok();
                        }
                        let key = lua.create_registry_value(func)?;
                        anim.scripts.insert(event, key);
                    }
            Ok(())
        });
    }
}

impl UserData for AnimHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        Self::add_duration_methods(methods);
        Self::add_delay_and_order_methods(methods);
        Self::add_property_methods(methods);
        Self::add_playback_methods(methods);
        Self::add_progress_methods(methods);
        Self::add_accessor_methods(methods);
        Self::add_script_methods(methods);
    }
}
