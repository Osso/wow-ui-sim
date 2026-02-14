//! Widget-specific methods: GameTooltip, EditBox, Slider, StatusBar, CheckButton,
//! Cooldown, ScrollFrame, Model, ColorSelect, dragging/moving, ScrollBox.
//!
//! Each widget type is implemented in its own submodule under `widget_*.rs`.

use mlua::Lua;

/// Add widget-specific methods to the shared frame methods table.
pub fn add_widget_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    super::widget_tooltip::add_tooltip_methods(lua, methods)?;
    super::widget_message_frame::add_message_frame_methods(lua, methods)?;
    super::widget_editbox::add_editbox_methods(lua, methods)?;
    super::widget_slider::add_slider_methods(lua, methods)?;
    super::widget_slider::add_statusbar_methods(lua, methods)?;
    super::widget_slider::add_checkbutton_methods(lua, methods)?;
    super::widget_cooldown::add_cooldown_methods(lua, methods)?;
    super::widget_scroll::add_scrollframe_methods(lua, methods)?;
    super::widget_model::add_model_methods(lua, methods)?;
    super::widget_model::add_model_scene_methods(lua, methods)?;
    super::widget_misc::add_colorselect_methods(lua, methods)?;
    super::widget_misc::add_drag_methods(lua, methods)?;
    super::widget_scroll::add_scrollbox_methods(lua, methods)?;
    super::widget_misc::add_simplehtml_methods(lua, methods)?;
    super::widget_slider::add_shared_value_methods(lua, methods)?;
    super::widget_misc::add_misc_widget_stubs(lua, methods)?;
    Ok(())
}
