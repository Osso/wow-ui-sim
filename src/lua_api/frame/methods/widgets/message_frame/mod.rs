//! MessageFrame widget methods: AddMessage, scrolling, fading, message history.
//!
//! Consolidates the three master-branch files:
//! - widget_message_frame.rs
//! - widget_message_frame_callbacks.rs
//! - widget_message_frame_scroll.rs

mod add;
mod callbacks;
mod font_string;
mod getters;
mod scroll;
mod transform;

pub(super) use getters::clear;

use crate::lua_api::methods::get_or_create_frame_fields;
use crate::lua_bridge::table_set_rust_fn;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

const METHODS: &[(&'static str, rilua::vm::closure::RustFn)] = &[
    // Add / backfill
    ("AddMessage", add::add_message),
    ("AddMsg", add::add_msg),
    ("_AddMessageSilent", add::add_message_silent),
    ("BackFillMessage", add::backfill_message),
    // Clear
    ("ClearText", getters::clear_text),
    // Count / max lines
    ("GetNumMessages", getters::get_num_messages),
    ("SetMaxLines", getters::set_max_lines),
    ("GetMaxLines", getters::get_max_lines),
    // Fading
    ("SetFading", getters::set_fading),
    ("GetFading", getters::get_fading),
    ("SetTimeVisible", getters::set_time_visible),
    ("GetTimeVisible", getters::get_time_visible),
    ("SetFadeDuration", getters::set_fade_duration),
    ("GetFadeDuration", getters::get_fade_duration),
    ("SetFadePower", getters::set_fade_power),
    ("GetFadePower", getters::get_fade_power),
    // Insert mode
    ("SetInsertMode", getters::set_insert_mode),
    ("GetInsertMode", getters::get_insert_mode),
    // Misc
    ("SetTextCopyable", getters::set_text_copyable),
    ("IsTextCopyable", getters::is_text_copyable),
    ("HasMessageByID", getters::has_message_by_id),
    ("GetMessageInfo", getters::get_message_info),
    // Scroll
    ("ScrollUp", scroll::scroll_up),
    ("ScrollDown", scroll::scroll_down),
    ("PageUp", scroll::page_up),
    ("PageDown", scroll::page_down),
    ("ScrollToTop", scroll::scroll_to_top),
    ("ScrollToBottom", scroll::scroll_to_bottom),
    ("AtTop", scroll::at_top),
    ("AtBottom", scroll::at_bottom),
    ("GetMaxScrollRange", scroll::get_max_scroll_range),
    ("SetScrollOffset", scroll::set_scroll_offset),
    ("GetScrollOffset", scroll::get_scroll_offset),
    ("SetScrollAllowed", scroll::set_scroll_allowed),
    ("IsScrollAllowed", scroll::is_scroll_allowed),
    // Callbacks / word-wrap
    ("SetIndentedWordWrap", getters::set_indented_word_wrap),
    ("GetIndentedWordWrap", getters::get_indented_word_wrap),
    (
        "SetOnScrollChangedCallback",
        callbacks::set_on_scroll_changed_callback,
    ),
    (
        "SetOnLineRightClickedCallback",
        callbacks::set_on_line_right_clicked_callback,
    ),
    (
        "AddOnDisplayRefreshedCallback",
        callbacks::add_on_display_refreshed_callback,
    ),
    (
        "SetOnTextCopiedCallback",
        callbacks::set_on_text_copied_callback,
    ),
    ("MarkDisplayDirty", callbacks::mark_display_dirty),
    ("ResetAllFadeTimes", callbacks::reset_all_fade_times),
    ("ResetMessageFadeByID", callbacks::reset_message_fade_by_id),
    ("GetFontStringByID", font_string::get_font_string_by_id),
    // Transforms
    (
        "RemoveMessagesByPredicate",
        transform::remove_messages_by_predicate,
    ),
    ("TransformMessages", transform::transform_messages),
    ("AdjustMessageColors", transform::adjust_message_colors),
];

pub fn register_message_frame(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    for (name, func) in METHODS {
        table_set_rust_fn(state, metatable, name, *func)?;
    }
    Ok(())
}

pub(crate) fn install_message_frame_fields(state: &mut LuaState, frame_id: u64) -> LuaResult<()> {
    let fields = get_or_create_frame_fields(state, frame_id);
    if let rilua::Val::Table(fields_ref) = fields {
        table_set_rust_fn_static(state, fields_ref, "SetMaxLines", getters::set_max_lines)?;
    }
    Ok(())
}
