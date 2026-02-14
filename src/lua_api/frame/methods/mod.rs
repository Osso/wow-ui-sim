//! Frame methods registered into the shared LightUserData methods table.
//!
//! Each submodule exports `add_*_methods(lua, &Table) -> Result<()>` which
//! sets method functions on the shared methods table.

mod methods_anchor;
mod methods_attribute;
mod methods_backdrop;
mod methods_button;
pub(crate) mod methods_core;
mod methods_create;
mod methods_event;
pub(crate) mod methods_helpers;
mod methods_line;
pub(crate) mod methods_hierarchy;
mod methods_misc;
mod methods_script;
mod methods_text;
mod methods_texture;
mod methods_widget;
mod widget_cooldown;
mod widget_editbox;
mod widget_message_frame;
mod widget_misc;
mod widget_model;
mod widget_scroll;
mod widget_slider;
mod widget_tooltip;

pub(crate) use methods_core::fire_on_show_recursive;

/// Register all ~200 frame methods into the shared methods table.
pub fn register_all_methods(lua: &mlua::Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods_core::add_core_methods(lua, methods)?;
    methods_hierarchy::add_hierarchy_methods(lua, methods)?;
    methods_misc::add_misc_methods(lua, methods)?;
    methods_anchor::add_anchor_methods(lua, methods)?;
    methods_event::add_event_methods(lua, methods)?;
    methods_script::add_script_methods(lua, methods)?;
    methods_attribute::add_attribute_methods(lua, methods)?;
    methods_backdrop::add_backdrop_methods(lua, methods)?;
    methods_create::add_create_methods(lua, methods)?;
    methods_texture::add_texture_methods(lua, methods)?;
    methods_text::add_text_methods(lua, methods)?;
    methods_button::add_button_methods(lua, methods)?;
    methods_widget::add_widget_methods(lua, methods)?;
    methods_line::add_line_methods(lua, methods)?;
    Ok(())
}
