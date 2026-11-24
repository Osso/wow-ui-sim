//! Retail scripted-animation effect records from the local UIScriptedAnimationEffect DB2.
//!
//! Retail Blizzard_APIDocumentationGenerated/ScriptedAnimationsDocumentation.lua:11-16,
//! 78-108 declare the function and record. INFERRED (R88fl, native evidence pending): DB2 units map
//! to the Lua record units used by the original ScriptedAnimations consumers.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_table, table_get, table_set, table_set_num,
};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const DB2_PERCENT_SCALE: f64 = 100.0;
const DB2_MILLISECONDS_PER_SECOND: f64 = 1000.0;
const USE_TARGET_AS_SOURCE: u32 = 1;

#[derive(Clone, Debug, Default)]
pub(crate) struct ScriptedAnimationEffect {
    id: u32,
    visual: u32,
    visual_scale: f64,
    duration: f64,
    trajectory: u32,
    yaw_radians: f64,
    pitch_radians: f64,
    roll_radians: f64,
    offset_x: f64,
    offset_y: f64,
    offset_z: f64,
    animation: u32,
    animation_speed: f64,
    alpha: f64,
    use_target_as_source: bool,
    start_behavior: Option<u32>,
    start_sound_kit_id: Option<u32>,
    finish_effect_id: Option<u32>,
    finish_behavior: Option<u32>,
    finish_sound_kit_id: Option<u32>,
    start_alpha_fade: Option<f64>,
    start_alpha_fade_duration: Option<f64>,
    end_alpha_fade: Option<f64>,
    end_alpha_fade_duration: Option<f64>,
    animation_start_offset: Option<f64>,
    looping_sound_kit_id: Option<u32>,
    particle_override_scale: Option<f64>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ScriptedAnimations")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAllScriptedAnimationEffects",
        get_all_scripted_animation_effects,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function(
        "SetScriptedAnimationEffects",
        set_scripted_animation_effects,
    )
}

fn get_all_scripted_animation_effects(state: &mut LuaState) -> LuaResult<u32> {
    let effects = borrow_state(state)?
        .scripted_animation_effects
        .clone()
        .unwrap_or_else(|| {
            sim_data_profile::scripted_animation_effects::ROWS
                .iter()
                .map(effect_from_db2)
                .collect()
        });
    let output = encode_effects(state, effects);
    state.push(output);
    Ok(1)
}

fn set_scripted_animation_effects(state: &mut LuaState) -> LuaResult<u32> {
    let input = stack_val(state, 1);
    let effects = decode_effects(state, input)?;
    borrow_state_mut(state)?.scripted_animation_effects = effects;
    Ok(0)
}

fn decode_effects(
    state: &mut LuaState,
    input: Val,
) -> LuaResult<Option<Vec<ScriptedAnimationEffect>>> {
    let Val::Table(table) = input else {
        if matches!(input, Val::Nil) {
            return Ok(None);
        }
        return Err(runtime_error(
            "A_Admin.SetScriptedAnimationEffects expects an array or nil",
        ));
    };
    let array = state.gc.tables.get(table).ok_or_else(|| {
        runtime_error("A_Admin.SetScriptedAnimationEffects received an invalid array")
    })?;
    let entries = (1..=array.len(&state.gc.string_arena))
        .map(|index| array.get_int(index as i64))
        .collect::<Vec<_>>();
    entries
        .into_iter()
        .map(|entry| decode_effect(state, entry))
        .collect::<LuaResult<Vec<_>>>()
        .map(Some)
}

