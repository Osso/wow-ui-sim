//! rilua RustFn equivalents for global functions from create_frame.rs,
//! global_frames.rs, and dropdown_api.rs.
//!
//! Each public function is a `rilua::RustFn` compatible signature:
//!   `fn foo(state: &mut LuaState) -> LuaResult<u32>`
//! Args start at index 1 (no self).
//!
//! `register_all` registers all globals on a rilua Lua state.

mod dropdown_api;
mod dropdown_children;
mod helpers;
pub mod helpers_shared;
mod simple_window;
mod template_chain;

pub(crate) use helpers::{append_parent_array_entry, apply_frame_mixin, apply_frame_mixins};
pub(crate) use helpers_shared::{
    apply_parent_sub, create_frame_instance, mark_frame_uses_forbidden_object_table,
};

use crate::lua_api::methods::{borrow_state, extract_frame_id, frame_ref, val_to_string};
use crate::lua_bridge::FromStack;
use crate::lua_bridge::stack_val;
use crate::widget::WidgetType;
use helpers::set_global_raw;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

// ---------------------------------------------------------------------------
// CreateFrame
// ---------------------------------------------------------------------------

pub fn create_frame(state: &mut LuaState) -> LuaResult<u32> {
    let mut args = parse_create_frame_args(state)?;
    let (parent_id, parent_explicit) =
        resolve_parent_id(state, args.parent_val, args.default_parent_allowed)?;
    let runtime_inherits = build_runtime_inherits(&args.frame_type, args.inherits.as_deref());
    let template_initializer = args.template_initializer;
    args.name = resolve_frame_name(state, args.name.take(), parent_id, parent_explicit)?;
    let frame_id = register_runtime_frame(state, args, parent_id, parent_explicit)?;
    apply_runtime_frame_templates(
        state,
        frame_id,
        runtime_inherits.as_deref(),
        template_initializer,
    )?;
    let frame_val = frame_ref(state, frame_id)?;
    state.push(frame_val);
    Ok(1)
}

pub fn enumerate_frames(state: &mut LuaState) -> LuaResult<u32> {
    let current = stack_val(state, 1);
    let after_id = if current == Val::Nil {
        0
    } else {
        extract_frame_id(state, current)
            .ok_or_else(|| runtime_error("bad argument #1 to 'EnumerateFrames' (Frame expected)"))?
    };

    let next_id = {
        let sim = borrow_state(state)?;
        sim.widgets.next_enumerable_id_after(after_id)
    };

    if let Some(next_id) = next_id {
        let frame = frame_ref(state, next_id)?;
        state.push(frame);
    } else {
        state.push(Val::Nil);
    }
    Ok(1)
}

pub(crate) use template_chain::{
    apply_runtime_template_chain, apply_runtime_template_chain_with_frame_overrides,
    apply_template_scripts, first_fast_install_miss, replay_runtime_template_parent_links,
    scripts_support_fast_install,
};
struct CreateFrameArgs {
    frame_type: String,
    widget_type: WidgetType,
    name: Option<String>,
    parent_val: Val,
    default_parent_allowed: bool,
    inherits: Option<String>,
    id: Option<i32>,
    template_initializer: Val,
}

fn register_runtime_frame(
    state: &mut LuaState,
    args: CreateFrameArgs,
    parent_id: u64,
    parent_explicit: bool,
) -> LuaResult<u64> {
    crate::lua_api::globals::create_frame::create_frame_instance(
        state,
        args.widget_type,
        &args.frame_type,
        args.name,
        (parent_id != 0).then_some(parent_id),
        parent_explicit,
        args.id,
    )
}

fn apply_runtime_frame_templates(
    state: &mut LuaState,
    frame_id: u64,
    runtime_inherits: Option<&str>,
    template_initializer: Val,
) -> LuaResult<()> {
    template_chain::ensure_runtime_slider_children(state, frame_id)?;
    let fire_on_load = borrow_state(state)?.suppress_runtime_on_load_depth == 0;
    template_chain::apply_runtime_template_chain_with_initializer(
        state,
        frame_id,
        runtime_inherits,
        fire_on_load,
        template_initializer,
    )?;
    replay_runtime_template_parent_links(state, frame_id, runtime_inherits)
}

