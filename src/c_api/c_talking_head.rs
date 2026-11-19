//! INFERRED (simulator mapping; native returns UNKNOWN, R88bf pending):
//! empty-default host inputs for Retail talking-head UI consumers.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug, Default)]
pub struct TalkingHeadAnimationInfo {
    pub animation_kit: Option<f64>,
    pub intro_animation: Option<f64>,
    pub loop_animation: Option<f64>,
    pub line_duration: Option<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct TalkingHeadLine {
    pub display_info: f64,
    pub camera_id: f64,
    pub voiceover: Option<f64>,
    pub duration: f64,
    pub line_number: f64,
    pub number_of_lines: f64,
    pub name: String,
    pub text: String,
    pub is_new: bool,
    pub texture_kit: Option<String>,
    pub animation: TalkingHeadAnimationInfo,
}

#[derive(Debug, Default)]
pub struct TalkingHeadState {
    pub current_line: Option<TalkingHeadLine>,
    pub conversations_deferred: bool,
    pub ignored: bool,
}

impl TalkingHeadState {
    fn line(&self) -> Option<TalkingHeadLine> {
        self.current_line.clone()
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_TalkingHead")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetConversationsDeferred",
        get_conversations_deferred,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCurrentLineAnimationInfo",
        get_current_line_animation_info,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCurrentLineInfo",
        get_current_line_info,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IgnoreCurrentTalkingHead",
        ignore_current_talking_head,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsCurrentTalkingHeadIgnored",
        is_current_talking_head_ignored,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetConversationsDeferred",
        set_conversations_deferred,
    )
}

fn get_conversations_deferred(state: &mut LuaState) -> LuaResult<u32> {
    let deferred = borrow_state(state)?.talking_head.conversations_deferred;
    state.push(Val::Bool(deferred));
    Ok(1)
}

fn set_conversations_deferred(state: &mut LuaState) -> LuaResult<u32> {
    let deferred = !matches!(stack_val(state, 1), Val::Nil | Val::Bool(false));
    borrow_state_mut(state)?.talking_head.conversations_deferred = deferred;
    Ok(0)
}

fn ignore_current_talking_head(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.talking_head.ignored = true;
    Ok(0)
}

fn is_current_talking_head_ignored(state: &mut LuaState) -> LuaResult<u32> {
    let ignored = borrow_state(state)?.talking_head.ignored;
    state.push(Val::Bool(ignored));
    Ok(1)
}

fn get_current_line_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(line) = borrow_state(state)?.talking_head.line() else {
        return Ok(0);
    };
    push_line_info(state, &line);
    Ok(10)
}

fn push_line_info(state: &mut LuaState, line: &TalkingHeadLine) {
    push_number(state, line.display_info);
    push_number(state, line.camera_id);
    push_optional_number(state, line.voiceover);
    push_number(state, line.duration);
    push_number(state, line.line_number);
    push_number(state, line.number_of_lines);
    push_string(state, &line.name);
    push_string(state, &line.text);
    state.push(Val::Bool(line.is_new));
    push_optional_string(state, line.texture_kit.as_deref());
}

fn get_current_line_animation_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(line) = borrow_state(state)?.talking_head.line() else {
        return Ok(0);
    };
    push_optional_number(state, line.animation.animation_kit);
    push_optional_number(state, line.animation.intro_animation);
    push_optional_number(state, line.animation.loop_animation);
    push_optional_number(state, line.animation.line_duration);
    Ok(4)
}

fn push_number(state: &mut LuaState, value: f64) {
    state.push(Val::Num(value));
}

fn push_optional_number(state: &mut LuaState, value: Option<f64>) {
    state.push(value.map_or(Val::Nil, Val::Num));
}

fn push_string(state: &mut LuaState, value: &str) {
    state.push(Val::Str(state.gc.intern_string(value.as_bytes())));
}

fn push_optional_string(state: &mut LuaState, value: Option<&str>) {
    let value = match value {
        Some(text) => Val::Str(state.gc.intern_string(text.as_bytes())),
        None => Val::Nil,
    };
    state.push(value);
}
