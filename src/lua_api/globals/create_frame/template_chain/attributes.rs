//! Initialize typed runtime XML attributes without notifying uninitialized frame scripts.
use crate::lua_api::methods::frame_ref;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn apply_xml_attributes(
    state: &mut LuaState,
    frame_id: u64,
    frame: &crate::xml::FrameXml,
) -> LuaResult<()> {
    if frame.xml_attributes().is_none() {
        return Ok(());
    }
    let mut code = String::from("local frame = ...");
    // Initial values precede OnLoad; ordinary SetAttribute would call mixin handlers
    // before their fields/children are initialized (for example ActionButton.icon).
    crate::loader::append_xml_attributes_code(&mut code, frame, "SetAttributeNoHandler");
    let function = crate::loader::chunk_cache::load_chunk(state, &code, "template-attributes")
        .map_err(|error| rilua::runtime_error(error.to_string()))?;
    let frame = frame_ref(state, frame_id)?;
    crate::lua_api::script_helpers::call_void_function_state(
        state,
        Val::Function(function.gc_ref()),
        &[frame],
    )
    .map_err(rilua::runtime_error)
}