fn parse_create_frame_args(state: &mut LuaState) -> LuaResult<CreateFrameArgs> {
    let frame_type: String = FromStack::from_stack(state, 1)?;
    let arg_count = state.top.saturating_sub(state.base);
    let arg2 = stack_val(state, 2);
    let arg3 = stack_val(state, 3);
    let arg4 = stack_val(state, 4);
    let arg5 = stack_val(state, 5);
    let arg6 = stack_val(state, 6);
    let name = if matches!(arg2, Val::Str(_)) || matches!(arg2, Val::Nil) {
        Option::<String>::from_stack(state, 2)?
    } else {
        None
    };
    let parent_val = if matches!(arg2, Val::Str(_) | Val::Nil) {
        arg3
    } else {
        Val::Nil
    };
    let default_parent_allowed = arg_count >= 2 && matches!(arg2, Val::Str(_) | Val::Nil);
    let inherits = val_to_string(state, arg4);
    let id = parse_frame_id(arg5);
    let template_initializer = parse_template_initializer(state, arg6)?;
    let widget_type = resolve_runtime_widget_type(&frame_type)?;
    Ok(CreateFrameArgs {
        frame_type,
        widget_type,
        name,
        parent_val,
        default_parent_allowed,
        inherits,
        id,
        template_initializer,
    })
}

fn parse_frame_id(value: Val) -> Option<i32> {
    match value {
        Val::Num(id) => Some(id as i32),
        _ => None,
    }
}

fn parse_template_initializer(state: &LuaState, value: Val) -> LuaResult<Val> {
    let is_xml_frame_creation = borrow_state(state)?.suppress_runtime_on_load_depth > 0;
    let initializer = if is_xml_frame_creation && matches!(value, Val::Function(_)) {
        value
    } else {
        Val::Nil
    };
    Ok(initializer)
}

fn resolve_runtime_widget_type(frame_type: &str) -> LuaResult<WidgetType> {
    let mapped_frame_type =
        crate::xml::widget_type_for_tag(frame_type).map(|(widget_type, _)| widget_type);
    let widget_type_name = mapped_frame_type.unwrap_or(frame_type);

    WidgetType::from_str(widget_type_name)
        .ok_or_else(|| rilua::runtime_error(format!("unknown frame type '{frame_type}'")))
}

fn resolve_frame_name(
    state: &mut LuaState,
    name: Option<String>,
    parent_id: u64,
    parent_explicit: bool,
) -> LuaResult<Option<String>> {
    let Some(name) = name else {
        return Ok(None);
    };
    let parent_for_name_sub = (parent_explicit && parent_id != 0).then_some(parent_id);
    let sim = borrow_state(state)?;
    Ok(Some(
        crate::lua_api::globals::create_frame::apply_parent_sub(&name, parent_for_name_sub, &sim),
    ))
}

fn resolve_parent_id(
    state: &mut LuaState,
    parent_val: Val,
    default_parent_allowed: bool,
) -> LuaResult<(u64, bool)> {
    let parent_explicit = !matches!(parent_val, Val::Nil);
    let parent_id = if parent_explicit {
        extract_frame_id(state, parent_val)
            .ok_or_else(|| rilua::runtime_error("CreateFrame parent must be a frame or nil"))?
    } else if !default_parent_allowed {
        0
    } else {
        let sim = borrow_state(state)?;
        sim.widgets.get_id_by_name("UIParent").unwrap_or_default()
    };
    Ok((parent_id, parent_explicit))
}

fn build_runtime_inherits(frame_type: &str, explicit_inherits: Option<&str>) -> Option<String> {
    let intrinsic =
        crate::xml::widget_type_for_tag(frame_type).and_then(|(_, intrinsic)| intrinsic);
    template_chain::build_child_inherits(intrinsic, explicit_inherits)
}

// ---------------------------------------------------------------------------
// Global frames registration
// ---------------------------------------------------------------------------

