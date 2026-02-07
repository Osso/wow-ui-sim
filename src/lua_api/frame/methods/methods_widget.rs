//! Widget-specific methods: GameTooltip, EditBox, Slider, StatusBar, CheckButton,
//! Cooldown, ScrollFrame, Model, ColorSelect, dragging/moving, ScrollBox.
//!
//! Each widget type is implemented in its own submodule under `widget_*.rs`.

use super::FrameHandle;
use mlua::UserDataMethods;

/// Add widget-specific methods to FrameHandle UserData.
pub fn add_widget_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    super::widget_tooltip::add_tooltip_methods(methods);
    super::widget_message_frame::add_message_frame_methods(methods);
    super::widget_editbox::add_editbox_methods(methods);
    super::widget_slider::add_slider_methods(methods);
    super::widget_slider::add_statusbar_methods(methods);
    super::widget_slider::add_checkbutton_methods(methods);
    super::widget_cooldown::add_cooldown_methods(methods);
    super::widget_scroll::add_scrollframe_methods(methods);
    super::widget_model::add_model_methods(methods);
    super::widget_model::add_model_scene_methods(methods);
    super::widget_misc::add_colorselect_methods(methods);
    super::widget_misc::add_drag_methods(methods);
    super::widget_scroll::add_scrollbox_methods(methods);
    super::widget_misc::add_simplehtml_methods(methods);
    super::widget_slider::add_shared_value_methods(methods);
    super::widget_misc::add_misc_widget_stubs(methods);
}
