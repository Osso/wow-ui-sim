//! Evidenced Retail/Forever login entry points (R40/R42).
use super::*;

pub(super) fn login(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::methods::{
        borrow_state_mut, frame_id_from_stack, frame_id_from_stack_unrestricted,
    };
    use crate::lua_bridge::stack_val;
    const USAGE: &str = "Usage: C_Login.Login(\"accountName\", AccountLoginPasswordEdit)";
    let account = stack_val(state, 1);
    let password_box = stack_val(state, 2);
    if !matches!(account, Val::Str(_) | Val::Num(_)) || !matches!(password_box, Val::Table(_)) {
        return Err(rilua::runtime_error(USAGE));
    }
    // Distinguish unresolved objects (Usage) from fallible access rejection.
    frame_id_from_stack_unrestricted(state, 2).map_err(|_| rilua::runtime_error(USAGE))?;
    let id = frame_id_from_stack(state, 2)?;
    {
        let sim = borrow_state(state)?;
        let frame = sim
            .widgets
            .get(id)
            .ok_or_else(|| rilua::runtime_error(USAGE))?;
        // Match the existing forbidden-frame guard used by CreateFontString.
        // The shared fallible resolver also enforces dynamic access restrictions.
        if frame.forbidden && !rilua::api::state_is_secure(state) {
            return Err(rilua::runtime_error(
                "Attempt to access forbidden object from code tainted by an AddOn",
            ));
        }
        if frame.widget_type != crate::widget::WidgetType::EditBox {
            // R40 object resolution distinguishes a wrong resolved type from
            // invalid arguments and an object that cannot be resolved.
            return Err(rilua::runtime_error("Wrong object type for function"));
        }
    }
    {
        let mut sim = borrow_state_mut(state)?;
        let login = &mut sim.glue_login;
        login.submitted = true;
        login.aurora_state = 3;
        login.connected_to_wow = false;
        login.received_connection_info = Default::default();
        login.pending_step = 1;
    }
    // R40 permits synchronous CONNECTING; this sequence is simulator policy.
    crate::lua_api::script_helpers::fire_named_event_state(state, "LOGIN_STATE_CHANGED", &[]);
    // The offline policy accepts all input. Do not copy, log or retain the password.
    crate::lua_api::frame::methods::text_attribute_event::set_frame_text_value(
        state,
        id,
        Some(String::new()),
        false,
    )?;
    Ok(0)
}

pub(super) fn get_logon_queue_info(state: &mut LuaState) -> LuaResult<u32> {
    let queue = borrow_state(state)?.glue_login.logon_queue;
    let (queued, position, seconds) = match queue {
        Some(queue) if queue.server_position > 1 => {
            (true, queue.server_position - 1, queue.estimated_seconds)
        }
        _ => (false, 0, 0),
    };
    state.push(Val::Bool(queued));
    state.push(Val::Num(f64::from(position)));
    state.push(Val::Num(f64::from(seconds)));
    Ok(3)
}
