//! Native forbidden-aspect state and XML declarations.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[cfg(feature = "forbidden-aspects")]
pub(crate) const INHERITANCE_PARENT: u64 = 1;
#[cfg(feature = "forbidden-aspects")]
pub(crate) const INHERITANCE_LAYOUT: u64 = 2;

#[cfg(feature = "forbidden-aspects")]
pub(crate) fn stored_forbidden_aspects(state: &LuaState, frame_id: u64) -> LuaResult<u64> {
    let sim = borrow_state(state)?;
    let frame = sim
        .widgets
        .get(frame_id)
        .ok_or_else(|| runtime_error("forbidden-aspect object does not exist"))?;
    Ok(frame.forbidden_aspects)
}

/// Forbidden aspects restrict addon (tainted) callers only: the 12.1.0 notes say they
/// "prevent addons from using certain functionality", and Blizzard's own secure
/// `AuraContainerPrivateMixin:UpdateEventRegistrations` registers events on
/// AuraContainers, which carry the EventRegistrations aspect.
pub(crate) fn ensure_forbidden_aspect_absent(
    state: &mut LuaState,
    frame_id: u64,
    aspect_name: &str,
    method_name: &str,
) -> LuaResult<()> {
    let mask = borrow_state(state)?
        .widgets
        .get(frame_id)
        .map_or(0, |frame| frame.forbidden_aspects);
    if mask == 0 || rilua::api::state_is_secure(state) {
        return Ok(());
    }
    let aspect = resolve_aspect(state, aspect_name)?;
    if mask & aspect != 0 {
        return Err(runtime_error(format!(
            "{method_name}: forbidden aspect {aspect_name} disallows this operation"
        )));
    }
    Ok(())
}

/// Whether UntrustedScriptExecution (any handler) or UntrustedLayoutScriptExecution
/// (layout handlers such as OnSizeChanged) suppresses addon-installed handlers on this
/// frame. Inheritance to children and anchored frames is already folded into the
/// frame's own mask.
pub(crate) fn suppresses_untrusted_handler(
    state: &mut LuaState,
    frame_id: u64,
    layout_handler: bool,
) -> LuaResult<bool> {
    let mask = borrow_state(state)?
        .widgets
        .get(frame_id)
        .map_or(0, |frame| frame.forbidden_aspects);
    if mask == 0 {
        return Ok(false);
    }
    let mut blocking = resolve_aspect(state, "UntrustedScriptExecution")?;
    if layout_handler {
        blocking |= resolve_aspect(state, "UntrustedLayoutScriptExecution")?;
    }
    Ok(mask & blocking != 0)
}

/// A handler is addon-installed when its closure carries addon taint. INFERRED: the
/// page's "lives in the Forbidden Partition and execution is untainted" is modeled as
/// the handler closure being untainted; the dispatching stack is not consulted, because
/// Blizzard intrinsic OnLoad handlers must still run for frames created by addons.
pub(crate) fn is_addon_installed_handler(state: &mut LuaState, handler: Val) -> bool {
    match handler {
        Val::Function(closure) => rilua::stdlib::taint::get_closure_taint(state, closure).is_some(),
        _ => false,
    }
}

/// HookScript chain gate `(frameId, layoutHandler, component) -> allowed`. A chain mixes
/// handlers installed by different callers, so each component is checked when it runs.
pub(crate) fn hooked_script_component_allowed(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = match crate::lua_bridge::stack_val(state, 1) {
        Val::Num(id) => id as u64,
        _ => return Err(runtime_error("hook gate requires a frame id")),
    };
    let layout_handler = matches!(crate::lua_bridge::stack_val(state, 2), Val::Bool(true));
    let component = crate::lua_bridge::stack_val(state, 3);
    let allowed = !suppresses_untrusted_handler(state, frame_id, layout_handler)?
        || !is_addon_installed_handler(state, component);
    state.push(Val::Bool(allowed));
    Ok(1)
}

/// AlwaysPropagateInput forces mouse and keyboard propagation for this frame.
pub(crate) fn input_propagation_forced(state: &mut LuaState, frame_id: u64) -> LuaResult<bool> {
    let mask = borrow_state(state)?
        .widgets
        .get(frame_id)
        .map_or(0, |frame| frame.forbidden_aspects);
    if mask == 0 {
        return Ok(false);
    }
    Ok(mask & resolve_aspect(state, "AlwaysPropagateInput")? != 0)
}

/// Read effective propagation without rewriting the caller's stored preference.
pub(crate) fn read_keyboard_input_propagation(
    state: &mut LuaState,
    frame_id: u64,
) -> LuaResult<bool> {
    let requested = borrow_state(state)?
        .widgets
        .get(frame_id)
        .is_some_and(|frame| frame.propagate_keyboard_input);
    Ok(requested || input_propagation_forced(state, frame_id)?)
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
    let mut mask = 0;
    if frame.intrinsic == Some(true) {
        for aspect in native_intrinsic_aspects(frame.name.as_deref()) {
            mask |= resolve_aspect(state, aspect)?;
        }
    }
    if let Some(aspects) = frame.forbidden_aspects() {
        for aspect in &aspects.aspects {
            mask |= resolve_aspect(state, &aspect.aspect)?;
        }
    }
    add_forbidden_aspects(state, frame_id, mask)
}

/// Aspects the client applies natively to intrinsics whose XML declares none.
fn native_intrinsic_aspects(intrinsic: Option<&str>) -> &'static [&'static str] {
    match intrinsic {
        // 12.1.0: "Aura Containers have had the EventRegistrations Forbidden Aspect applied."
        Some("AuraContainer") => &["EventRegistrations"],
        _ => &[],
    }
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

#[cfg(feature = "forbidden-aspects")]
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

#[cfg(feature = "forbidden-aspects")]
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
