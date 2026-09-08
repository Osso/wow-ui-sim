//! Native forbidden-aspect state and XML declarations.

#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::methods::borrow_state;
use crate::lua_api::methods::{borrow_state_mut, table_get};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[cfg(feature = "retail-12-1-0")]
pub(crate) const INHERITANCE_PARENT: u64 = 1;
#[cfg(feature = "retail-12-1-0")]
pub(crate) const INHERITANCE_LAYOUT: u64 = 2;

#[cfg(feature = "retail-12-1-0")]
pub(crate) fn stored_forbidden_aspects(state: &LuaState, frame_id: u64) -> LuaResult<u64> {
    let sim = borrow_state(state)?;
    let frame = sim
        .widgets
        .get(frame_id)
        .ok_or_else(|| runtime_error("forbidden-aspect object does not exist"))?;
    Ok(frame.forbidden_aspects)
}

pub(crate) fn add_forbidden_aspects(
    state: &mut LuaState,
    frame_id: u64,
    mask: u64,
) -> LuaResult<()> {
    if mask == 0 {
        return Ok(());
    }
    let reset = resolve_aspect(state, "SetToDefaults")?;
    let layout = resolve_aspect(state, "UntrustedLayoutScriptExecution")?;
    let hierarchy = resolve_aspect(state, "UntrustedScriptExecution")?
        | layout
        | resolve_aspect(state, "AlwaysPropagateInput")?;
    let mut sim = borrow_state_mut(state)?;
    let frame = sim
        .widgets
        .get_mut(frame_id)
        .ok_or_else(|| runtime_error("forbidden-aspect object does not exist"))?;
    frame.forbidden_aspects |= mask | reset;
    frame.inheritable_forbidden_aspects_parent |= mask & hierarchy;
    frame.inheritable_forbidden_aspects_layout |= mask & layout;
    Ok(())
}

pub(crate) fn apply_xml_forbidden_aspects(
    state: &mut LuaState,
    frame_id: u64,
    frame: &crate::xml::FrameXml,
) -> LuaResult<()> {
    let Some(aspects) = frame.forbidden_aspects() else {
        return Ok(());
    };
    let mut mask = 0;
    for aspect in &aspects.aspects {
        mask |= resolve_aspect(state, &aspect.aspect)?;
    }
    add_forbidden_aspects(state, frame_id, mask)
}

fn resolve_aspect(state: &mut LuaState, name: &str) -> LuaResult<u64> {
    let enums = table_get(state, Val::Table(state.global), "Enum");
    let aspects = table_get(state, enums, "ForbiddenAspect");
    match table_get(state, aspects, name) {
        Val::Num(value) if value.is_finite() && value > 0.0 && value.fract() == 0.0 => {
            Ok(value as u64)
        }
        _ => Err(runtime_error(format!(
            "Unknown forbidden aspect '{name}' in active Enum.ForbiddenAspect"
        ))),
    }
}

#[cfg(feature = "retail-12-1-0")]
pub(crate) fn stored_inheritable_forbidden_aspects(
    state: &LuaState,
    frame_id: u64,
    inheritance: u64,
) -> LuaResult<u64> {
    let sim = borrow_state(state)?;
    let frame = sim
        .widgets
        .get(frame_id)
        .ok_or_else(|| runtime_error("forbidden-aspect object does not exist"))?;
    let mut mask = 0;
    if inheritance & INHERITANCE_PARENT != 0 {
        mask |= frame.inheritable_forbidden_aspects_parent;
    }
    if inheritance & INHERITANCE_LAYOUT != 0 {
        mask |= frame.inheritable_forbidden_aspects_layout;
    }
    Ok(mask)
}

#[cfg(feature = "retail-12-1-0")]
pub(crate) fn ensure_forbidden_aspects_already_owned(
    state: &mut LuaState,
    frame_id: u64,
    source_frame_id: u64,
    inheritance: u64,
    method_name: &str,
) -> LuaResult<()> {
    let source_aspects = stored_inheritable_forbidden_aspects(state, source_frame_id, inheritance)?;
    let frame_aspects = stored_forbidden_aspects(state, frame_id)?;
    if source_aspects & !frame_aspects == 0 {
        return Ok(());
    }
    Err(runtime_error(format!(
        "Action[{method_name}] failed because[Cannot implicitly gain forbidden aspects]"
    )))
}