fn decode_effect(state: &mut LuaState, row: Val) -> LuaResult<ScriptedAnimationEffect> {
    if !matches!(row, Val::Table(_)) {
        return Err(runtime_error(
            "A_Admin.SetScriptedAnimationEffects entries must be tables",
        ));
    }
    let mut effect = ScriptedAnimationEffect {
        id: field(state, row, "id")?,
        visual: field(state, row, "visual")?,
        visual_scale: field(state, row, "visualScale")?,
        duration: field(state, row, "duration")?,
        trajectory: field(state, row, "trajectory")?,
        yaw_radians: field(state, row, "yawRadians")?,
        pitch_radians: field(state, row, "pitchRadians")?,
        roll_radians: field(state, row, "rollRadians")?,
        offset_x: field(state, row, "offsetX")?,
        offset_y: field(state, row, "offsetY")?,
        offset_z: field(state, row, "offsetZ")?,
        animation: field(state, row, "animation")?,
        animation_speed: field(state, row, "animationSpeed")?,
        alpha: field(state, row, "alpha")?,
        use_target_as_source: field(state, row, "useTargetAsSource")?,
        ..ScriptedAnimationEffect::default()
    };
    decode_optional_effect_fields(state, row, &mut effect)?;
    Ok(effect)
}

fn decode_optional_effect_fields(
    state: &mut LuaState,
    row: Val,
    effect: &mut ScriptedAnimationEffect,
) -> LuaResult<()> {
    effect.start_behavior = optional_field(state, row, "startBehavior")?;
    effect.start_sound_kit_id = optional_field(state, row, "startSoundKitID")?;
    effect.finish_effect_id = optional_field(state, row, "finishEffectID")?;
    effect.finish_behavior = optional_field(state, row, "finishBehavior")?;
    effect.finish_sound_kit_id = optional_field(state, row, "finishSoundKitID")?;
    effect.start_alpha_fade = optional_field(state, row, "startAlphaFade")?;
    effect.start_alpha_fade_duration = optional_field(state, row, "startAlphaFadeDuration")?;
    effect.end_alpha_fade = optional_field(state, row, "endAlphaFade")?;
    effect.end_alpha_fade_duration = optional_field(state, row, "endAlphaFadeDuration")?;
    effect.animation_start_offset = optional_field(state, row, "animationStartOffset")?;
    effect.looping_sound_kit_id = optional_field(state, row, "loopingSoundKitID")?;
    effect.particle_override_scale = optional_field(state, row, "particleOverrideScale")?;
    Ok(())
}

fn field<T: FromStack>(state: &mut LuaState, row: Val, name: &str) -> LuaResult<T> {
    let value = table_get(state, row, name);
    let top = state.top;
    state.push(value);
    let result = T::from_stack(state, -1);
    state.top = top;
    result
}

fn optional_field<T: FromStack>(
    state: &mut LuaState,
    row: Val,
    name: &str,
) -> LuaResult<Option<T>> {
    match table_get(state, row, name) {
        Val::Nil => Ok(None),
        value => {
            let top = state.top;
            state.push(value);
            let result = T::from_stack(state, -1);
            state.top = top;
            result.map(Some)
        }
    }
}

fn effect_from_db2(
    row: &sim_data_profile::scripted_animation_effects::Db2Row,
) -> ScriptedAnimationEffect {
    let mut effect = ScriptedAnimationEffect {
        id: row.id,
        visual: row.visual,
        visual_scale: f64::from(row.visual_scale) / DB2_PERCENT_SCALE,
        duration: f64::from(row.duration) / DB2_MILLISECONDS_PER_SECOND,
        trajectory: row.trajectory,
        yaw_radians: f64::from(row.yaw_radians),
        pitch_radians: f64::from(row.pitch_radians),
        roll_radians: f64::from(row.roll_radians),
        offset_x: f64::from(row.offset_x),
        offset_y: f64::from(row.offset_y),
        offset_z: f64::from(row.offset_z),
        animation: row.animation,
        animation_speed: f64::from(row.animation_speed),
        alpha: f64::from(row.alpha) / DB2_PERCENT_SCALE,
        use_target_as_source: row.flags & USE_TARGET_AS_SOURCE != 0,
        ..ScriptedAnimationEffect::default()
    };
    set_optional_db2_fields(row, &mut effect);
    effect
}

