use crate::lua_api::LoaderEnv;
use rilua::LuaResult;

pub(super) fn apply_loader_chain_layers(
    loader_env: &LoaderEnv,
    inherits: &str,
    frame_id: u64,
    frame_name: &str,
    name_parent: &str,
    timing: &mut crate::loader::LoadTiming,
) -> LuaResult<()> {
    let parent_ref_name = format!("__frame_{frame_id}");
    for entry in &*crate::xml::get_template_chain(inherits) {
        crate::loader::xml_layer_batch::create_layer_children_batched_with_name_parent(
            loader_env,
            &entry.frame,
            frame_name,
            &parent_ref_name,
            name_parent,
            timing,
        )
        .map_err(|error| rilua::runtime_error(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn apply_loader_frame_extras(
    loader_env: &LoaderEnv,
    frame: &crate::xml::FrameXml,
    frame_id: u64,
    frame_name: &str,
    name_parent: &str,
    inherits: &str,
    timing: &mut crate::loader::LoadTiming,
) -> LuaResult<()> {
    let parent_ref_name = format!("__frame_{frame_id}");
    apply_loader_frame_layers(
        loader_env,
        frame,
        frame_name,
        name_parent,
        timing,
        &parent_ref_name,
    )?;
    apply_loader_frame_button_parts(loader_env, frame, frame_name, inherits, &parent_ref_name)?;
    apply_loader_frame_xml_extras(loader_env, frame, frame_id, frame_name, inherits)?;
    Ok(())
}

fn apply_loader_frame_layers(
    loader_env: &LoaderEnv,
    frame: &crate::xml::FrameXml,
    frame_name: &str,
    name_parent: &str,
    timing: &mut crate::loader::LoadTiming,
    parent_ref_name: &str,
) -> LuaResult<()> {
    crate::loader::xml_layer_batch::create_layer_children_batched_with_name_parent(
        loader_env,
        frame,
        frame_name,
        parent_ref_name,
        name_parent,
        timing,
    )
    .map_err(|error| rilua::runtime_error(error.to_string()))
}

fn apply_loader_frame_button_parts(
    loader_env: &LoaderEnv,
    frame: &crate::xml::FrameXml,
    frame_name: &str,
    inherits: &str,
    parent_ref_name: &str,
) -> LuaResult<()> {
    crate::loader::button::apply_button_textures_with_ref(
        loader_env,
        frame,
        frame_name,
        parent_ref_name,
        inherits,
    )
    .map_err(|error| rilua::runtime_error(error.to_string()))?;
    crate::loader::button::apply_button_text_with_ref(
        loader_env,
        frame,
        frame_name,
        parent_ref_name,
        inherits,
    )
    .map_err(|error| rilua::runtime_error(error.to_string()))?;
    crate::loader::button::apply_button_fonts_with_ref(
        loader_env,
        frame,
        frame_name,
        parent_ref_name,
        inherits,
    )
    .map_err(|error| rilua::runtime_error(error.to_string()))
}

fn apply_loader_frame_xml_extras(
    loader_env: &LoaderEnv,
    frame: &crate::xml::FrameXml,
    frame_id: u64,
    frame_name: &str,
    inherits: &str,
) -> LuaResult<()> {
    crate::loader::xml_frame_extras::apply_animation_groups(loader_env, frame, frame_id, inherits)
        .map_err(|error| rilua::runtime_error(error.to_string()))?;
    crate::loader::xml_frame_extras::apply_bar_texture(loader_env, frame, frame_name, inherits)
        .map_err(|error| rilua::runtime_error(error.to_string()))?;
    crate::loader::xml_frame_extras::apply_thumb_texture(loader_env, frame, frame_name, inherits)
        .map_err(|error| rilua::runtime_error(error.to_string()))?;
    crate::loader::xml_frame_extras::apply_colorselect_textures(
        loader_env, frame, frame_name, inherits,
    )
    .map_err(|error| rilua::runtime_error(error.to_string()))?;
    crate::loader::xml_frame_extras::init_action_bar_tables(loader_env, frame, frame_name);
    Ok(())
}