pub fn register_global_frames(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    let named_frames = {
        let sim = borrow_state(state)?;
        sim.widgets
            .named_frames()
            .map(|(id, name)| (id, name.clone()))
            .collect::<Vec<_>>()
    };
    for (id, name) in named_frames {
        let frame_val = frame_ref(state, id)?;
        set_global_raw(state, &name, frame_val);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// register_all
// ---------------------------------------------------------------------------

/// Register all globals from create_frame.rs, global_frames.rs, and dropdown_api.rs
/// onto the rilua Lua state.
pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    super::loader_script_bindings::register_all(lua)?;
    LuaApiMut::register_function(lua, "CreateFrame", create_frame)?;
    LuaApiMut::register_function(lua, "CreateWindow", simple_window::create_window)?;
    LuaApiMut::register_function(lua, "EnumerateFrames", enumerate_frames)?;
    register_global_frames(lua)?;
    dropdown_api::register_dropdown_constants(lua)?;
    dropdown_api::register_dropdown_mutators(lua)?;
    dropdown_api::register_dropdown_selections(lua)?;
    dropdown_api::register_dropdown_queries(lua)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn create_frame_registers_named_global_and_parent() {
        let env = WowLuaEnv::new().expect("env");

        env.exec(
            r#"
            local child = CreateFrame("Frame", "RiluaCreateFrameChild", UIParent)
            assert(child ~= nil, "CreateFrame should return a frame")
            assert(type(child) == "table", "CreateFrame should expose frames as tables")
            assert(RiluaCreateFrameChild == child, "named frame should be global")
            assert(child:GetParent() == UIParent, "parent should be assigned")
        "#,
        )
        .expect("CreateFrame should create a named child frame");

        let parent_name: Option<String> = env
            .eval("local p = RiluaCreateFrameChild:GetParent(); return p and p:GetName()")
            .expect("eval parent name");
        assert_eq!(parent_name.as_deref(), Some("UIParent"));
    }

    #[test]
    fn create_frame_accepts_intrinsic_widget_aliases() {
        let env = WowLuaEnv::new().expect("env");

        let result: String = env
            .eval(
                r#"
                local dropdown = CreateFrame("DropDownToggleButton", "RuntimeDropDownToggle", UIParent)
                local eventButton = CreateFrame("EventButton", "RuntimeEventButton", UIParent)
                return table.concat({
                    dropdown:GetObjectType(),
                    tostring(dropdown:IsObjectType("Button")),
                    eventButton:GetObjectType(),
                    tostring(eventButton:IsObjectType("Button")),
                }, "|")
            "#,
            )
            .expect("CreateFrame should accept intrinsic aliases");

        assert_eq!(result, "Button|true|Button|true");
    }

    #[test]
    fn create_frame_named_ui_parent_preserves_existing_global() {
        let env = WowLuaEnv::new().expect("env");

        let result: String = env
            .eval(
                r#"
                local original = UIParent
                local replacement = CreateFrame("Frame", "UIParent")

                if UIParent ~= original then
                    return "global_replaced"
                end
                if replacement == original then
                    return "returned_original"
                end

                local ok, err = pcall(function()
                    replacement:SetAllPoints(UIParent)
                end)
                if not ok then
                    return "set_all_points_failed:" .. tostring(err)
                end

                return "ok"
            "#,
            )
            .expect("duplicate UIParent CreateFrame probe");

        assert_eq!(
            result, "ok",
            "CreateFrame(\"Frame\", \"UIParent\") must not clobber the existing global UIParent: {result}"
        );
    }

    #[test]
    fn create_frame_omitted_parent_is_nil() {
        let env = WowLuaEnv::new().expect("env");

        let parents: (bool, Option<String>) = env
            .eval(
                r#"
                local omitted = CreateFrame("Frame")
                local explicitNilName = CreateFrame("Frame", nil)
                local explicitNilParent = explicitNilName:GetParent()

                return omitted:GetParent() == nil,
                       explicitNilParent and explicitNilParent:GetName()
            "#,
            )
            .expect("CreateFrame parent defaults should be observable");

        assert!(
            parents.0,
            "omitting the parent argument must leave parent nil"
        );
        assert_eq!(
            parents.1.as_deref(),
            Some("UIParent"),
            "passing nil as the name still uses the legacy UIParent default"
        );
    }

    #[test]
    fn enumerate_frames_walks_created_frames() {
        let env = WowLuaEnv::new().expect("env");

        let result: String = env
            .eval(
                r#"
                local first = EnumerateFrames()
                if first == nil then
                    return "missing_first"
                end

                local second = EnumerateFrames(first)
                if second == nil or second == first then
                    return "missing_second"
                end

                local target = CreateFrame("Frame", "EnumerateFramesTarget", UIParent)
                local object = EnumerateFrames()
                while object do
                    if object == target then
                        return "ok"
                    end
                    object = EnumerateFrames(object)
                end
                return "target_not_found"
            "#,
            )
            .expect("EnumerateFrames should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn enumerate_frames_skips_forbidden_frames() {
        let env = WowLuaEnv::new().expect("env");

        let result: (bool, bool, bool) = env
            .eval(
                r#"
                local forbidden = CreateFrame("Frame", "EnumerateForbiddenProbe", UIParent)
                forbidden:SetForbidden(true)
                local visible = CreateFrame("Frame", "EnumerateVisibleProbe", UIParent)

                local sawForbidden = false
                local sawVisible = false
                local object = EnumerateFrames()
                while object do
                    if object == forbidden then
                        sawForbidden = true
                    end
                    if object == visible then
                        sawVisible = true
                    end
                    object = EnumerateFrames(object)
                end

                return sawForbidden, sawVisible, forbidden:IsForbidden()
            "#,
            )
            .expect(
                "EnumerateFrames should skip forbidden frames without invalidating direct handles",
            );

        assert_eq!(
            result,
            (false, true, true),
            "forbidden frames should remain directly usable but hidden from EnumerateFrames"
        );
    }
}