fn set_optional_db2_fields(
    row: &sim_data_profile::scripted_animation_effects::Db2Row,
    effect: &mut ScriptedAnimationEffect,
) {
    effect.start_behavior = nonzero(row.start_behavior);
    effect.start_sound_kit_id = nonzero(row.start_sound_kit_id);
    effect.finish_effect_id = nonzero(row.finish_effect_id);
    effect.finish_behavior = nonzero(row.finish_behavior);
    effect.finish_sound_kit_id = nonzero(row.finish_sound_kit_id);
    effect.start_alpha_fade = optional_percent(row.start_alpha_fade);
    effect.start_alpha_fade_duration = optional_milliseconds(row.start_alpha_fade_duration);
    effect.end_alpha_fade = optional_percent(row.end_alpha_fade);
    effect.end_alpha_fade_duration = optional_milliseconds(row.end_alpha_fade_duration);
    effect.animation_start_offset = optional_milliseconds(row.animation_start_offset);
    effect.looping_sound_kit_id = nonzero(row.looping_sound_kit_id);
    effect.particle_override_scale = optional_percent(row.particle_override_scale);
}

fn nonzero(value: u32) -> Option<u32> {
    (value != 0).then_some(value)
}

fn optional_percent(value: i32) -> Option<f64> {
    (value != 0).then_some(f64::from(value) / DB2_PERCENT_SCALE)
}

fn optional_milliseconds(value: i32) -> Option<f64> {
    (value != 0).then_some(f64::from(value) / DB2_MILLISECONDS_PER_SECOND)
}

fn encode_effects(state: &mut LuaState, effects: Vec<ScriptedAnimationEffect>) -> Val {
    let Val::Table(output) = create_table(state) else {
        unreachable!("create_table returns a table");
    };
    for (index, effect) in effects.into_iter().enumerate() {
        let row = encode_effect(state, &effect);
        table_set_num(state, output, (index + 1) as f64, row);
    }
    Val::Table(output)
}

fn encode_effect(state: &mut LuaState, effect: &ScriptedAnimationEffect) -> Val {
    let Val::Table(row) = create_table(state) else {
        unreachable!("create_table returns a table");
    };
    let row = Val::Table(row);
    encode_numeric_effect_fields(state, row, effect);
    table_set(
        state,
        row,
        "useTargetAsSource",
        Val::Bool(effect.use_target_as_source),
    );
    encode_optional_effect_fields(state, row, effect);
    row
}

fn encode_numeric_effect_fields(state: &mut LuaState, row: Val, effect: &ScriptedAnimationEffect) {
    for (name, value) in [
        ("id", f64::from(effect.id)),
        ("visual", f64::from(effect.visual)),
        ("visualScale", effect.visual_scale),
        ("duration", effect.duration),
        ("trajectory", f64::from(effect.trajectory)),
        ("yawRadians", effect.yaw_radians),
        ("pitchRadians", effect.pitch_radians),
        ("rollRadians", effect.roll_radians),
        ("offsetX", effect.offset_x),
        ("offsetY", effect.offset_y),
        ("offsetZ", effect.offset_z),
        ("animation", f64::from(effect.animation)),
        ("animationSpeed", effect.animation_speed),
        ("alpha", effect.alpha),
    ] {
        table_set(state, row, name, Val::Num(value));
    }
}

fn encode_optional_effect_fields(state: &mut LuaState, row: Val, effect: &ScriptedAnimationEffect) {
    for (name, value) in [
        ("startBehavior", effect.start_behavior.map(f64::from)),
        ("startSoundKitID", effect.start_sound_kit_id.map(f64::from)),
        ("finishEffectID", effect.finish_effect_id.map(f64::from)),
        ("finishBehavior", effect.finish_behavior.map(f64::from)),
        (
            "finishSoundKitID",
            effect.finish_sound_kit_id.map(f64::from),
        ),
        ("startAlphaFade", effect.start_alpha_fade),
        ("startAlphaFadeDuration", effect.start_alpha_fade_duration),
        ("endAlphaFade", effect.end_alpha_fade),
        ("endAlphaFadeDuration", effect.end_alpha_fade_duration),
        ("animationStartOffset", effect.animation_start_offset),
        (
            "loopingSoundKitID",
            effect.looping_sound_kit_id.map(f64::from),
        ),
        ("particleOverrideScale", effect.particle_override_scale),
    ] {
        table_set(state, row, name, value.map_or(Val::Nil, Val::Num));
    }
}
