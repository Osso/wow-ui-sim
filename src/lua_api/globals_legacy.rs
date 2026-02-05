//! Global WoW API functions.

use super::globals::addon_api::register_addon_api;
use super::globals::create_frame::create_frame_function;
use super::globals::font_api::create_standard_font_objects;
use super::globals::c_collection_api::register_c_collection_api;
use super::globals::constants_api::register_constants_api;
use super::globals::c_item_api::register_c_item_api;
use super::globals::c_map_api::register_c_map_api;
use super::globals::c_misc_api::register_c_misc_api;
use super::globals::c_quest_api::register_c_quest_api;
use super::globals::c_system_api::register_c_system_api;
use super::globals::cvar_api::register_cvar_api;
use super::globals::dropdown_api::register_dropdown_api;
use super::globals::enum_api::register_enum_api;
use super::globals::font_api::register_font_api;
use super::globals::global_frames::register_global_frames;
use super::globals::item_api::register_item_api;
use super::globals::locale_api::register_locale_api;
use super::globals::mixin_api::register_mixin_api;
use super::globals::player_api::register_player_api;
use super::globals::pool_api::register_pool_api;
use super::globals::quest_frames::register_quest_frames;
use super::globals::settings_api::register_settings_api;
use super::globals::spell_api::register_spell_api;
use super::globals::system_api::register_system_api;
use super::globals::timer_api::register_timer_api;
use super::globals::tooltip_api::register_tooltip_frames;
use super::globals::ui_frames::register_ui_frames;
use super::globals::register_all_ui_strings;
use super::globals::unit_api::register_unit_api;
use super::globals::utility_api::register_utility_api;
use super::frame::FrameHandle;
use super::SimState;
use mlua::{Lua, ObjectLike, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all global WoW API functions.
pub fn register_globals(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    // Override print to capture output to console
    let state_for_print = Rc::clone(&state);
    let print_func = lua.create_function(move |_lua, args: mlua::Variadic<Value>| {
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
        // Print to terminal too
        println!("{}", output);
        // Store in console buffer
        state_for_print.borrow_mut().console_output.push(output);
        Ok(())
    })?;
    globals.set("print", print_func)?;

    // Store the original ipairs for tables
    let original_ipairs: mlua::Function = lua.globals().get("ipairs")?;

    // Custom ipairs that supports frame userdata (for iterating over header children)
    let state_for_ipairs = Rc::clone(&state);
    let custom_ipairs = lua.create_function(move |lua, value: Value| {
        // If it's a frame userdata, return an iterator over its children
        if let Value::UserData(ud) = &value {
            if let Ok(handle) = ud.borrow::<FrameHandle>() {
                let frame_id = handle.id;
                let state_rc = Rc::clone(&handle.state);
                drop(handle);

                // Get children IDs
                let children: Vec<u64> = {
                    let state = state_rc.borrow();
                    state.widgets.get(frame_id).map(|f| f.children.clone()).unwrap_or_default()
                };

                // Create iterator function that returns (index, child) pairs
                let iterator_state = Rc::clone(&state_for_ipairs);
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

                // Return iterator, nil (stateless), 0 (starting index)
                return Ok(mlua::MultiValue::from_vec(vec![
                    Value::Function(iterator),
                    Value::Nil,
                    Value::Integer(0),
                ]));
            }
        }

        // Fall back to original ipairs for tables
        let original_ipairs: mlua::Function = lua.globals().get("__original_ipairs")?;
        original_ipairs.call(value)
    })?;
    globals.set("__original_ipairs", original_ipairs)?;
    globals.set("ipairs", custom_ipairs)?;

    // Custom getmetatable that returns a proper metatable structure for frame userdata
    // WoW addons expect getmetatable(frame).__index to be a table they can iterate over
    let custom_getmetatable = lua.create_function(|lua, value: Value| {
        // Check if it's our frame userdata
        if let Value::UserData(ud) = &value {
            if ud.borrow::<FrameHandle>().is_ok() {
                // Return a fake metatable with __index as a table of method names
                // This allows addons to do: for name, func in pairs(getmetatable(frame).__index) do
                let mt = lua.create_table()?;
                let index_table = lua.create_table()?;

                // Add all frame method names to the __index table
                // The values are the actual methods from the userdata
                let method_names = [
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
                    "GetAnimationGroups", "CreateLine", "GetNumRegions", "SetText", "GetText", "SetTitle", "GetTitle",
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
                    "GetResizeBounds", "SetDontSavePosition", "GetDontSavePosition",
                    "SetWindow", "GetWindow", "SetHyperlinksEnabled", "GetHyperlinksEnabled",
                    "AdjustPointsOffset", "SetTexture", "GetTexture", "SetTexCoord",
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
                    "GetNumChildren", "PlaySoundFile", "ClearNineSlice", "ApplyLayout",
                    "MarkDirty", "IsLayoutFrame", "AddLayoutChildren", "ClearLayout",
                    "SetAutomaticFrameLevelEnabled", "IsAutomaticFrameLevelEnabled",
                    // Button methods
                    "Click", "SetNormalTexture", "GetNormalTexture", "SetPushedTexture",
                    "GetPushedTexture", "SetHighlightTexture", "GetHighlightTexture",
                    "SetDisabledTexture", "GetDisabledTexture", "SetNormalFontObject",
                    "GetNormalFontObject", "SetHighlightFontObject", "GetHighlightFontObject",
                    "SetDisabledFontObject", "GetDisabledFontObject", "SetPushedTextOffset",
                    "GetPushedTextOffset", "Enable", "Disable", "IsEnabled", "SetEnabled",
                    "SetButtonState", "GetButtonState", "LockHighlight", "UnlockHighlight",
                    "RegisterForClicks", "RegisterForMouse", "GetMotionScriptsWhileDisabled",
                    "SetMotionScriptsWhileDisabled", "GetFontString", "SetFontString",
                    "GetTextWidth", "GetTextHeight", "GetNumLines", "GetMaxLines", "GetUnboundedStringWidth", "GetFontObjectForAlphabet",
                    // EditBox methods
                    "SetMaxLetters", "GetMaxLetters", "SetMaxBytes", "GetMaxBytes",
                    "SetNumber", "GetNumber", "SetMultiLine", "IsMultiLine",
                    "SetAutoFocus", "HasFocus", "SetFocus", "ClearFocus", "Insert",
                    "SetCursorPosition", "GetCursorPosition", "SetTextInsets",
                    "GetTextInsets", "SetHistoryLines", "GetHistoryLines", "AddHistoryLine",
                    "HighlightText", "GetHighlightedText", "SetBlinkSpeed",
                    "SetNumeric", "IsNumeric", "SetPassword", "IsPassword",
                    "SetCountInvisibleLetters", "IsCountInvisibleLetters",
                    "SetSecurityDisablePaste", "SetSecurityDisableSetText", "SetSecureText",
                    "SetVisibleTextByteLimit", "GetUTF8CursorPosition", "SetEnabled",
                    // Slider methods
                    "SetMinMaxValues", "GetMinMaxValues", "SetValue", "GetValue",
                    "SetValueStep", "GetValueStep", "SetStepsPerPage", "GetStepsPerPage",
                    "SetOrientation", "GetOrientation", "SetThumbTexture", "GetThumbTexture",
                    "SetObeyStepOnDrag", "GetObeyStepOnDrag",
                    // ScrollFrame methods
                    "SetScrollChild", "GetScrollChild", "SetHorizontalScroll",
                    "GetHorizontalScroll", "SetVerticalScroll", "GetVerticalScroll",
                    "GetHorizontalScrollRange", "GetVerticalScrollRange", "UpdateScrollChildRect",
                    // StatusBar methods
                    "SetStatusBarTexture", "GetStatusBarTexture", "SetStatusBarColor",
                    "GetStatusBarColor", "SetStatusBarDesaturated", "GetStatusBarDesaturated",
                    "SetStatusBarAtlas", "SetMinMaxValues", "GetMinMaxValues",
                    "SetValue", "GetValue", "SetFillStyle", "GetFillStyle",
                    "SetReverseFill", "GetReverseFill", "SetRotatesTexture", "GetRotatesTexture",
                    // CheckButton methods
                    "SetChecked", "GetChecked", "GetCheckedTexture", "SetCheckedTexture",
                    // Model methods
                    "SetModel", "GetModel", "SetModelScale", "GetModelScale",
                    "SetPosition", "GetPosition", "SetFacing", "GetFacing",
                    "SetSequence", "GetSequence", "SetCamera", "GetCamera",
                    "ClearModel", "SetDisplayInfo", "SetCreature", "SetUnit",
                    "RefreshUnit", "RefreshCamera", "SetItem", "SetItemAppearance",
                    "SetKeepModelOnHide", "GetKeepModelOnHide", "SetLight",
                    "SetModelDrawLayer", "GetModelDrawLayer", "UseModelCenterToTransform",
                    "SetCamDistanceScale", "GetCamDistanceScale", "SetPortraitZoom",
                    "SetDesaturation", "SetRotation", "SetSequenceTime", "SetAnimation",
                    // ColorSelect methods
                    "SetColorRGB", "GetColorRGB", "SetColorHSV", "GetColorHSV",
                    // Cooldown methods
                    "SetCooldown", "Clear", "GetCooldownTimes", "SetCooldownDuration",
                    "GetCooldownDuration", "SetHideCountdownNumbers", "SetDrawSwipe",
                    "SetDrawBling", "SetDrawEdge", "SetSwipeColor", "SetSwipeTexture",
                    "SetBlingTexture", "SetEdgeTexture", "SetEdgeScale", "SetUseCircularEdge",
                    "SetReverse", "GetReverse", "SetRotation", "GetRotation",
                    // MessageFrame methods
                    "AddMessage", "AddMsg", "SetFading", "GetFading", "SetFadeDuration",
                    "GetFadeDuration", "SetFadePower", "GetFadePower", "SetTimeVisible",
                    "GetTimeVisible", "SetInsertMode", "GetInsertMode",
                    // GameTooltip methods
                    "SetOwner", "GetOwner", "AddLine", "AddDoubleLine", "SetPadding",
                    "GetPadding", "NumLines", "GetLine", "ClearLines", "SetMinimumWidth",
                    "SetAnchorType", "GetAnchorType", "SetPoint", "SetHyperlink",
                    "SetSpellByID", "SetItemByID", "SetUnitBuff", "SetUnitDebuff",
                    "SetUnitAura", "SetAction", "SetBagItem", "SetInventoryItem",
                    "FadeOut", "AppendText", "SetFrameStrata", "IsForbidden",
                ];

                for name in method_names {
                    // Get the method from the userdata
                    if let Ok(method) = ud.get::<mlua::Function>(name) {
                        index_table.set(name, method)?;
                    }
                }

                mt.set("__index", index_table)?;
                return Ok(Value::Table(mt));
            }
        }

        // For non-frame values, use the real getmetatable
        let real_getmetatable: mlua::Function = lua.globals().get("__real_getmetatable")?;
        real_getmetatable.call(value)
    })?;

    // Save the original getmetatable and install our custom one
    let real_getmetatable: mlua::Function = lua.globals().get("getmetatable")?;
    globals.set("__real_getmetatable", real_getmetatable)?;
    globals.set("getmetatable", custom_getmetatable)?;

    // CreateFrame from separate module
    let create_frame = create_frame_function(lua, Rc::clone(&state))?;
    globals.set("CreateFrame", create_frame)?;
    // Register functions from split modules
    // These override any duplicates registered above (will be cleaned up in future refactoring)
    register_locale_api(lua)?;
    register_addon_api(lua, Rc::clone(&state))?;
    register_unit_api(lua)?;
    register_player_api(lua)?;
    register_pool_api(lua, Rc::clone(&state))?;
    register_timer_api(lua, Rc::clone(&state))?;
    register_enum_api(lua)?;
    register_constants_api(lua)?;
    register_c_map_api(lua)?;
    register_c_quest_api(lua)?;
    register_c_collection_api(lua)?;
    register_c_item_api(lua)?;
    register_c_misc_api(lua)?;
    register_c_system_api(lua)?;
    register_mixin_api(lua)?;
    register_dropdown_api(lua, Rc::clone(&state))?;
    register_utility_api(lua)?;
    register_settings_api(lua)?;
    register_spell_api(lua)?;
    register_cvar_api(lua, Rc::clone(&state))?;
    register_system_api(lua, Rc::clone(&state))?;
    register_item_api(lua)?;
    register_font_api(lua)?;
    register_global_frames(lua, Rc::clone(&state))?;
    register_tooltip_frames(lua, Rc::clone(&state))?;
    register_quest_frames(lua, Rc::clone(&state))?;
    register_ui_frames(lua, Rc::clone(&state))?;

    // Register UI strings
    let globals = lua.globals();
    register_all_ui_strings(lua, &globals)?;

    // Create standard WoW font objects
    // These are font objects that many addons expect to exist
    create_standard_font_objects(lua)?;

    Ok(())
}
