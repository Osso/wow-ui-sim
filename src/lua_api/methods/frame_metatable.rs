use super::{
    FRAME_MT_CACHE_KEY, create_table, registry_table_or_create, table_get_static, table_set_static,
};
use crate::widget::WidgetType;
use rilua::Val;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

pub(super) fn frame_metatable_for_widget_type(
    state: &mut LuaState,
    base_mt_ref: GcRef<Table>,
    widget_type: WidgetType,
    plain_wrath_message_frame: bool,
    cinematic_model: bool,
    model_scene_actor: bool,
) -> Val {
    let cinematic_model = cinematic_model
        && widget_type == WidgetType::PlayerModel
        && crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Wrath;
    let cache = registry_table_or_create(state, FRAME_MT_CACHE_KEY);
    // WorldFrame reports Frame but retains the otherwise complete base index.
    let key = match widget_type {
        WidgetType::WorldFrame => "WorldFrame",
        WidgetType::PlayerModel if cinematic_model => "CinematicModel",
        WidgetType::Frame if model_scene_actor => "ModelSceneActor",
        WidgetType::MessageFrame if plain_wrath_message_frame => "WrathMessageFrame",
        _ => widget_type.as_str(),
    };
    if let cached @ Val::Table(_) = table_get_static(state, cache, key) {
        return cached;
    }

    let Some(mt) = build_widget_metatable(
        state,
        base_mt_ref,
        widget_type,
        plain_wrath_message_frame,
        cinematic_model,
        model_scene_actor,
    ) else {
        return Val::Table(base_mt_ref);
    };
    table_set_static(state, cache, key, mt);
    mt
}

fn build_widget_metatable(
    state: &mut LuaState,
    base_mt_ref: GcRef<Table>,
    widget_type: WidgetType,
    plain_wrath_message_frame: bool,
    cinematic_model: bool,
    model_scene_actor: bool,
) -> Option<Val> {
    let source_index = metatable_index_table(state, base_mt_ref)?;
    let index_clone = clone_index_for_widget(state, source_index, widget_type)?;
    if plain_wrath_message_frame {
        crate::lua_api::frame::methods::widgets::message_frame::retain_plain_message_frame_methods(
            state,
            index_clone,
        );
    }
    if cinematic_model {
        crate::lua_api::frame::methods::widgets::register_cinematic_methods(state, index_clone)
            .expect("register CinematicModel methods");
    }
    if model_scene_actor
        && crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Wrath
    {
        crate::lua_api::frame::methods::widgets::register_model_scene_actor_methods(
            state,
            index_clone,
        )
        .expect("register ModelSceneActor methods");
    }
    let mt_clone = clone_metatable_with_index(state, base_mt_ref, index_clone)?;
    Some(Val::Table(mt_clone))
}

fn metatable_index_table(state: &mut LuaState, mt_ref: GcRef<Table>) -> Option<GcRef<Table>> {
    let index_key = state.gc.intern_string_static(b"__index");
    match table_str_value(state, mt_ref, index_key) {
        Val::Table(index_ref) => Some(index_ref),
        _ => None,
    }
}

fn hidden_scroll_frame_method(name: &str) -> bool {
    matches!(
        name,
        "GetVerticalScroll"
            | "SetVerticalScroll"
            | "GetVerticalScrollRange"
            | "GetScrollChild"
            | "SetScrollChild"
            | "UpdateScrollChildRect"
    )
}

fn hidden_message_frame_method(name: &str) -> bool {
    matches!(
        name,
        "RemoveMessagesByAccessID"
            | "ScrollUp"
            | "ScrollDown"
            | "PageUp"
            | "PageDown"
            | "ScrollToTop"
            | "ScrollToBottom"
            | "AtTop"
            | "AtBottom"
            | "GetMaxScrollRange"
            | "SetScrollOffset"
            | "GetScrollOffset"
            | "SetScrollAllowed"
            | "IsScrollAllowed"
    )
}

fn hidden_method_for_widget(widget_type: WidgetType, name: &str) -> bool {
    if matches!(name, "GetFileWidth" | "GetFileHeight") {
        return widget_type != WidgetType::Texture;
    }
    match widget_type {
        WidgetType::WorldFrame => false,
        WidgetType::MessageFrame => hidden_scroll_frame_method(name),
        WidgetType::ScrollFrame => name == "SetMaxLines" || hidden_message_frame_method(name),
        WidgetType::StatusBar => hidden_scroll_frame_method(name) || name == "SetStatusBarAtlas",
        _ => hidden_scroll_frame_method(name) || hidden_message_frame_method(name),
    }
}

fn clone_index_for_widget(
    state: &mut LuaState,
    source_index: GcRef<Table>,
    widget_type: WidgetType,
) -> Option<GcRef<Table>> {
    let Val::Table(index_clone) = create_table(state) else {
        return None;
    };

    for (entry_key, entry_value) in table_entries(state, source_index) {
        if !is_hidden_string_key(state, entry_key, widget_type) {
            raw_set(state, index_clone, entry_key, entry_value);
        }
    }
    state.gc.barrier_back(index_clone);
    Some(index_clone)
}

fn clone_metatable_with_index(
    state: &mut LuaState,
    base_mt_ref: GcRef<Table>,
    index_clone: GcRef<Table>,
) -> Option<GcRef<Table>> {
    let Val::Table(mt_clone) = create_table(state) else {
        return None;
    };

    for (entry_key, entry_value) in table_entries(state, base_mt_ref) {
        let value = if is_string_key(state, entry_key, "__index") {
            Val::Table(index_clone)
        } else {
            entry_value
        };
        raw_set(state, mt_clone, entry_key, value);
    }
    state.gc.barrier_back(mt_clone);
    Some(mt_clone)
}

fn table_entries(state: &LuaState, table_ref: GcRef<Table>) -> Vec<(Val, Val)> {
    state
        .gc
        .tables
        .get(table_ref)
        .map(Table::hash_entries)
        .unwrap_or_default()
}

fn table_str_value(
    state: &LuaState,
    table_ref: GcRef<Table>,
    key_ref: GcRef<rilua::vm::string::LuaString>,
) -> Val {
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

fn is_hidden_string_key(state: &LuaState, key: Val, widget_type: WidgetType) -> bool {
    string_key_name(state, key).is_some_and(|name| hidden_method_for_widget(widget_type, name))
}

fn is_string_key(state: &LuaState, key: Val, expected: &str) -> bool {
    string_key_name(state, key) == Some(expected)
}

fn string_key_name(state: &LuaState, key: Val) -> Option<&str> {
    let Val::Str(str_ref) = key else {
        return None;
    };

    state
        .gc
        .string_arena
        .get(str_ref)
        .and_then(|name| std::str::from_utf8(name.data()).ok())
}

fn raw_set(state: &mut LuaState, table_ref: GcRef<Table>, key: Val, value: Val) {
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(key, value, &state.gc.string_arena);
    }
}
