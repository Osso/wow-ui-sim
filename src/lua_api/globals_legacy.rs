//! Global WoW API functions.
//!
//! Orchestrates registration of all WoW API globals by delegating to
//! sub-modules and registering core Lua overrides (print, ipairs, getmetatable).

use super::frame::FrameHandle;
use super::globals::addon_api::register_addon_api;
use super::globals::c_collection_api::register_c_collection_api;
use super::globals::c_item_api::register_c_item_api;
use super::globals::c_map_api::register_c_map_api;
use super::globals::c_misc_api::register_c_misc_api;
use super::globals::c_editmode_api::register_c_editmode_api;
use super::globals::c_stubs_api::register_c_stubs_api;
use super::globals::c_quest_api::register_c_quest_api;
use super::globals::c_system_api::register_c_system_api;
use super::globals::constants_api::register_constants_api;
use super::globals::create_frame::create_frame_function;
use super::globals::cvar_api::register_cvar_api;
use super::globals::dropdown_api::register_dropdown_api;
use super::globals::enum_api::register_enum_api;
use super::globals::font_api::{create_standard_font_objects, register_font_api};
use super::globals::global_frames::register_global_frames;
use super::globals::item_api::register_item_api;
use super::globals::locale_api::register_locale_api;
use super::globals::mixin_api::register_mixin_api;
use super::globals::player_api::register_player_api;
use super::globals::pool_api::register_pool_api;
use super::globals::quest_frames::register_quest_frames;
use super::globals::register_all_ui_strings;
use super::globals::settings_api::register_settings_api;
use super::globals::sound_api::register_sound_api;
use super::globals::spell_api::register_spell_api;
use super::globals::system_api::register_system_api;
use super::globals::timer_api::register_timer_api;
use super::globals::tooltip_api::register_tooltip_frames;
use super::globals::unit_api::register_unit_api;
use super::globals::utility_api::register_utility_api;
use super::SimState;
use mlua::{Lua, ObjectLike, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all global WoW API functions.
pub fn register_globals(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_print(lua, Rc::clone(&state))?;
    register_custom_ipairs(lua, Rc::clone(&state))?;
    register_custom_getmetatable(lua)?;
    register_create_frame(lua, Rc::clone(&state))?;
    register_submodule_apis(lua, &state)?;
    register_ui_strings_and_fonts(lua)?;
    patch_string_format(lua)?;
    Ok(())
}

/// Patch string.format to support:
/// - %F (uppercase float) which Lua 5.1 lacks; converted to %f
/// - Positional arguments (%1$s, %2$d, %11$s) which WoW's patched LuaJIT supports
///   but standard Lua 5.1 does not; converted by reordering arguments
fn patch_string_format(lua: &Lua) -> Result<()> {
    lua.load(r#"
        local _format = string.format
        string.format = function(fmt, ...)
            if type(fmt) ~= "string" then return _format(fmt, ...) end
            fmt = fmt:gsub("%%(%d*%.?%d*)F", "%%%1f")
            if not fmt:find("%%%d+%$") then return _format(fmt, ...) end
            local args = {...}
            local out, new_args, seq = {}, {}, 0
            local i, len = 1, #fmt
            while i <= len do
                if fmt:sub(i,i) ~= "%" then
                    out[#out+1] = fmt:sub(i,i); i = i + 1
                elseif fmt:sub(i+1,i+1) == "%" then
                    out[#out+1] = "%%"; i = i + 2
                else
                    local n, a = fmt:match("^%%(%d+)%$()", i)
                    if n then
                        new_args[#new_args+1] = args[tonumber(n)]
                        out[#out+1] = "%"; i = a
                    else
                        seq = seq + 1
                        new_args[#new_args+1] = args[seq]
                        out[#out+1] = "%"; i = i + 1
                    end
                    while i <= len and fmt:sub(i,i):find("[%-+ #0]") do
                        out[#out+1] = fmt:sub(i,i); i = i + 1
                    end
                    while i <= len and fmt:sub(i,i):find("%d") do
                        out[#out+1] = fmt:sub(i,i); i = i + 1
                    end
                    if i <= len and fmt:sub(i,i) == "." then
                        out[#out+1] = "."; i = i + 1
                        while i <= len and fmt:sub(i,i):find("%d") do
                            out[#out+1] = fmt:sub(i,i); i = i + 1
                        end
                    end
                    if i <= len and fmt:sub(i,i):find("[diouxXeEfgGaAcspqn]") then
                        out[#out+1] = fmt:sub(i,i); i = i + 1
                    end
                end
            end
            return _format(table.concat(out), unpack(new_args))
        end
        format = string.format
    "#).exec()
}

/// Override `print` to capture output to the console buffer.
fn register_print(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let print_func = lua.create_function(move |_lua, args: mlua::Variadic<Value>| {
        let output = format_print_args(&args);
        println!("{}", output);
        state.borrow_mut().console_output.push(output);
        Ok(())
    })?;
    lua.globals().set("print", print_func)
}

/// Format variadic print arguments with tab separators, matching WoW's print behavior.
fn format_print_args(args: &[Value]) -> String {
    let mut output = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            output.push('\t');
        }
        match arg {
            Value::Nil => output.push_str("nil"),
            Value::Boolean(b) => output.push_str(if *b { "true" } else { "false" }),
            Value::Integer(n) => output.push_str(&n.to_string()),
            Value::Number(n) => output.push_str(&n.to_string()),
            Value::String(s) => output.push_str(&s.to_string_lossy()),
            Value::Table(_) => output.push_str("table"),
            Value::Function(_) => output.push_str("function"),
            Value::UserData(_) => output.push_str("userdata"),
            _ => output.push_str(&format!("{:?}", arg)),
        }
    }
    output
}

/// Override `ipairs` to support iterating over frame userdata children.
///
/// WoW addons iterate frame children with `for i, child in ipairs(frame)`.
/// Falls back to the original `ipairs` for regular tables.
fn register_custom_ipairs(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    let original_ipairs: mlua::Function = globals.get("ipairs")?;

    let custom_ipairs = lua.create_function(move |lua, value: Value| {
        if let Value::UserData(ud) = &value
            && let Ok(handle) = ud.borrow::<FrameHandle>() {
                return create_frame_children_iterator(lua, &handle, &state);
            }
        // Fall back to original ipairs for tables
        let original_ipairs: mlua::Function = lua.globals().get("__original_ipairs")?;
        original_ipairs.call(value)
    })?;

    globals.set("__original_ipairs", original_ipairs)?;
    globals.set("ipairs", custom_ipairs)
}

/// Create a stateless iterator over a frame's children for use with `ipairs`.
///
/// Returns `(iterator_fn, nil, 0)` matching Lua's generic for protocol.
fn create_frame_children_iterator(
    lua: &Lua,
    handle: &FrameHandle,
    state: &Rc<RefCell<SimState>>,
) -> Result<mlua::MultiValue> {
    let frame_id = handle.id;
    let state_rc = Rc::clone(&handle.state);

    let children: Vec<u64> = {
        let st = state_rc.borrow();
        st.widgets.get(frame_id).map(|f| f.children.clone()).unwrap_or_default()
    };

    let iterator_state = Rc::clone(state);
    let iterator = lua.create_function(move |lua, (_, idx): (Value, i32)| {
        let next_idx = idx + 1;
        if next_idx as usize > children.len() {
            return Ok(mlua::MultiValue::new());
        }

        let child_id = children[(next_idx - 1) as usize];
        let handle = FrameHandle {
            id: child_id,
            state: Rc::clone(&iterator_state),
        };
        let ud = lua.create_userdata(handle)?;
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(next_idx as i64),
            Value::UserData(ud),
        ]))
    })?;

    Ok(mlua::MultiValue::from_vec(vec![
        Value::Function(iterator),
        Value::Nil,
        Value::Integer(0),
    ]))
}

/// Override `getmetatable` to return a proper metatable for frame userdata.
///
/// WoW addons expect `getmetatable(frame).__index` to be an iterable table
/// of method names mapped to functions.
fn register_custom_getmetatable(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    let custom_getmetatable = lua.create_function(|lua, value: Value| {
        if let Value::UserData(ud) = &value
            && ud.borrow::<FrameHandle>().is_ok() {
                return build_frame_metatable(lua, ud);
            }
        let real_getmetatable: mlua::Function = lua.globals().get("__real_getmetatable")?;
        real_getmetatable.call(value)
    })?;

    let real_getmetatable: mlua::Function = globals.get("getmetatable")?;
    globals.set("__real_getmetatable", real_getmetatable)?;
    globals.set("getmetatable", custom_getmetatable)
}

/// Build a fake metatable for frame userdata with `__index` populated from actual methods.
fn build_frame_metatable(lua: &Lua, ud: &mlua::AnyUserData) -> Result<Value> {
    let mt = lua.create_table()?;
    let index_table = lua.create_table()?;
    populate_method_index(ud, &index_table)?;
    mt.set("__index", index_table)?;
    Ok(Value::Table(mt))
}

/// Populate an index table with all frame method names from the categorized lists.
fn populate_method_index(ud: &mlua::AnyUserData, index_table: &mlua::Table) -> Result<()> {
    for methods in ALL_METHOD_GROUPS {
        for &name in *methods {
            if let Ok(method) = ud.get::<mlua::Function>(name) {
                index_table.set(name, method)?;
            }
        }
    }
    Ok(())
}

/// All method name groups, organized by widget type.
const ALL_METHOD_GROUPS: &[&[&str]] = &[
    FRAME_BASE_METHODS, TEXTURE_METHODS, BUTTON_METHODS,
    EDITBOX_METHODS, SLIDER_METHODS, SCROLLFRAME_METHODS,
    STATUSBAR_METHODS, CHECKBUTTON_METHODS, MODEL_METHODS,
    COLORSELECT_METHODS, COOLDOWN_METHODS, MESSAGEFRAME_METHODS,
    GAMETOOLTIP_METHODS,
];

const FRAME_BASE_METHODS: &[&str] = &[
    "GetName", "GetWidth", "GetHeight", "SetSize", "SetWidth", "SetHeight",
    "SetPoint", "ClearAllPoints", "GetPoint", "GetNumPoints", "Show", "Hide",
    "IsShown", "IsVisible", "SetShown", "SetAlpha", "GetAlpha", "SetScale",
    "GetScale", "GetParent", "SetParent", "GetChildren", "GetRegions",
    "GetFrameLevel", "SetFrameLevel", "GetFrameStrata", "SetFrameStrata",
    "EnableMouse", "IsMouseEnabled", "EnableMouseWheel", "IsMouseWheelEnabled",
    "SetMovable", "IsMovable", "SetResizable",
    "IsResizable", "SetClampedToScreen", "IsClampedToScreen", "SetID", "GetID",
    "GetObjectType", "IsObjectType", "GetDebugName", "SetScript", "GetScript",
    "HookScript", "HasScript", "RegisterEvent", "UnregisterEvent",
    "UnregisterAllEvents", "IsEventRegistered", "RegisterForDrag",
    "RegisterUnitEvent", "SetAttribute", "GetAttribute", "ClearAttributes", "SetBackdrop",
    "ApplyBackdrop", "SetBackdropColor", "SetBackdropBorderColor", "GetBackdrop",
    "GetBackdropColor", "GetBackdropBorderColor", "CreateTexture",
    "CreateMaskTexture", "CreateFontString", "CreateAnimationGroup",
    "GetAnimationGroups", "GetNumRegions", "SetText", "GetText", "SetTitle", "GetTitle",
    "SetTextColor", "GetTextColor", "SetFontObject", "GetFontObject", "SetFont",
    "GetFont", "SetJustifyH", "GetJustifyH", "SetJustifyV", "GetJustifyV",
    "SetWordWrap", "GetWordWrap", "CanChangeAttribute", "SetToplevel",
    "IsToplevel", "Raise", "Lower", "GetEffectiveScale", "GetEffectiveAlpha",
    "SetPropagateKeyboardInput", "GetPropagateKeyboardInput", "SetIgnoreParentScale",
    "SetIgnoreParentAlpha", "SetFlattensRenderLayers", "GetFlattensRenderLayers",
    "SetDrawLayerEnabled", "GetDrawLayerEnabled", "GetTop", "GetBottom",
    "GetLeft", "GetRight", "GetCenter", "GetBounds", "GetRect", "GetSize",
    "GetScaledRect", "SetClipsChildren", "DoesClipChildren", "SetHitRectInsets",
    "GetHitRectInsets", "EnableKeyboard", "IsKeyboardEnabled",
    "SetMouseClickEnabled", "IsMouseClickEnabled", "SetMouseMotionEnabled",
    "IsMouseMotionEnabled", "SetPassThroughButtons", "GetPassThroughButtons",
    "SetFixedFrameLevel", "HasFixedFrameLevel", "SetFixedFrameStrata",
    "HasFixedFrameStrata", "SetUsingParentLevel", "IsUsingParentLevel",
    "EnableGamePadButton", "IsGamePadButtonEnabled", "EnableGamePadStick",
    "IsGamePadStickEnabled", "CanChangeProtectedState", "SetForbidden",
    "IsForbidden", "SetUserPlaced", "IsUserPlaced", "SetResizeBounds",
    "GetResizeBounds", "SetMinResize", "SetMaxResize",
    "SetDontSavePosition", "GetDontSavePosition",
    "SetWindow", "GetWindow", "SetHyperlinksEnabled", "GetHyperlinksEnabled",
    "AdjustPointsOffset", "ClearPoint", "ClearPointsOffset",
    "RegisterAllEvents", "IsRectValid", "IsObjectLoaded",
    "IsMouseOver", "StopAnimating", "GetSourceLocation",
    "Intersects", "SetAlphaFromBoolean", "EnableMouseMotion",
    "ClearScripts", "IsDrawLayerEnabled",
    "SetParentKey", "GetParentKey",
];

const TEXTURE_METHODS: &[&str] = &[
    "SetTexture", "GetTexture", "SetTexCoord",
    "GetTexCoord", "SetVertexColor", "GetVertexColor", "SetDesaturated",
    "IsDesaturated", "SetBlendMode", "GetBlendMode", "SetRotation",
    "GetRotation", "SetAtlas", "GetAtlas", "SetColorTexture", "SetGradient",
    "SetAllPoints",
    "SetSnapToPixelGrid", "IsSnappingToPixelGrid", "SetTexelSnappingBias",
    "GetTexelSnappingBias", "ClearTextureSlice", "SetTextureSliceMode",
    "GetTextureSliceMode", "SetTextureSliceMargins", "GetTextureSliceMargins",
    "AddMaskTexture", "RemoveMaskTexture", "GetMaskTexture", "GetNumMaskTextures",
    "SetDrawLayer", "GetDrawLayer", "SetVertexOffset", "GetVertexOffset",
    "SetHorizTile", "GetHorizTile", "SetVertTile", "GetVertTile",
    "SetNonBlocking", "GetNonBlocking", "SetBlockingLoadsRequested",
    "IsBlockingLoadRequested", "GetNumRegionsByLayer", "GetRegionsByLayer",
    "GetNumChildren", "PlaySoundFile", "ClearNineSlice",
    "SetAutomaticFrameLevelEnabled", "IsAutomaticFrameLevelEnabled",
    "SetVisuals",
];

const BUTTON_METHODS: &[&str] = &[
    "Click", "SetNormalTexture", "GetNormalTexture", "SetPushedTexture",
    "GetPushedTexture", "SetHighlightTexture", "GetHighlightTexture",
    "SetDisabledTexture", "GetDisabledTexture", "SetNormalFontObject",
    "GetNormalFontObject", "SetHighlightFontObject", "GetHighlightFontObject",
    "SetDisabledFontObject", "GetDisabledFontObject", "SetPushedTextOffset",
    "GetPushedTextOffset", "Enable", "Disable", "IsEnabled", "SetEnabled",
    "SetButtonState", "GetButtonState", "LockHighlight", "UnlockHighlight",
    "RegisterForClicks", "RegisterForMouse", "GetMotionScriptsWhileDisabled",
    "SetMotionScriptsWhileDisabled", "GetFontString", "SetFontString",
    "GetTextWidth", "GetTextHeight", "GetNumLines", "GetMaxLines",
    "GetUnboundedStringWidth", "GetFontObjectForAlphabet",
];

const EDITBOX_METHODS: &[&str] = &[
    "SetMaxLetters", "GetMaxLetters", "SetMaxBytes", "GetMaxBytes",
    "SetNumber", "GetNumber", "SetMultiLine", "IsMultiLine",
    "SetAutoFocus", "HasFocus", "SetFocus", "ClearFocus", "Insert",
    "SetCursorPosition", "GetCursorPosition", "SetTextInsets",
    "GetTextInsets", "SetHistoryLines", "GetHistoryLines", "AddHistoryLine",
    "HighlightText", "GetHighlightedText", "SetBlinkSpeed",
    "SetNumeric", "IsNumeric", "SetPassword", "IsPassword",
    "SetCountInvisibleLetters", "IsCountInvisibleLetters",
    "SetSecurityDisablePaste", "SetSecurityDisableSetText", "SetSecureText",
    "SetVisibleTextByteLimit", "GetUTF8CursorPosition",
];

const SLIDER_METHODS: &[&str] = &[
    "SetMinMaxValues", "GetMinMaxValues", "SetValue", "GetValue",
    "SetValueStep", "GetValueStep", "SetStepsPerPage", "GetStepsPerPage",
    "SetOrientation", "GetOrientation", "SetThumbTexture", "GetThumbTexture",
    "SetObeyStepOnDrag", "GetObeyStepOnDrag",
];

const SCROLLFRAME_METHODS: &[&str] = &[
    "SetScrollChild", "GetScrollChild", "SetHorizontalScroll",
    "GetHorizontalScroll", "SetVerticalScroll", "GetVerticalScroll",
    "GetHorizontalScrollRange", "GetVerticalScrollRange", "UpdateScrollChildRect",
];

const STATUSBAR_METHODS: &[&str] = &[
    "SetStatusBarTexture", "GetStatusBarTexture", "SetStatusBarColor",
    "GetStatusBarColor", "SetStatusBarDesaturated", "GetStatusBarDesaturated",
    "SetStatusBarAtlas", "SetFillStyle", "GetFillStyle",
    "SetReverseFill", "GetReverseFill", "SetRotatesTexture", "GetRotatesTexture",
];

const CHECKBUTTON_METHODS: &[&str] = &[
    "SetChecked", "GetChecked", "GetCheckedTexture", "SetCheckedTexture",
];

const MODEL_METHODS: &[&str] = &[
    "SetModel", "GetModel", "SetModelScale", "GetModelScale",
    "SetPosition", "GetPosition", "SetFacing", "GetFacing",
    "SetSequence", "GetSequence", "SetCamera", "GetCamera",
    "ClearModel", "SetDisplayInfo", "SetCreature", "SetUnit",
    "RefreshUnit", "RefreshCamera", "SetItem", "SetItemAppearance",
    "SetKeepModelOnHide", "GetKeepModelOnHide", "SetLight",
    "SetModelDrawLayer", "GetModelDrawLayer", "UseModelCenterToTransform",
    "SetCamDistanceScale", "GetCamDistanceScale", "SetPortraitZoom",
    "SetDesaturation", "SetSequenceTime", "SetAnimation",
];

const COLORSELECT_METHODS: &[&str] = &[
    "SetColorRGB", "GetColorRGB", "SetColorHSV", "GetColorHSV",
];

const COOLDOWN_METHODS: &[&str] = &[
    "SetCooldown", "Clear", "GetCooldownTimes", "SetCooldownDuration",
    "GetCooldownDuration", "SetHideCountdownNumbers", "SetDrawSwipe",
    "SetDrawBling", "SetDrawEdge", "SetSwipeColor", "SetSwipeTexture",
    "SetBlingTexture", "SetEdgeTexture", "SetEdgeScale", "SetUseCircularEdge",
    "SetReverse", "GetReverse",
];

const MESSAGEFRAME_METHODS: &[&str] = &[
    "AddMessage", "AddMsg", "SetFading", "GetFading", "SetFadeDuration",
    "GetFadeDuration", "SetFadePower", "GetFadePower", "SetTimeVisible",
    "GetTimeVisible", "SetInsertMode", "GetInsertMode",
];

const GAMETOOLTIP_METHODS: &[&str] = &[
    "SetOwner", "GetOwner", "AddLine", "AddDoubleLine", "SetPadding",
    "GetPadding", "NumLines", "GetLine", "ClearLines", "SetMinimumWidth",
    "SetAnchorType", "GetAnchorType", "SetHyperlink",
    "SetSpellByID", "SetItemByID", "SetUnitBuff", "SetUnitDebuff",
    "SetUnitAura", "SetAction", "SetBagItem", "SetInventoryItem",
    "FadeOut", "AppendText",
];

/// Register `CreateFrame` from its dedicated sub-module.
fn register_create_frame(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let create_frame = create_frame_function(lua, state)?;
    lua.globals().set("CreateFrame", create_frame)
}

/// Register all sub-module APIs (locale, addon, unit, timer, etc.).
fn register_submodule_apis(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    // Stateless APIs
    register_locale_api(lua)?;
    register_player_api(lua, state.clone())?;
    register_enum_api(lua)?;
    register_constants_api(lua)?;
    register_c_map_api(lua)?;
    register_c_quest_api(lua)?;
    register_c_collection_api(lua)?;
    register_c_item_api(lua)?;
    register_c_misc_api(lua)?;
    register_c_system_api(lua)?;
    register_c_stubs_api(lua, Rc::clone(state))?;
    register_c_editmode_api(lua)?;
    register_mixin_api(lua)?;
    register_utility_api(lua)?;
    register_settings_api(lua)?;
    register_spell_api(lua, Rc::clone(state))?;
    register_item_api(lua)?;
    register_font_api(lua)?;

    // Stateful APIs (need SimState)
    register_sound_api(lua, Rc::clone(state))?;
    register_unit_api(lua, Rc::clone(state))?;
    register_addon_api(lua, Rc::clone(state))?;
    register_pool_api(lua, Rc::clone(state))?;
    register_timer_api(lua, Rc::clone(state))?;
    register_dropdown_api(lua, Rc::clone(state))?;
    register_cvar_api(lua, Rc::clone(state))?;
    register_system_api(lua, Rc::clone(state))?;

    // Frame registration (creates global frame objects)
    register_global_frames(lua, Rc::clone(state))?;
    register_tooltip_frames(lua, Rc::clone(state))?;
    register_quest_frames(lua, Rc::clone(state))?;

    Ok(())
}

/// Register UI string constants and create standard font objects.
fn register_ui_strings_and_fonts(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    register_all_ui_strings(lua, &globals)?;
    create_standard_font_objects(lua)
}
