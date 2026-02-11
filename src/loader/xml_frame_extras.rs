//! Extra frame helpers: animations, bar textures, action bar init.

use crate::lua_api::LoaderEnv;

use super::error::LoadError;
use super::helpers::{escape_lua_string, lua_global_ref, rand_id};
use super::helpers_anim::generate_animation_group_code;

/// Apply animation groups from the frame and its inherited templates.
pub(super) fn apply_animation_groups(
    env: &LoaderEnv<'_>, frame: &crate::xml::FrameXml, name: &str, inherits: &str,
) -> Result<(), LoadError> {
    if let Some(anims) = frame.animations() {
        exec_animation_groups(env, anims, name);
    }
    if !inherits.is_empty() {
        for template_entry in &crate::xml::get_template_chain(inherits) {
            if let Some(anims) = template_entry.frame.animations() {
                exec_animation_groups(env, anims, name);
            }
        }
    }
    Ok(())
}

/// Generate and execute Lua code for a set of animation groups on a frame.
fn exec_animation_groups(env: &LoaderEnv<'_>, anims: &crate::xml::AnimationsXml, name: &str) {
    let mut anim_code = format!(
        r#"
            local frame = {}
            "#,
        lua_global_ref(name)
    );
    for anim_group_xml in &anims.animations {
        if anim_group_xml.is_virtual == Some(true) {
            if let Some(ref name) = anim_group_xml.name {
                crate::xml::register_anim_group_template(name, anim_group_xml.clone());
            }
            continue;
        }
        anim_code.push_str(&generate_animation_group_code(anim_group_xml, "frame"));
    }
    if let Err(e) = env.exec(&anim_code) {
        eprintln!("[AnimSetup] error: {}", e);
    }
}

/// Create the bar texture for a StatusBar from its inline `<BarTexture>` XML element.
pub(super) fn apply_bar_texture(
    env: &LoaderEnv<'_>, frame: &crate::xml::FrameXml, name: &str,
) -> Result<(), LoadError> {
    let Some(bar) = frame.bar_texture() else { return Ok(()) };
    let bar_name = bar
        .name
        .as_ref()
        .map(|n| n.replace("$parent", name))
        .unwrap_or_else(|| format!("__bar_{}", rand_id()));

    let parent_ref = lua_global_ref(name);
    let mut code = build_bar_texture_header(&parent_ref, &bar_name);
    append_bar_texture_properties(&mut code, bar);
    code.push_str("            parent:SetStatusBarTexture(bar)\n");
    let parent_key = bar.parent_key.as_deref().unwrap_or("Bar");
    code.push_str(&format!("            parent.{} = bar\n", parent_key));
    if bar.name.is_some() {
        code.push_str(&format!(
            "            _G[\"{}\"] = bar\n",
            escape_lua_string(&bar_name)
        ));
    }
    code.push_str("        end\n");
    env.exec(&code).map_err(|e| {
        LoadError::Lua(format!("Failed to create bar texture on {}: {}", name, e))
    })?;
    Ok(())
}

fn build_bar_texture_header(parent_ref: &str, bar_name: &str) -> String {
    format!(
        r#"
        local parent = {parent_ref}
        if parent and parent.SetStatusBarTexture then
            local bar = parent:CreateTexture("{}", "ARTWORK")
        "#,
        escape_lua_string(bar_name),
    )
}

fn append_bar_texture_properties(code: &mut String, bar: &crate::xml::TextureXml) {
    if let Some(file) = &bar.file {
        code.push_str(&format!(
            "            bar:SetTexture(\"{}\")\n",
            escape_lua_string(file)
        ));
    }
    if let Some(atlas) = &bar.atlas {
        code.push_str(&format!(
            "            bar:SetAtlas(\"{}\")\n",
            escape_lua_string(atlas)
        ));
    }
    if let Some(color) = &bar.color {
        code.push_str(&format!(
            "            bar:SetColorTexture({}, {}, {}, {})\n",
            color.r.unwrap_or(1.0),
            color.g.unwrap_or(1.0),
            color.b.unwrap_or(1.0),
            color.a.unwrap_or(1.0)
        ));
    }
}

/// Initialize tables expected by action bar OnLoad handlers.
/// Frames with `numButtons` KeyValue are action bars that need `actionButtons = {}`.
pub(super) fn init_action_bar_tables(env: &LoaderEnv<'_>, name: &str) {
    let code = format!(
        r#"do local f = {}
        if f and f.numButtons and not f.actionButtons then
            f.actionButtons = {{}}
        end end"#,
        lua_global_ref(name)
    );
    let _ = env.exec(&code);
}
