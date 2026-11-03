use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, extract_frame_id, extract_frame_slot_zero_id};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_widget_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_Widget")?;
    table_set_rust_fn_static(state, table_ref, "IsFrameWidget", is_frame_widget)?;
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        table_set_rust_fn_static(state, table_ref, "IsWidget", is_widget)?;
    }
    Ok(())
}

fn is_widget(state: &mut LuaState) -> LuaResult<u32> {
    let id = extract_frame_slot_zero_id(state, stack_val(state, 1));
    let sim = borrow_state(state)?;
    let recognized = id.is_some_and(|id| {
        sim.widgets.get(id).is_some_and(|frame| {
            frame.object_type_name.as_deref() != Some("ControlPoint")
                && !sim.anim_frame_to_group.contains_key(&id)
                && !sim.anim_frame_to_anim.contains_key(&id)
        })
    });
    drop(sim);
    state.push(Val::Bool(recognized));
    Ok(1)
}

fn is_frame_widget(state: &mut LuaState) -> LuaResult<u32> {
    let is_frame = extract_frame_id(state, stack_val(state, 1)).is_some();
    state.push(Val::Bool(is_frame));
    Ok(1)
}
