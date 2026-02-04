//! Global WoW API functions.

use super::{next_timer_id, PendingTimer, SimState};
use crate::widget::{Anchor, AnchorPoint, AttributeValue, Frame, FrameStrata, WidgetType};
use crate::xml::get_template_info;
use mlua::{Lua, MetaMethod, ObjectLike, Result, UserData, UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

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

    // CreateFrame(frameType, name, parent, template, id)
    let state_clone = Rc::clone(&state);
    let create_frame = lua.create_function(move |lua, args: mlua::MultiValue| {
        let mut args_iter = args.into_iter();

        let frame_type: String = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Frame".to_string());

        let name_raw: Option<String> = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string());

        let parent_id: Option<u64> = args_iter.next().and_then(|v| {
            if let Value::UserData(ud) = v {
                ud.borrow::<FrameHandle>().ok().map(|h| h.id)
            } else {
                None
            }
        });

        // Get template parameter (4th argument)
        let template: Option<String> = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string());

        // Get parent ID (default to UIParent)
        let parent_id = parent_id.or_else(|| {
            let state = state_clone.borrow();
            state.widgets.get_id_by_name("UIParent")
        });

        // Handle $parent/$Parent name substitution
        let name: Option<String> = name_raw.map(|n| {
            if n.contains("$parent") || n.contains("$Parent") {
                if let Some(pid) = parent_id {
                    let state = state_clone.borrow();
                    if let Some(parent_name) = state.widgets.get(pid).and_then(|f| f.name.clone())
                    {
                        n.replace("$parent", &parent_name)
                            .replace("$Parent", &parent_name)
                    } else {
                        n.replace("$parent", "").replace("$Parent", "")
                    }
                } else {
                    n.replace("$parent", "").replace("$Parent", "")
                }
            } else {
                n
            }
        });

        let widget_type = WidgetType::from_str(&frame_type).unwrap_or(WidgetType::Frame);
        let frame = Frame::new(widget_type, name.clone(), parent_id);
        let frame_id = frame.id;

        // Track if we need to create tooltip FontStrings after userdata creation
        let mut needs_tooltip_fontstrings = false;
        // Track if we need to register HybridScrollBarTemplate children as globals
        let mut needs_hybrid_scroll_globals = false;

        let mut state = state_clone.borrow_mut();
        state.widgets.register(frame);

        if let Some(pid) = parent_id {
            state.widgets.add_child(pid, frame_id);

            // Inherit strata and level from parent (like wowless does)
            let parent_props = state.widgets.get(pid).map(|p| (p.frame_strata, p.frame_level));
            if let Some((parent_strata, parent_level)) = parent_props {
                if let Some(f) = state.widgets.get_mut(frame_id) {
                    f.frame_strata = parent_strata;
                    f.frame_level = parent_level + 1;
                }
            }
        }

        // Handle template-based child elements
        // UICheckButtonTemplate and similar templates create a Text FontString
        if let Some(ref tmpl) = template {
            if tmpl.contains("CheckButton") || tmpl.contains("CheckBox") {
                // Create Text FontString child for checkbox templates
                let text_name = name.as_ref().map(|n| format!("{}Text", n));
                let text_frame = Frame::new(WidgetType::FontString, text_name.clone(), Some(frame_id));
                let text_id = text_frame.id;
                state.widgets.register(text_frame);
                state.widgets.add_child(frame_id, text_id);

                // Store the text child reference on the parent frame
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Text".to_string(), text_id);
                }
            }

            // HybridScrollBarTemplate creates track textures directly on the scrollbar frame
            if tmpl.contains("HybridScroll") {
                // Create ThumbTexture
                let thumb_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let thumb_id = thumb_tex.id;
                state.widgets.register(thumb_tex);
                state.widgets.add_child(frame_id, thumb_id);

                // Create ScrollBarTop texture
                let top_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let top_id = top_tex.id;
                state.widgets.register(top_tex);
                state.widgets.add_child(frame_id, top_id);

                // Create ScrollBarMiddle texture
                let mid_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let mid_id = mid_tex.id;
                state.widgets.register(mid_tex);
                state.widgets.add_child(frame_id, mid_id);

                // Create ScrollBarBottom texture
                let bot_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let bot_id = bot_tex.id;
                state.widgets.register(bot_tex);
                state.widgets.add_child(frame_id, bot_id);

                // Create ScrollUpButton
                let up_name = name.as_ref().map(|n| format!("{}ScrollUpButton", n));
                let up_button = Frame::new(WidgetType::Button, up_name.clone(), Some(frame_id));
                let up_id = up_button.id;
                state.widgets.register(up_button);
                state.widgets.add_child(frame_id, up_id);

                // Create ScrollDownButton
                let down_name = name.as_ref().map(|n| format!("{}ScrollDownButton", n));
                let down_button = Frame::new(WidgetType::Button, down_name.clone(), Some(frame_id));
                let down_id = down_button.id;
                state.widgets.register(down_button);
                state.widgets.add_child(frame_id, down_id);

                if let Some(sb) = state.widgets.get_mut(frame_id) {
                    sb.children_keys.insert("ThumbTexture".to_string(), thumb_id);
                    sb.children_keys.insert("ScrollBarTop".to_string(), top_id);
                    sb.children_keys.insert("ScrollBarMiddle".to_string(), mid_id);
                    sb.children_keys.insert("ScrollBarBottom".to_string(), bot_id);
                    sb.children_keys.insert("ScrollUpButton".to_string(), up_id);
                    sb.children_keys.insert("ScrollDownButton".to_string(), down_id);
                }
                // Register children as globals after state is dropped (if frame has a name)
                if name.is_some() {
                    needs_hybrid_scroll_globals = true;
                }
            }

            // FauxScrollFrameTemplate and ScrollFrameTemplate create ScrollBar with buttons as child
            if tmpl.contains("ScrollFrame") || (tmpl.contains("ScrollBar") && !tmpl.contains("HybridScroll")) {
                // Create ScrollBar slider child
                let scrollbar_name = name.as_ref().map(|n| format!("{}ScrollBar", n));
                let mut scrollbar = Frame::new(WidgetType::Slider, scrollbar_name.clone(), Some(frame_id));
                scrollbar.visible = false; // Usually hidden by default
                let scrollbar_id = scrollbar.id;
                state.widgets.register(scrollbar);
                state.widgets.add_child(frame_id, scrollbar_id);

                // Create ThumbTexture for ScrollBar
                let thumb_tex = Frame::new(WidgetType::Texture, None, Some(scrollbar_id));
                let thumb_id = thumb_tex.id;
                state.widgets.register(thumb_tex);
                state.widgets.add_child(scrollbar_id, thumb_id);

                // Create ScrollBarTop texture (scrollbar track top)
                let top_tex = Frame::new(WidgetType::Texture, None, Some(scrollbar_id));
                let top_id = top_tex.id;
                state.widgets.register(top_tex);
                state.widgets.add_child(scrollbar_id, top_id);

                // Create ScrollBarMiddle texture (scrollbar track middle)
                let mid_tex = Frame::new(WidgetType::Texture, None, Some(scrollbar_id));
                let mid_id = mid_tex.id;
                state.widgets.register(mid_tex);
                state.widgets.add_child(scrollbar_id, mid_id);

                // Create ScrollBarBottom texture (scrollbar track bottom)
                let bot_tex = Frame::new(WidgetType::Texture, None, Some(scrollbar_id));
                let bot_id = bot_tex.id;
                state.widgets.register(bot_tex);
                state.widgets.add_child(scrollbar_id, bot_id);

                if let Some(sb) = state.widgets.get_mut(scrollbar_id) {
                    sb.children_keys.insert("ThumbTexture".to_string(), thumb_id);
                    sb.children_keys.insert("ScrollBarTop".to_string(), top_id);
                    sb.children_keys.insert("ScrollBarMiddle".to_string(), mid_id);
                    sb.children_keys.insert("ScrollBarBottom".to_string(), bot_id);
                }

                // Create ScrollUpButton child of ScrollBar
                let up_name = name.as_ref().map(|n| format!("{}ScrollBarScrollUpButton", n));
                let up_button = Frame::new(WidgetType::Button, up_name.clone(), Some(scrollbar_id));
                let up_id = up_button.id;
                state.widgets.register(up_button);
                state.widgets.add_child(scrollbar_id, up_id);

                // Create Normal texture for ScrollUpButton
                let up_normal = Frame::new(WidgetType::Texture, None, Some(up_id));
                let up_normal_id = up_normal.id;
                state.widgets.register(up_normal);
                state.widgets.add_child(up_id, up_normal_id);

                // Create Pushed texture for ScrollUpButton
                let up_pushed = Frame::new(WidgetType::Texture, None, Some(up_id));
                let up_pushed_id = up_pushed.id;
                state.widgets.register(up_pushed);
                state.widgets.add_child(up_id, up_pushed_id);

                // Create Disabled texture for ScrollUpButton
                let up_disabled = Frame::new(WidgetType::Texture, None, Some(up_id));
                let up_disabled_id = up_disabled.id;
                state.widgets.register(up_disabled);
                state.widgets.add_child(up_id, up_disabled_id);

                // Store texture references on ScrollUpButton
                if let Some(up_btn) = state.widgets.get_mut(up_id) {
                    up_btn.children_keys.insert("Normal".to_string(), up_normal_id);
                    up_btn.children_keys.insert("Pushed".to_string(), up_pushed_id);
                    up_btn.children_keys.insert("Disabled".to_string(), up_disabled_id);
                }

                // Create ScrollDownButton child of ScrollBar
                let down_name = name.as_ref().map(|n| format!("{}ScrollBarScrollDownButton", n));
                let down_button = Frame::new(WidgetType::Button, down_name.clone(), Some(scrollbar_id));
                let down_id = down_button.id;
                state.widgets.register(down_button);
                state.widgets.add_child(scrollbar_id, down_id);

                // Create Normal texture for ScrollDownButton
                let down_normal = Frame::new(WidgetType::Texture, None, Some(down_id));
                let down_normal_id = down_normal.id;
                state.widgets.register(down_normal);
                state.widgets.add_child(down_id, down_normal_id);

                // Create Pushed texture for ScrollDownButton
                let down_pushed = Frame::new(WidgetType::Texture, None, Some(down_id));
                let down_pushed_id = down_pushed.id;
                state.widgets.register(down_pushed);
                state.widgets.add_child(down_id, down_pushed_id);

                // Create Disabled texture for ScrollDownButton
                let down_disabled = Frame::new(WidgetType::Texture, None, Some(down_id));
                let down_disabled_id = down_disabled.id;
                state.widgets.register(down_disabled);
                state.widgets.add_child(down_id, down_disabled_id);

                // Store texture references on ScrollDownButton
                if let Some(down_btn) = state.widgets.get_mut(down_id) {
                    down_btn.children_keys.insert("Normal".to_string(), down_normal_id);
                    down_btn.children_keys.insert("Pushed".to_string(), down_pushed_id);
                    down_btn.children_keys.insert("Disabled".to_string(), down_disabled_id);
                }

                // Store child references on frames
                if let Some(scrollbar_frame) = state.widgets.get_mut(scrollbar_id) {
                    scrollbar_frame.children_keys.insert("ScrollUpButton".to_string(), up_id);
                    scrollbar_frame.children_keys.insert("ScrollDownButton".to_string(), down_id);
                }
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("ScrollBar".to_string(), scrollbar_id);
                }
            }

            // PanelTabButtonTemplate creates Text FontString
            if tmpl.contains("PanelTabButton") || tmpl.contains("TabButtonTemplate") {
                // Create Text FontString child
                let text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let text_id = text_fs.id;
                state.widgets.register(text_fs);
                state.widgets.add_child(frame_id, text_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Text".to_string(), text_id);
                }
            }

            // DefaultPanelTemplate creates Bg texture and TitleText FontString
            if tmpl.contains("DefaultPanelTemplate") || tmpl.contains("BasicPanel") {
                // Create Bg texture child
                let bg_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let bg_id = bg_tex.id;
                state.widgets.register(bg_tex);
                state.widgets.add_child(frame_id, bg_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Bg".to_string(), bg_id);
                }

                // Create TitleText FontString child
                let title_text = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let title_id = title_text.id;
                state.widgets.register(title_text);
                state.widgets.add_child(frame_id, title_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("TitleText".to_string(), title_id);
                }
            }

            // ObjectiveTrackerContainerHeaderTemplate creates Text FontString, MinimizeButton, and Background
            if tmpl.contains("ObjectiveTrackerContainerHeader") || tmpl.contains("ObjectiveTrackerHeader") {
                // Create Text FontString child
                let text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let text_id = text_fs.id;
                state.widgets.register(text_fs);
                state.widgets.add_child(frame_id, text_id);

                // Create MinimizeButton child
                let min_button = Frame::new(WidgetType::Button, None, Some(frame_id));
                let min_id = min_button.id;
                state.widgets.register(min_button);
                state.widgets.add_child(frame_id, min_id);

                // Create Background texture child (used by WorldQuestTracker)
                let bg_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let bg_id = bg_tex.id;
                state.widgets.register(bg_tex);
                state.widgets.add_child(frame_id, bg_id);

                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Text".to_string(), text_id);
                    parent_frame.children_keys.insert("MinimizeButton".to_string(), min_id);
                    parent_frame.children_keys.insert("Background".to_string(), bg_id);
                }
            }

            // Tooltip templates create NineSlice child frame (required by SharedTooltipTemplates)
            if tmpl.contains("TooltipTemplate") || tmpl.contains("Tooltip") {
                let nine_slice = Frame::new(WidgetType::Frame, None, Some(frame_id));
                let nine_slice_id = nine_slice.id;
                state.widgets.register(nine_slice);
                state.widgets.add_child(frame_id, nine_slice_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame
                        .children_keys
                        .insert("NineSlice".to_string(), nine_slice_id);
                }
            }

            // GameTooltipTemplate - mark that we need to create TextLeftN/TextRightN FontStrings
            // This is handled after the main frame userdata is created
            if tmpl.contains("GameTooltipTemplate") {
                needs_tooltip_fontstrings = true;
            }

            // NOTE: PortraitFrameTemplate and ButtonFrameTemplate children (TitleContainer, NineSlice,
            // CloseButton, PortraitContainer) are now created by the XML template system via
            // instantiate_template_children in loader.rs. The parentKey handling there also
            // updates children_keys for SetTitle and other lookups.

            // NOTE: ButtonFrameTemplate handling is now done via proper template inheritance
            // in loader.rs (instantiate_template_children). The template registry stores
            // virtual frames and their children are automatically instantiated when a frame
            // inherits from them.

            // EditModeSystemSelectionTemplate creates a Label FontString (used by LibEditMode)
            // Also sets parent.Selection = self for addon access patterns
            if tmpl.contains("EditModeSystemSelection") {
                let label_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let label_id = label_fs.id;
                state.widgets.register(label_fs);
                state.widgets.add_child(frame_id, label_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Label".to_string(), label_id);
                }

                // Set the Selection key on the parent frame so addon code can access via parent.Selection
                if let Some(pid) = parent_id {
                    if let Some(actual_parent) = state.widgets.get_mut(pid) {
                        actual_parent.children_keys.insert("Selection".to_string(), frame_id);
                    }
                }
            }

            // SettingsCheckBoxControlTemplate creates Text and Checkbox children
            if tmpl.contains("SettingsCheckBoxControl") || tmpl.contains("SettingsCheckbox") {
                // Create Text FontString child
                let text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let text_id = text_fs.id;
                state.widgets.register(text_fs);
                state.widgets.add_child(frame_id, text_id);

                // Create Checkbox child
                let checkbox = Frame::new(WidgetType::CheckButton, None, Some(frame_id));
                let checkbox_id = checkbox.id;
                state.widgets.register(checkbox);
                state.widgets.add_child(frame_id, checkbox_id);

                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Text".to_string(), text_id);
                    parent_frame.children_keys.insert("Checkbox".to_string(), checkbox_id);
                }
            }

            // SettingsListTemplate creates Header and ScrollBox children (used by Settings UI)
            if tmpl.contains("SettingsListTemplate") || tmpl.contains("SettingsList") {
                // Create Header frame
                let header = Frame::new(WidgetType::Frame, None, Some(frame_id));
                let header_id = header.id;
                state.widgets.register(header);
                state.widgets.add_child(frame_id, header_id);

                // Create Header.DefaultsButton
                let defaults_btn = Frame::new(WidgetType::Button, None, Some(header_id));
                let defaults_btn_id = defaults_btn.id;
                state.widgets.register(defaults_btn);
                state.widgets.add_child(header_id, defaults_btn_id);

                // Create Header.Title FontString
                let title = Frame::new(WidgetType::FontString, None, Some(header_id));
                let title_id = title.id;
                state.widgets.register(title);
                state.widgets.add_child(header_id, title_id);

                // Store Header children keys
                if let Some(header_frame) = state.widgets.get_mut(header_id) {
                    header_frame.children_keys.insert("DefaultsButton".to_string(), defaults_btn_id);
                    header_frame.children_keys.insert("Title".to_string(), title_id);
                }

                // Create ScrollBox frame
                let scrollbox = Frame::new(WidgetType::Frame, None, Some(frame_id));
                let scrollbox_id = scrollbox.id;
                state.widgets.register(scrollbox);
                state.widgets.add_child(frame_id, scrollbox_id);

                // Create ScrollBox.ScrollTarget
                let scroll_target = Frame::new(WidgetType::Frame, None, Some(scrollbox_id));
                let scroll_target_id = scroll_target.id;
                state.widgets.register(scroll_target);
                state.widgets.add_child(scrollbox_id, scroll_target_id);

                if let Some(sb) = state.widgets.get_mut(scrollbox_id) {
                    sb.children_keys.insert("ScrollTarget".to_string(), scroll_target_id);
                }

                // Store children keys on main frame
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Header".to_string(), header_id);
                    parent_frame.children_keys.insert("ScrollBox".to_string(), scrollbox_id);
                }
            }

            // SecureGroupHeaderTemplate and SecureGroupPetHeaderTemplate create indexed child frames
            if tmpl == "SecureGroupHeaderTemplate" || tmpl == "SecureGroupPetHeaderTemplate" {
                // Create at least 5 child frames indexed numerically [1], [2], etc.
                for _i in 1..=5 {
                    let child = Frame::new(WidgetType::Button, None, Some(frame_id));
                    let child_id = child.id;
                    state.widgets.register(child);
                    state.widgets.add_child(frame_id, child_id);
                }
            }

            // PlumberSettingsPanelLayoutTemplate creates FrameContainer with nested child frames
            if tmpl == "PlumberSettingsPanelLayoutTemplate" {
                // Create FrameContainer
                let frame_container = Frame::new(WidgetType::Frame, None, Some(frame_id));
                let frame_container_id = frame_container.id;
                state.widgets.register(frame_container);
                state.widgets.add_child(frame_id, frame_container_id);

                // Create child frames inside FrameContainer
                let child_keys = ["LeftSection", "RightSection", "CentralSection", "SideTab",
                                  "TabButtonContainer", "ModuleTab", "ChangelogTab"];
                for key in child_keys {
                    let child = Frame::new(WidgetType::Frame, None, Some(frame_container_id));
                    let child_id = child.id;
                    state.widgets.register(child);
                    state.widgets.add_child(frame_container_id, child_id);
                    if let Some(fc) = state.widgets.get_mut(frame_container_id) {
                        fc.children_keys.insert(key.to_string(), child_id);
                    }
                }

                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("FrameContainer".to_string(), frame_container_id);
                }
            }

            // AddonListEntryTemplate creates Title, Status, Reload FontStrings, Enabled CheckButton, LoadAddonButton
            // Used by Blizzard_AddOnList for addon list entries
            if tmpl.contains("AddonListEntry") {
                // Create Title FontString (300x12)
                let mut title_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                title_fs.width = 300.0;
                title_fs.height = 16.0; // Taller for better visibility
                title_fs.font_size = 14.0;
                title_fs.text = Some("TestAddon".to_string()); // Placeholder text
                title_fs.text_color = crate::widget::Color::new(1.0, 0.78, 0.0, 1.0); // Gold color
                // Anchor Title at LEFT + 32px (leaving room for checkbox)
                title_fs.anchors.push(Anchor {
                    point: AnchorPoint::Left,
                    relative_to: None,
                    relative_to_id: Some(frame_id as usize),
                    relative_point: AnchorPoint::Left,
                    x_offset: 32.0,
                    y_offset: 0.0,
                });
                let title_id = title_fs.id;
                state.widgets.register(title_fs);
                state.widgets.add_child(frame_id, title_id);

                // Create Status FontString (anchored RIGHT)
                let mut status_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                status_fs.width = 100.0;
                status_fs.height = 12.0;
                status_fs.anchors.push(Anchor {
                    point: AnchorPoint::Right,
                    relative_to: None,
                    relative_to_id: Some(frame_id as usize),
                    relative_point: AnchorPoint::Right,
                    x_offset: 0.0,
                    y_offset: 0.0,
                });
                let status_id = status_fs.id;
                state.widgets.register(status_fs);
                state.widgets.add_child(frame_id, status_id);

                // Create Reload FontString (220x12, anchored RIGHT)
                let mut reload_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                reload_fs.width = 220.0;
                reload_fs.height = 12.0;
                reload_fs.visible = false; // Hidden by default
                reload_fs.anchors.push(Anchor {
                    point: AnchorPoint::Right,
                    relative_to: None,
                    relative_to_id: Some(frame_id as usize),
                    relative_point: AnchorPoint::Right,
                    x_offset: 0.0,
                    y_offset: 0.0,
                });
                let reload_id = reload_fs.id;
                state.widgets.register(reload_fs);
                state.widgets.add_child(frame_id, reload_id);

                // Create Enabled CheckButton (24x24, anchored LEFT)
                let mut enabled_cb = Frame::new(WidgetType::CheckButton, None, Some(frame_id));
                enabled_cb.width = 24.0;
                enabled_cb.height = 24.0;
                enabled_cb.anchors.push(Anchor {
                    point: AnchorPoint::Left,
                    relative_to: None,
                    relative_to_id: Some(frame_id as usize),
                    relative_point: AnchorPoint::Left,
                    x_offset: 4.0,
                    y_offset: 0.0,
                });
                let enabled_id = enabled_cb.id;
                state.widgets.register(enabled_cb);
                state.widgets.add_child(frame_id, enabled_id);

                // Create CheckedTexture for the CheckButton
                let mut checked_tex = Frame::new(WidgetType::Texture, None, Some(enabled_id));
                checked_tex.width = 24.0;
                checked_tex.height = 24.0;
                checked_tex.visible = false; // Hidden until SetChecked(true) is called
                // Set atlas texture to checkmark-minimal
                checked_tex.attributes.insert("__atlas".to_string(), AttributeValue::String("checkmark-minimal".to_string()));
                let checked_tex_id = checked_tex.id;
                state.widgets.register(checked_tex);
                state.widgets.add_child(enabled_id, checked_tex_id);
                if let Some(cb) = state.widgets.get_mut(enabled_id) {
                    cb.children_keys.insert("CheckedTexture".to_string(), checked_tex_id);
                }

                // Create LoadAddonButton (100x22, hidden by default)
                let mut load_btn = Frame::new(WidgetType::Button, None, Some(frame_id));
                load_btn.width = 100.0;
                load_btn.height = 22.0;
                load_btn.visible = false;
                let load_btn_id = load_btn.id;
                state.widgets.register(load_btn);
                state.widgets.add_child(frame_id, load_btn_id);

                // Store children keys
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Title".to_string(), title_id);
                    parent_frame.children_keys.insert("Status".to_string(), status_id);
                    parent_frame.children_keys.insert("Reload".to_string(), reload_id);
                    parent_frame.children_keys.insert("Enabled".to_string(), enabled_id);
                    parent_frame.children_keys.insert("LoadAddonButton".to_string(), load_btn_id);
                }
            }

            // AddonListCategoryTemplate creates Title FontString
            // Used by Blizzard_AddOnList for category headers
            if tmpl.contains("AddonListCategory") && !tmpl.contains("Entry") {
                // Create Title FontString
                let mut title_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                title_fs.width = 300.0;
                title_fs.height = 12.0;
                let title_id = title_fs.id;
                state.widgets.register(title_fs);
                state.widgets.add_child(frame_id, title_id);

                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Title".to_string(), title_id);
                }
            }
        }

        // Button widget types get NormalTexture, PushedTexture, HighlightTexture, DisabledTexture children
        if widget_type == WidgetType::Button || widget_type == WidgetType::CheckButton {
            // Create NormalTexture
            let normal_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let normal_id = normal_tex.id;
            state.widgets.register(normal_tex);
            state.widgets.add_child(frame_id, normal_id);

            // Create PushedTexture
            let pushed_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let pushed_id = pushed_tex.id;
            state.widgets.register(pushed_tex);
            state.widgets.add_child(frame_id, pushed_id);

            // Create HighlightTexture
            let highlight_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let highlight_id = highlight_tex.id;
            state.widgets.register(highlight_tex);
            state.widgets.add_child(frame_id, highlight_id);

            // Create DisabledTexture
            let disabled_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let disabled_id = disabled_tex.id;
            state.widgets.register(disabled_tex);
            state.widgets.add_child(frame_id, disabled_id);

            // Create Icon texture (for action buttons)
            let icon_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let icon_id = icon_tex.id;
            state.widgets.register(icon_tex);
            state.widgets.add_child(frame_id, icon_id);

            // Create IconOverlay texture (for action buttons)
            let icon_overlay_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let icon_overlay_id = icon_overlay_tex.id;
            state.widgets.register(icon_overlay_tex);
            state.widgets.add_child(frame_id, icon_overlay_id);

            // Create Border texture (for action buttons)
            let border_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let border_id = border_tex.id;
            state.widgets.register(border_tex);
            state.widgets.add_child(frame_id, border_id);

            // Store texture references as children_keys
            if let Some(btn) = state.widgets.get_mut(frame_id) {
                btn.children_keys.insert("NormalTexture".to_string(), normal_id);
                btn.children_keys.insert("PushedTexture".to_string(), pushed_id);
                btn.children_keys.insert("HighlightTexture".to_string(), highlight_id);
                btn.children_keys.insert("DisabledTexture".to_string(), disabled_id);
                btn.children_keys.insert("Icon".to_string(), icon_id);
                btn.children_keys.insert("IconOverlay".to_string(), icon_overlay_id);
                btn.children_keys.insert("Border".to_string(), border_id);
            }
        }

        // Slider widget types get Low, High, Text fontstrings and ThumbTexture
        if widget_type == WidgetType::Slider {
            // Create Low fontstring (min value label)
            let low_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            let low_id = low_fs.id;
            state.widgets.register(low_fs);
            state.widgets.add_child(frame_id, low_id);

            // Create High fontstring (max value label)
            let high_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            let high_id = high_fs.id;
            state.widgets.register(high_fs);
            state.widgets.add_child(frame_id, high_id);

            // Create Text fontstring (current value label)
            let text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            let text_id = text_fs.id;
            state.widgets.register(text_fs);
            state.widgets.add_child(frame_id, text_id);

            // Create ThumbTexture
            let thumb_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let thumb_id = thumb_tex.id;
            state.widgets.register(thumb_tex);
            state.widgets.add_child(frame_id, thumb_id);

            // Store references as children_keys
            if let Some(slider) = state.widgets.get_mut(frame_id) {
                slider.children_keys.insert("Low".to_string(), low_id);
                slider.children_keys.insert("High".to_string(), high_id);
                slider.children_keys.insert("Text".to_string(), text_id);
                slider.children_keys.insert("ThumbTexture".to_string(), thumb_id);
            }
        }

        drop(state); // Release borrow before creating userdata

        // Create userdata handle
        let handle = FrameHandle {
            id: frame_id,
            state: Rc::clone(&state_clone),
        };

        let ud = lua.create_userdata(handle)?;

        // Store reference in globals if named
        if let Some(ref n) = name {
            lua.globals().set(n.as_str(), ud.clone())?;
        }

        // Store reference for event dispatch
        let frame_key = format!("__frame_{}", frame_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        // Create TextLeftN/TextRightN FontStrings for GameTooltipTemplate
        if needs_tooltip_fontstrings {
            if let Some(ref frame_name) = name {
                // Create TextLeft1..4 and TextRight1..4 as globals using Lua
                let lua_code = format!(
                    r#"
                    local tooltip = {}
                    for i = 1, 4 do
                        local leftName = "{}TextLeft" .. i
                        local rightName = "{}TextRight" .. i
                        _G[leftName] = tooltip:CreateFontString(leftName, "ARTWORK", "GameTooltipText")
                        _G[rightName] = tooltip:CreateFontString(rightName, "ARTWORK", "GameTooltipText")
                    end
                    "#,
                    frame_name, frame_name, frame_name
                );
                let _ = lua.load(&lua_code).exec();
            }
        }

        // Register HybridScrollBarTemplate children as globals
        if needs_hybrid_scroll_globals {
            if let Some(ref frame_name) = name {
                // Register scroll buttons and thumb texture as globals using Lua
                let lua_code = format!(
                    r#"
                    local frame = {}
                    local frameName = "{}"
                    _G[frameName .. "ScrollUpButton"] = frame.ScrollUpButton
                    _G[frameName .. "ScrollDownButton"] = frame.ScrollDownButton
                    _G[frameName .. "ThumbTexture"] = frame.ThumbTexture
                    "#,
                    frame_name, frame_name
                );
                let _ = lua.load(&lua_code).exec();
            }
        }

        Ok(ud)
    })?;
    globals.set("CreateFrame", create_frame)?;

    // CreateFont(name) - Creates a named font object that can be used with SetFontObject
    let create_font = lua.create_function(|lua, name: Option<String>| {
        // Create a font object table with font properties and methods
        let font = lua.create_table()?;

        // Internal state for the font
        font.set("__fontPath", "Fonts\\FRIZQT__.TTF")?;
        font.set("__fontHeight", 12.0)?;
        font.set("__fontFlags", "")?;
        font.set("__textColorR", 1.0)?;
        font.set("__textColorG", 1.0)?;
        font.set("__textColorB", 1.0)?;
        font.set("__textColorA", 1.0)?;
        font.set("__shadowColorR", 0.0)?;
        font.set("__shadowColorG", 0.0)?;
        font.set("__shadowColorB", 0.0)?;
        font.set("__shadowColorA", 0.0)?;
        font.set("__shadowOffsetX", 0.0)?;
        font.set("__shadowOffsetY", 0.0)?;
        font.set("__justifyH", "CENTER")?;
        font.set("__justifyV", "MIDDLE")?;

        // SetFont(fontPath, height, flags)
        font.set(
            "SetFont",
            lua.create_function(
                |_, (this, path, height, flags): (mlua::Table, String, f64, Option<String>)| {
                    this.set("__fontPath", path)?;
                    this.set("__fontHeight", height)?;
                    this.set("__fontFlags", flags.unwrap_or_default())?;
                    Ok(())
                },
            )?,
        )?;

        // GetFont() -> fontPath, height, flags
        font.set(
            "GetFont",
            lua.create_function(|_, this: mlua::Table| {
                let path: String = this.get("__fontPath")?;
                let height: f64 = this.get("__fontHeight")?;
                let flags: String = this.get("__fontFlags")?;
                Ok((path, height, flags))
            })?,
        )?;

        // SetTextColor(r, g, b, a)
        font.set(
            "SetTextColor",
            lua.create_function(
                |_,
                 (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                    this.set("__textColorR", r)?;
                    this.set("__textColorG", g)?;
                    this.set("__textColorB", b)?;
                    this.set("__textColorA", a.unwrap_or(1.0))?;
                    Ok(())
                },
            )?,
        )?;

        // GetTextColor() -> r, g, b, a
        font.set(
            "GetTextColor",
            lua.create_function(|_, this: mlua::Table| {
                let r: f64 = this.get("__textColorR")?;
                let g: f64 = this.get("__textColorG")?;
                let b: f64 = this.get("__textColorB")?;
                let a: f64 = this.get("__textColorA")?;
                Ok((r, g, b, a))
            })?,
        )?;

        // SetShadowColor(r, g, b, a)
        font.set(
            "SetShadowColor",
            lua.create_function(
                |_,
                 (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                    this.set("__shadowColorR", r)?;
                    this.set("__shadowColorG", g)?;
                    this.set("__shadowColorB", b)?;
                    this.set("__shadowColorA", a.unwrap_or(1.0))?;
                    Ok(())
                },
            )?,
        )?;

        // GetShadowColor() -> r, g, b, a
        font.set(
            "GetShadowColor",
            lua.create_function(|_, this: mlua::Table| {
                let r: f64 = this.get("__shadowColorR")?;
                let g: f64 = this.get("__shadowColorG")?;
                let b: f64 = this.get("__shadowColorB")?;
                let a: f64 = this.get("__shadowColorA")?;
                Ok((r, g, b, a))
            })?,
        )?;

        // SetShadowOffset(x, y)
        font.set(
            "SetShadowOffset",
            lua.create_function(|_, (this, x, y): (mlua::Table, f64, f64)| {
                this.set("__shadowOffsetX", x)?;
                this.set("__shadowOffsetY", y)?;
                Ok(())
            })?,
        )?;

        // GetShadowOffset() -> x, y
        font.set(
            "GetShadowOffset",
            lua.create_function(|_, this: mlua::Table| {
                let x: f64 = this.get("__shadowOffsetX")?;
                let y: f64 = this.get("__shadowOffsetY")?;
                Ok((x, y))
            })?,
        )?;

        // SetJustifyH(justify)
        font.set(
            "SetJustifyH",
            lua.create_function(|_, (this, justify): (mlua::Table, String)| {
                this.set("__justifyH", justify)?;
                Ok(())
            })?,
        )?;

        // GetJustifyH() -> justify
        font.set(
            "GetJustifyH",
            lua.create_function(|_, this: mlua::Table| {
                let justify: String = this.get("__justifyH")?;
                Ok(justify)
            })?,
        )?;

        // SetJustifyV(justify)
        font.set(
            "SetJustifyV",
            lua.create_function(|_, (this, justify): (mlua::Table, String)| {
                this.set("__justifyV", justify)?;
                Ok(())
            })?,
        )?;

        // GetJustifyV() -> justify
        font.set(
            "GetJustifyV",
            lua.create_function(|_, this: mlua::Table| {
                let justify: String = this.get("__justifyV")?;
                Ok(justify)
            })?,
        )?;

        // SetSpacing(spacing)
        font.set(
            "SetSpacing",
            lua.create_function(|_, (this, spacing): (mlua::Table, f64)| {
                this.set("__spacing", spacing)?;
                Ok(())
            })?,
        )?;

        // GetSpacing() -> spacing
        font.set(
            "GetSpacing",
            lua.create_function(|_, this: mlua::Table| {
                let spacing: f64 = this.get("__spacing").unwrap_or(0.0);
                Ok(spacing)
            })?,
        )?;

        // CopyFontObject(fontObject or fontName)
        font.set(
            "CopyFontObject",
            lua.create_function(|lua, (this, src): (mlua::Table, Value)| {
                // If src is a string, look up the font object by name
                let src_table: Option<mlua::Table> = match src {
                    Value::String(name) => {
                        let name_str = name.to_string_lossy().to_string();
                        lua.globals().get::<Option<mlua::Table>>(name_str).ok().flatten()
                    }
                    Value::Table(t) => Some(t),
                    _ => None,
                };

                if let Some(src) = src_table {
                    // Copy all font properties from src to this
                    if let Ok(v) = src.get::<String>("__fontPath") {
                        this.set("__fontPath", v)?;
                    }
                    if let Ok(v) = src.get::<f64>("__fontHeight") {
                        this.set("__fontHeight", v)?;
                    }
                    if let Ok(v) = src.get::<String>("__fontFlags") {
                        this.set("__fontFlags", v)?;
                    }
                    for key in &[
                        "__textColorR",
                        "__textColorG",
                        "__textColorB",
                        "__textColorA",
                        "__shadowColorR",
                        "__shadowColorG",
                        "__shadowColorB",
                        "__shadowColorA",
                        "__shadowOffsetX",
                        "__shadowOffsetY",
                    ] {
                        if let Ok(v) = src.get::<f64>(*key) {
                            this.set(*key, v)?;
                        }
                    }
                    if let Ok(v) = src.get::<String>("__justifyH") {
                        this.set("__justifyH", v)?;
                    }
                    if let Ok(v) = src.get::<String>("__justifyV") {
                        this.set("__justifyV", v)?;
                    }
                }
                Ok(())
            })?,
        )?;

        // GetName() -> name
        font.set("__name", name.clone())?;
        font.set(
            "GetName",
            lua.create_function(|_, this: mlua::Table| {
                let name: Option<String> = this.get("__name").ok();
                Ok(name)
            })?,
        )?;

        // GetFontObjectForAlphabet(alphabet) -> returns self (font localization stub)
        // In WoW this returns a different font for different alphabets (Latin, Cyrillic, etc.)
        // For simulation, just return self
        font.set(
            "GetFontObjectForAlphabet",
            lua.create_function(|_, this: mlua::Table| Ok(this))?,
        )?;

        // Register the font globally if it has a name
        if let Some(ref n) = name {
            lua.globals().set(n.as_str(), font.clone())?;
        }

        Ok(font)
    })?;
    globals.set("CreateFont", create_font)?;

    // GetFonts() - returns a list of registered font names
    let get_fonts = lua.create_function(|lua, ()| {
        // Return an empty table - in simulation we don't track font objects
        lua.create_table()
    })?;
    globals.set("GetFonts", get_fonts)?;

    // GetFontInfo(fontName or fontObject) - returns font information for a registered font
    let get_font_info = lua.create_function(|lua, font_input: Value| {
        // Return a table with font information
        let info = lua.create_table()?;

        match font_input {
            Value::String(name) => {
                let name_str = name.to_string_lossy().to_string();
                info.set("name", name_str.clone())?;
                // Try to get the font object from globals
                if let Ok(font_obj) = lua.globals().get::<mlua::Table>(name_str) {
                    let height: f64 = font_obj.get("__fontHeight").unwrap_or(12.0);
                    let outline: String = font_obj.get("__fontFlags").unwrap_or_default();
                    info.set("height", height)?;
                    info.set("outline", outline)?;
                    // Add color info
                    let color = lua.create_table()?;
                    color.set("r", font_obj.get::<f64>("__textColorR").unwrap_or(1.0))?;
                    color.set("g", font_obj.get::<f64>("__textColorG").unwrap_or(1.0))?;
                    color.set("b", font_obj.get::<f64>("__textColorB").unwrap_or(1.0))?;
                    color.set("a", font_obj.get::<f64>("__textColorA").unwrap_or(1.0))?;
                    info.set("color", color)?;
                } else {
                    info.set("height", 12.0)?;
                    info.set("outline", "")?;
                }
            }
            Value::Table(font_obj) => {
                let name: String = font_obj.get("__name").unwrap_or_default();
                let height: f64 = font_obj.get("__fontHeight").unwrap_or(12.0);
                let outline: String = font_obj.get("__fontFlags").unwrap_or_default();
                info.set("name", name)?;
                info.set("height", height)?;
                info.set("outline", outline)?;
                // Add color info
                let color = lua.create_table()?;
                color.set("r", font_obj.get::<f64>("__textColorR").unwrap_or(1.0))?;
                color.set("g", font_obj.get::<f64>("__textColorG").unwrap_or(1.0))?;
                color.set("b", font_obj.get::<f64>("__textColorB").unwrap_or(1.0))?;
                color.set("a", font_obj.get::<f64>("__textColorA").unwrap_or(1.0))?;
                info.set("color", color)?;
            }
            _ => {
                info.set("name", "")?;
                info.set("height", 12.0)?;
                info.set("outline", "")?;
            }
        }
        Ok(info)
    })?;
    globals.set("GetFontInfo", get_font_info)?;

    // CreateFontFamily(name, members) - creates a font family with different fonts for different alphabets
    // members is an array of {alphabet, file, height, flags} tables
    let create_font_family = lua.create_function(|lua, (name, members): (String, mlua::Table)| {
        // Create a font object similar to CreateFont
        let font = lua.create_table()?;
        font.set("__name", name.clone())?;
        font.set("__fontPath", "Fonts\\FRIZQT__.TTF")?;
        font.set("__fontHeight", 12.0)?;
        font.set("__fontFlags", "")?;
        font.set("__textColorR", 1.0)?;
        font.set("__textColorG", 1.0)?;
        font.set("__textColorB", 1.0)?;
        font.set("__textColorA", 1.0)?;
        font.set("__shadowColorR", 0.0)?;
        font.set("__shadowColorG", 0.0)?;
        font.set("__shadowColorB", 0.0)?;
        font.set("__shadowColorA", 0.0)?;
        font.set("__shadowOffsetX", 0.0)?;
        font.set("__shadowOffsetY", 0.0)?;

        // Try to get font info from first member
        if let Ok(first_member) = members.get::<mlua::Table>(1) {
            if let Ok(file) = first_member.get::<String>("file") {
                font.set("__fontPath", file)?;
            }
            if let Ok(height) = first_member.get::<f64>("height") {
                font.set("__fontHeight", height)?;
            }
            if let Ok(flags) = first_member.get::<String>("flags") {
                font.set("__fontFlags", flags)?;
            }
        }

        // SetFont(path, height, flags)
        font.set("SetFont", lua.create_function(
            |_, (this, path, height, flags): (mlua::Table, String, f64, Option<String>)| {
                this.set("__fontPath", path)?;
                this.set("__fontHeight", height)?;
                this.set("__fontFlags", flags.unwrap_or_default())?;
                Ok(())
            })?)?;

        // GetFont() -> path, height, flags
        font.set("GetFont", lua.create_function(|_, this: mlua::Table| {
            let path: String = this.get("__fontPath")?;
            let height: f64 = this.get("__fontHeight")?;
            let flags: String = this.get("__fontFlags")?;
            Ok((path, height, flags))
        })?)?;

        // SetTextColor(r, g, b, a)
        font.set("SetTextColor", lua.create_function(
            |_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                this.set("__textColorR", r)?;
                this.set("__textColorG", g)?;
                this.set("__textColorB", b)?;
                this.set("__textColorA", a.unwrap_or(1.0))?;
                Ok(())
            })?)?;

        // GetTextColor() -> r, g, b, a
        font.set("GetTextColor", lua.create_function(|_, this: mlua::Table| {
            let r: f64 = this.get("__textColorR")?;
            let g: f64 = this.get("__textColorG")?;
            let b: f64 = this.get("__textColorB")?;
            let a: f64 = this.get("__textColorA")?;
            Ok((r, g, b, a))
        })?)?;

        // SetShadowColor(r, g, b, a)
        font.set("SetShadowColor", lua.create_function(
            |_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                this.set("__shadowColorR", r)?;
                this.set("__shadowColorG", g)?;
                this.set("__shadowColorB", b)?;
                this.set("__shadowColorA", a.unwrap_or(1.0))?;
                Ok(())
            })?)?;

        // GetShadowColor() -> r, g, b, a
        font.set("GetShadowColor", lua.create_function(|_, this: mlua::Table| {
            let r: f64 = this.get("__shadowColorR")?;
            let g: f64 = this.get("__shadowColorG")?;
            let b: f64 = this.get("__shadowColorB")?;
            let a: f64 = this.get("__shadowColorA")?;
            Ok((r, g, b, a))
        })?)?;

        // SetShadowOffset(x, y)
        font.set("SetShadowOffset", lua.create_function(
            |_, (this, x, y): (mlua::Table, f64, f64)| {
                this.set("__shadowOffsetX", x)?;
                this.set("__shadowOffsetY", y)?;
                Ok(())
            })?)?;

        // GetShadowOffset() -> x, y
        font.set("GetShadowOffset", lua.create_function(|_, this: mlua::Table| {
            let x: f64 = this.get("__shadowOffsetX")?;
            let y: f64 = this.get("__shadowOffsetY")?;
            Ok((x, y))
        })?)?;

        // GetName() -> name
        font.set("GetName", lua.create_function(|_, this: mlua::Table| {
            let name: Option<String> = this.get("__name").ok();
            Ok(name)
        })?)?;

        // GetFontObjectForAlphabet(alphabet) -> returns self
        font.set("GetFontObjectForAlphabet", lua.create_function(|_, this: mlua::Table| {
            Ok(this)
        })?)?;

        // CopyFontObject(fontObject or fontName)
        font.set("CopyFontObject", lua.create_function(|lua, (this, src): (mlua::Table, Value)| {
            let src_table: Option<mlua::Table> = match src {
                Value::String(s) => lua.globals().get::<Option<mlua::Table>>(s.to_string_lossy().to_string()).ok().flatten(),
                Value::Table(t) => Some(t),
                _ => None,
            };
            if let Some(src) = src_table {
                if let Ok(v) = src.get::<String>("__fontPath") { this.set("__fontPath", v)?; }
                if let Ok(v) = src.get::<f64>("__fontHeight") { this.set("__fontHeight", v)?; }
                if let Ok(v) = src.get::<String>("__fontFlags") { this.set("__fontFlags", v)?; }
                for key in &["__textColorR", "__textColorG", "__textColorB", "__textColorA",
                             "__shadowColorR", "__shadowColorG", "__shadowColorB", "__shadowColorA",
                             "__shadowOffsetX", "__shadowOffsetY"] {
                    if let Ok(v) = src.get::<f64>(*key) { this.set(*key, v)?; }
                }
            }
            Ok(())
        })?)?;

        // Register globally
        lua.globals().set(name.as_str(), font.clone())?;
        Ok(font)
    })?;
    globals.set("CreateFontFamily", create_font_family)?;

    // CreateTexturePool(parent, layer, subLayer, textureTemplate, resetterFunc)
    // Creates a pool for managing reusable textures
    let create_texture_pool_state = Rc::clone(&state);
    let create_texture_pool = lua.create_function(move |lua, args: mlua::MultiValue| {
        let _state = create_texture_pool_state.borrow();
        let _args: Vec<Value> = args.into_iter().collect();
        // Create a simple pool table with Acquire/Release methods
        let pool = lua.create_table()?;
        let pool_storage = lua.create_table()?;
        pool.set("__storage", pool_storage)?;
        pool.set("__active", lua.create_table()?)?;
        pool.set(
            "Acquire",
            lua.create_function(|lua, this: mlua::Table| {
                // Return a new texture-like table
                let texture = lua.create_table()?;
                texture.set("SetTexture", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetTexCoord", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetVertexColor", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetBlendMode", lua.create_function(|_, _: String| Ok(()))?)?;
                texture.set("SetDrawLayer", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetAllPoints", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetPoint", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("ClearAllPoints", lua.create_function(|_, ()| Ok(()))?)?;
                texture.set("SetAlpha", lua.create_function(|_, _: f64| Ok(()))?)?;
                texture.set("SetSize", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("Show", lua.create_function(|_, ()| Ok(()))?)?;
                texture.set("Hide", lua.create_function(|_, ()| Ok(()))?)?;
                texture.set("SetParent", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                // Track in active list
                let active: mlua::Table = this.get("__active")?;
                active.set(active.raw_len() + 1, texture.clone())?;
                Ok(texture)
            })?,
        )?;
        pool.set(
            "Release",
            lua.create_function(|_, (_this, _texture): (mlua::Table, mlua::Table)| Ok(()))?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, _this: mlua::Table| Ok(()))?,
        )?;
        Ok(pool)
    })?;
    globals.set("CreateTexturePool", create_texture_pool)?;

    // CreateFramePool(frameType, parent, template, resetterFunc, forbidden)
    // Creates a pool for managing reusable frames
    let create_frame_pool = lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();
        let frame_type: String = args.get(0)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Frame".to_string());
        // Parent can be nil, a string (global name), or a frame object
        let parent_val = args.get(1).cloned().unwrap_or(Value::Nil);
        let template: Option<String> = args.get(2)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let pool = lua.create_table()?;
        pool.set("__active", lua.create_table()?)?;
        pool.set("__inactive", lua.create_table()?)?;
        pool.set("__frame_type", frame_type.clone())?;
        pool.set("__parent", parent_val.clone())?;
        if let Some(ref tmpl) = template {
            pool.set("__template", tmpl.clone())?;
        }
        pool.set(
            "Acquire",
            lua.create_function(move |lua, this: mlua::Table| {
                // First check inactive pool for a frame to reuse
                let inactive: mlua::Table = this.get("__inactive")?;
                let inactive_len = inactive.raw_len();
                if inactive_len > 0 {
                    // Reuse an existing frame
                    let frame: Value = inactive.get(inactive_len)?;
                    inactive.set(inactive_len, Value::Nil)?;
                    let active: mlua::Table = this.get("__active")?;
                    active.set(active.raw_len() + 1, frame.clone())?;
                    return Ok((frame, false)); // false = not new
                }

                // Create a new frame via CreateFrame
                let create_frame: mlua::Function = lua.globals().get("CreateFrame")?;
                let frame_type: String = this.get("__frame_type")?;
                let parent: Value = this.get("__parent")?;
                let template: Option<String> = this.get("__template").ok();

                let frame = if let Some(tmpl) = template {
                    create_frame.call::<Value>((frame_type, Value::Nil, parent, tmpl))?
                } else {
                    create_frame.call::<Value>((frame_type, Value::Nil, parent))?
                };

                let active: mlua::Table = this.get("__active")?;
                active.set(active.raw_len() + 1, frame.clone())?;
                Ok((frame, true)) // true = new frame
            })?,
        )?;
        pool.set(
            "Release",
            lua.create_function(|_, (this, frame): (mlua::Table, Value)| {
                // Move from active to inactive
                let inactive: mlua::Table = this.get("__inactive")?;
                inactive.set(inactive.raw_len() + 1, frame)?;
                Ok(())
            })?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, this: mlua::Table| {
                let active: mlua::Table = this.get("__active")?;
                let inactive: mlua::Table = this.get("__inactive")?;
                // Move all active to inactive
                for pair in active.pairs::<i64, Value>() {
                    let (_, frame) = pair?;
                    inactive.set(inactive.raw_len() + 1, frame)?;
                }
                // Clear active
                for i in 1..=active.raw_len() {
                    active.set(i, Value::Nil)?;
                }
                Ok(())
            })?,
        )?;
        pool.set(
            "GetNumActive",
            lua.create_function(|_, this: mlua::Table| {
                let active: mlua::Table = this.get("__active")?;
                Ok(active.raw_len())
            })?,
        )?;
        pool.set(
            "EnumerateActive",
            lua.create_function(|lua, this: mlua::Table| {
                let active: mlua::Table = this.get("__active")?;
                let iter_state = lua.create_table()?;
                iter_state.set("tbl", active)?;
                iter_state.set("idx", 0i64)?;
                let iter_fn = lua.create_function(|_, state: mlua::Table| {
                    let tbl: mlua::Table = state.get("tbl")?;
                    let idx: i64 = state.get("idx")?;
                    let next_idx = idx + 1;
                    if next_idx <= tbl.raw_len() as i64 {
                        state.set("idx", next_idx)?;
                        let val: Value = tbl.get(next_idx)?;
                        Ok((Some(val), Value::Nil))
                    } else {
                        Ok((None, Value::Nil))
                    }
                })?;
                Ok((iter_fn, iter_state, Value::Nil))
            })?,
        )?;
        Ok(pool)
    })?;
    globals.set("CreateFramePool", create_frame_pool)?;

    // CreateObjectPool(creatorFunc, resetterFunc) - generic object pool
    let create_object_pool = lua.create_function(|lua, (creator_func, _resetter_func): (mlua::Function, Option<mlua::Function>)| {
        let pool = lua.create_table()?;
        pool.set("__creator", creator_func.clone())?;
        pool.set("__active", lua.create_table()?)?;
        pool.set("__inactive", lua.create_table()?)?;
        pool.set(
            "Acquire",
            lua.create_function(|_lua, this: mlua::Table| {
                let creator: mlua::Function = this.get("__creator")?;
                let obj = creator.call::<Value>(())?;
                let active: mlua::Table = this.get("__active")?;
                active.set(active.raw_len() + 1, obj.clone())?;
                Ok((obj, true)) // Return object and isNew flag
            })?,
        )?;
        pool.set(
            "Release",
            lua.create_function(|_, (_this, _obj): (mlua::Table, Value)| Ok(()))?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, _this: mlua::Table| Ok(()))?,
        )?;
        pool.set(
            "GetNumActive",
            lua.create_function(|_, this: mlua::Table| {
                let active: mlua::Table = this.get("__active")?;
                Ok(active.raw_len())
            })?,
        )?;
        pool.set(
            "EnumerateActive",
            lua.create_function(|lua, _this: mlua::Table| {
                // Return an iterator function (simple stub)
                lua.create_function(|_, ()| Ok(Value::Nil))
            })?,
        )?;
        Ok(pool)
    })?;
    globals.set("CreateObjectPool", create_object_pool)?;

    // UIParent reference
    let ui_parent_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("UIParent").unwrap()
    };
    let ui_parent = lua.create_userdata(FrameHandle {
        id: ui_parent_id,
        state: Rc::clone(&state),
    })?;
    globals.set("UIParent", ui_parent)?;

    // UIPanelWindows - registry for UI panel positioning/behavior
    globals.set("UIPanelWindows", lua.create_table()?)?;

    // WorldFrame reference (3D world rendering frame)
    let world_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("WorldFrame").unwrap()
    };
    let world_frame = lua.create_userdata(FrameHandle {
        id: world_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("WorldFrame", world_frame)?;

    // Minimap reference (built-in UI element)
    let minimap_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("Minimap").unwrap()
    };
    let minimap = lua.create_userdata(FrameHandle {
        id: minimap_id,
        state: Rc::clone(&state),
    })?;
    globals.set("Minimap", minimap)?;

    // DEFAULT_CHAT_FRAME reference (main chat window)
    let default_chat_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("DEFAULT_CHAT_FRAME").unwrap()
    };
    let default_chat_frame = lua.create_userdata(FrameHandle {
        id: default_chat_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("DEFAULT_CHAT_FRAME", default_chat_frame)?;

    // ChatFrame1 reference (same as DEFAULT_CHAT_FRAME)
    let chat_frame1_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ChatFrame1").unwrap()
    };
    let chat_frame1 = lua.create_userdata(FrameHandle {
        id: chat_frame1_id,
        state: Rc::clone(&state),
    })?;
    globals.set("ChatFrame1", chat_frame1)?;

    // ChatTypeGroup - maps chat type groups to arrays of chat message types
    let chat_type_group = lua.create_table()?;
    // System messages
    let system_group = lua.create_table()?;
    system_group.set(1, "SYSTEM")?;
    system_group.set(2, "ERROR")?;
    system_group.set(3, "IGNORED")?;
    system_group.set(4, "CHANNEL_NOTICE")?;
    system_group.set(5, "CHANNEL_NOTICE_USER")?;
    chat_type_group.set("SYSTEM", system_group)?;
    // Say messages
    let say_group = lua.create_table()?;
    say_group.set(1, "SAY")?;
    chat_type_group.set("SAY", say_group)?;
    // Yell messages
    let yell_group = lua.create_table()?;
    yell_group.set(1, "YELL")?;
    chat_type_group.set("YELL", yell_group)?;
    // Whisper messages
    let whisper_group = lua.create_table()?;
    whisper_group.set(1, "WHISPER")?;
    whisper_group.set(2, "WHISPER_INFORM")?;
    chat_type_group.set("WHISPER", whisper_group)?;
    // Party messages
    let party_group = lua.create_table()?;
    party_group.set(1, "PARTY")?;
    party_group.set(2, "PARTY_LEADER")?;
    chat_type_group.set("PARTY", party_group)?;
    // Raid messages
    let raid_group = lua.create_table()?;
    raid_group.set(1, "RAID")?;
    raid_group.set(2, "RAID_LEADER")?;
    raid_group.set(3, "RAID_WARNING")?;
    chat_type_group.set("RAID", raid_group)?;
    // Guild messages
    let guild_group = lua.create_table()?;
    guild_group.set(1, "GUILD")?;
    guild_group.set(2, "OFFICER")?;
    chat_type_group.set("GUILD", guild_group)?;
    // Emote messages
    let emote_group = lua.create_table()?;
    emote_group.set(1, "EMOTE")?;
    emote_group.set(2, "TEXT_EMOTE")?;
    chat_type_group.set("EMOTE", emote_group)?;
    // Channel messages
    let channel_group = lua.create_table()?;
    channel_group.set(1, "CHANNEL")?;
    chat_type_group.set("CHANNEL", channel_group)?;
    // Instance messages
    let instance_group = lua.create_table()?;
    instance_group.set(1, "INSTANCE_CHAT")?;
    instance_group.set(2, "INSTANCE_CHAT_LEADER")?;
    chat_type_group.set("INSTANCE_CHAT", instance_group)?;
    // BattleNet messages
    let bn_group = lua.create_table()?;
    bn_group.set(1, "BN_WHISPER")?;
    bn_group.set(2, "BN_WHISPER_INFORM")?;
    bn_group.set(3, "BN_CONVERSATION")?;
    chat_type_group.set("BN_WHISPER", bn_group)?;
    globals.set("ChatTypeGroup", chat_type_group)?;

    // ChatFrameUtil - utility functions for chat frames
    let chat_frame_util = lua.create_table()?;
    chat_frame_util.set(
        "ProcessMessageEventFilters",
        lua.create_function(|_, (_, event, args): (Value, String, mlua::Variadic<Value>)| {
            // Return the event args unchanged - no filtering in sim
            Ok((false, event, args))
        })?,
    )?;
    chat_frame_util.set(
        "GetChatWindowName",
        lua.create_function(|_, frame_id: i32| {
            Ok(format!("Chat Window {}", frame_id))
        })?,
    )?;
    globals.set("ChatFrameUtil", chat_frame_util)?;

    // EventToastManagerFrame reference (event toast/notification UI)
    let event_toast_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("EventToastManagerFrame").unwrap()
    };
    let event_toast_frame = lua.create_userdata(FrameHandle {
        id: event_toast_id,
        state: Rc::clone(&state),
    })?;
    globals.set("EventToastManagerFrame", event_toast_frame)?;

    // EditModeManagerFrame reference (Edit Mode UI manager)
    let edit_mode_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("EditModeManagerFrame").unwrap()
    };
    let edit_mode_frame = lua.create_userdata(FrameHandle {
        id: edit_mode_id,
        state: Rc::clone(&state),
    })?;
    globals.set("EditModeManagerFrame", edit_mode_frame)?;

    // RolePollPopup reference (role selection popup for groups)
    let role_poll_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("RolePollPopup").unwrap()
    };
    let role_poll_popup = lua.create_userdata(FrameHandle {
        id: role_poll_id,
        state: Rc::clone(&state),
    })?;
    globals.set("RolePollPopup", role_poll_popup)?;

    // TimerTracker reference (displays dungeon/raid instance timers)
    let timer_tracker_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("TimerTracker").unwrap()
    };
    let timer_tracker = lua.create_userdata(FrameHandle {
        id: timer_tracker_id,
        state: Rc::clone(&state),
    })?;
    globals.set("TimerTracker", timer_tracker)?;

    // WorldMapFrame reference (world map display frame)
    let world_map_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("WorldMapFrame").unwrap()
    };
    let world_map_frame = lua.create_userdata(FrameHandle {
        id: world_map_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("WorldMapFrame", world_map_frame)?;

    // Set up WorldMapFrame.pinPools table (used by HereBeDragons map pins)
    let frame_fields: mlua::Table = lua
        .globals()
        .get::<mlua::Table>("__frame_fields")
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.globals().set("__frame_fields", t.clone()).unwrap();
            t
        });
    let wm_fields: mlua::Table = frame_fields
        .get::<mlua::Table>(world_map_frame_id)
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            frame_fields.set(world_map_frame_id, t.clone()).unwrap();
            t
        });
    wm_fields.set("pinPools", lua.create_table()?)?;
    // Add overlayFrames (used by WorldQuestTracker)
    wm_fields.set("overlayFrames", lua.create_table()?)?;

    // PlayerFrame reference (player unit frame)
    let player_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("PlayerFrame").unwrap()
    };
    let player_frame = lua.create_userdata(FrameHandle {
        id: player_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("PlayerFrame", player_frame)?;

    // TargetFrame reference (target unit frame)
    let target_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("TargetFrame").unwrap()
    };
    let target_frame = lua.create_userdata(FrameHandle {
        id: target_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("TargetFrame", target_frame)?;

    // FocusFrame reference (focus target unit frame)
    let focus_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("FocusFrame").unwrap()
    };
    let focus_frame = lua.create_userdata(FrameHandle {
        id: focus_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("FocusFrame", focus_frame)?;

    // FocusFrameSpellBar reference (focus cast bar)
    let focus_spell_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("FocusFrameSpellBar").unwrap()
    };
    let focus_spell_bar = lua.create_userdata(FrameHandle {
        id: focus_spell_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("FocusFrameSpellBar", focus_spell_bar)?;

    // BuffFrame reference (player buff display)
    let buff_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("BuffFrame").unwrap()
    };
    let buff_frame = lua.create_userdata(FrameHandle {
        id: buff_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("BuffFrame", buff_frame)?;

    // Set iconScale on BuffFrame.AuraContainer
    {
        let aura_container_id = {
            let state = state.borrow();
            state.widgets.get_id_by_name("BuffFrameAuraContainer").unwrap()
        };
        // Get or create __frame_fields table
        let fields_table: mlua::Table = lua.globals().get::<mlua::Table>("__frame_fields").unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.globals().set("__frame_fields", t.clone()).unwrap();
            t
        });
        // Create field table for AuraContainer
        let aura_fields = lua.create_table()?;
        aura_fields.set("iconScale", 1.0)?;
        fields_table.set(aura_container_id, aura_fields)?;
    }

    // TargetFrameSpellBar reference (target cast bar)
    let target_spell_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("TargetFrameSpellBar").unwrap()
    };
    let target_spell_bar = lua.create_userdata(FrameHandle {
        id: target_spell_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("TargetFrameSpellBar", target_spell_bar)?;

    // Minimap reference (minimap frame)
    let minimap_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("Minimap").unwrap()
    };
    let minimap = lua.create_userdata(FrameHandle {
        id: minimap_id,
        state: Rc::clone(&state),
    })?;
    globals.set("Minimap", minimap)?;

    // MinimapCluster reference (minimap container frame)
    let minimap_cluster_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("MinimapCluster").unwrap()
    };
    let minimap_cluster = lua.create_userdata(FrameHandle {
        id: minimap_cluster_id,
        state: Rc::clone(&state),
    })?;
    globals.set("MinimapCluster", minimap_cluster)?;

    // ObjectiveTrackerFrame reference (quest/objectives tracker)
    let objective_tracker_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ObjectiveTrackerFrame").unwrap()
    };
    let objective_tracker = lua.create_userdata(FrameHandle {
        id: objective_tracker_id,
        state: Rc::clone(&state),
    })?;
    globals.set("ObjectiveTrackerFrame", objective_tracker)?;

    // SettingsPanel reference (game settings UI)
    let settings_panel_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("SettingsPanel").unwrap()
    };
    let settings_panel = lua.create_userdata(FrameHandle {
        id: settings_panel_id,
        state: Rc::clone(&state),
    })?;
    globals.set("SettingsPanel", settings_panel)?;

    // Add Container structure to SettingsPanel (used by DynamicCam)
    lua.load(r#"
        SettingsPanel.Container = {
            SettingsList = {
                ScrollBox = {
                    ScrollTarget = {
                        GetChildren = function() return end
                    }
                },
                Header = {
                    Title = {
                        GetText = function() return "" end
                    }
                }
            }
        }
    "#).exec()?;

    // Add Header.MinimizeButton structure to ObjectiveTrackerFrame (used by WorldQuestTracker)
    lua.load(r#"
        ObjectiveTrackerFrame.Header = CreateFrame("Frame", nil, ObjectiveTrackerFrame)
        ObjectiveTrackerFrame.Header.MinimizeButton = CreateFrame("Button", nil, ObjectiveTrackerFrame.Header)
    "#).exec()?;

    // ObjectiveTrackerManager - manages objective tracker modules (used by !KalielsTracker)
    lua.load(r#"
        ObjectiveTrackerManager = {
            modules = {},
            containers = {},
            AssignModulesOrder = function(self, modules) end,
            AddContainer = function(self, container) end,
            HasAnyModules = function(self) return false end,
            UpdateAll = function(self) end,
            UpdateModule = function(self, module) end,
            GetContainerForModule = function(self, module) return nil end,
            SetModuleContainer = function(self, module, container) end,
            AcquireFrame = function(self, parent, template) return nil end,
            ReleaseFrame = function(self, frame) end,
            SetOpacity = function(self, opacity) end,
            SetTextSize = function(self, textSize) end,
            ShowRewardsToast = function(self, rewards, module, block, headerText, callback) end,
            HideRewardsToast = function(self, rewardsToast) end,
            HasRewardsToastForBlock = function(self, block) return false end,
            UpdatePOIEnabled = function(self, enabled) end,
            OnVariablesLoaded = function(self) end,
            OnCVarChanged = function(self, cvar, value) end,
            CanShowPOIs = function(self, module) return false end,
            EnumerateActiveBlocksByTag = function(self, tag, callback) end,
        }
    "#).exec()?;

    // PlayerCastingBarFrame reference (player cast bar)
    let player_casting_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("PlayerCastingBarFrame").unwrap()
    };
    let player_casting_bar = lua.create_userdata(FrameHandle {
        id: player_casting_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("PlayerCastingBarFrame", player_casting_bar)?;

    // PartyFrame reference (party member container)
    let party_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("PartyFrame").unwrap()
    };
    let party_frame = lua.create_userdata(FrameHandle {
        id: party_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("PartyFrame", party_frame)?;

    // PetFrame reference (pet unit frame)
    let pet_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("PetFrame").unwrap()
    };
    let pet_frame = lua.create_userdata(FrameHandle {
        id: pet_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("PetFrame", pet_frame)?;

    // AlternatePowerBar reference (alternate power resource bar)
    let alternate_power_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("AlternatePowerBar").unwrap()
    };
    let alternate_power_bar = lua.create_userdata(FrameHandle {
        id: alternate_power_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("AlternatePowerBar", alternate_power_bar)?;

    // MonkStaggerBar reference (monk stagger resource bar)
    let monk_stagger_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("MonkStaggerBar").unwrap()
    };
    let monk_stagger_bar = lua.create_userdata(FrameHandle {
        id: monk_stagger_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("MonkStaggerBar", monk_stagger_bar)?;

    // LFGListFrame reference (Looking For Group list frame)
    let lfg_list_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("LFGListFrame").unwrap()
    };
    let lfg_list_frame = lua.create_userdata(FrameHandle {
        id: lfg_list_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("LFGListFrame", lfg_list_frame)?;

    // Add SearchPanel.SearchBox structure to LFGListFrame (used by WorldQuestTracker)
    lua.load(r#"
        LFGListFrame.SearchPanel = CreateFrame("Frame", nil, LFGListFrame)
        LFGListFrame.SearchPanel.SearchBox = CreateFrame("EditBox", nil, LFGListFrame.SearchPanel)
    "#).exec()?;

    // AlertFrame reference (alert/popup management frame)
    let alert_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("AlertFrame").unwrap()
    };
    let alert_frame = lua.create_userdata(FrameHandle {
        id: alert_frame_id,
        state: Rc::clone(&state),
    })?;
    // Add alertFrameSubSystems table for DynamicCam
    let alert_sub_systems = lua.create_table()?;
    alert_frame.set("alertFrameSubSystems", alert_sub_systems)?;
    globals.set("AlertFrame", alert_frame)?;

    // LFGEventFrame reference (LFG event handling frame)
    let lfg_event_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("LFGEventFrame").unwrap()
    };
    let lfg_event_frame = lua.create_userdata(FrameHandle {
        id: lfg_event_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("LFGEventFrame", lfg_event_frame)?;

    // NamePlateDriverFrame reference (nameplate management frame)
    let nameplate_driver_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("NamePlateDriverFrame").unwrap()
    };
    let nameplate_driver_frame = lua.create_userdata(FrameHandle {
        id: nameplate_driver_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("NamePlateDriverFrame", nameplate_driver_frame)?;

    // UIErrorsFrame reference (error message display frame)
    let ui_errors_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("UIErrorsFrame").unwrap()
    };
    let ui_errors_frame = lua.create_userdata(FrameHandle {
        id: ui_errors_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("UIErrorsFrame", ui_errors_frame)?;

    // InterfaceOptionsFrame reference (legacy interface options)
    let interface_options_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("InterfaceOptionsFrame").unwrap()
    };
    let interface_options_frame = lua.create_userdata(FrameHandle {
        id: interface_options_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("InterfaceOptionsFrame", interface_options_frame)?;

    // AuctionHouseFrame reference (auction house UI)
    let auction_house_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("AuctionHouseFrame").unwrap()
    };
    let auction_house_frame = lua.create_userdata(FrameHandle {
        id: auction_house_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("AuctionHouseFrame", auction_house_frame)?;

    // SideDressUpFrame reference (side dressing room)
    let side_dressup_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("SideDressUpFrame").unwrap()
    };
    let side_dressup_frame = lua.create_userdata(FrameHandle {
        id: side_dressup_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("SideDressUpFrame", side_dressup_frame)?;

    // ContainerFrameContainer reference (bag frame container for combined bags)
    let container_frame_container_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ContainerFrameContainer").unwrap()
    };
    let container_frame_container = lua.create_userdata(FrameHandle {
        id: container_frame_container_id,
        state: Rc::clone(&state),
    })?;
    // ContainerFrames is an empty array (individual bag frames would be added here)
    let container_frames = lua.create_table()?;
    container_frame_container.set("ContainerFrames", container_frames)?;
    globals.set("ContainerFrameContainer", container_frame_container)?;

    // ContainerFrameCombinedBags reference (combined bag frame)
    let container_combined_bags_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ContainerFrameCombinedBags").unwrap()
    };
    let container_combined_bags = lua.create_userdata(FrameHandle {
        id: container_combined_bags_id,
        state: Rc::clone(&state),
    })?;
    globals.set("ContainerFrameCombinedBags", container_combined_bags)?;

    // LootFrame reference (loot window)
    let loot_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("LootFrame").unwrap()
    };
    let loot_frame = lua.create_userdata(FrameHandle {
        id: loot_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("LootFrame", loot_frame)?;

    // AddonCompartmentFrame (retail UI element for addon buttons)
    let addon_compartment_id = {
        let mut state = state.borrow_mut();
        let frame = Frame::new(WidgetType::Frame, Some("AddonCompartmentFrame".to_string()), None);
        state.widgets.register(frame)
    };
    let addon_compartment = lua.create_userdata(FrameHandle {
        id: addon_compartment_id,
        state: Rc::clone(&state),
    })?;
    // Add RegisterAddon/UnregisterAddon methods via custom fields
    // These accept variadic args since they're called as methods (self is first arg)
    {
        let fields_table: mlua::Table = lua.globals().get::<mlua::Table>("__frame_fields").unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.globals().set("__frame_fields", t.clone()).unwrap();
            t
        });
        let frame_fields = lua.create_table()?;
        frame_fields.set("RegisterAddon", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
        frame_fields.set("UnregisterAddon", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
        frame_fields.set("registeredAddons", lua.create_table()?)?;
        fields_table.set(addon_compartment_id, frame_fields)?;
    }
    globals.set("AddonCompartmentFrame", addon_compartment)?;

    // ScenarioObjectiveTracker reference (objective tracker for scenarios/M+)
    let scenario_tracker_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ScenarioObjectiveTracker").unwrap()
    };
    let scenario_tracker = lua.create_userdata(FrameHandle {
        id: scenario_tracker_id,
        state: Rc::clone(&state),
    })?;
    globals.set("ScenarioObjectiveTracker", scenario_tracker)?;

    // RaidWarningFrame reference (raid warning message display)
    let raid_warning_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("RaidWarningFrame").unwrap()
    };
    let raid_warning_frame = lua.create_userdata(FrameHandle {
        id: raid_warning_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("RaidWarningFrame", raid_warning_frame)?;

    // GossipFrame reference (NPC interaction dialog)
    let gossip_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("GossipFrame").unwrap()
    };
    let gossip_frame = lua.create_userdata(FrameHandle {
        id: gossip_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("GossipFrame", gossip_frame)?;

    // FriendsFrame - friends list panel (used by GlobalIgnoreList)
    let friends_frame_id = {
        let mut state_ref = state.borrow_mut();
        let friends_frame = Frame::new(WidgetType::Frame, Some("FriendsFrame".to_string()), None);
        let friends_frame_id = friends_frame.id;
        state_ref.widgets.register(friends_frame);
        friends_frame_id
    };
    let friends_ud = lua.create_userdata(FrameHandle {
        id: friends_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("FriendsFrame", friends_ud)?;

    // PartyMemberFramePool - pool of party member frames (used by Clicked)
    let party_frame_pool = lua.create_table()?;
    party_frame_pool.set("EnumerateActive", lua.create_function(|lua, _self: Value| {
        // Return an empty iterator
        let iter_func = lua.create_function(|_, ()| Ok(Value::Nil))?;
        Ok(iter_func)
    })?)?;
    party_frame_pool.set("GetNumActive", lua.create_function(|_, _self: Value| Ok(0i32))?)?;
    globals.set("PartyMemberFramePool", party_frame_pool)?;

    // UISpecialFrames - table of frame names that close on Escape
    let ui_special_frames = lua.create_table()?;
    globals.set("UISpecialFrames", ui_special_frames)?;

    // StaticPopupDialogs - table for popup dialog definitions
    let static_popup_dialogs = lua.create_table()?;
    globals.set("StaticPopupDialogs", static_popup_dialogs)?;

    // StaticPopup_Show(name, text1, text2, ...) - show a static popup
    globals.set(
        "StaticPopup_Show",
        lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(Value::Nil))?,
    )?;
    globals.set(
        "StaticPopup_Hide",
        lua.create_function(|_, _name: String| Ok(()))?,
    )?;

    // StaticPopup1-4 frames with EditBox children (used by !KalielsTracker, etc)
    lua.load(r#"
        for i = 1, 4 do
            local popup = CreateFrame("Frame", "StaticPopup"..i, UIParent)
            popup.EditBox = CreateFrame("EditBox", "StaticPopup"..i.."EditBox", popup)
            popup.text = popup:CreateFontString(nil, "ARTWORK")
            popup.button1 = CreateFrame("Button", "StaticPopup"..i.."Button1", popup)
            popup.button2 = CreateFrame("Button", "StaticPopup"..i.."Button2", popup)
        end
    "#).exec()?;

    // POIButtonMixin - mixin for quest POI buttons on world map
    let poi_button_mixin = lua.create_table()?;
    poi_button_mixin.set("OnLoad", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnShow", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnHide", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnEnter", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnLeave", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("OnClick", lua.create_function(|_, (_self, _button): (Value, Option<String>)| Ok(()))?)?;
    poi_button_mixin.set("UpdateButtonStyle", lua.create_function(|_, _self: Value| Ok(()))?)?;
    poi_button_mixin.set("SetSelected", lua.create_function(|_, (_self, _selected): (Value, bool)| Ok(()))?)?;
    poi_button_mixin.set("GetSelected", lua.create_function(|_, _self: Value| Ok(false))?)?;
    globals.set("POIButtonMixin", poi_button_mixin)?;

    // TaggableObjectMixin - mixin for objects that can have tags (used by MapCanvasPinMixin)
    let taggable_object_mixin = lua.create_table()?;
    taggable_object_mixin.set("AddTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    taggable_object_mixin.set("RemoveTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    taggable_object_mixin.set("MatchesTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(false))?)?;
    taggable_object_mixin.set("MatchesAnyTag", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    taggable_object_mixin.set("MatchesAllTags", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    globals.set("TaggableObjectMixin", taggable_object_mixin)?;

    // MapCanvasPinMixin - mixin for map pins on WorldMapFrame canvas (inherits from TaggableObjectMixin)
    let map_canvas_pin_mixin = lua.create_table()?;
    // Methods from TaggableObjectMixin (duplicated for mixin inheritance pattern)
    map_canvas_pin_mixin.set("AddTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    map_canvas_pin_mixin.set("RemoveTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(()))?)?;
    map_canvas_pin_mixin.set("MatchesTag", lua.create_function(|_, (_self, _tag): (Value, i32)| Ok(false))?)?;
    map_canvas_pin_mixin.set("MatchesAnyTag", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    map_canvas_pin_mixin.set("MatchesAllTags", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(false))?)?;
    // MapCanvasPinMixin specific methods
    map_canvas_pin_mixin.set("OnLoad", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnAcquired", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnReleased", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnClick", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnMouseEnter", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnMouseLeave", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnMouseDown", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("OnMouseUp", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("GetMap", lua.create_function(|_, _self: Value| Ok(Value::Nil))?)?;
    map_canvas_pin_mixin.set("SetPosition", lua.create_function(|_, (_self, _x, _y): (Value, f64, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetFrameLevelType", lua.create_function(|_, (_self, _type): (Value, String)| Ok(()))?)?;
    map_canvas_pin_mixin.set("GetFrameLevelType", lua.create_function(|lua, _self: Value| Ok(Value::String(lua.create_string("PIN_FRAME_LEVEL_DEFAULT")?)))?)?;
    map_canvas_pin_mixin.set("UseFrameLevelType", lua.create_function(|_, (_self, _type): (Value, String)| Ok(()))?)?;
    map_canvas_pin_mixin.set("ApplyFrameLevel", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("ApplyCurrentPosition", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("ApplyCurrentScale", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("ApplyCurrentAlpha", lua.create_function(|_, _self: Value| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetScalingLimits", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetAlphaLimits", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeSourceRadius", lua.create_function(|_, (_self, _radius): (Value, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeSourceMagnitude", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeTargetFactor", lua.create_function(|_, (_self, _factor): (Value, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeZoomedInFactor", lua.create_function(|_, (_self, _factor): (Value, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("SetNudgeZoomedOutFactor", lua.create_function(|_, (_self, _factor): (Value, f64)| Ok(()))?)?;
    map_canvas_pin_mixin.set("DisableInheritedMotionScriptsWarning", lua.create_function(|_, _self: Value| Ok(false))?)?;
    map_canvas_pin_mixin.set("ShouldMouseButtonBePassthrough", lua.create_function(|_, (_self, _button): (Value, String)| Ok(false))?)?;
    map_canvas_pin_mixin.set("CheckMouseButtonPassthrough", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    map_canvas_pin_mixin.set("AddIconWidgets", lua.create_function(|_, _self: Value| Ok(()))?)?;
    globals.set("MapCanvasPinMixin", map_canvas_pin_mixin)?;

    // Menu - new context menu system (WoW 10.0+)
    let menu = lua.create_table()?;
    menu.set("GetOpenMenu", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    menu.set("GetOpenMenuTags", lua.create_function(|lua, ()| Ok(lua.create_table()?))?)?;
    menu.set("PopupMenu", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    menu.set("OpenMenu", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    menu.set("CloseAll", lua.create_function(|_, ()| Ok(()))?)?;
    menu.set("ModifyMenu", lua.create_function(|_, (_owner, _generator_fn): (Value, mlua::Function)| Ok(()))?)?;
    let menu_response = lua.create_table()?;
    menu_response.set("Close", 0)?;
    menu_response.set("Open", 1)?;
    menu_response.set("Refresh", 2)?;
    menu_response.set("CloseAll", 3)?;
    menu.set("Response", menu_response)?;
    globals.set("Menu", menu)?;

    // MenuUtil - utility functions for the new menu system
    let menu_util = lua.create_table()?;
    menu_util.set("CreateRootMenuDescription", lua.create_function(|lua, _menu_tag: Option<String>| {
        let desc = lua.create_table()?;
        desc.set("CreateButton", lua.create_function(|_, (_self, _text, _callback): (Value, String, Option<mlua::Function>)| Ok(Value::Nil))?)?;
        desc.set("CreateTitle", lua.create_function(|_, (_self, _text): (Value, String)| Ok(Value::Nil))?)?;
        desc.set("CreateDivider", lua.create_function(|_, _self: Value| Ok(Value::Nil))?)?;
        Ok(desc)
    })?)?;
    menu_util.set("SetElementData", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
    menu_util.set("GetElementData", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(Value::Nil))?)?;
    globals.set("MenuUtil", menu_util)?;

    // print() - already exists in Lua but we can customize if needed

    // strsplit(delimiter, str, limit) - WoW string utility
    let strsplit = lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();

        let delimiter = args
            .first()
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| " ".to_string());

        let input = args
            .get(1)
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let limit = args
            .get(2)
            .and_then(|v| {
                if let Value::Integer(n) = v {
                    Some(*n as usize)
                } else if let Value::Number(n) = v {
                    Some(*n as usize)
                } else {
                    None
                }
            });

        let parts: Vec<&str> = if let Some(limit) = limit {
            input.splitn(limit, &delimiter).collect()
        } else {
            input.split(&delimiter).collect()
        };

        let mut result = mlua::MultiValue::new();
        for part in parts {
            result.push_back(Value::String(lua.create_string(part)?));
        }
        Ok(result)
    })?;
    globals.set("strsplit", strsplit)?;

    // getglobal(name) - Get a global variable by name (old WoW API)
    let getglobal_fn = lua.create_function(|lua, name: String| {
        let globals = lua.globals();
        let value: Value = globals.get(name.as_str()).unwrap_or(Value::Nil);
        Ok(value)
    })?;
    globals.set("getglobal", getglobal_fn)?;

    // setglobal(name, value) - Set a global variable by name (old WoW API)
    let setglobal_fn = lua.create_function(|lua, (name, value): (String, Value)| {
        lua.globals().set(name.as_str(), value)?;
        Ok(())
    })?;
    globals.set("setglobal", setglobal_fn)?;

    // loadstring(code, name) - Compile a string of Lua code and return it as a function
    // This is a Lua 5.1 function that WoW uses (replaced by load() in Lua 5.2+)
    let loadstring_fn = lua.create_function(|lua, (code, name): (String, Option<String>)| {
        let chunk_name = name.unwrap_or_else(|| "=(loadstring)".to_string());
        match lua.load(&code).set_name(&chunk_name).into_function() {
            Ok(func) => Ok((Value::Function(func), Value::Nil)),
            Err(e) => Ok((Value::Nil, Value::String(lua.create_string(&e.to_string())?))),
        }
    })?;
    globals.set("loadstring", loadstring_fn)?;

    // Override type() to return "table" for our frame userdata
    // WoW frames behave like tables (they have methods via __index), and some addons
    // check `type(frame) == "table"` to validate UI objects
    let type_fn = lua.create_function(|_, value: Value| {
        let type_str = match &value {
            Value::Nil => "nil",
            Value::Boolean(_) => "boolean",
            Value::Integer(_) | Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Table(_) => "table",
            Value::Function(_) => "function",
            Value::Thread(_) => "thread",
            Value::UserData(ud) => {
                // Check if this is a FrameHandle userdata - treat it as "table"
                // This matches WoW's behavior where frames pass `type(frame) == "table"` checks
                if ud.is::<FrameHandle>() {
                    "table"
                } else {
                    "userdata"
                }
            }
            Value::LightUserData(_) => "userdata",
            Value::Error(_) => "error",
            Value::Other(_) => "userdata",
        };
        Ok(type_str)
    })?;
    globals.set("type", type_fn)?;

    // Override rawget() to handle userdata gracefully
    // Blizzard's Dump.lua does `rawget(v, 0)` on things that pass `type(v) == "table"`
    // Since our FrameHandle passes that check, rawget needs to handle it
    let rawget_fn = lua.create_function(|lua, (table, key): (Value, Value)| {
        match table {
            Value::Table(t) => t.raw_get(key),
            Value::UserData(_) => {
                // UserData doesn't support rawget, return nil instead of erroring
                Ok(Value::Nil)
            }
            _ => {
                // Call the original rawget for proper error
                let original: mlua::Function = lua.globals().raw_get("__original_rawget")?;
                original.call((table, key))
            }
        }
    })?;
    // Save original and install custom
    let original_rawget: mlua::Function = globals.raw_get("rawget")?;
    globals.raw_set("__original_rawget", original_rawget)?;
    globals.set("rawget", rawget_fn)?;

    // xpcall(func, errorhandler, ...) - Call function with error handler and varargs
    // Lua 5.1's native xpcall doesn't support varargs, but WoW's Lua does (Lua 5.2+ feature)
    // This is critical for AceAddon's safecall function to work
    let xpcall_fn = lua.create_function(|lua, args: mlua::MultiValue| {
        let mut args_vec: Vec<Value> = args.into_iter().collect();
        if args_vec.len() < 2 {
            return Err(mlua::Error::RuntimeError(
                "xpcall requires at least 2 arguments".to_string(),
            ));
        }

        let func = match args_vec.remove(0) {
            Value::Function(f) => f,
            _ => {
                return Err(mlua::Error::RuntimeError(
                    "bad argument #1 to 'xpcall' (function expected)".to_string(),
                ))
            }
        };

        let error_handler = match args_vec.remove(0) {
            Value::Function(f) => f,
            _ => {
                return Err(mlua::Error::RuntimeError(
                    "bad argument #2 to 'xpcall' (function expected)".to_string(),
                ))
            }
        };

        // Remaining args are passed to the function
        let call_args: mlua::MultiValue = args_vec.into_iter().collect();

        // Call the function with the varargs
        match func.call::<mlua::MultiValue>(call_args) {
            Ok(results) => {
                // Success: return true followed by all results
                let mut ret = mlua::MultiValue::new();
                ret.push_back(Value::Boolean(true));
                for v in results {
                    ret.push_back(v);
                }
                Ok(ret)
            }
            Err(e) => {
                // Error: call error handler with the error message
                let error_msg = lua.create_string(&e.to_string())?;
                let handler_result = error_handler.call::<Value>(Value::String(error_msg));

                let mut ret = mlua::MultiValue::new();
                ret.push_back(Value::Boolean(false));
                match handler_result {
                    Ok(v) => ret.push_back(v),
                    Err(he) => ret.push_back(Value::String(lua.create_string(&he.to_string())?)),
                }
                Ok(ret)
            }
        }
    })?;
    globals.set("xpcall", xpcall_fn)?;

    // wipe(table) - Clear a table in place
    let wipe = lua.create_function(|_, table: mlua::Table| {
        // Get all keys first to avoid modification during iteration
        let keys: Vec<Value> = table
            .pairs::<Value, Value>()
            .filter_map(|r| r.ok().map(|(k, _)| k))
            .collect();

        for key in keys {
            table.set(key, Value::Nil)?;
        }
        Ok(table)
    })?;
    globals.set("wipe", wipe)?;

    // tinsert - alias for table.insert
    let tinsert = lua.create_function(|lua, args: mlua::MultiValue| {
        let table_insert: mlua::Function = lua.globals().get::<mlua::Table>("table")?.get("insert")?;
        table_insert.call::<()>(args)?;
        Ok(())
    })?;
    globals.set("tinsert", tinsert)?;

    // tremove - alias for table.remove
    let tremove = lua.create_function(|lua, args: mlua::MultiValue| {
        let table_remove: mlua::Function = lua.globals().get::<mlua::Table>("table")?.get("remove")?;
        table_remove.call::<Value>(args)
    })?;
    globals.set("tremove", tremove)?;

    // tInvert - invert table (swap keys and values)
    let tinvert = lua.create_function(|lua, tbl: mlua::Table| {
        let result = lua.create_table()?;
        for pair in tbl.pairs::<Value, Value>() {
            let (k, v) = pair?;
            result.set(v, k)?;
        }
        Ok(result)
    })?;
    globals.set("tInvert", tinvert)?;

    // tContains - check if table contains value
    let tcontains = lua.create_function(|_, (tbl, value): (mlua::Table, Value)| {
        for pair in tbl.pairs::<Value, Value>() {
            let (_, v) = pair?;
            if v == value {
                return Ok(true);
            }
        }
        Ok(false)
    })?;
    globals.set("tContains", tcontains)?;

    // tIndexOf - get index of value in array-like table
    let tindexof = lua.create_function(|_, (tbl, value): (mlua::Table, Value)| {
        for pair in tbl.pairs::<i32, Value>() {
            let (k, v) = pair?;
            if v == value {
                return Ok(Value::Integer(k as i64));
            }
        }
        Ok(Value::Nil)
    })?;
    globals.set("tIndexOf", tindexof)?;

    // tFilter - filter table with predicate (in-place)
    globals.set(
        "tFilter",
        lua.create_function(|_, (tbl, pred, _keep_order): (mlua::Table, mlua::Function, Option<bool>)| {
            let mut to_remove = Vec::new();
            for pair in tbl.pairs::<Value, Value>() {
                let (k, v) = pair?;
                let keep: bool = pred.call((v.clone(),))?;
                if !keep {
                    to_remove.push(k);
                }
            }
            for k in to_remove {
                tbl.set(k, Value::Nil)?;
            }
            Ok(tbl)
        })?,
    )?;

    // CopyTable - deep copy a table
    globals.set(
        "CopyTable",
        lua.create_function(|lua, (tbl, seen): (mlua::Table, Option<mlua::Table>)| {
            let seen = seen.unwrap_or_else(|| lua.create_table().unwrap());
            let result = lua.create_table()?;
            seen.set(tbl.clone(), result.clone())?;
            for pair in tbl.pairs::<Value, Value>() {
                let (k, v) = pair?;
                let new_v = if let Value::Table(inner) = v.clone() {
                    if let Ok(cached) = seen.get::<mlua::Table>(inner.clone()) {
                        Value::Table(cached)
                    } else {
                        // Recursively copy
                        let copy_table: mlua::Function = lua.globals().get("CopyTable")?;
                        copy_table.call((inner, seen.clone()))?
                    }
                } else {
                    v
                };
                result.set(k, new_v)?;
            }
            Ok(result)
        })?,
    )?;

    // MergeTable - merge source into dest
    globals.set(
        "MergeTable",
        lua.create_function(|_, (dest, source): (mlua::Table, mlua::Table)| {
            for pair in source.pairs::<Value, Value>() {
                let (k, v) = pair?;
                dest.set(k, v)?;
            }
            Ok(dest)
        })?,
    )?;

    // SecureCmdOptionParse - parse secure command option strings
    globals.set(
        "SecureCmdOptionParse",
        lua.create_function(|lua, options: String| {
            // Returns the result of parsing a secure option string like "[mod:shift] action1; action2"
            // In simulation, just return the default (last) option
            if let Some(last) = options.split(';').last() {
                Ok(Value::String(lua.create_string(last.trim())?))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;

    // issecure() - check if current execution is in secure context
    globals.set("issecure", lua.create_function(|_, ()| Ok(false))?)?;

    // issecurevariable(table, variable) - check if variable is secure
    globals.set(
        "issecurevariable",
        lua.create_function(|_, (_table, _var): (Option<Value>, String)| {
            // Returns: isSecure, taint
            Ok((true, Value::Nil))
        })?,
    )?;

    // securecall(func, ...) - call a function in secure context
    globals.set(
        "securecall",
        lua.create_function(|_, (func, args): (mlua::Function, mlua::MultiValue)| {
            func.call::<mlua::MultiValue>(args)
        })?,
    )?;

    // forceinsecure() - mark current execution as insecure
    globals.set("forceinsecure", lua.create_function(|_, ()| Ok(()))?)?;

    // hooksecurefunc(name, hook) or hooksecurefunc(table, name, hook)
    let hooksecurefunc = lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();

        let (table, name, hook) = if args.len() == 2 {
            // hooksecurefunc("FuncName", hookFunc)
            let name = if let Value::String(s) = &args[0] {
                s.to_string_lossy().to_string()
            } else {
                String::new()
            };
            let hook = args[1].clone();
            (lua.globals(), name, hook)
        } else if args.len() >= 3 {
            // hooksecurefunc(someTable, "FuncName", hookFunc)
            let table = if let Value::Table(t) = &args[0] {
                t.clone()
            } else {
                lua.globals()
            };
            let name = if let Value::String(s) = &args[1] {
                s.to_string_lossy().to_string()
            } else {
                String::new()
            };
            let hook = args[2].clone();
            (table, name, hook)
        } else {
            return Ok(());
        };

        // Get the original function
        let original: Value = table.get::<Value>(name.as_str())?;

        if let (Value::Function(orig_fn), Value::Function(hook_fn)) = (original, hook) {
            // Create a wrapper that calls original then hook
            let wrapper = lua.create_function(move |_, args: mlua::MultiValue| {
                // Call original
                let result = orig_fn.call::<mlua::MultiValue>(args.clone())?;
                // Call hook (ignoring its result)
                let _ = hook_fn.call::<mlua::MultiValue>(args);
                Ok(result)
            })?;

            table.set(name.as_str(), wrapper)?;
        }

        Ok(())
    })?;
    globals.set("hooksecurefunc", hooksecurefunc)?;

    // GetBuildInfo() - Return mock game version
    let get_build_info = lua.create_function(|lua, ()| {
        // Return: version, build, date, tocversion, localizedVersion, buildType
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("11.0.0")?),  // version
            Value::String(lua.create_string("99999")?),   // build
            Value::String(lua.create_string("Jan 1 2025")?), // date
            Value::Integer(110000),                        // tocversion
            Value::String(lua.create_string("11.0.0")?),  // localizedVersion
            Value::String(lua.create_string("Release")?), // buildType
        ]))
    })?;
    globals.set("GetBuildInfo", get_build_info)?;

    // GetRealmName() - Return mock realm name
    let get_realm_name = lua.create_function(|lua, ()| {
        Ok(Value::String(lua.create_string("SimulatedRealm")?))
    })?;
    globals.set("GetRealmName", get_realm_name)?;

    // GetNormalizedRealmName() - Return mock normalized realm name
    let get_normalized_realm_name = lua.create_function(|lua, ()| {
        Ok(Value::String(lua.create_string("SimulatedRealm")?))
    })?;
    globals.set("GetNormalizedRealmName", get_normalized_realm_name)?;

    // GetLocale() - Return mock locale
    let get_locale = lua.create_function(|lua, ()| {
        Ok(Value::String(lua.create_string("enUS")?))
    })?;
    globals.set("GetLocale", get_locale)?;

    // IsMacClient() - Return false (we're simulating, not on Mac)
    globals.set("IsMacClient", lua.create_function(|_, ()| Ok(false))?)?;

    // IsWindowsClient() - Return true (simulate Windows)
    globals.set("IsWindowsClient", lua.create_function(|_, ()| Ok(true))?)?;

    // IsLinuxClient() - Return false
    globals.set("IsLinuxClient", lua.create_function(|_, ()| Ok(false))?)?;

    // IsTestBuild() - Return false (not a test/beta build)
    globals.set("IsTestBuild", lua.create_function(|_, ()| Ok(false))?)?;

    // IsBetaBuild() - Return false
    globals.set("IsBetaBuild", lua.create_function(|_, ()| Ok(false))?)?;

    // IsPTRClient() - Return false (not PTR)
    globals.set("IsPTRClient", lua.create_function(|_, ()| Ok(false))?)?;

    // IsTrialAccount() - Return false
    globals.set("IsTrialAccount", lua.create_function(|_, ()| Ok(false))?)?;

    // IsVeteranTrialAccount() - Return false
    globals.set("IsVeteranTrialAccount", lua.create_function(|_, ()| Ok(false))?)?;

    // SlashCmdList table
    let slash_cmd_list = lua.create_table()?;
    globals.set("SlashCmdList", slash_cmd_list)?;

    // FireEvent - simulator utility to fire events for testing
    let state_for_fire = Rc::clone(&state);
    let fire_event = lua.create_function(move |lua, args: mlua::Variadic<Value>| {
        let mut args_iter = args.into_iter();
        let event_name: String = match args_iter.next() {
            Some(Value::String(s)) => s.to_str()?.to_string(),
            _ => return Err(mlua::Error::runtime("FireEvent requires event name as first argument")),
        };

        // Collect remaining arguments
        let event_args: Vec<Value> = args_iter.collect();

        // Get listeners for this event
        let listeners = {
            let state = state_for_fire.borrow();
            state.widgets.get_event_listeners(&event_name)
        };

        // Fire to each listener
        for widget_id in listeners {
            let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();

            if let Some(table) = scripts_table {
                let frame_key = format!("{}_OnEvent", widget_id);
                let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

                if let Some(handler) = handler {
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    let frame: Value = lua.globals().get(frame_ref_key.as_str()).unwrap_or(Value::Nil);

                    let mut call_args = vec![frame, Value::String(lua.create_string(&event_name)?)];
                    call_args.extend(event_args.iter().cloned());

                    handler.call::<()>(mlua::MultiValue::from_vec(call_args)).ok();
                }
            }
        }

        Ok(())
    })?;
    globals.set("FireEvent", fire_event)?;

    // ReloadUI - reload the interface (fires startup events again)
    let state_for_reload = Rc::clone(&state);
    let reload_ui = lua.create_function(move |lua, ()| {
        // Fire ADDON_LOADED
        let addon_loaded_listeners = {
            let state = state_for_reload.borrow();
            state.widgets.get_event_listeners("ADDON_LOADED")
        };
        for widget_id in addon_loaded_listeners {
            if let Ok(Some(table)) = lua.globals().get::<Option<mlua::Table>>("__scripts") {
                let frame_key = format!("{}_OnEvent", widget_id);
                if let Ok(Some(handler)) = table.get::<Option<mlua::Function>>(frame_key.as_str()) {
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    if let Ok(frame) = lua.globals().get::<Value>(frame_ref_key.as_str()) {
                        let event_str = lua.create_string("ADDON_LOADED")?;
                        let addon_name = lua.create_string("WoWUISim")?;
                        let _ = handler.call::<()>((frame, Value::String(event_str), Value::String(addon_name)));
                    }
                }
            }
        }

        // Fire PLAYER_LOGIN
        let login_listeners = {
            let state = state_for_reload.borrow();
            state.widgets.get_event_listeners("PLAYER_LOGIN")
        };
        for widget_id in login_listeners {
            if let Ok(Some(table)) = lua.globals().get::<Option<mlua::Table>>("__scripts") {
                let frame_key = format!("{}_OnEvent", widget_id);
                if let Ok(Some(handler)) = table.get::<Option<mlua::Function>>(frame_key.as_str()) {
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    if let Ok(frame) = lua.globals().get::<Value>(frame_ref_key.as_str()) {
                        let event_str = lua.create_string("PLAYER_LOGIN")?;
                        let _ = handler.call::<()>((frame, Value::String(event_str)));
                    }
                }
            }
        }

        // Fire PLAYER_ENTERING_WORLD
        let entering_listeners = {
            let state = state_for_reload.borrow();
            state.widgets.get_event_listeners("PLAYER_ENTERING_WORLD")
        };
        for widget_id in entering_listeners {
            if let Ok(Some(table)) = lua.globals().get::<Option<mlua::Table>>("__scripts") {
                let frame_key = format!("{}_OnEvent", widget_id);
                if let Ok(Some(handler)) = table.get::<Option<mlua::Function>>(frame_key.as_str()) {
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    if let Ok(frame) = lua.globals().get::<Value>(frame_ref_key.as_str()) {
                        let event_str = lua.create_string("PLAYER_ENTERING_WORLD")?;
                        let _ = handler.call::<()>((frame, Value::String(event_str), Value::Boolean(false), Value::Boolean(true)));
                    }
                }
            }
        }

        state_for_reload.borrow_mut().console_output.push("UI Reloaded".to_string());
        Ok(())
    })?;
    globals.set("ReloadUI", reload_ui)?;

    // Enum table (WoW uses this for various enumerations)
    let enum_table = lua.create_table()?;

    // Enum.LFGRole
    let lfg_role = lua.create_table()?;
    lfg_role.set("Tank", 0)?;
    lfg_role.set("Healer", 1)?;
    lfg_role.set("Damage", 2)?;
    enum_table.set("LFGRole", lfg_role)?;

    // Enum.UnitSex
    let unit_sex = lua.create_table()?;
    unit_sex.set("Male", 2)?;
    unit_sex.set("Female", 3)?;
    enum_table.set("UnitSex", unit_sex)?;

    // Enum.GameMode
    let game_mode = lua.create_table()?;
    game_mode.set("Standard", 0)?;
    game_mode.set("Plunderstorm", 1)?;
    game_mode.set("WoWHack", 2)?;
    enum_table.set("GameMode", game_mode)?;

    // Enum.Profession
    let profession = lua.create_table()?;
    profession.set("Mining", 1)?;
    profession.set("Skinning", 2)?;
    profession.set("Herbalism", 3)?;
    profession.set("Blacksmithing", 4)?;
    profession.set("Leatherworking", 5)?;
    profession.set("Alchemy", 6)?;
    profession.set("Tailoring", 7)?;
    profession.set("Engineering", 8)?;
    profession.set("Enchanting", 9)?;
    profession.set("Fishing", 10)?;
    profession.set("Cooking", 11)?;
    profession.set("Jewelcrafting", 12)?;
    profession.set("Inscription", 13)?;
    profession.set("Archaeology", 14)?;
    enum_table.set("Profession", profession)?;

    // Enum.VasTransactionPurchaseResult - all values used by VASErrorLookup.lua
    let vas_result = lua.create_table()?;
    for (i, name) in [
        "Ok", "NotAvailable", "InProgress", "OnlyOneVasAtATime",
        "InvalidDestinationAccount", "InvalidSourceAccount", "InvalidCharacter",
        "NotEnoughMoney", "NotEligible", "TransferServiceDisabled",
        "DifferentRegion", "RealmNotEligible", "CharacterNotOnAccount",
        "TooManyCharacters", "InternalError", "PendingOtherProduct",
        "PendingItemDelivery", "PurchaseInProgress", "GenericError",
        "DisallowedSourceAccount", "DisallowedDestinationAccount", "LowerBoxLevel",
        "MaxCharactersOnServer", "CantAffordService", "ServiceAvailable",
        "CharacterHasGuildBank", "NameNotAvailable", "CharacterBelongsToGuild",
        "LockedForVas", "MoveInProgress", "AgeRestriction", "UnderMinAge",
        "BoostedTooRecently", "NewPlayerRestrictions", "CannotRestore",
        "GuildHasGuildBank", "CharacterArenaTeam", "CharacterTransferInProgress",
        "CharacterTransferPending", "RaceClassComboNotEligible", "InvalidStartingLevel",
        // Proxy errors
        "ProxyBadRequestContained", "ProxyCharacterTransferredNoBoostInProgress",
        // Database errors
        "DbRealmNotEligible", "DbCannotMoveGuildmaster", "DbMaxCharactersOnServer",
        "DbNoMixedAlliance", "DbDuplicateCharacterName", "DbHasMail", "DbMoveInProgress",
        "DbUnderMinLevelReq", "DbIneligibleTargetRealm", "DbTransferDateTooSoon",
        "DbCharLocked", "DbAllianceNotEligible", "DbTooMuchMoneyForLevel",
        "DbHasAuctions", "DbLastSaveTooRecent", "DbNameNotAvailable",
        "DbLastRenameTooRecent", "DbAlreadyRenameFlagged", "DbCustomizeAlreadyRequested",
        "DbLastCustomizeTooSoon", "DbFactionChangeTooSoon", "DbRaceClassComboIneligible",
        "DbPendingItemAudit", "DbGuildRankInsufficient", "DbCharacterWithoutGuild",
        "DbGmSenorityInsufficient", "DbAuthenticatorInsufficient", "DbIneligibleMapID",
        "DbBpayDeliveryPending", "DbHasBpayToken", "DbHasHeirloomItem",
        "DbResultAccountRestricted", "DbLastSaveTooDistant", "DbCagedPetInInventory",
        "DbOnBoostCooldown", "DbPvEPvPTransferNotAllowed", "DbNewLeaderInvalid",
        "DbNeedsLevelSquish", "DbHasNewPlayerExperienceRestriction", "DbHasCraftingOrders",
        "DbInvalidName", "DbNeedsEraChoice", "DbCannotMoveArenaCaptn",
    ].iter().enumerate() {
        vas_result.set(*name, i as i32)?;
    }
    enum_table.set("VasTransactionPurchaseResult", vas_result)?;

    // Enum.StoreError - store error codes
    let store_error = lua.create_table()?;
    for (i, name) in [
        "InvalidPaymentMethod", "PaymentFailed", "WrongCurrency", "BattlepayDisabled",
        "InsufficientBalance", "Other", "AlreadyOwned", "ParentalControlsNoPurchase",
        "PurchaseDenied", "ConsumableTokenOwned", "TooManyTokens", "ItemUnavailable",
        "ClientRestricted",
    ].iter().enumerate() {
        store_error.set(*name, i as i32)?;
    }
    enum_table.set("StoreError", store_error)?;

    // Enum.GameRule - game rule identifiers
    let game_rule = lua.create_table()?;
    for (i, name) in [
        "PlayerCastBarDisabled", "TargetCastBarDisabled", "NameplateCastBarDisabled",
        "UserAddonsDisabled", "EncounterJournalDisabled", "EjSuggestedContentDisabled",
        "EjDungeonsDisabled", "EjRaidsDisabled", "EjItemSetsDisabled",
        "ExperienceBarDisabled", "ActionButtonTypeOverlayStrategy",
    ].iter().enumerate() {
        game_rule.set(*name, i as i32)?;
    }
    enum_table.set("GameRule", game_rule)?;

    // Enum.ScriptedAnimationBehavior (many values needed)
    let animation_behavior = lua.create_table()?;
    for (i, name) in [
        "None", "FollowsCaster", "FollowsTarget", "SourceRecoil",
        "SourceCollideWithTarget", "TargetShake", "TargetKnockBack",
        "UIParentShake", "TargetCenter", "TargetCenterToSource",
    ].iter().enumerate() {
        animation_behavior.set(*name, i as i32)?;
    }
    enum_table.set("ScriptedAnimationBehavior", animation_behavior)?;

    // Enum.ScriptedAnimationTrajectory
    let animation_trajectory = lua.create_table()?;
    for (i, name) in [
        "AtSource", "Straight", "CurveLeft", "CurveRight", "CurveRandom",
        "AtTarget", "HalfwayBetween", "SourceToTarget", "TargetToSource",
    ].iter().enumerate() {
        animation_trajectory.set(*name, i as i32)?;
    }
    enum_table.set("ScriptedAnimationTrajectory", animation_trajectory)?;

    // Enum.UIWidgetVisualizationType - UI widget visual types
    let widget_vis_type = lua.create_table()?;
    for (i, name) in [
        "IconAndText", "CaptureBar", "StatusBar", "DoubleStatusBar",
        "IconTextAndBackground", "DoubleIconAndText", "StackedResourceTracker",
        "IconTextAndCurrencies", "TextWithState", "HorizontalCurrencies",
        "BulletTextList", "ScenarioHeaderCurrenciesAndBackground", "TextureAndText",
        "SpellDisplay", "DoubleStateIconRow", "TextureAndTextRow", "ZoneControl",
        "CaptureZone", "TextureWithAnimation", "DiscreteProgressSteps",
        "ScenarioHeaderTimer", "TextColumnRow", "Spacer", "UnitPowerBar",
        "FillUpFrames", "TextWithSubtext", "MapPinAnimation", "ItemDisplay",
    ].iter().enumerate() {
        widget_vis_type.set(*name, i as i32)?;
    }
    enum_table.set("UIWidgetVisualizationType", widget_vis_type)?;

    // Enum.UIWidgetTooltipLocation - where tooltips appear
    let widget_tooltip_loc = lua.create_table()?;
    for (i, name) in [
        "Default", "BottomLeft", "BottomRight", "TopLeft", "TopRight",
        "Right", "Left", "Top", "Bottom", "Custom",
    ].iter().enumerate() {
        widget_tooltip_loc.set(*name, i as i32)?;
    }
    enum_table.set("UIWidgetTooltipLocation", widget_tooltip_loc)?;

    // Enum.UIWidgetTextSizeType - text size options
    let widget_text_size = lua.create_table()?;
    widget_text_size.set("Small10Pt", 0)?;
    widget_text_size.set("Small11Pt", 1)?;
    widget_text_size.set("Small12Pt", 2)?;
    widget_text_size.set("Standard14Pt", 3)?;
    widget_text_size.set("Medium16Pt", 4)?;
    widget_text_size.set("Medium18Pt", 5)?;
    widget_text_size.set("Large20Pt", 6)?;
    widget_text_size.set("Large24Pt", 7)?;
    widget_text_size.set("Huge27Pt", 8)?;
    enum_table.set("UIWidgetTextSizeType", widget_text_size)?;

    // Enum.UIWidgetFlag - widget flags
    let widget_flag = lua.create_table()?;
    widget_flag.set("UniversalWidget", 0)?;
    enum_table.set("UIWidgetFlag", widget_flag)?;

    // Enum.FlightPathFaction - flight path faction IDs
    let flight_path_faction = lua.create_table()?;
    flight_path_faction.set("Horde", 0)?;
    flight_path_faction.set("Alliance", 1)?;
    flight_path_faction.set("Neutral", 2)?;
    enum_table.set("FlightPathFaction", flight_path_faction)?;

    // Enum.UIWidgetScale - widget scale percentages
    let widget_scale = lua.create_table()?;
    widget_scale.set("OneHundred", 0)?;
    widget_scale.set("Ninty", 1)?;
    widget_scale.set("Eighty", 2)?;
    widget_scale.set("Seventy", 3)?;
    widget_scale.set("Sixty", 4)?;
    widget_scale.set("Fifty", 5)?;
    widget_scale.set("OneHundredTen", 6)?;
    widget_scale.set("OneHundredTwenty", 7)?;
    widget_scale.set("OneHundredThirty", 8)?;
    widget_scale.set("OneHundredForty", 9)?;
    widget_scale.set("OneHundredFifty", 10)?;
    widget_scale.set("OneHundredSixty", 11)?;
    widget_scale.set("OneHundredSeventy", 12)?;
    widget_scale.set("OneHundredEighty", 13)?;
    widget_scale.set("OneHundredNinety", 14)?;
    widget_scale.set("TwoHundred", 15)?;
    enum_table.set("UIWidgetScale", widget_scale)?;

    // Enum.UIWidgetRewardShownState - reward display state
    let reward_shown_state = lua.create_table()?;
    reward_shown_state.set("Hidden", 0)?;
    reward_shown_state.set("Shown", 1)?;
    reward_shown_state.set("ShownEarned", 2)?;
    reward_shown_state.set("ShownUnearned", 3)?;
    reward_shown_state.set("Unavailable", 4)?;
    enum_table.set("UIWidgetRewardShownState", reward_shown_state)?;

    // Enum.WidgetIconSizeType - icon size options
    let widget_icon_size = lua.create_table()?;
    widget_icon_size.set("Small", 0)?;
    widget_icon_size.set("Medium", 1)?;
    widget_icon_size.set("Large", 2)?;
    widget_icon_size.set("Standard", 3)?;
    enum_table.set("WidgetIconSizeType", widget_icon_size)?;

    // Enum.SpellDisplayBorderColor - spell display border colors
    let spell_border_color = lua.create_table()?;
    spell_border_color.set("None", 0)?;
    spell_border_color.set("Black", 1)?;
    spell_border_color.set("White", 2)?;
    spell_border_color.set("Red", 3)?;
    spell_border_color.set("Yellow", 4)?;
    spell_border_color.set("Orange", 5)?;
    spell_border_color.set("Purple", 6)?;
    spell_border_color.set("Green", 7)?;
    spell_border_color.set("Blue", 8)?;
    enum_table.set("SpellDisplayBorderColor", spell_border_color)?;

    // Enum.SpellDisplayIconDisplayType
    let spell_icon_display = lua.create_table()?;
    spell_icon_display.set("Buff", 0)?;
    spell_icon_display.set("Debuff", 1)?;
    spell_icon_display.set("Circular", 2)?;
    enum_table.set("SpellDisplayIconDisplayType", spell_icon_display)?;

    // Enum.SpellDisplayTextShownStateType
    let spell_text_shown = lua.create_table()?;
    spell_text_shown.set("NotShown", 0)?;
    spell_text_shown.set("Shown", 1)?;
    enum_table.set("SpellDisplayTextShownStateType", spell_text_shown)?;

    // Enum.SpellDisplayTint
    let spell_tint = lua.create_table()?;
    spell_tint.set("None", 0)?;
    spell_tint.set("Red", 1)?;
    spell_tint.set("Yellow", 2)?;
    spell_tint.set("Disabled", 3)?;
    enum_table.set("SpellDisplayTint", spell_tint)?;

    // Enum.StatusBarColorTintValue
    let status_bar_tint = lua.create_table()?;
    status_bar_tint.set("None", 0)?;
    status_bar_tint.set("Black", 1)?;
    status_bar_tint.set("White", 2)?;
    status_bar_tint.set("Red", 3)?;
    status_bar_tint.set("Yellow", 4)?;
    status_bar_tint.set("Orange", 5)?;
    status_bar_tint.set("Purple", 6)?;
    status_bar_tint.set("Green", 7)?;
    status_bar_tint.set("Blue", 8)?;
    enum_table.set("StatusBarColorTintValue", status_bar_tint)?;

    // Enum.StatusBarOverrideBarTextShownType
    let bar_text_shown = lua.create_table()?;
    bar_text_shown.set("Never", 0)?;
    bar_text_shown.set("Always", 1)?;
    bar_text_shown.set("OnlyOnMouseover", 2)?;
    bar_text_shown.set("OnlyNotOnMouseover", 3)?;
    enum_table.set("StatusBarOverrideBarTextShownType", bar_text_shown)?;

    // Enum.StatusBarValueTextType
    let bar_value_text = lua.create_table()?;
    bar_value_text.set("Hidden", 0)?;
    bar_value_text.set("Percentage", 1)?;
    bar_value_text.set("Value", 2)?;
    bar_value_text.set("Time", 3)?;
    bar_value_text.set("TimeShowOneLevelOnly", 4)?;
    bar_value_text.set("ValueOverMax", 5)?;
    bar_value_text.set("ValueOverMaxNormalized", 6)?;
    enum_table.set("StatusBarValueTextType", bar_value_text)?;

    // Enum.WidgetShownState
    let widget_shown = lua.create_table()?;
    widget_shown.set("Hidden", 0)?;
    widget_shown.set("Shown", 1)?;
    enum_table.set("WidgetShownState", widget_shown)?;

    // Enum.WidgetEnabledState
    let widget_enabled = lua.create_table()?;
    widget_enabled.set("Disabled", 0)?;
    widget_enabled.set("Enabled", 1)?;
    widget_enabled.set("Red", 2)?;
    widget_enabled.set("White", 3)?;
    widget_enabled.set("Green", 4)?;
    widget_enabled.set("Gold", 5)?;
    enum_table.set("WidgetEnabledState", widget_enabled)?;

    // Enum.WidgetOpacityType - opacity percentages
    let widget_opacity = lua.create_table()?;
    widget_opacity.set("OneHundred", 0)?;
    widget_opacity.set("Ninety", 1)?;
    widget_opacity.set("Eighty", 2)?;
    widget_opacity.set("Seventy", 3)?;
    widget_opacity.set("Sixty", 4)?;
    widget_opacity.set("Fifty", 5)?;
    widget_opacity.set("Forty", 6)?;
    widget_opacity.set("Thirty", 7)?;
    widget_opacity.set("Twenty", 8)?;
    widget_opacity.set("Ten", 9)?;
    widget_opacity.set("Zero", 10)?;
    enum_table.set("WidgetOpacityType", widget_opacity)?;

    // Enum.WidgetAnimationType
    let widget_animation = lua.create_table()?;
    widget_animation.set("None", 0)?;
    widget_animation.set("Fade", 1)?;
    enum_table.set("WidgetAnimationType", widget_animation)?;

    // Enum.WidgetShowGlowState
    let widget_glow = lua.create_table()?;
    widget_glow.set("HideGlow", 0)?;
    widget_glow.set("ShowGlow", 1)?;
    enum_table.set("WidgetShowGlowState", widget_glow)?;

    // Enum.WidgetGlowAnimType
    let widget_glow_anim = lua.create_table()?;
    widget_glow_anim.set("None", 0)?;
    widget_glow_anim.set("Pulse", 1)?;
    enum_table.set("WidgetGlowAnimType", widget_glow_anim)?;

    // Enum.IconAndTextWidgetState
    let icon_text_state = lua.create_table()?;
    icon_text_state.set("Hidden", 0)?;
    icon_text_state.set("Shown", 1)?;
    icon_text_state.set("ShownWithDynamicIconFlashing", 2)?;
    icon_text_state.set("ShownWithDynamicIconNotFlashing", 3)?;
    enum_table.set("IconAndTextWidgetState", icon_text_state)?;

    // Enum.IconState
    let icon_state = lua.create_table()?;
    icon_state.set("Hidden", 0)?;
    icon_state.set("ShowState1", 1)?;
    icon_state.set("ShowState2", 2)?;
    enum_table.set("IconState", icon_state)?;

    // Enum.ZoneControlState / ZoneControlMode / etc
    let zone_control_state = lua.create_table()?;
    zone_control_state.set("State1", 0)?;
    zone_control_state.set("State2", 1)?;
    zone_control_state.set("State3", 2)?;
    enum_table.set("ZoneControlState", zone_control_state)?;

    let zone_control_mode = lua.create_table()?;
    zone_control_mode.set("BothStatesHaveFullBar", 0)?;
    zone_control_mode.set("SingleState", 1)?;
    enum_table.set("ZoneControlMode", zone_control_mode)?;

    let zone_control_active = lua.create_table()?;
    zone_control_active.set("Inactive", 0)?;
    zone_control_active.set("State1Active", 1)?;
    zone_control_active.set("State2Active", 2)?;
    enum_table.set("ZoneControlActiveState", zone_control_active)?;

    let zone_control_fill = lua.create_table()?;
    zone_control_fill.set("SingleFillClockwise", 0)?;
    zone_control_fill.set("SingleFillCounterClockwise", 1)?;
    zone_control_fill.set("DoubleFillClockwise", 2)?;
    zone_control_fill.set("DoubleFillCounterClockwise", 3)?;
    enum_table.set("ZoneControlFillType", zone_control_fill)?;

    let zone_danger_flash = lua.create_table()?;
    zone_danger_flash.set("ShowNone", 0)?;
    zone_danger_flash.set("ShowOnState1", 1)?;
    zone_danger_flash.set("ShowOnState2", 2)?;
    zone_danger_flash.set("ShowOnBoth", 3)?;
    enum_table.set("ZoneControlDangerFlashType", zone_danger_flash)?;

    let zone_leading_edge = lua.create_table()?;
    zone_leading_edge.set("ShowNone", 0)?;
    zone_leading_edge.set("ShowOnState1", 1)?;
    zone_leading_edge.set("ShowOnState2", 2)?;
    zone_leading_edge.set("ShowOnBoth", 3)?;
    enum_table.set("ZoneControlLeadingEdgeType", zone_leading_edge)?;

    // Enum.CaptureBarWidgetFillDirectionType
    let capture_fill = lua.create_table()?;
    capture_fill.set("RightToLeft", 0)?;
    capture_fill.set("LeftToRight", 1)?;
    enum_table.set("CaptureBarWidgetFillDirectionType", capture_fill)?;

    // Enum.UIWidgetTextureAndTextSizeType
    let texture_text_size = lua.create_table()?;
    texture_text_size.set("Small", 0)?;
    texture_text_size.set("Medium", 1)?;
    texture_text_size.set("Large", 2)?;
    texture_text_size.set("Huge", 3)?;
    texture_text_size.set("Standard", 4)?;
    texture_text_size.set("Medium2", 5)?;
    texture_text_size.set("Standard20", 6)?;
    texture_text_size.set("Standard14", 7)?;
    enum_table.set("UIWidgetTextureAndTextSizeType", texture_text_size)?;

    // Enum.MapPinAnimationType
    let map_pin_anim = lua.create_table()?;
    map_pin_anim.set("BounceIn", 0)?;
    map_pin_anim.set("FadeIn", 1)?;
    map_pin_anim.set("RiseIn", 2)?;
    enum_table.set("MapPinAnimationType", map_pin_anim)?;

    // Enum.TugOfWarMarkerArrowShownState
    let tug_arrow = lua.create_table()?;
    tug_arrow.set("None", 0)?;
    tug_arrow.set("Left", 1)?;
    tug_arrow.set("Right", 2)?;
    enum_table.set("TugOfWarMarkerArrowShownState", tug_arrow)?;

    // Enum.IconAndTextShiftTextType
    let icon_shift_text = lua.create_table()?;
    icon_shift_text.set("None", 0)?;
    icon_shift_text.set("Up", 1)?;
    icon_shift_text.set("Down", 2)?;
    enum_table.set("IconAndTextShiftTextType", icon_shift_text)?;

    // Enum.ItemDisplayTextDisplayStyle
    let item_text_display = lua.create_table()?;
    item_text_display.set("Hidden", 0)?;
    item_text_display.set("ItemName", 1)?;
    item_text_display.set("ItemCount", 2)?;
    enum_table.set("ItemDisplayTextDisplayStyle", item_text_display)?;

    // Enum.WidgetIconSourceType
    let icon_source = lua.create_table()?;
    icon_source.set("Spell", 0)?;
    icon_source.set("Item", 1)?;
    icon_source.set("Currency", 2)?;
    icon_source.set("File", 3)?;
    enum_table.set("WidgetIconSourceType", icon_source)?;

    // Enum.WidgetTextHorizontalAlignmentType
    let text_align = lua.create_table()?;
    text_align.set("Left", 0)?;
    text_align.set("Center", 1)?;
    text_align.set("Right", 2)?;
    enum_table.set("WidgetTextHorizontalAlignmentType", text_align)?;

    // Enum.BagIndex - bag slot indices
    let bag_index = lua.create_table()?;
    bag_index.set("Backpack", 0)?;
    bag_index.set("Bag_1", 1)?;
    bag_index.set("Bag_2", 2)?;
    bag_index.set("Bag_3", 3)?;
    bag_index.set("Bag_4", 4)?;
    bag_index.set("Bank", -1)?;
    bag_index.set("Keyring", -2)?;
    bag_index.set("BankBag_1", 5)?;
    bag_index.set("BankBag_2", 6)?;
    bag_index.set("BankBag_3", 7)?;
    bag_index.set("BankBag_4", 8)?;
    bag_index.set("BankBag_5", 9)?;
    bag_index.set("BankBag_6", 10)?;
    bag_index.set("BankBag_7", 11)?;
    bag_index.set("Reagentbank", -3)?;
    bag_index.set("AccountBankTab_1", 13)?;
    bag_index.set("AccountBankTab_2", 14)?;
    bag_index.set("AccountBankTab_3", 15)?;
    bag_index.set("AccountBankTab_4", 16)?;
    bag_index.set("AccountBankTab_5", 17)?;
    enum_table.set("BagIndex", bag_index)?;

    // Enum.WidgetUnitPowerBarFlashMomentType
    let flash_moment = lua.create_table()?;
    flash_moment.set("None", 0)?;
    flash_moment.set("ValueFull", 1)?;
    flash_moment.set("OnIncrement", 2)?;
    enum_table.set("WidgetUnitPowerBarFlashMomentType", flash_moment)?;

    // Enum.UIWidgetFontType
    let widget_font = lua.create_table()?;
    widget_font.set("Normal", 0)?;
    widget_font.set("Shadow", 1)?;
    widget_font.set("Outline", 2)?;
    enum_table.set("UIWidgetFontType", widget_font)?;

    // Enum.UIWidgetBlendModeType
    let blend_mode = lua.create_table()?;
    blend_mode.set("Opaque", 0)?;
    blend_mode.set("Additive", 1)?;
    blend_mode.set("AlphaKey", 2)?;
    enum_table.set("UIWidgetBlendModeType", blend_mode)?;

    // Enum.UIWidgetMotionType
    let motion_type = lua.create_table()?;
    motion_type.set("Instant", 0)?;
    motion_type.set("Smooth", 1)?;
    enum_table.set("UIWidgetMotionType", motion_type)?;

    // Enum.UIWidgetUpdateAnimType
    let update_anim = lua.create_table()?;
    update_anim.set("None", 0)?;
    update_anim.set("Flash", 1)?;
    enum_table.set("UIWidgetUpdateAnimType", update_anim)?;

    // Enum.UIWidgetOverrideState
    let override_state = lua.create_table()?;
    override_state.set("None", 0)?;
    override_state.set("State1", 1)?;
    override_state.set("State2", 2)?;
    enum_table.set("UIWidgetOverrideState", override_state)?;

    // Enum.UIWidgetTextFormatType
    let text_format = lua.create_table()?;
    text_format.set("Value", 0)?;
    text_format.set("ValueOverMax", 1)?;
    text_format.set("ValueOverMaxNormalized", 2)?;
    text_format.set("Percentage", 3)?;
    enum_table.set("UIWidgetTextFormatType", text_format)?;

    // Enum.UIWidgetSpellButtonCooldownType
    let spell_cooldown = lua.create_table()?;
    spell_cooldown.set("None", 0)?;
    spell_cooldown.set("Cooldown", 1)?;
    spell_cooldown.set("Loss", 2)?;
    enum_table.set("UIWidgetSpellButtonCooldownType", spell_cooldown)?;

    // Enum.UIWidgetButtonEnabledState
    let button_enabled = lua.create_table()?;
    button_enabled.set("Disabled", 0)?;
    button_enabled.set("Enabled", 1)?;
    enum_table.set("UIWidgetButtonEnabledState", button_enabled)?;

    // Enum.UIWidgetButtonIconType
    let button_icon = lua.create_table()?;
    button_icon.set("None", 0)?;
    button_icon.set("Exit", 1)?;
    button_icon.set("Speak", 2)?;
    button_icon.set("Undo", 3)?;
    button_icon.set("Checkmark", 4)?;
    button_icon.set("RedX", 5)?;
    button_icon.set("Spell", 6)?;
    button_icon.set("Item", 7)?;
    enum_table.set("UIWidgetButtonIconType", button_icon)?;

    // Enum.UIWidgetHorizontalDirection
    let h_direction = lua.create_table()?;
    h_direction.set("Left", 0)?;
    h_direction.set("Right", 1)?;
    enum_table.set("UIWidgetHorizontalDirection", h_direction)?;

    // Enum.UIWidgetLayoutDirection
    let layout_dir = lua.create_table()?;
    layout_dir.set("Default", 0)?;
    layout_dir.set("Vertical", 1)?;
    layout_dir.set("Horizontal", 2)?;
    layout_dir.set("Overlap", 3)?;
    enum_table.set("UIWidgetLayoutDirection", layout_dir)?;

    // Enum.UIWidgetModelSceneLayer
    let model_layer = lua.create_table()?;
    model_layer.set("None", 0)?;
    model_layer.set("Front", 1)?;
    model_layer.set("Back", 2)?;
    enum_table.set("UIWidgetModelSceneLayer", model_layer)?;

    // Enum.TugOfWarStyleValue - tug of war widget style
    let tug_of_war_style = lua.create_table()?;
    tug_of_war_style.set("None", 0)?;
    tug_of_war_style.set("DefaultYellow", 1)?;
    tug_of_war_style.set("ArchaeologyBrown", 2)?;
    tug_of_war_style.set("Arrow", 3)?;
    tug_of_war_style.set("Flames", 4)?;
    enum_table.set("TugOfWarStyleValue", tug_of_war_style)?;

    // Enum.UIWidgetSetLayoutDirection - layout direction
    let layout_direction = lua.create_table()?;
    layout_direction.set("Vertical", 0)?;
    layout_direction.set("Horizontal", 1)?;
    layout_direction.set("HorizontalReverse", 2)?;
    layout_direction.set("VerticalReverse", 3)?;
    enum_table.set("UIWidgetSetLayoutDirection", layout_direction)?;

    // Enum.InventoryType - item inventory slot types
    let inventory_type = lua.create_table()?;
    inventory_type.set("IndexNonEquipType", 0)?;
    inventory_type.set("IndexHeadType", 1)?;
    inventory_type.set("IndexNeckType", 2)?;
    inventory_type.set("IndexShoulderType", 3)?;
    inventory_type.set("IndexBodyType", 4)?;
    inventory_type.set("IndexChestType", 5)?;
    inventory_type.set("IndexWaistType", 6)?;
    inventory_type.set("IndexLegsType", 7)?;
    inventory_type.set("IndexFeetType", 8)?;
    inventory_type.set("IndexWristType", 9)?;
    inventory_type.set("IndexHandType", 10)?;
    inventory_type.set("IndexFingerType", 11)?;
    inventory_type.set("IndexTrinketType", 12)?;
    inventory_type.set("IndexWeaponType", 13)?;
    inventory_type.set("IndexShieldType", 14)?;
    inventory_type.set("IndexRangedType", 15)?;
    inventory_type.set("IndexCloakType", 16)?;
    inventory_type.set("Index2HweaponType", 17)?;
    inventory_type.set("IndexBagType", 18)?;
    inventory_type.set("IndexTabardType", 19)?;
    inventory_type.set("IndexRobeType", 20)?;
    inventory_type.set("IndexWeaponmainhandType", 21)?;
    inventory_type.set("IndexWeaponoffhandType", 22)?;
    inventory_type.set("IndexHoldableType", 23)?;
    inventory_type.set("IndexAmmoType", 24)?;
    inventory_type.set("IndexThrownType", 25)?;
    inventory_type.set("IndexRangedrightType", 26)?;
    inventory_type.set("IndexQuiverType", 27)?;
    inventory_type.set("IndexRelicType", 28)?;
    inventory_type.set("IndexProfessionToolType", 29)?;
    inventory_type.set("IndexProfessionGearType", 30)?;
    inventory_type.set("IndexEquipablespellOffensiveType", 31)?;
    inventory_type.set("IndexEquipablespellUtilityType", 32)?;
    inventory_type.set("IndexEquipablespellDefensiveType", 33)?;
    inventory_type.set("IndexEquipablespellWeaponType", 34)?;
    enum_table.set("InventoryType", inventory_type)?;

    // Enum.ItemWeaponSubclass - weapon subclass types
    let item_weapon_subclass = lua.create_table()?;
    item_weapon_subclass.set("Axe1H", 0)?;
    item_weapon_subclass.set("Axe2H", 1)?;
    item_weapon_subclass.set("Bows", 2)?;
    item_weapon_subclass.set("Guns", 3)?;
    item_weapon_subclass.set("Mace1H", 4)?;
    item_weapon_subclass.set("Mace2H", 5)?;
    item_weapon_subclass.set("Polearm", 6)?;
    item_weapon_subclass.set("Sword1H", 7)?;
    item_weapon_subclass.set("Sword2H", 8)?;
    item_weapon_subclass.set("Warglaive", 9)?;
    item_weapon_subclass.set("Staff", 10)?;
    item_weapon_subclass.set("Bearclaw", 11)?;
    item_weapon_subclass.set("Catclaw", 12)?;
    item_weapon_subclass.set("Unarmed", 13)?;
    item_weapon_subclass.set("Generic", 14)?;
    item_weapon_subclass.set("Dagger", 15)?;
    item_weapon_subclass.set("Thrown", 16)?;
    item_weapon_subclass.set("Obsolete3", 17)?;
    item_weapon_subclass.set("Crossbow", 18)?;
    item_weapon_subclass.set("Wand", 19)?;
    item_weapon_subclass.set("Fishingpole", 20)?;
    enum_table.set("ItemWeaponSubclass", item_weapon_subclass)?;

    // Enum.ItemArmorSubclass - armor subclass types
    let item_armor_subclass = lua.create_table()?;
    item_armor_subclass.set("Generic", 0)?;
    item_armor_subclass.set("Cloth", 1)?;
    item_armor_subclass.set("Leather", 2)?;
    item_armor_subclass.set("Mail", 3)?;
    item_armor_subclass.set("Plate", 4)?;
    item_armor_subclass.set("Cosmetic", 5)?;
    item_armor_subclass.set("Shield", 6)?;
    item_armor_subclass.set("Libram", 7)?;
    item_armor_subclass.set("Idol", 8)?;
    item_armor_subclass.set("Totem", 9)?;
    item_armor_subclass.set("Sigil", 10)?;
    item_armor_subclass.set("Relic", 11)?;
    enum_table.set("ItemArmorSubclass", item_armor_subclass)?;

    // Enum.ItemQuality - item quality/rarity levels
    let item_quality = lua.create_table()?;
    item_quality.set("Poor", 0)?;
    item_quality.set("Common", 1)?;
    item_quality.set("Uncommon", 2)?;
    item_quality.set("Good", 2)?; // Alias for Uncommon (used by Auctionator)
    item_quality.set("Rare", 3)?;
    item_quality.set("Epic", 4)?;
    item_quality.set("Legendary", 5)?;
    item_quality.set("Artifact", 6)?;
    item_quality.set("Heirloom", 7)?;
    item_quality.set("WoWToken", 8)?;
    enum_table.set("ItemQuality", item_quality)?;

    // Enum.ItemMiscellaneousSubclass - miscellaneous item subclasses
    let item_misc_subclass = lua.create_table()?;
    item_misc_subclass.set("Junk", 0)?;
    item_misc_subclass.set("Reagent", 1)?;
    item_misc_subclass.set("CompanionPet", 2)?;
    item_misc_subclass.set("Holiday", 3)?;
    item_misc_subclass.set("Other", 4)?;
    item_misc_subclass.set("Mount", 5)?;
    item_misc_subclass.set("MountEquipment", 6)?;
    enum_table.set("ItemMiscellaneousSubclass", item_misc_subclass)?;

    // Enum.MountTypeMeta - mount type metadata
    let mount_type_meta = lua.create_table()?;
    mount_type_meta.set("NumValues", 20)?; // Approximate number of mount types
    enum_table.set("MountTypeMeta", mount_type_meta)?;

    // Enum.AddOnEnableState - addon enable states
    let addon_enable_state = lua.create_table()?;
    addon_enable_state.set("None", 0)?;
    addon_enable_state.set("Some", 1)?;
    addon_enable_state.set("All", 2)?;
    enum_table.set("AddOnEnableState", addon_enable_state)?;

    // Enum.WeeklyRewardChestThresholdType - Great Vault reward types
    let weekly_reward_threshold = lua.create_table()?;
    weekly_reward_threshold.set("None", 0)?;
    weekly_reward_threshold.set("Activities", 1)?;
    weekly_reward_threshold.set("Raid", 2)?;
    weekly_reward_threshold.set("MythicPlus", 3)?;
    weekly_reward_threshold.set("RankedPvP", 4)?;
    weekly_reward_threshold.set("World", 5)?;
    enum_table.set("WeeklyRewardChestThresholdType", weekly_reward_threshold)?;

    // Enum.TransmogCollectionType - transmog appearance collection categories
    let transmog_collection_type = lua.create_table()?;
    transmog_collection_type.set("Head", 0)?;
    transmog_collection_type.set("Shoulder", 1)?;
    transmog_collection_type.set("Back", 2)?;
    transmog_collection_type.set("Chest", 3)?;
    transmog_collection_type.set("Shirt", 4)?;
    transmog_collection_type.set("Tabard", 5)?;
    transmog_collection_type.set("Wrist", 6)?;
    transmog_collection_type.set("Hands", 7)?;
    transmog_collection_type.set("Waist", 8)?;
    transmog_collection_type.set("Legs", 9)?;
    transmog_collection_type.set("Feet", 10)?;
    transmog_collection_type.set("Wand", 11)?;
    transmog_collection_type.set("OneHAxe", 12)?;
    transmog_collection_type.set("OneHSword", 13)?;
    transmog_collection_type.set("OneHMace", 14)?;
    transmog_collection_type.set("Dagger", 15)?;
    transmog_collection_type.set("Fist", 16)?;
    transmog_collection_type.set("TwoHAxe", 17)?;
    transmog_collection_type.set("TwoHSword", 18)?;
    transmog_collection_type.set("TwoHMace", 19)?;
    transmog_collection_type.set("Staff", 20)?;
    transmog_collection_type.set("Polearm", 21)?;
    transmog_collection_type.set("Bow", 22)?;
    transmog_collection_type.set("Gun", 23)?;
    transmog_collection_type.set("Crossbow", 24)?;
    transmog_collection_type.set("Warglaives", 25)?;
    transmog_collection_type.set("Paired", 26)?;
    transmog_collection_type.set("Shield", 27)?;
    transmog_collection_type.set("Holdable", 28)?;
    enum_table.set("TransmogCollectionType", transmog_collection_type)?;

    // Enum.GarrisonType - garrison/expansion landing page types
    let garrison_type = lua.create_table()?;
    garrison_type.set("Type_6_0", 2)?;
    garrison_type.set("Type_7_0", 3)?;
    garrison_type.set("Type_8_0", 111)?;
    garrison_type.set("Type_9_0", 123)?;
    garrison_type.set("Type_10_0", 124)?;
    garrison_type.set("Type_11_0", 125)?;
    enum_table.set("GarrisonType", garrison_type)?;

    // Enum.HousingItemToastType - housing item acquisition toast types
    let housing_toast_type = lua.create_table()?;
    housing_toast_type.set("None", 0)?;
    housing_toast_type.set("PlaceableItem", 1)?;
    housing_toast_type.set("Cosmetic", 2)?;
    enum_table.set("HousingItemToastType", housing_toast_type)?;

    // Enum.HouseEditorMode - housing editor mode types
    let house_editor_mode = lua.create_table()?;
    house_editor_mode.set("None", 0)?;
    house_editor_mode.set("BasicDecor", 1)?;
    house_editor_mode.set("ExpertDecor", 2)?;
    house_editor_mode.set("Layout", 3)?;
    house_editor_mode.set("Customize", 4)?;
    enum_table.set("HouseEditorMode", house_editor_mode)?;

    // Enum.EditModeSettingDisplayType - types of setting controls in Edit Mode
    let edit_mode_setting_type = lua.create_table()?;
    edit_mode_setting_type.set("Dropdown", 0)?;
    edit_mode_setting_type.set("Checkbox", 1)?;
    edit_mode_setting_type.set("Slider", 2)?;
    enum_table.set("EditModeSettingDisplayType", edit_mode_setting_type)?;

    // Enum.UITextureSliceMode - nine-slice texture modes
    let ui_texture_slice_mode = lua.create_table()?;
    ui_texture_slice_mode.set("Stretched", 0)?;
    ui_texture_slice_mode.set("Tiled", 1)?;
    enum_table.set("UITextureSliceMode", ui_texture_slice_mode)?;

    // Enum.UIMapType - types of UI maps
    let ui_map_type = lua.create_table()?;
    ui_map_type.set("Cosmic", 0)?;
    ui_map_type.set("World", 1)?;
    ui_map_type.set("Continent", 2)?;
    ui_map_type.set("Zone", 3)?;
    ui_map_type.set("Dungeon", 4)?;
    ui_map_type.set("Micro", 5)?;
    ui_map_type.set("Orphan", 6)?;
    enum_table.set("UIMapType", ui_map_type)?;

    // Enum.QuestClassification - quest types
    let quest_classification = lua.create_table()?;
    quest_classification.set("Normal", 0)?;
    quest_classification.set("Questline", 1)?;
    quest_classification.set("Important", 2)?;
    quest_classification.set("Legendary", 3)?;
    quest_classification.set("Campaign", 4)?;
    quest_classification.set("Calling", 5)?;
    quest_classification.set("Meta", 6)?;
    quest_classification.set("Recurring", 7)?;
    quest_classification.set("BonusObjective", 8)?;
    quest_classification.set("Threat", 9)?;
    quest_classification.set("WorldQuest", 10)?;
    enum_table.set("QuestClassification", quest_classification)?;

    // Enum.QuestTagType - quest tag types (world quests, etc)
    let quest_tag_type = lua.create_table()?;
    quest_tag_type.set("Tag", 0)?;
    quest_tag_type.set("Profession", 1)?;
    quest_tag_type.set("Normal", 2)?;
    quest_tag_type.set("PvP", 3)?;
    quest_tag_type.set("PetBattle", 4)?;
    quest_tag_type.set("Bounty", 5)?;
    quest_tag_type.set("Dungeon", 6)?;
    quest_tag_type.set("Invasion", 7)?;
    quest_tag_type.set("Raid", 8)?;
    quest_tag_type.set("Contribution", 9)?;
    quest_tag_type.set("RatedReward", 10)?;
    quest_tag_type.set("InvasionWrapper", 11)?;
    quest_tag_type.set("FactionAssault", 12)?;
    quest_tag_type.set("Islands", 13)?;
    quest_tag_type.set("Threat", 14)?;
    quest_tag_type.set("CovenantCalling", 15)?;
    quest_tag_type.set("DragonRiderRacing", 16)?;
    quest_tag_type.set("Capstone", 17)?;
    quest_tag_type.set("WorldBoss", 18)?;
    enum_table.set("QuestTagType", quest_tag_type)?;

    // Enum.QuestTag - legacy quest tags (dungeon, raid, etc)
    let quest_tag = lua.create_table()?;
    quest_tag.set("Dungeon", 62)?;
    quest_tag.set("Raid", 63)?;
    quest_tag.set("Raid10", 82)?;
    quest_tag.set("Raid25", 83)?;
    quest_tag.set("Scenario", 98)?;
    quest_tag.set("Group", 1)?;
    quest_tag.set("Heroic", 104)?;
    quest_tag.set("PvP", 41)?;
    quest_tag.set("Account", 102)?;
    quest_tag.set("Legendary", 128)?;
    quest_tag.set("Delve", 288)?;
    enum_table.set("QuestTag", quest_tag)?;

    // Enum.ContentTrackingTargetType - content tracking types (adventures, etc)
    let content_tracking = lua.create_table()?;
    content_tracking.set("JournalEncounter", 0)?;
    content_tracking.set("Vendor", 1)?;
    content_tracking.set("Achievement", 2)?;
    content_tracking.set("Profession", 3)?;
    content_tracking.set("Quest", 4)?;
    enum_table.set("ContentTrackingTargetType", content_tracking)?;

    // Enum.QuestRewardContextFlags - quest reward context flags
    let quest_reward_context = lua.create_table()?;
    quest_reward_context.set("None", 0)?;
    quest_reward_context.set("FirstCompletionBonus", 1)?;
    quest_reward_context.set("RepeatCompletionBonus", 2)?;
    enum_table.set("QuestRewardContextFlags", quest_reward_context)?;

    // Enum.HousingPlotOwnerType - housing plot owner types (Delves housing)
    let housing_owner_type = lua.create_table()?;
    housing_owner_type.set("None", 0)?;
    housing_owner_type.set("Stranger", 1)?;
    housing_owner_type.set("Friend", 2)?;
    housing_owner_type.set("Self", 3)?;
    enum_table.set("HousingPlotOwnerType", housing_owner_type)?;

    // Enum.QuestCompleteSpellType - quest reward spell types
    let quest_complete_spell_type = lua.create_table()?;
    quest_complete_spell_type.set("Follower", 0)?;
    quest_complete_spell_type.set("Companion", 1)?;
    quest_complete_spell_type.set("Tradeskill", 2)?;
    quest_complete_spell_type.set("Ability", 3)?;
    quest_complete_spell_type.set("Aura", 4)?;
    quest_complete_spell_type.set("Spell", 5)?;
    enum_table.set("QuestCompleteSpellType", quest_complete_spell_type)?;

    // Enum.WorldQuestQuality - world quest quality/rarity levels
    let world_quest_quality = lua.create_table()?;
    world_quest_quality.set("Common", 1)?;
    world_quest_quality.set("Rare", 2)?;
    world_quest_quality.set("Epic", 3)?;
    enum_table.set("WorldQuestQuality", world_quest_quality)?;

    // Enum.TooltipDataType - tooltip data types for TooltipDataProcessor
    let tooltip_data_type = lua.create_table()?;
    tooltip_data_type.set("Item", 0)?;
    tooltip_data_type.set("Spell", 1)?;
    tooltip_data_type.set("Unit", 2)?;
    tooltip_data_type.set("Corpse", 3)?;
    tooltip_data_type.set("Object", 4)?;
    tooltip_data_type.set("Currency", 5)?;
    tooltip_data_type.set("Achievement", 6)?;
    tooltip_data_type.set("Quest", 7)?;
    tooltip_data_type.set("QuestItem", 8)?;
    tooltip_data_type.set("BattlePet", 9)?;
    tooltip_data_type.set("CompanionPet", 10)?;
    tooltip_data_type.set("Mount", 11)?;
    tooltip_data_type.set("Macro", 12)?;
    tooltip_data_type.set("EquipmentSet", 13)?;
    tooltip_data_type.set("Hyperlink", 14)?;
    tooltip_data_type.set("Toy", 15)?;
    tooltip_data_type.set("RecipeRankInfo", 16)?;
    tooltip_data_type.set("Totem", 17)?;
    tooltip_data_type.set("UnitAura", 18)?;
    tooltip_data_type.set("QuestPartyProgress", 19)?; // For AllTheThings
    tooltip_data_type.set("InstanceLock", 20)?; // For AllTheThings
    tooltip_data_type.set("MinimapMouseover", 21)?;
    tooltip_data_type.set("CorruptionReplacementEffect", 22)?;
    enum_table.set("TooltipDataType", tooltip_data_type)?;

    // Enum.PlayerInteractionType - types of NPC interactions
    let player_interaction = lua.create_table()?;
    player_interaction.set("None", 0)?;
    player_interaction.set("TradeSkill", 1)?;
    player_interaction.set("MailInfo", 2)?;
    player_interaction.set("Merchant", 3)?; // For Baganator
    player_interaction.set("Banker", 4)?;
    player_interaction.set("MerchantGuild", 5)?;
    player_interaction.set("GuildBanker", 6)?;
    player_interaction.set("Registrar", 7)?;
    player_interaction.set("Vendor", 8)?;
    player_interaction.set("Trainer", 9)?;
    player_interaction.set("Gossip", 10)?;
    player_interaction.set("QuestGiver", 11)?;
    player_interaction.set("TaxiNode", 12)?;
    player_interaction.set("Auctioneer", 13)?;
    player_interaction.set("ItemUpgrade", 14)?;
    player_interaction.set("Transmogrifier", 15)?;
    player_interaction.set("VoidStorageBanker", 16)?;
    player_interaction.set("BlackMarketAuctioneer", 17)?;
    player_interaction.set("AdventureMap", 18)?;
    player_interaction.set("ScrappingMachine", 19)?;
    player_interaction.set("ItemInteraction", 20)?;
    player_interaction.set("ChromieTime", 21)?;
    player_interaction.set("Soulbind", 22)?;
    player_interaction.set("CovenantSanctum", 23)?;
    player_interaction.set("AnimaDiversion", 24)?;
    player_interaction.set("LegendaryCrafting", 25)?;
    player_interaction.set("WeeklyRewards", 26)?;
    player_interaction.set("Renown", 27)?;
    player_interaction.set("PerkProgram", 28)?;
    player_interaction.set("MajorFaction", 29)?;
    player_interaction.set("Delves", 30)?;
    player_interaction.set("TradePartner", 31)?;
    player_interaction.set("Barbershop", 32)?;
    player_interaction.set("AlliedRaceDetails", 33)?;
    player_interaction.set("ProfessionsCraftingOrder", 34)?;
    player_interaction.set("Professions", 35)?;
    player_interaction.set("ProfessionsCustomerOrder", 36)?;
    enum_table.set("PlayerInteractionType", player_interaction)?;

    // Enum.SpellBookSpellBank - spellbook spell storage banks
    let spell_book_spell_bank = lua.create_table()?;
    spell_book_spell_bank.set("Player", 0)?;
    spell_book_spell_bank.set("Pet", 1)?;
    enum_table.set("SpellBookSpellBank", spell_book_spell_bank)?;

    // Enum.SpellBookItemType - types of items in spellbook
    let spell_book_item_type = lua.create_table()?;
    spell_book_item_type.set("None", 0)?;
    spell_book_item_type.set("Spell", 1)?;
    spell_book_item_type.set("Flyout", 2)?;
    spell_book_item_type.set("PetAction", 3)?;
    spell_book_item_type.set("FutureSpell", 4)?;
    enum_table.set("SpellBookItemType", spell_book_item_type)?;

    // Enum.CompressionMethod - compression methods for C_EncodingUtil
    let compression_method = lua.create_table()?;
    compression_method.set("Deflate", 0)?;
    compression_method.set("Huffman", 1)?;
    enum_table.set("CompressionMethod", compression_method)?;

    // Enum.PowerType - power/resource types
    let power_type = lua.create_table()?;
    power_type.set("Mana", 0)?;
    power_type.set("Rage", 1)?;
    power_type.set("Focus", 2)?;
    power_type.set("Energy", 3)?;
    power_type.set("ComboPoints", 4)?;
    power_type.set("Runes", 5)?;
    power_type.set("RunicPower", 6)?;
    power_type.set("SoulShards", 7)?;
    power_type.set("LunarPower", 8)?;
    power_type.set("HolyPower", 9)?;
    power_type.set("Alternate", 10)?;
    power_type.set("Maelstrom", 11)?;
    power_type.set("Chi", 12)?;
    power_type.set("Insanity", 13)?;
    power_type.set("Obsolete", 14)?;
    power_type.set("Obsolete2", 15)?;
    power_type.set("ArcaneCharges", 16)?;
    power_type.set("Fury", 17)?;
    power_type.set("Pain", 18)?;
    power_type.set("Essence", 19)?;
    power_type.set("RuneBlood", 20)?;
    power_type.set("RuneFrost", 21)?;
    power_type.set("RuneUnholy", 22)?;
    power_type.set("AlternateQuest", 23)?;
    power_type.set("AlternateEncounter", 24)?;
    power_type.set("AlternateMount", 25)?;
    power_type.set("NumPowerTypes", 26)?;
    power_type.set("HealthCost", -2)?;
    enum_table.set("PowerType", power_type)?;

    globals.set("Enum", enum_table)?;

    // C_UIColor namespace (color utilities)
    let c_ui_color = lua.create_table()?;
    let get_colors = lua.create_function(|lua, ()| {
        // Return an empty table of colors
        lua.create_table()
    })?;
    c_ui_color.set("GetColors", get_colors)?;
    globals.set("C_UIColor", c_ui_color)?;

    // C_ClassColor namespace
    let c_class_color = lua.create_table()?;
    let get_class_color = lua.create_function(|lua, _class_name: String| {
        // Return a color object with methods (same as CreateColor)
        let r = 1.0f32;
        let g = 1.0f32;
        let b = 1.0f32;
        let a = 1.0f32;

        let color = lua.create_table()?;
        color.set("r", r)?;
        color.set("g", g)?;
        color.set("b", b)?;
        color.set("a", a)?;

        let get_rgb = lua.create_function(move |_, ()| Ok((r, g, b)))?;
        color.set("GetRGB", get_rgb)?;

        let get_rgba = lua.create_function(move |_, ()| Ok((r, g, b, a)))?;
        color.set("GetRGBA", get_rgba)?;

        let generate_hex = lua.create_function(move |lua, ()| {
            let hex = format!("{:02x}{:02x}{:02x}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
            Ok(Value::String(lua.create_string(&hex)?))
        })?;
        color.set("GenerateHexColor", generate_hex)?;

        let wrap_text = lua.create_function(move |lua, text: String| {
            let hex = format!("{:02x}{:02x}{:02x}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
            let wrapped = format!("|cff{}{}|r", hex, text);
            Ok(Value::String(lua.create_string(&wrapped)?))
        })?;
        color.set("WrapTextInColorCode", wrap_text)?;

        Ok(color)
    })?;
    c_class_color.set("GetClassColor", get_class_color)?;
    globals.set("C_ClassColor", c_class_color)?;

    // C_GameRules namespace
    let c_game_rules = lua.create_table()?;
    let is_active = lua.create_function(|_, _rule: Value| {
        Ok(false) // No special game rules in simulation
    })?;
    c_game_rules.set("IsGameRuleActive", is_active)?;

    let get_active_game_mode = lua.create_function(|_, ()| {
        Ok(0) // Enum.GameMode.Standard
    })?;
    c_game_rules.set("GetActiveGameMode", get_active_game_mode)?;

    let get_game_rule_as_float = lua.create_function(|_, _rule: Value| {
        Ok(0.0f32) // Default value for numeric game rules
    })?;
    c_game_rules.set("GetGameRuleAsFloat", get_game_rule_as_float)?;

    let is_standard = lua.create_function(|_, ()| {
        Ok(true) // Always standard mode in simulation
    })?;
    c_game_rules.set("IsStandard", is_standard)?;

    globals.set("C_GameRules", c_game_rules)?;

    // C_ScriptedAnimations namespace
    let c_scripted_anims = lua.create_table()?;
    let get_all_effects = lua.create_function(|lua, ()| {
        // Return empty array - no scripted animation effects in simulation
        lua.create_table()
    })?;
    c_scripted_anims.set("GetAllScriptedAnimationEffects", get_all_effects)?;
    globals.set("C_ScriptedAnimations", c_scripted_anims)?;

    // C_Glue namespace (glue screen utilities)
    let c_glue = lua.create_table()?;
    let is_on_glue_screen = lua.create_function(|_, ()| {
        Ok(false) // Not on character select/login screen
    })?;
    c_glue.set("IsOnGlueScreen", is_on_glue_screen)?;
    globals.set("C_Glue", c_glue)?;

    // IsPublicTestClient() - returns true if running on PTR
    globals.set("IsPublicTestClient", lua.create_function(|_, ()| Ok(false))?)?;

    // IsBetaBuild() - returns true if running on beta
    globals.set("IsBetaBuild", lua.create_function(|_, ()| Ok(false))?)?;

    // IsPublicBuild() - returns true if running on live servers
    globals.set("IsPublicBuild", lua.create_function(|_, ()| Ok(true))?)?;

    // Unit info functions (stubs for simulation)
    let unit_race = lua.create_function(|lua, _unit: String| {
        // Return: raceName, raceFile
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("Human")?),
            Value::String(lua.create_string("Human")?),
        ]))
    })?;
    globals.set("UnitRace", unit_race)?;

    let unit_sex = lua.create_function(|_, _unit: String| {
        // Return: 2 for male, 3 for female (matches Enum.UnitSex)
        Ok(2)
    })?;
    globals.set("UnitSex", unit_sex)?;

    let unit_class = lua.create_function(|lua, _unit: String| {
        // Return: className, classFile, classID
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("Warrior")?),
            Value::String(lua.create_string("WARRIOR")?),
            Value::Integer(1),
        ]))
    })?;
    globals.set("UnitClass", unit_class)?;

    // UnitClassBase(unit) - Returns class file name only (no localization)
    let unit_class_base = lua.create_function(|lua, _unit: String| {
        // Return: classFile only
        Ok(Value::String(lua.create_string("WARRIOR")?))
    })?;
    globals.set("UnitClassBase", unit_class_base)?;

    // GetNumClasses() - Returns number of playable classes
    globals.set("GetNumClasses", lua.create_function(|_, ()| Ok(13i32))?)?; // 13 classes in retail

    // GetClassInfo(classIndex) - Returns className, classFile, classID
    globals.set(
        "GetClassInfo",
        lua.create_function(|lua, class_index: i32| {
            let (name, file) = match class_index {
                1 => ("Warrior", "WARRIOR"),
                2 => ("Paladin", "PALADIN"),
                3 => ("Hunter", "HUNTER"),
                4 => ("Rogue", "ROGUE"),
                5 => ("Priest", "PRIEST"),
                6 => ("Death Knight", "DEATHKNIGHT"),
                7 => ("Shaman", "SHAMAN"),
                8 => ("Mage", "MAGE"),
                9 => ("Warlock", "WARLOCK"),
                10 => ("Monk", "MONK"),
                11 => ("Druid", "DRUID"),
                12 => ("Demon Hunter", "DEMONHUNTER"),
                13 => ("Evoker", "EVOKER"),
                _ => ("Unknown", "UNKNOWN"),
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::String(lua.create_string(file)?),
                Value::Integer(class_index as i64),
            ]))
        })?,
    )?;

    // LocalizedClassList(isFemale) - Returns table mapping classFile to localized name
    globals.set(
        "LocalizedClassList",
        lua.create_function(|lua, _is_female: Option<bool>| {
            let classes = lua.create_table()?;
            classes.set("WARRIOR", "Warrior")?;
            classes.set("PALADIN", "Paladin")?;
            classes.set("HUNTER", "Hunter")?;
            classes.set("ROGUE", "Rogue")?;
            classes.set("PRIEST", "Priest")?;
            classes.set("DEATHKNIGHT", "Death Knight")?;
            classes.set("SHAMAN", "Shaman")?;
            classes.set("MAGE", "Mage")?;
            classes.set("WARLOCK", "Warlock")?;
            classes.set("MONK", "Monk")?;
            classes.set("DRUID", "Druid")?;
            classes.set("DEMONHUNTER", "Demon Hunter")?;
            classes.set("EVOKER", "Evoker")?;
            Ok(classes)
        })?,
    )?;

    // GetWeaponEnchantInfo() - Returns mainhand/offhand enchant info
    globals.set(
        "GetWeaponEnchantInfo",
        lua.create_function(|_, ()| {
            // Returns: hasMainHandEnchant, mainHandExpiration, mainHandCharges, mainHandEnchantID,
            //          hasOffHandEnchant, offHandExpiration, offHandCharges, offHandEnchantID
            Ok((false, 0i32, 0i32, 0i32, false, 0i32, 0i32, 0i32))
        })?,
    )?;

    // UnitName(unit) - Return name and realm
    let unit_name = lua.create_function(|lua, unit: String| {
        let name = match unit.as_str() {
            "player" => "SimPlayer",
            _ => "SimUnit",
        };
        // Return: name, realm (realm is nil for same-realm units)
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string(name)?),
            Value::Nil,
        ]))
    })?;
    globals.set("UnitName", unit_name)?;

    // UnitNameUnmodified(unit) - Return raw name (used for BattleTag lookups)
    let unit_name_unmodified = lua.create_function(|lua, unit: String| {
        let name = match unit.as_str() {
            "player" => "SimPlayer",
            _ => "SimUnit",
        };
        // Return: name, realm (realm is nil for same-realm units)
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string(name)?),
            Value::Nil,
        ]))
    })?;
    globals.set("UnitNameUnmodified", unit_name_unmodified)?;

    // UnitFullName(unit) - Return name with realm
    let unit_full_name = lua.create_function(|lua, unit: String| {
        let name = match unit.as_str() {
            "player" => "SimPlayer",
            _ => "SimUnit",
        };
        // Return: name, realm
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string(name)?),
            Value::String(lua.create_string("SimRealm")?),
        ]))
    })?;
    globals.set("UnitFullName", unit_full_name)?;

    // GetUnitName(unit, showServerName) - alias for UnitName with server name option
    let get_unit_name = lua.create_function(|lua, (unit, _show_server): (String, Option<bool>)| {
        let name = match unit.as_str() {
            "player" => "SimPlayer",
            _ => "SimUnit",
        };
        Ok(Value::String(lua.create_string(name)?))
    })?;
    globals.set("GetUnitName", get_unit_name)?;

    // UnitGUID(unit) - Return unit GUID
    let unit_guid = lua.create_function(|lua, unit: String| {
        let guid = match unit.as_str() {
            "player" => "Player-0000-00000001",
            _ => "Creature-0000-00000000",
        };
        Ok(Value::String(lua.create_string(guid)?))
    })?;
    globals.set("UnitGUID", unit_guid)?;

    // UnitLevel(unit) - Return unit level
    let unit_level = lua.create_function(|_, _unit: String| Ok(70))?;
    globals.set("UnitLevel", unit_level)?;

    // UnitExists(unit) - Check if unit exists
    let unit_exists = lua.create_function(|_, unit: String| {
        Ok(matches!(unit.as_str(), "player" | "target" | "pet"))
    })?;
    globals.set("UnitExists", unit_exists)?;

    // UnitFactionGroup(unit) - Return faction
    let unit_faction_group = lua.create_function(|lua, _unit: String| {
        // Return: englishFaction, localizedFaction
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("Alliance")?),
            Value::String(lua.create_string("Alliance")?),
        ]))
    })?;
    globals.set("UnitFactionGroup", unit_faction_group)?;

    // IsLoggedIn() - Check if player is logged in (always true in sim)
    globals.set("IsLoggedIn", lua.create_function(|_, ()| Ok(false))?)?;

    // InGlue() - Check if in glue screen (character selection). Always false in sim.
    globals.set("InGlue", lua.create_function(|_, ()| Ok(false))?)?;

    // Group/raid functions
    globals.set("IsInGroup", lua.create_function(|_, _instance_type: Option<String>| Ok(false))?)?;
    globals.set("IsInRaid", lua.create_function(|_, _instance_type: Option<String>| Ok(false))?)?;
    globals.set("IsInGuild", lua.create_function(|_, ()| Ok(false))?)?;
    // GetGuildInfo(unit) - Returns guild name, rank, rank index for a unit
    // If not in guild or unit doesn't exist, returns nil
    globals.set(
        "GetGuildInfo",
        lua.create_function(|_, _unit: String| {
            // Not in a guild in simulation
            Ok(mlua::MultiValue::new())
        })?,
    )?;
    globals.set("IsInInstance", lua.create_function(|_, ()| {
        // Returns: inInstance, instanceType ("none", "pvp", "arena", "party", "raid", "scenario")
        Ok((false, "none"))
    })?)?;
    globals.set("InCombatLockdown", lua.create_function(|_, ()| {
        // Returns true if in combat lockdown (can't modify protected frames)
        Ok(false)
    })?)?;
    globals.set("GetInstanceInfo", lua.create_function(|_, ()| {
        // Returns: name, instanceType, difficultyID, difficultyName, maxPlayers, dynamicDifficulty, isDynamic, instanceID, instanceGroupSize, LfgDungeonID
        Ok((
            "",        // name
            "none",    // instanceType
            0i32,      // difficultyID
            "",        // difficultyName
            0i32,      // maxPlayers
            0i32,      // dynamicDifficulty
            false,     // isDynamic
            0i32,      // instanceID
            0i32,      // instanceGroupSize
            0i32,      // LfgDungeonID
        ))
    })?)?;
    globals.set("GetNumGroupMembers", lua.create_function(|_, _instance_type: Option<String>| Ok(0))?)?;
    globals.set("GetNumSubgroupMembers", lua.create_function(|_, _instance_type: Option<String>| Ok(0))?)?;
    globals.set("UnitInParty", lua.create_function(|_, _unit: String| Ok(false))?)?;
    globals.set("UnitInRaid", lua.create_function(|_, _unit: String| Ok(Value::Nil))?)?;
    globals.set("GetRaidRosterInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;

    // Unit state functions
    globals.set("UnitIsDeadOrGhost", lua.create_function(|_, _unit: String| Ok(false))?)?;
    globals.set("UnitIsDead", lua.create_function(|_, _unit: String| Ok(false))?)?;
    globals.set("UnitIsGhost", lua.create_function(|_, _unit: String| Ok(false))?)?;
    globals.set("UnitIsAFK", lua.create_function(|_, _unit: String| Ok(false))?)?;
    globals.set("UnitIsDND", lua.create_function(|_, _unit: String| Ok(false))?)?;
    globals.set("UnitIsConnected", lua.create_function(|_, _unit: String| Ok(true))?)?;
    globals.set("UnitIsPlayer", lua.create_function(|_, unit: String| Ok(unit == "player"))?)?;
    globals.set("UnitPlayerControlled", lua.create_function(|_, unit: String| Ok(unit == "player" || unit == "pet"))?)?;

    // Unit aura functions - no auras in simulation, return nil/empty
    globals.set(
        "UnitAura",
        lua.create_function(|_, (_unit, _index, _filter): (String, i32, Option<String>)| {
            // Returns: name, icon, count, debuffType, duration, expirationTime, source, isStealable,
            // nameplateShowPersonal, spellId, canApplyAura, isBossDebuff, castByPlayer, nameplateShowAll, timeMod, ...
            Ok(Value::Nil) // No auras
        })?,
    )?;
    globals.set(
        "UnitBuff",
        lua.create_function(|_, (_unit, _index, _filter): (String, i32, Option<String>)| {
            // Same returns as UnitAura but specifically for buffs
            Ok(Value::Nil)
        })?,
    )?;
    globals.set(
        "UnitDebuff",
        lua.create_function(|_, (_unit, _index, _filter): (String, i32, Option<String>)| {
            // Same returns as UnitAura but specifically for debuffs
            Ok(Value::Nil)
        })?,
    )?;
    globals.set(
        "GetPlayerAuraBySpellID",
        lua.create_function(|_, _spell_id: i32| {
            // Returns aura info for player by spell ID
            Ok(Value::Nil)
        })?,
    )?;

    // AuraUtil - utility functions for auras
    let aura_util = lua.create_table()?;
    aura_util.set(
        "ForEachAura",
        lua.create_function(|_, (_unit, _filter, _max_count, _callback, _use_packed): (String, String, Option<i32>, mlua::Function, Option<bool>)| {
            // No auras in simulation, so callback is never called
            Ok(())
        })?,
    )?;
    aura_util.set(
        "FindAura",
        lua.create_function(|_, (_predicate, _unit, _filter, _spell_id, _caster): (mlua::Function, String, String, Option<i32>, Option<String>)| {
            // No auras in simulation
            Ok(Value::Nil)
        })?,
    )?;
    aura_util.set(
        "UnpackAuraData",
        lua.create_function(|_, _aura_data: Value| {
            // Unpack aura data table into individual values
            // Returns: name, icon, count, dispelType, duration, expirationTime, source, isStealable,
            //          nameplateShowPersonal, spellId, canApplyAura, isBossDebuff, castByPlayer, nameplateShowAll, timeMod
            Ok(Value::Nil)
        })?,
    )?;
    aura_util.set(
        "FindAuraByName",
        lua.create_function(|_, (_name, _unit, _filter): (String, String, String)| {
            Ok(Value::Nil)
        })?,
    )?;
    globals.set("AuraUtil", aura_util)?;

    // Realm/server functions
    globals.set("GetAutoCompleteRealms", lua.create_function(|lua, ()| lua.create_table())?)?;
    globals.set("GetRealmName", lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("SimRealm")?)))?)?;
    globals.set("GetNormalizedRealmName", lua.create_function(|lua, ()| Ok(Value::String(lua.create_string("SimRealm")?)))?)?;
    globals.set("GetRealmID", lua.create_function(|_, ()| Ok(1i32))?)?; // Return mock realm ID

    // Secure state driver functions
    globals.set(
        "RegisterStateDriver",
        lua.create_function(|_, (_frame, _attribute, _state_driver): (Value, String, String)| {
            // Secure state drivers are not fully implemented in simulation
            Ok(())
        })?,
    )?;
    globals.set(
        "UnregisterStateDriver",
        lua.create_function(|_, (_frame, _attribute): (Value, String)| Ok(()))?,
    )?;
    globals.set(
        "RegisterAttributeDriver",
        lua.create_function(|_, (_frame, _attribute, _driver): (Value, String, String)| Ok(()))?,
    )?;
    globals.set(
        "UnregisterAttributeDriver",
        lua.create_function(|_, (_frame, _attribute): (Value, String)| Ok(()))?,
    )?;

    // Battle.net functions
    globals.set("BNFeaturesEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNFeaturesEnabledAndConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNGetFriendInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    globals.set("BNGetNumFriends", lua.create_function(|_, ()| Ok((0, 0)))?)?; // online, total
    globals.set("BNGetInfo", lua.create_function(|lua, ()| {
        // Return: presenceID, battleTag, toonID, currentBroadcast, bnetAFK, bnetDND, isRIDEnabled
        Ok((
            Value::Integer(0),
            Value::String(lua.create_string("SimPlayer#0000")?),
            Value::Nil,
            Value::String(lua.create_string("")?),
            Value::Boolean(false),
            Value::Boolean(false),
            Value::Boolean(false),
        ))
    })?)?;

    // Specialization functions
    globals.set("GetSpecialization", lua.create_function(|_, ()| Ok(1))?)?; // Returns current spec index (1-4)
    globals.set("GetSpecializationInfo", lua.create_function(|lua, spec_index: i32| {
        // Returns: specID, name, description, icon, role, primaryStat
        let (id, name, role) = match spec_index {
            1 => (62, "Arcane", "DAMAGER"),
            2 => (63, "Fire", "DAMAGER"),
            3 => (64, "Frost", "DAMAGER"),
            _ => (62, "Arcane", "DAMAGER"),
        };
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(id),
            Value::String(lua.create_string(name)?),
            Value::String(lua.create_string("Spec description")?),
            Value::Integer(136116), // icon texture ID
            Value::String(lua.create_string(role)?),
            Value::Integer(4), // primaryStat (INT)
        ]))
    })?)?;
    globals.set("GetNumSpecializations", lua.create_function(|_, ()| Ok(4))?)?;
    globals.set("GetNumSpecializationsForClassID", lua.create_function(|_, _class_id: i32| Ok(3))?)?;
    globals.set("GetSpecializationInfoByID", lua.create_function(|lua, _spec_id: i32| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(62),
            Value::String(lua.create_string("Arcane")?),
            Value::String(lua.create_string("Spec description")?),
            Value::Integer(136116),
            Value::String(lua.create_string("DAMAGER")?),
            Value::String(lua.create_string("MAGE")?),
        ]))
    })?)?;
    // Alias for GetSpecializationInfoByID (used by Cell)
    globals.set("GetSpecializationInfoForSpecID", lua.create_function(|lua, _spec_id: i32| {
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(62),
            Value::String(lua.create_string("Arcane")?),
            Value::String(lua.create_string("Spec description")?),
            Value::Integer(136116),
            Value::String(lua.create_string("DAMAGER")?),
            Value::String(lua.create_string("MAGE")?),
        ]))
    })?)?;
    globals.set("GetSpecializationInfoForClassID", lua.create_function(|lua, (_class_id, spec_index): (i32, i32)| {
        // Returns: specID, name, description, icon, role, isRecommended, isAllowed
        // Return nil if spec_index is out of range (most classes have 3-4 specs)
        if spec_index < 1 || spec_index > 4 {
            return Ok(mlua::MultiValue::new());
        }
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(62i64 + spec_index as i64 - 1),
            Value::String(lua.create_string("Spec")?),
            Value::String(lua.create_string("Description")?),
            Value::Integer(136116),
            Value::String(lua.create_string("DAMAGER")?),
            Value::Boolean(false),
            Value::Boolean(true),
        ]))
    })?)?;
    globals.set("GetSpecializationRoleByID", lua.create_function(|lua, _spec_id: i32| {
        Ok(Value::String(lua.create_string("DAMAGER")?))
    })?)?;

    // GetNumSpecializationsForClassID(classID, sex) - returns number of specs for a class
    globals.set(
        "GetNumSpecializationsForClassID",
        lua.create_function(|_, (_class_id, _sex): (Option<i32>, Option<i32>)| {
            // Most classes have 3 specs, return 0 if classID is nil
            Ok(_class_id.map_or(0, |_| 3i32))
        })?,
    )?;

    // Action bar functions - no actions in simulation
    globals.set("HasAction", lua.create_function(|_, _slot: i32| Ok(false))?)?;
    globals.set("GetActionInfo", lua.create_function(|_, _slot: i32| Ok(Value::Nil))?)?;
    globals.set("GetActionTexture", lua.create_function(|_, _slot: i32| Ok(Value::Nil))?)?;
    globals.set("GetActionText", lua.create_function(|_, _slot: i32| Ok(Value::Nil))?)?;
    globals.set("GetActionCount", lua.create_function(|_, _slot: i32| Ok(0))?)?;
    globals.set("GetActionCooldown", lua.create_function(|_, _slot: i32| {
        // Returns: start, duration, enable, modRate
        Ok((0.0_f64, 0.0_f64, 1, 1.0_f64))
    })?)?;
    globals.set("IsUsableAction", lua.create_function(|_, _slot: i32| Ok((false, false)))?)?;
    globals.set("IsConsumableAction", lua.create_function(|_, _slot: i32| Ok(false))?)?;
    globals.set("IsStackableAction", lua.create_function(|_, _slot: i32| Ok(false))?)?;
    globals.set("IsAttackAction", lua.create_function(|_, _slot: i32| Ok(false))?)?;
    globals.set("IsAutoRepeatAction", lua.create_function(|_, _slot: i32| Ok(false))?)?;
    globals.set("IsCurrentAction", lua.create_function(|_, _slot: i32| Ok(false))?)?;
    globals.set("GetActionCharges", lua.create_function(|_, _slot: i32| {
        // Returns: currentCharges, maxCharges, cooldownStart, cooldownDuration, chargeModRate
        Ok((0, 0, 0.0_f64, 0.0_f64, 1.0_f64))
    })?)?;
    globals.set("GetPossessInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    globals.set("GetActionBarPage", lua.create_function(|_, ()| Ok(1))?)?;
    globals.set("GetBonusBarOffset", lua.create_function(|_, ()| Ok(0))?)?;
    globals.set("GetOverrideBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetVehicleBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetTempShapeshiftBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("IsPossessBarVisible", lua.create_function(|_, ()| Ok(false))?)?;

    // GetCurrentRegion() - Return region ID
    let get_current_region = lua.create_function(|_, ()| {
        // 1=US, 2=Korea, 3=Europe, 4=Taiwan, 5=China
        Ok(1)
    })?;
    globals.set("GetCurrentRegion", get_current_region)?;

    // GetCurrentRegionName() - Return region name
    let get_current_region_name = lua.create_function(|lua, ()| {
        Ok(Value::String(lua.create_string("US")?))
    })?;
    globals.set("GetCurrentRegionName", get_current_region_name)?;

    // GetExpansionLevel() - Returns the current expansion level (0-10)
    // 0 = Classic, 1 = TBC, 2 = WotLK, 3 = Cata, 4 = MoP, 5 = WoD, 6 = Legion, 7 = BfA, 8 = SL, 9 = DF, 10 = TWW
    globals.set(
        "GetExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?, // The War Within
    )?;

    // GetMaxLevelForPlayerExpansion() - Returns max level for player's expansion
    globals.set(
        "GetMaxLevelForPlayerExpansion",
        lua.create_function(|_, ()| Ok(80))?, // TWW max level
    )?;

    // GetMaxLevelForExpansionLevel(expansion) - Returns max level for given expansion
    globals.set(
        "GetMaxLevelForExpansionLevel",
        lua.create_function(|_, expansion: i32| {
            let max_level = match expansion {
                0 => 60,  // Classic
                1 => 70,  // TBC
                2 => 80,  // WotLK
                3 => 85,  // Cata
                4 => 90,  // MoP
                5 => 100, // WoD
                6 => 110, // Legion
                7 => 120, // BfA
                8 => 60,  // Shadowlands (level squish)
                9 => 70,  // Dragonflight
                10 => 80, // The War Within
                _ => 80,
            };
            Ok(max_level)
        })?,
    )?;

    // GetServerExpansionLevel() - Server's expansion level
    globals.set(
        "GetServerExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?,
    )?;

    // GetClientDisplayExpansionLevel() - Client display expansion
    globals.set(
        "GetClientDisplayExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?,
    )?;

    // GetMinimumExpansionLevel() - Minimum expansion level for trial accounts
    globals.set(
        "GetMinimumExpansionLevel",
        lua.create_function(|_, ()| Ok(0))?,
    )?;

    // GetMaximumExpansionLevel() - Maximum expansion level
    globals.set(
        "GetMaximumExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?,
    )?;

    // GetAccountExpansionLevel() - Account's expansion level
    globals.set(
        "GetAccountExpansionLevel",
        lua.create_function(|_, ()| Ok(10))?,
    )?;

    // GetBuildInfo() - Return game version info
    let get_build_info = lua.create_function(|lua, ()| {
        // Returns: version, build, date, tocversion
        // 11.0.7 is The War Within (TWW)
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("11.0.7")?), // version
            Value::String(lua.create_string("58238")?),  // build
            Value::String(lua.create_string("Jan 7 2025")?), // date
            Value::Integer(110007), // tocversion
        ]))
    })?;
    globals.set("GetBuildInfo", get_build_info)?;

    // GetDifficultyInfo(difficultyID) - Return difficulty info
    // GetDifficultyInfo(difficultyID) - Return difficulty info
    // Complete list from https://warcraft.wiki.gg/wiki/DifficultyID
    globals.set(
        "GetDifficultyInfo",
        lua.create_function(|lua, difficulty_id: i32| {
            // Return: name, groupType, isHeroic, isChallengeMode, displayHeroic, displayMythic
            let (name, group_type, is_heroic, is_mythic) = match difficulty_id {
                // Dungeon difficulties
                1 => ("Normal", "party", false, false),
                2 => ("Heroic", "party", true, false),
                8 => ("Mythic Keystone", "party", false, true),
                23 => ("Mythic", "party", false, true),
                24 => ("Timewalking", "party", false, false),
                // Raid difficulties (legacy)
                3 => ("10 Player", "raid", false, false),
                4 => ("25 Player", "raid", false, false),
                5 => ("10 Player (Heroic)", "raid", true, false),
                6 => ("25 Player (Heroic)", "raid", true, false),
                7 => ("Looking For Raid", "raid", false, false),
                9 => ("40 Player", "raid", false, false),
                // Modern raid difficulties
                14 => ("Normal", "raid", false, false),
                15 => ("Heroic", "raid", true, false),
                16 => ("Mythic", "raid", false, true),
                17 => ("Looking For Raid", "raid", false, false),
                33 => ("Timewalking", "raid", false, false),
                // Scenario difficulties
                10 => ("Challenge Mode", "party", false, true),
                11 => ("Heroic Scenario", "scenario", true, false),
                12 => ("Normal Scenario", "scenario", false, false),
                13 => ("Challenge Mode Scenario", "scenario", false, true),
                // Event/special
                18 => ("Event", "raid", false, false),
                19 => ("Event", "party", false, false),
                20 => ("Event Scenario", "scenario", false, false),
                // More instance difficulties (Shadowlands, Dragonflight, TWW, etc.)
                // IDs 21-260 cover various dungeon and instance scenarios
                21..=81 => ("Dungeon", "party", false, false),
                82..=260 => ("Instance", "party", false, false),
                // All other IDs - use generic stub
                _ => ("Instance", "party", false, false),
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::String(lua.create_string(group_type)?),
                Value::Boolean(is_heroic),
                Value::Boolean(false), // isChallengeMode
                Value::Boolean(is_heroic),
                Value::Boolean(is_mythic),
                Value::Nil, // toggleDifficultyID
            ]))
        })?,
    )?;

    // GetRaidDifficultyID() - Return current raid difficulty ID
    globals.set(
        "GetRaidDifficultyID",
        lua.create_function(|_, ()| {
            Ok(14i32) // Normal raid difficulty
        })?,
    )?;

    // GetDungeonDifficultyID() - Return current dungeon difficulty ID
    globals.set(
        "GetDungeonDifficultyID",
        lua.create_function(|_, ()| {
            Ok(1i32) // Normal dungeon difficulty
        })?,
    )?;

    // SetRaidDifficultyID() - Set raid difficulty
    globals.set(
        "SetRaidDifficultyID",
        lua.create_function(|_, _difficulty_id: i32| Ok(()))?,
    )?;

    // SetDungeonDifficultyID() - Set dungeon difficulty
    globals.set(
        "SetDungeonDifficultyID",
        lua.create_function(|_, _difficulty_id: i32| Ok(()))?,
    )?;

    // GetNumShapeshiftForms() - Return number of shapeshift forms
    globals.set(
        "GetNumShapeshiftForms",
        lua.create_function(|_, ()| Ok(0))?,
    )?;

    // GetShapeshiftFormInfo(index) - Return shapeshift form info
    globals.set(
        "GetShapeshiftFormInfo",
        lua.create_function(|_, _index: i32| {
            // Return: texture, name, isActive, isCastable
            Ok(Value::Nil)
        })?,
    )?;

    // GetShapeshiftFormID() - Return current shapeshift form ID (0 if none)
    globals.set(
        "GetShapeshiftFormID",
        lua.create_function(|_, ()| {
            // 0 = no form, 1+ = various forms (Bear, Cat, Travel, etc.)
            Ok(0)
        })?,
    )?;

    // GetPhysicalScreenSize() - Return physical screen dimensions
    let get_physical_screen_size = lua.create_function(|_, ()| {
        // Return simulated 1920x1080 screen
        Ok((1280, 720))
    })?;
    globals.set("GetPhysicalScreenSize", get_physical_screen_size)?;

    // GetScreenWidth() - Return screen width in UI units
    globals.set(
        "GetScreenWidth",
        lua.create_function(|_, ()| Ok(1280.0))?,
    )?;

    // GetScreenHeight() - Return screen height in UI units
    globals.set(
        "GetScreenHeight",
        lua.create_function(|_, ()| Ok(720.0))?,
    )?;

    // UnitAttackSpeed(unit) - Return attack speed info
    globals.set(
        "UnitAttackSpeed",
        lua.create_function(|_, _unit: String| {
            // Return: mainHandSpeed, offHandSpeed
            Ok((2.0, Value::Nil))
        })?,
    )?;

    // GetTexCoordsByGrid(row, col, rows, cols) - Calculate texture coordinates for grid
    globals.set(
        "GetTexCoordsByGrid",
        lua.create_function(|_, (col, row, grid_cols, grid_rows): (i32, i32, Option<i32>, Option<i32>)| {
            let cols = grid_cols.unwrap_or(1);
            let rows = grid_rows.unwrap_or(1);
            if cols == 0 || rows == 0 {
                return Ok((0.0, 1.0, 0.0, 1.0));
            }
            let cell_width = 1.0 / cols as f64;
            let cell_height = 1.0 / rows as f64;
            let left = (col - 1) as f64 * cell_width;
            let right = col as f64 * cell_width;
            let top = (row - 1) as f64 * cell_height;
            let bottom = row as f64 * cell_height;
            Ok((left, right, top, bottom))
        })?,
    )?;

    // IsAddonMessagePrefixRegistered(prefix) - Check if addon message prefix is registered
    globals.set(
        "IsAddonMessagePrefixRegistered",
        lua.create_function(|_, _prefix: String| Ok(false))?,
    )?;

    // RegisterAddonMessagePrefix(prefix) - Register addon message prefix
    globals.set(
        "RegisterAddonMessagePrefix",
        lua.create_function(|_, _prefix: String| Ok(true))?,
    )?;

    // CreateTextureMarkup(file, fileWidth, fileHeight, width, height, left, right, top, bottom) - create texture markup string
    globals.set(
        "CreateTextureMarkup",
        lua.create_function(
            |lua, (file, _file_width, _file_height, width, height, _left, _right, _top, _bottom): (
                String,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<f64>,
                Option<f64>,
                Option<f64>,
                Option<f64>,
            )| {
                let w = width.unwrap_or(0);
                let h = height.unwrap_or(0);
                let markup = format!("|T{}:{}:{}|t", file, h, w);
                Ok(Value::String(lua.create_string(&markup)?))
            },
        )?,
    )?;

    // CreateAtlasMarkup(atlas, width, height, offsetX, offsetY) - create atlas texture markup string
    globals.set(
        "CreateAtlasMarkup",
        lua.create_function(
            |lua, (atlas, width, height, _offset_x, _offset_y): (
                String,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<i32>,
            )| {
                let w = width.unwrap_or(0);
                let h = height.unwrap_or(0);
                let markup = format!("|A:{}:{}:{}|a", atlas, h, w);
                Ok(Value::String(lua.create_string(&markup)?))
            },
        )?,
    )?;

    // SetRaidTargetIconTexture(texture, raidTargetIndex) - set a texture to a raid target marker icon
    globals.set(
        "SetRaidTargetIconTexture",
        lua.create_function(|_, (_texture, _index): (Value, i32)| {
            // This sets a texture to display one of the 8 raid target markers
            // In simulation, we just accept the call without doing anything
            Ok(())
        })?,
    )?;

    // SetPortraitToTexture(texture, fileID) - set portrait texture
    globals.set(
        "SetPortraitToTexture",
        lua.create_function(|_, (_texture, _file_id): (Value, Value)| {
            // Set texture to display a portrait
            Ok(())
        })?,
    )?;

    // CooldownFrame_Set(frame, start, duration, enable) - set cooldown on frame
    globals.set(
        "CooldownFrame_Set",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;

    // GetInventoryItemTexture(unit, slot) - get texture ID for equipped item
    globals.set(
        "GetInventoryItemTexture",
        lua.create_function(|_, (_unit, _slot): (String, i32)| {
            // Return nil - no items equipped in simulation
            Ok(Value::Nil)
        })?,
    )?;

    // GetInventoryItemID(unit, slot) - get item ID for equipped item
    globals.set(
        "GetInventoryItemID",
        lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(Value::Nil))?,
    )?;

    // GetInventoryItemLink(unit, slot) - get item link for equipped item
    globals.set(
        "GetInventoryItemLink",
        lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(Value::Nil))?,
    )?;

    // GetInventoryItemDurability(slot) - get durability for equipped item
    globals.set(
        "GetInventoryItemDurability",
        lua.create_function(|_, _slot: i32| {
            // Returns: current, maximum (nil if item has no durability)
            Ok((100i32, 100i32))
        })?,
    )?;

    // UnitPlayerControlled(unit) - Check if unit is player controlled
    let unit_player_controlled = lua.create_function(|_, unit: String| {
        // Player, party, raid members are player controlled
        Ok(unit.starts_with("player")
            || unit.starts_with("party")
            || unit.starts_with("raid")
            || unit == "pet")
    })?;
    globals.set("UnitPlayerControlled", unit_player_controlled)?;

    // UnitIsTapDenied(unit) - Check if unit is tapped by another player
    let unit_is_tap_denied = lua.create_function(|_, _unit: String| {
        // In simulation, nothing is tapped
        Ok(false)
    })?;
    globals.set("UnitIsTapDenied", unit_is_tap_denied)?;

    // PixelUtil namespace - pixel snapping utilities
    let pixel_util = lua.create_table()?;
    pixel_util.set(
        "SetWidth",
        lua.create_function(|_, (frame, width): (mlua::AnyUserData, f64)| {
            // Just forward to frame:SetWidth
            frame.call_method::<()>("SetWidth", width)?;
            Ok(())
        })?,
    )?;
    pixel_util.set(
        "SetHeight",
        lua.create_function(|_, (frame, height): (mlua::AnyUserData, f64)| {
            frame.call_method::<()>("SetHeight", height)?;
            Ok(())
        })?,
    )?;
    pixel_util.set(
        "SetSize",
        lua.create_function(|_, (frame, width, height): (mlua::AnyUserData, f64, f64)| {
            frame.call_method::<()>("SetSize", (width, height))?;
            Ok(())
        })?,
    )?;
    pixel_util.set(
        "SetPoint",
        lua.create_function(|_, args: mlua::MultiValue| {
            let mut args_iter = args.into_iter();
            if let Some(Value::UserData(frame)) = args_iter.next() {
                // Forward remaining args to frame:SetPoint
                let remaining: Vec<Value> = args_iter.collect();
                frame.call_method::<()>("SetPoint", mlua::MultiValue::from_vec(remaining))?;
            }
            Ok(())
        })?,
    )?;
    pixel_util.set(
        "GetPixelToUIUnitFactor",
        lua.create_function(|_, ()| Ok(1.0))?,
    )?;
    globals.set("PixelUtil", pixel_util)?;

    // Constants table (WoW uses this for various constants)
    let constants_table = lua.create_table()?;
    // LFG role constants
    let lfg_role_constants = lua.create_table()?;
    lfg_role_constants.set("LFG_ROLE_TANK", 0)?;
    lfg_role_constants.set("LFG_ROLE_HEALER", 1)?;
    lfg_role_constants.set("LFG_ROLE_DAMAGE", 2)?;
    lfg_role_constants.set("LFG_ROLE_NO_ROLE", 3)?;
    constants_table.set("LFG_ROLEConstants", lfg_role_constants)?;

    // AccountStoreConsts
    let account_store_consts = lua.create_table()?;
    account_store_consts.set("PlunderstormStoreFrontID", 1)?;
    account_store_consts.set("WowhackStoreFrontID", 2)?;
    constants_table.set("AccountStoreConsts", account_store_consts)?;

    // TraitConsts - talent system constants
    let trait_consts = lua.create_table()?;
    trait_consts.set("VIEW_TRAIT_CONFIG_ID", -1)?; // Special constant for viewing talents
    trait_consts.set("MAX_CONFIG_ID", 0)?;
    constants_table.set("TraitConsts", trait_consts)?;

    globals.set("Constants", constants_table)?;

    // GetCurrentEnvironment() - returns the current global environment table
    let get_current_environment = lua.create_function(|lua, ()| {
        // Return _G (the global environment table)
        Ok(lua.globals())
    })?;
    globals.set("GetCurrentEnvironment", get_current_environment)?;

    // WOW_PROJECT constants
    globals.set("WOW_PROJECT_MAINLINE", 1)?;
    globals.set("WOW_PROJECT_CLASSIC", 2)?;
    globals.set("WOW_PROJECT_BURNING_CRUSADE_CLASSIC", 5)?;
    globals.set("WOW_PROJECT_WRATH_CLASSIC", 11)?;
    globals.set("WOW_PROJECT_CATACLYSM_CLASSIC", 14)?;
    globals.set("WOW_PROJECT_ID", 1)?; // Mainline

    // nop() - no-operation function
    let nop = lua.create_function(|_, _: mlua::MultiValue| {
        Ok(())
    })?;
    globals.set("nop", nop)?;

    // securecallfunction(func, ...) - calls a function in protected mode
    let securecallfunction = lua.create_function(|_, (func, args): (mlua::Function, mlua::MultiValue)| {
        // In WoW this provides taint protection, but for simulation we just call it
        func.call::<mlua::MultiValue>(args)
    })?;
    globals.set("securecallfunction", securecallfunction)?;

    // securecall(func, ...) - alias for securecallfunction
    let securecall = lua.create_function(|_, (func, args): (mlua::Function, mlua::MultiValue)| {
        func.call::<mlua::MultiValue>(args)
    })?;
    globals.set("securecall", securecall)?;

    // secureexecuterange(tbl, func, ...) - calls func(key, value, ...) for each entry in tbl
    let secureexecuterange =
        lua.create_function(|_lua, (tbl, func, args): (mlua::Table, mlua::Function, mlua::MultiValue)| {
            // Iterate through the table and call func(key, value, ...) for each entry
            for pair in tbl.pairs::<mlua::Value, mlua::Value>() {
                if let Ok((key, value)) = pair {
                    let mut call_args = mlua::MultiValue::new();
                    call_args.push_front(value);
                    call_args.push_front(key);
                    // Append the extra arguments
                    for arg in args.iter() {
                        call_args.push_back(arg.clone());
                    }
                    if let Err(e) = func.call::<()>(call_args) {
                        // Log but don't propagate errors (WoW behavior)
                        tracing::warn!("secureexecuterange callback error: {}", e);
                    }
                }
            }
            Ok(())
        })?;
    globals.set("secureexecuterange", secureexecuterange)?;

    // geterrorhandler() - returns error handler function
    let geterrorhandler = lua.create_function(|lua, ()| {
        // Return a simple error handler that just prints
        let handler = lua.create_function(|_, msg: String| {
            println!("Lua error: {}", msg);
            Ok(())
        })?;
        Ok(handler)
    })?;
    globals.set("geterrorhandler", geterrorhandler)?;

    // seterrorhandler(func) - sets the error handler function
    let seterrorhandler = lua.create_function(|_, _handler: mlua::Function| {
        // Accept the handler but don't actually use it (stub)
        Ok(())
    })?;
    globals.set("seterrorhandler", seterrorhandler)?;

    // SecureHandler functions (secure frame management stubs)
    let secure_handler_set_frame_ref = lua.create_function(
        |_, (_frame, _name, _target): (mlua::Value, String, mlua::Value)| Ok(()),
    )?;
    globals.set("SecureHandlerSetFrameRef", secure_handler_set_frame_ref)?;

    let secure_handler_execute = lua.create_function(
        |_, (_frame, _body, _args): (mlua::Value, String, mlua::MultiValue)| Ok(()),
    )?;
    globals.set("SecureHandlerExecute", secure_handler_execute)?;

    let secure_handler_wrap_script = lua.create_function(
        |_, (_frame, _script, _body): (mlua::Value, String, String)| Ok(()),
    )?;
    globals.set("SecureHandlerWrapScript", secure_handler_wrap_script)?;

    // Lua stdlib global aliases (WoW compatibility)
    lua.load(r##"
        -- String library aliases
        strlen = string.len
        strsub = string.sub
        strfind = string.find
        strmatch = string.match
        strbyte = string.byte
        strchar = string.char
        strrep = string.rep
        strrev = string.reverse
        strlower = string.lower
        strupper = string.upper
        strtrim = function(s) return (s:gsub("^%s*(.-)%s*$", "%1")) end
        strsplittable = function(del, str) local t = {} for v in string.gmatch(str, "([^"..del.."]+)") do t[#t+1] = v end return t end
        strjoin = function(delimiter, ...) return table.concat({...}, delimiter) end
        string.join = strjoin
        format = string.format

        -- Add string:split method (WoW extension)
        function string:split(delimiter)
            local result = {}
            local from = 1
            local delim_from, delim_to = string.find(self, delimiter, from, true)
            while delim_from do
                table.insert(result, string.sub(self, from, delim_from - 1))
                from = delim_to + 1
                delim_from, delim_to = string.find(self, delimiter, from, true)
            end
            table.insert(result, string.sub(self, from))
            return result
        end
        gsub = string.gsub
        gmatch = string.gmatch

        -- Math library aliases
        abs = math.abs
        ceil = math.ceil
        floor = math.floor
        max = math.max
        min = math.min
        mod = math.fmod
        sqrt = math.sqrt
        sin = function(x) return math.sin(math.rad(x)) end
        cos = function(x) return math.cos(math.rad(x)) end
        tan = function(x) return math.tan(math.rad(x)) end
        asin = function(x) return math.deg(math.asin(x)) end
        acos = function(x) return math.deg(math.acos(x)) end
        atan = function(x) return math.deg(math.atan(x)) end
        atan2 = function(x, y) return math.deg(math.atan2(x, y)) end
        deg = math.deg
        rad = math.rad
        exp = math.exp
        log = math.log
        log10 = math.log10
        frexp = math.frexp
        ldexp = math.ldexp
        random = math.random
        PI = math.pi

        -- WoW math utility functions
        function Round(value)
            if value < 0 then
                return math.ceil(value - 0.5)
            else
                return math.floor(value + 0.5)
            end
        end

        function Lerp(startValue, endValue, amount)
            return startValue + (endValue - startValue) * amount
        end

        function Clamp(value, min, max)
            if value < min then return min end
            if value > max then return max end
            return value
        end

        function Saturate(value)
            return Clamp(value, 0.0, 1.0)
        end

        function ClampedPercentageBetween(value, min, max)
            if max <= min then return 0.0 end
            return Saturate((value - min) / (max - min))
        end

        -- Table library aliases
        foreach = table.foreach
        foreachi = table.foreachi
        getn = table.getn or function(t) return #t end
        sort = table.sort
        table.wipe = wipe

        -- Bit operations (pure Lua 5.1 implementation)
        bit = {}

        local function tobits(n)
            local t = {}
            while n > 0 do
                t[#t + 1] = n % 2
                n = math.floor(n / 2)
            end
            return t
        end

        local function frombits(t)
            local n = 0
            for i = 1, #t do
                n = n + t[i] * (2 ^ (i - 1))
            end
            return n
        end

        function bit.band(a, b)
            local ta, tb = tobits(a), tobits(b)
            local result = {}
            local len = math.max(#ta, #tb)
            for i = 1, len do
                result[i] = ((ta[i] or 0) == 1 and (tb[i] or 0) == 1) and 1 or 0
            end
            return frombits(result)
        end

        function bit.bor(a, b)
            local ta, tb = tobits(a), tobits(b)
            local result = {}
            local len = math.max(#ta, #tb)
            for i = 1, len do
                result[i] = ((ta[i] or 0) == 1 or (tb[i] or 0) == 1) and 1 or 0
            end
            return frombits(result)
        end

        function bit.bxor(a, b)
            local ta, tb = tobits(a), tobits(b)
            local result = {}
            local len = math.max(#ta, #tb)
            for i = 1, len do
                result[i] = ((ta[i] or 0) ~= (tb[i] or 0)) and 1 or 0
            end
            return frombits(result)
        end

        function bit.bnot(a)
            -- 32-bit NOT
            return 4294967295 - a
        end

        function bit.lshift(a, n)
            return math.floor(a * (2 ^ n)) % 4294967296
        end

        function bit.rshift(a, n)
            return math.floor(a / (2 ^ n))
        end

        function bit.arshift(a, n)
            local r = bit.rshift(a, n)
            if a >= 2147483648 then
                r = r + (2 ^ 32 - 2 ^ (32 - n))
            end
            return r
        end

        function bit.mod(a, b)
            return a % b
        end

        -- Lua 5.2 compatibility: bit32 is an alias for bit with different names
        bit32 = {
            band = bit.band,
            bor = bit.bor,
            bxor = bit.bxor,
            bnot = bit.bnot,
            lshift = bit.lshift,
            rshift = bit.rshift,
            arshift = bit.arshift,
            -- bit32-specific functions
            extract = function(n, field, width)
                width = width or 1
                return bit.band(bit.rshift(n, field), (2 ^ width) - 1)
            end,
            replace = function(n, v, field, width)
                width = width or 1
                local mask = (2 ^ width) - 1
                return bit.bor(bit.band(n, bit.bnot(bit.lshift(mask, field))), bit.lshift(bit.band(v, mask), field))
            end,
            btest = function(...)
                return bit.band(...) ~= 0
            end,
        }

        -- Mixin system (WoW C++ intrinsics)
        function Mixin(object, ...)
            for i = 1, select("#", ...) do
                local mixin = select(i, ...)
                if mixin then
                    for k, v in pairs(mixin) do
                        object[k] = v
                    end
                end
            end
            return object
        end

        function CreateFromMixins(...)
            return Mixin({}, ...)
        end

        function CreateAndInitFromMixin(mixin, ...)
            local object = CreateFromMixins(mixin)
            if object.Init then
                object:Init(...)
            end
            return object
        end

        -- Security functions (always "secure" in simulation)
        function issecure()
            return true
        end

        function issecurevariable(table, variable)
            return true, "secure"
        end

        function forceinsecure()
            -- no-op in simulation
        end

        -- Debug functions
        function debugstack(start, count1, count2)
            start = start or 1
            count1 = count1 or 12
            count2 = count2 or 12

            local result = {}
            local level = start + 1  -- +1 to skip debugstack itself

            for i = 1, count1 do
                local info = debug.getinfo(level, "Sln")
                if not info then break end

                local source = info.source or "?"
                -- Convert @path to just path
                if source:sub(1, 1) == "@" then
                    source = source:sub(2)
                end

                local line = info.currentline or 0
                local name = info.name or ""

                if name ~= "" then
                    table.insert(result, source .. ":" .. line .. ": in function `" .. name .. "'")
                else
                    table.insert(result, source .. ":" .. line .. ": in main chunk")
                end

                level = level + 1
            end

            return table.concat(result, "\n")
        end

        function debuglocals(level)
            return ""
        end

        -- Time functions
        function GetTime()
            return os.clock()
        end

        function GetTimePreciseSec()
            -- High-precision time in seconds (same as GetTime but explicitly high-precision)
            return os.clock()
        end

        function GetServerTime()
            return os.time()
        end

        -- SecondsFormatter class - formats seconds into time strings
        SecondsFormatter = {}
        SecondsFormatter.__index = SecondsFormatter

        -- Constants
        SecondsFormatter.Abbreviation = {
            None = 0,
            Truncate = 1,
            OneLetter = 2,
        }

        -- Interval descriptions (used by WoW for time formatting)
        SecondsFormatter.IntervalDescription = {
            { seconds = 86400, formatString = { "%d Day", "%d Days", "d" } },
            { seconds = 3600, formatString = { "%d Hour", "%d Hours", "h" } },
            { seconds = 60, formatString = { "%d Min", "%d Mins", "m" } },
            { seconds = 1, formatString = { "%d Sec", "%d Secs", "s" } },
        }

        function SecondsFormatter:Init(interval, abbreviation, roundUpLastUnit, convertToLower)
            self.interval = interval or 0
            self.abbreviation = abbreviation or SecondsFormatter.Abbreviation.None
            self.roundUpLastUnit = roundUpLastUnit or false
            self.convertToLower = convertToLower or false
        end

        function SecondsFormatter:SetDesiredUnitCount(count)
            self.desiredUnitCount = count
        end

        function SecondsFormatter:SetStripIntervalWhitespace(strip)
            self.stripIntervalWhitespace = strip
        end

        function SecondsFormatter:Format(seconds)
            if not seconds or seconds < 0 then
                return ""
            end
            local days = math.floor(seconds / 86400)
            local hours = math.floor((seconds % 86400) / 3600)
            local minutes = math.floor((seconds % 3600) / 60)
            local secs = math.floor(seconds % 60)

            if days > 0 then
                return string.format("%d d %02d h", days, hours)
            elseif hours > 0 then
                return string.format("%d h %02d m", hours, minutes)
            elseif minutes > 0 then
                return string.format("%d m %02d s", minutes, secs)
            else
                return string.format("%d s", secs)
            end
        end

        function CreateSecondsFormatter(interval, abbreviation, roundUpLastUnit, convertToLower)
            local formatter = setmetatable({}, SecondsFormatter)
            formatter:Init(interval, abbreviation, roundUpLastUnit, convertToLower)
            return formatter
        end

        -- SecondsFormatterMixin alias (some code uses this)
        SecondsFormatterMixin = SecondsFormatter

        function time()
            return os.time()
        end

        function date(fmt, t)
            return os.date(fmt, t)
        end

        function difftime(t2, t1)
            return os.difftime(t2, t1)
        end

        -- CopyTable - deep copy a table
        function CopyTable(settings, shallow)
            if type(settings) ~= "table" then
                return settings
            end
            local copy = {}
            for k, v in pairs(settings) do
                if type(v) == "table" and not shallow then
                    copy[k] = CopyTable(v, shallow)
                else
                    copy[k] = v
                end
            end
            return copy
        end

        -- MergeTable - merge source into destination
        function MergeTable(destination, source)
            for k, v in pairs(source) do
                destination[k] = v
            end
            return destination
        end

        -- ChatFrame message filter (store filters but don't actually filter in simulation)
        __chatFilters = {}
        function ChatFrame_AddMessageEventFilter(event, filter)
            __chatFilters[event] = __chatFilters[event] or {}
            table.insert(__chatFilters[event], filter)
        end

        function ChatFrame_RemoveMessageEventFilter(event, filter)
            if __chatFilters[event] then
                for i, f in ipairs(__chatFilters[event]) do
                    if f == filter then
                        table.remove(__chatFilters[event], i)
                        break
                    end
                end
            end
        end
    "##).exec()?;

    // C_Timer namespace
    let c_timer = lua.create_table()?;

    // C_Timer.After(seconds, callback) - one-shot timer, no handle returned
    let state_timer_after = Rc::clone(&state);
    let c_timer_after = lua.create_function(move |lua, (seconds, callback): (f64, mlua::Function)| {
        let id = next_timer_id();
        let callback_key = lua.create_registry_value(callback)?;
        let fire_at = Instant::now() + Duration::from_secs_f64(seconds);

        let timer = PendingTimer {
            id,
            fire_at,
            callback_key,
            interval: None,
            remaining: None,
            cancelled: false,
            handle_key: None, // After() doesn't pass handle to callback
        };

        state_timer_after.borrow_mut().timers.push_back(timer);
        Ok(())
    })?;
    c_timer.set("After", c_timer_after)?;

    // C_Timer.NewTicker(seconds, callback, iterations) - repeating timer with handle
    let state_timer_ticker = Rc::clone(&state);
    let c_timer_new_ticker = lua.create_function(move |lua, (seconds, callback, iterations): (f64, mlua::Function, Option<i32>)| {
        let id = next_timer_id();
        let callback_key = lua.create_registry_value(callback)?;
        let fire_at = Instant::now() + Duration::from_secs_f64(seconds);
        let interval = Duration::from_secs_f64(seconds);

        // Create the ticker handle table first so we can pass it to callbacks
        let ticker = lua.create_table()?;
        ticker.set("_id", id)?;
        ticker.set("_cancelled", false)?;

        let state_cancel = Rc::clone(&state_timer_ticker);
        let ticker_clone = ticker.clone();
        let cancel = lua.create_function(move |_, ()| {
            // Mark as cancelled in the handle table
            ticker_clone.set("_cancelled", true)?;
            // Also mark in the timer queue
            let mut state = state_cancel.borrow_mut();
            for timer in state.timers.iter_mut() {
                if timer.id == id {
                    timer.cancelled = true;
                    break;
                }
            }
            Ok(())
        })?;
        ticker.set("Cancel", cancel)?;

        // IsCancelled method checks the _cancelled field
        let ticker_for_is_cancelled = ticker.clone();
        let is_cancelled = lua.create_function(move |_, ()| {
            let cancelled: bool = ticker_for_is_cancelled.get("_cancelled").unwrap_or(false);
            Ok(cancelled)
        })?;
        ticker.set("IsCancelled", is_cancelled)?;

        // Store the handle in registry so we can pass it to callback
        let handle_key = lua.create_registry_value(ticker.clone())?;

        let timer = PendingTimer {
            id,
            fire_at,
            callback_key,
            interval: Some(interval),
            remaining: iterations,
            cancelled: false,
            handle_key: Some(handle_key),
        };

        state_timer_ticker.borrow_mut().timers.push_back(timer);

        Ok(ticker)
    })?;
    c_timer.set("NewTicker", c_timer_new_ticker)?;

    // C_Timer.NewTimer(seconds, callback) - one-shot timer with handle
    let state_timer_new = Rc::clone(&state);
    let c_timer_new_timer = lua.create_function(move |lua, (seconds, callback): (f64, mlua::Function)| {
        let id = next_timer_id();
        let callback_key = lua.create_registry_value(callback)?;
        let fire_at = Instant::now() + Duration::from_secs_f64(seconds);

        // Create the timer handle table first so we can pass it to callback
        let timer_handle = lua.create_table()?;
        timer_handle.set("_id", id)?;
        timer_handle.set("_cancelled", false)?;

        let state_cancel = Rc::clone(&state_timer_new);
        let handle_clone = timer_handle.clone();
        let cancel = lua.create_function(move |_, ()| {
            // Mark as cancelled in the handle table
            handle_clone.set("_cancelled", true)?;
            // Also mark in the timer queue
            let mut state = state_cancel.borrow_mut();
            for timer in state.timers.iter_mut() {
                if timer.id == id {
                    timer.cancelled = true;
                    break;
                }
            }
            Ok(())
        })?;
        timer_handle.set("Cancel", cancel)?;

        // IsCancelled method checks the _cancelled field
        let handle_for_is_cancelled = timer_handle.clone();
        let is_cancelled = lua.create_function(move |_, ()| {
            let cancelled: bool = handle_for_is_cancelled.get("_cancelled").unwrap_or(false);
            Ok(cancelled)
        })?;
        timer_handle.set("IsCancelled", is_cancelled)?;

        // Store the handle in registry so we can pass it to callback
        let handle_key = lua.create_registry_value(timer_handle.clone())?;

        let timer = PendingTimer {
            id,
            fire_at,
            callback_key,
            interval: None,
            remaining: None,
            cancelled: false,
            handle_key: Some(handle_key),
        };

        state_timer_new.borrow_mut().timers.push_back(timer);

        Ok(timer_handle)
    })?;
    c_timer.set("NewTimer", c_timer_new_timer)?;

    globals.set("C_Timer", c_timer)?;

    // C_Map namespace - map and area information
    let c_map = lua.create_table()?;
    c_map.set(
        "GetAreaInfo",
        lua.create_function(|lua, area_id: i32| {
            // Return area name for the given area ID
            // In simulation, return a placeholder
            Ok(Value::String(lua.create_string(&format!("Area_{}", area_id))?))
        })?,
    )?;
    c_map.set(
        "GetMapInfo",
        lua.create_function(|lua, map_id: i32| {
            // Return map info table
            let info = lua.create_table()?;
            info.set("mapID", map_id)?;
            info.set("name", format!("Map_{}", map_id))?;
            info.set("mapType", 3)?; // Zone type
            info.set("parentMapID", 0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_map.set(
        "GetBestMapForUnit",
        lua.create_function(|_, _unit: String| {
            // Return a default map ID
            Ok(Value::Integer(1)) // Durotar
        })?,
    )?;
    c_map.set(
        "GetPlayerMapPosition",
        lua.create_function(|lua, (_map_id, _unit): (i32, String)| {
            // Return a position vector (x, y)
            let pos = lua.create_table()?;
            pos.set("x", 0.5)?;
            pos.set("y", 0.5)?;
            Ok(Value::Table(pos))
        })?,
    )?;
    c_map.set(
        "GetMapChildrenInfo",
        lua.create_function(|lua, (_map_id, _map_type, _all_descendants): (i32, Option<i32>, Option<bool>)| {
            // Return empty table of child maps
            lua.create_table()
        })?,
    )?;
    c_map.set(
        "GetWorldPosFromMapPos",
        lua.create_function(|lua, (map_id, pos): (i32, Value)| {
            // pos is a Vector2DMixin with x, y fields
            // Returns (instanceID, Vector2DMixin with world coordinates)
            let (x, y) = if let Value::Table(ref t) = pos {
                let x: f64 = t.get("x").unwrap_or(0.5);
                let y: f64 = t.get("y").unwrap_or(0.5);
                (x, y)
            } else {
                (0.5, 0.5)
            };
            // Convert map coords (0-1) to world coords (arbitrary scale)
            // Use a simple scale of 1000 units per map
            let world_x = x * 1000.0;
            let world_y = y * 1000.0;
            let world_pos = lua.create_table()?;
            world_pos.set("x", world_x)?;
            world_pos.set("y", world_y)?;
            // Add GetXY method
            world_pos.set(
                "GetXY",
                lua.create_function(move |_, _: Value| Ok((world_x, world_y)))?,
            )?;
            // Instance ID is typically same as map_id for simplicity
            Ok((map_id, world_pos))
        })?,
    )?;
    c_map.set(
        "GetMapWorldSize",
        lua.create_function(|_, _map_id: i32| {
            // Return width, height in world units (arbitrary scale)
            Ok((1000.0f64, 1000.0f64))
        })?,
    )?;
    globals.set("C_Map", c_map)?;

    // Zone text functions
    globals.set(
        "GetRealZoneText",
        lua.create_function(|_, ()| Ok("Stormwind City"))?,
    )?;
    globals.set(
        "GetZoneText",
        lua.create_function(|_, ()| Ok("Stormwind City"))?,
    )?;
    globals.set(
        "GetSubZoneText",
        lua.create_function(|_, ()| Ok("Trade District"))?,
    )?;
    globals.set(
        "GetMinimapZoneText",
        lua.create_function(|_, ()| Ok("Trade District"))?,
    )?;

    // UiMapPoint - map point creation helper
    let ui_map_point = lua.create_table()?;
    ui_map_point.set(
        "CreateFromVector2D",
        lua.create_function(|lua, (map_id, pos): (i32, Value)| {
            // Extract x, y from position table
            let (x, y) = if let Value::Table(ref t) = pos {
                let x: f64 = t.get("x").unwrap_or(0.5);
                let y: f64 = t.get("y").unwrap_or(0.5);
                (x, y)
            } else {
                (0.5, 0.5)
            };
            // Create a map point table
            let point = lua.create_table()?;
            point.set("uiMapID", map_id)?;
            point.set("x", x)?;
            point.set("y", y)?;
            Ok(point)
        })?,
    )?;
    ui_map_point.set(
        "CreateFromCoordinates",
        lua.create_function(|lua, (map_id, x, y): (i32, f64, f64)| {
            let point = lua.create_table()?;
            point.set("uiMapID", map_id)?;
            point.set("x", x)?;
            point.set("y", y)?;
            Ok(point)
        })?,
    )?;
    globals.set("UiMapPoint", ui_map_point)?;

    // C_MapExplorationInfo namespace - map exploration data
    let c_map_exploration = lua.create_table()?;
    c_map_exploration.set(
        "GetExploredAreaIDsAtPosition",
        lua.create_function(|lua, (_map_id, _pos): (i32, Value)| {
            // Return empty table (no explored areas in sim)
            lua.create_table()
        })?,
    )?;
    c_map_exploration.set(
        "GetExploredMapTextures",
        lua.create_function(|lua, _map_id: i32| {
            // Return empty table (no textures in sim)
            lua.create_table()
        })?,
    )?;
    globals.set("C_MapExplorationInfo", c_map_exploration)?;

    // C_DateAndTime namespace - date/time utilities
    let c_date_time = lua.create_table()?;
    c_date_time.set(
        "GetCurrentCalendarTime",
        lua.create_function(|lua, ()| {
            let info = lua.create_table()?;
            info.set("year", 2024)?;
            info.set("month", 1)?;
            info.set("monthDay", 1)?;
            info.set("weekday", 1)?;
            info.set("hour", 12)?;
            info.set("minute", 0)?;
            Ok(info)
        })?,
    )?;
    c_date_time.set(
        "GetServerTimeLocal",
        lua.create_function(|_, ()| Ok(0i64))?,
    )?;
    c_date_time.set(
        "GetSecondsUntilDailyReset",
        lua.create_function(|_, ()| Ok(86400i32))?,
    )?;
    c_date_time.set(
        "GetSecondsUntilWeeklyReset",
        lua.create_function(|_, ()| Ok(604800i32))?,
    )?;
    globals.set("C_DateAndTime", c_date_time)?;

    // C_Minimap namespace - minimap utilities
    let c_minimap = lua.create_table()?;
    c_minimap.set(
        "IsInsideQuestBlob",
        lua.create_function(|_, (_quest_id, _x, _y): (i32, f64, f64)| Ok(false))?,
    )?;
    c_minimap.set(
        "GetViewRadius",
        lua.create_function(|_, ()| Ok(200.0f64))?,
    )?;
    c_minimap.set(
        "SetPlayerTexture",
        lua.create_function(|_, (_file_id, _icon_id): (i32, i32)| Ok(()))?,
    )?;
    globals.set("C_Minimap", c_minimap)?;

    // C_Navigation namespace - quest navigation waypoints
    let c_navigation = lua.create_table()?;
    c_navigation.set(
        "GetFrame",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_navigation.set(
        "GetDistance",
        lua.create_function(|_, ()| Ok(0.0f64))?,
    )?;
    c_navigation.set(
        "GetDestination",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_navigation.set(
        "IsAutoFollowEnabled",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_navigation.set(
        "SetAutoFollowEnabled",
        lua.create_function(|_, _enabled: bool| Ok(()))?,
    )?;
    globals.set("C_Navigation", c_navigation)?;

    // C_QuestLog namespace - quest log utilities
    let c_quest_log = lua.create_table()?;
    c_quest_log.set(
        "IsQuestFlaggedCompleted",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    c_quest_log.set(
        "GetNumQuestLogEntries",
        lua.create_function(|_, ()| Ok((0i32, 0i32)))?,
    )?;
    c_quest_log.set(
        "GetInfo",
        lua.create_function(|_, _quest_index: i32| Ok(Value::Nil))?,
    )?;
    c_quest_log.set(
        "GetQuestIDForLogIndex",
        lua.create_function(|_, _quest_index: i32| Ok(0i32))?,
    )?;
    c_quest_log.set(
        "IsComplete",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    c_quest_log.set(
        "IsOnQuest",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    c_quest_log.set(
        "GetQuestObjectives",
        lua.create_function(|lua, _quest_id: i32| lua.create_table())?,
    )?;
    c_quest_log.set(
        "GetMaxNumQuestsCanAccept",
        lua.create_function(|_, ()| Ok(35i32))?,
    )?;
    c_quest_log.set(
        "GetTitleForQuestID",
        lua.create_function(|lua, _quest_id: i32| {
            Ok(Value::String(lua.create_string("Quest")?))
        })?,
    )?;
    c_quest_log.set(
        "GetQuestTagInfo",
        lua.create_function(|lua, _quest_id: i32| {
            let info = lua.create_table()?;
            info.set("tagID", 0)?;
            info.set("tagName", "Quest")?;
            info.set("worldQuestType", Value::Nil)?;
            info.set("quality", 1)?;
            info.set("isElite", false)?;
            info.set("displayExpiration", false)?;
            Ok(info)
        })?,
    )?;
    globals.set("C_QuestLog", c_quest_log)?;

    // C_TaskQuest namespace - world quest/task utilities
    let c_task_quest = lua.create_table()?;
    c_task_quest.set(
        "IsActive",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    c_task_quest.set(
        "GetQuestsOnMap",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    c_task_quest.set(
        "GetQuestInfoByQuestID",
        lua.create_function(|_, _quest_id: i32| Ok(Value::Nil))?,
    )?;
    c_task_quest.set(
        "GetQuestLocation",
        lua.create_function(|_, (_quest_id, _map_id): (i32, i32)| Ok((0.0f64, 0.0f64)))?,
    )?;
    c_task_quest.set(
        "GetQuestsForPlayerByMapID",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    c_task_quest.set(
        "GetQuestTimeLeftMinutes",
        lua.create_function(|_, _quest_id: i32| Ok(0i32))?,
    )?;
    globals.set("C_TaskQuest", c_task_quest)?;

    // C_TalkingHead namespace - talking head popup utilities
    let c_talking_head = lua.create_table()?;
    c_talking_head.set(
        "GetCurrentLineInfo",
        lua.create_function(|_, ()| {
            Ok((Value::Nil, Value::Nil, Value::Nil, 0i32, Value::Nil))
        })?,
    )?;
    c_talking_head.set(
        "IgnoreCurrentTalkingHead",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    globals.set("C_TalkingHead", c_talking_head)?;

    // C_MerchantFrame namespace - merchant/vendor utilities
    let c_merchant_frame = lua.create_table()?;
    c_merchant_frame.set(
        "IsMerchantItemRefundable",
        lua.create_function(|_, _index: i32| Ok(false))?,
    )?;
    c_merchant_frame.set(
        "GetBuybackItemID",
        lua.create_function(|_, _index: i32| Ok(0i32))?,
    )?;
    globals.set("C_MerchantFrame", c_merchant_frame)?;

    // C_HousingCatalog namespace - player housing utilities
    let c_housing_catalog = lua.create_table()?;
    c_housing_catalog.set(
        "GetAllPlacedFurniture",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_housing_catalog.set(
        "GetFurnitureInfo",
        lua.create_function(|_, _furniture_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_HousingCatalog", c_housing_catalog)?;

    // C_DyeColor namespace - dye/color utilities for player housing
    let c_dye_color = lua.create_table()?;
    c_dye_color.set(
        "GetDyeColorsByItemID",
        lua.create_function(|lua, _item_id: i32| lua.create_table())?,
    )?;
    c_dye_color.set(
        "GetDyeInfo",
        lua.create_function(|_, _dye_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_DyeColor", c_dye_color)?;

    // C_TaxiMap namespace - flight path utilities
    let c_taxi_map = lua.create_table()?;
    c_taxi_map.set(
        "GetAllTaxiNodes",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    c_taxi_map.set(
        "GetTaxiNodesForMap",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    c_taxi_map.set(
        "ShouldMapShowTaxiNodes",
        lua.create_function(|_, _map_id: i32| Ok(true))?,
    )?;
    globals.set("C_TaxiMap", c_taxi_map)?;

    // C_PetJournal namespace - battle pet utilities
    let c_pet_journal = lua.create_table()?;
    c_pet_journal.set(
        "GetNumPets",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_pet_journal.set(
        "GetPetInfoByIndex",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_pet_journal.set(
        "GetPetInfoByPetID",
        lua.create_function(|_, _pet_id: String| Ok(Value::Nil))?,
    )?;
    c_pet_journal.set(
        "GetPetInfoBySpeciesID",
        lua.create_function(|_, _species_id: i32| Ok(Value::Nil))?,
    )?;
    c_pet_journal.set(
        "PetIsSummonable",
        lua.create_function(|_, _pet_id: String| Ok(false))?,
    )?;
    c_pet_journal.set(
        "GetNumCollectedInfo",
        lua.create_function(|_, _species_id: i32| Ok((0i32, 0i32)))?,
    )?;
    globals.set("C_PetJournal", c_pet_journal)?;

    // C_MountJournal namespace - mount collection
    let c_mount_journal = lua.create_table()?;
    c_mount_journal.set(
        "GetNumMounts",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_mount_journal.set(
        "GetNumDisplayedMounts",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_mount_journal.set(
        "GetMountInfoByID",
        lua.create_function(|_, _mount_id: i32| {
            // Returns: name, spellID, icon, isActive, isUsable, sourceType, isFavorite,
            // isFactionSpecific, faction, shouldHideOnChar, isCollected, mountID, ...
            Ok((
                Value::Nil, // name
                Value::Nil, // spellID
                Value::Nil, // icon
                false,      // isActive
                false,      // isUsable
                0i32,       // sourceType
                false,      // isFavorite
                false,      // isFactionSpecific
                Value::Nil, // faction
                false,      // shouldHideOnChar
                false,      // isCollected
                0i32,       // mountID
            ))
        })?,
    )?;
    c_mount_journal.set(
        "GetMountIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_mount_journal.set(
        "GetCollectedFilterSetting",
        lua.create_function(|_, _filter_index: i32| Ok(true))?,
    )?;
    c_mount_journal.set(
        "SetCollectedFilterSetting",
        lua.create_function(|_, (_filter_index, _is_checked): (i32, bool)| Ok(()))?,
    )?;
    c_mount_journal.set(
        "GetIsFavorite",
        lua.create_function(|_, _mount_index: i32| Ok((false, false)))?,
    )?;
    c_mount_journal.set(
        "SetIsFavorite",
        lua.create_function(|_, (_mount_index, _is_favorite): (i32, bool)| Ok(()))?,
    )?;
    c_mount_journal.set(
        "Summon",
        lua.create_function(|_, _mount_id: i32| Ok(()))?,
    )?;
    c_mount_journal.set(
        "Dismiss",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    globals.set("C_MountJournal", c_mount_journal)?;

    // C_Housing namespace - player housing (Delves housing feature)
    let c_housing = lua.create_table()?;
    c_housing.set(
        "GetHomeInfo",
        lua.create_function(|lua, ()| {
            let info = lua.create_table()?;
            info.set("hasHome", false)?;
            Ok(info)
        })?,
    )?;
    c_housing.set(
        "IsHomeOwner",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_housing.set(
        "GetNumPlacedFurniture",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    globals.set("C_Housing", c_housing)?;

    // C_DelvesUI namespace - Delves dungeon UI
    let c_delves_ui = lua.create_table()?;
    c_delves_ui.set(
        "GetCurrentDelvesSeasonNumber",
        lua.create_function(|_, ()| Ok(1i32))?,
    )?;
    c_delves_ui.set(
        "GetFactionForDelve",
        lua.create_function(|_, _delve_map_id: i32| Ok(Value::Nil))?,
    )?;
    c_delves_ui.set(
        "GetDelvesForSeason",
        lua.create_function(|lua, _season: i32| lua.create_table())?,
    )?;
    c_delves_ui.set(
        "HasActiveDelve",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_delves_ui.set(
        "GetDelveInfo",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    globals.set("C_DelvesUI", c_delves_ui)?;

    // C_ToyBox namespace - toy collection
    let c_toy_box = lua.create_table()?;
    c_toy_box.set(
        "GetToyInfo",
        lua.create_function(|_, _item_id: i32| {
            // Returns: itemID, toyName, icon, isFavorite, hasFanfare, itemQuality
            Ok((0i32, "", 0i32, false, false, 0i32))
        })?,
    )?;
    c_toy_box.set(
        "IsToyUsable",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;
    c_toy_box.set(
        "GetNumToys",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_toy_box.set(
        "GetToyFromIndex",
        lua.create_function(|_, _index: i32| Ok(0i32))?,
    )?;
    c_toy_box.set(
        "GetNumFilteredToys",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    globals.set("C_ToyBox", c_toy_box)?;

    // C_TransmogCollection namespace - transmog/appearance collection
    let c_transmog_collection = lua.create_table()?;
    c_transmog_collection.set(
        "GetAppearanceSources",
        lua.create_function(|lua, _appearance_id: i32| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "GetSourceInfo",
        lua.create_function(|lua, _source_id: i32| {
            // Returns sourceInfo table
            let info = lua.create_table()?;
            info.set("sourceID", 0)?;
            info.set("visualID", 0)?;
            info.set("categoryID", 0)?;
            info.set("itemID", 0)?;
            info.set("isCollected", false)?;
            Ok(info)
        })?,
    )?;
    c_transmog_collection.set(
        "PlayerHasTransmog",
        lua.create_function(|_, (_item_id, _appearance_mod): (i32, Option<i32>)| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "PlayerHasTransmogByItemInfo",
        lua.create_function(|_, _item_info: String| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "PlayerHasTransmogItemModifiedAppearance",
        lua.create_function(|_, _item_modified_appearance_id: i32| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "GetItemInfo",
        lua.create_function(|_, _item_modified_appearance_id: i32| Ok(Value::Nil))?,
    )?;
    c_transmog_collection.set(
        "GetAllAppearanceSources",
        lua.create_function(|lua, _visual_id: i32| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "GetIllusions",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "GetOutfits",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "GetNumMaxOutfits",
        lua.create_function(|_, ()| Ok(20i32))?,
    )?;
    c_transmog_collection.set(
        "GetOutfitInfo",
        lua.create_function(|_, _outfit_id: i32| {
            Ok((Value::Nil, Value::Nil)) // name, icon
        })?,
    )?;
    c_transmog_collection.set(
        "GetAppearanceCameraID",
        lua.create_function(|_, _appearance_id: i32| Ok(0i32))?,
    )?;
    c_transmog_collection.set(
        "GetCategoryAppearances",
        lua.create_function(|lua, (_category, _location): (i32, Value)| lua.create_table())?,
    )?;
    c_transmog_collection.set(
        "PlayerKnowsSource",
        lua.create_function(|_, _source_id: i32| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "IsAppearanceHiddenVisual",
        lua.create_function(|_, _appearance_id: i32| Ok(false))?,
    )?;
    c_transmog_collection.set(
        "IsSourceTypeFilterChecked",
        lua.create_function(|_, _filter: i32| Ok(true))?,
    )?;
    c_transmog_collection.set(
        "GetShowMissingSourceInItemTooltips",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    globals.set("C_TransmogCollection", c_transmog_collection)?;

    // C_Transmog namespace - transmogrification API
    let c_transmog = lua.create_table()?;
    c_transmog.set(
        "GetAllSetAppearancesByID",
        lua.create_function(|lua, _set_id: i32| {
            // Returns array of appearance info for a transmog set
            lua.create_table()
        })?,
    )?;
    c_transmog.set(
        "GetAppliedSourceID",
        lua.create_function(|_, _slot: i32| Ok(Value::Nil))?,
    )?;
    c_transmog.set(
        "GetSlotInfo",
        lua.create_function(|_, _slot: i32| {
            Ok((false, false, false, false, false, Value::Nil))
        })?,
    )?;
    globals.set("C_Transmog", c_transmog)?;

    // TransmogUtil - utility functions for transmog system
    let transmog_util = lua.create_table()?;
    transmog_util.set(
        "GetTransmogLocation",
        lua.create_function(|lua, (slot, transmog_type, modification): (String, i32, i32)| {
            // Return a transmog location table
            let location = lua.create_table()?;
            location.set("slotName", slot)?;
            location.set("transmogType", transmog_type)?;
            location.set("modification", modification)?;
            Ok(location)
        })?,
    )?;
    transmog_util.set(
        "CreateTransmogLocation",
        lua.create_function(|lua, (slot_id, transmog_type, modification): (i32, i32, i32)| {
            let location = lua.create_table()?;
            location.set("slotID", slot_id)?;
            location.set("transmogType", transmog_type)?;
            location.set("modification", modification)?;
            Ok(location)
        })?,
    )?;
    transmog_util.set(
        "GetBestItemModifiedAppearanceID",
        lua.create_function(|_, _item_loc: mlua::Value| Ok(Value::Nil))?,
    )?;
    globals.set("TransmogUtil", transmog_util)?;

    // C_HousingCustomizeMode namespace - housing decoration customization
    let c_housing_customize = lua.create_table()?;
    c_housing_customize.set(
        "IsHoveringDecor",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_housing_customize.set(
        "GetHoveredDecorInfo",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_housing_customize.set(
        "GetDecorDyeSlots",
        lua.create_function(|lua, _decor_id: i32| lua.create_table())?,
    )?;
    globals.set("C_HousingCustomizeMode", c_housing_customize)?;

    // C_DyeColor namespace - dye color information
    let c_dye_color = lua.create_table()?;
    c_dye_color.set(
        "GetDyeColorInfo",
        lua.create_function(|lua, _dye_color_id: i32| {
            // Returns dye color info table
            let info = lua.create_table()?;
            info.set("name", "Dye")?;
            info.set("dyeColorID", 0)?;
            info.set("baseColor", 0xFFFFFFu32)?;
            info.set("highlightColor", 0xFFFFFFu32)?;
            info.set("shadowColor", 0x000000u32)?;
            Ok(info)
        })?,
    )?;
    globals.set("C_DyeColor", c_dye_color)?;

    // C_ScenarioInfo namespace - scenario/dungeon info
    let c_scenario_info = lua.create_table()?;
    c_scenario_info.set(
        "GetScenarioInfo",
        lua.create_function(|_lua, ()| {
            // Returns: scenarioName, currentStage, numStages, flags, hasBonusStep, isBonusStepComplete, ...
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Nil, // Not in scenario
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(0),
                Value::Boolean(false),
                Value::Boolean(false),
            ]))
        })?,
    )?;
    c_scenario_info.set(
        "GetScenarioStepInfo",
        lua.create_function(|_, _step: Option<i32>| {
            Ok((Value::Nil, Value::Nil, Value::Integer(0), Value::Integer(0)))
        })?,
    )?;
    c_scenario_info.set(
        "GetCriteriaInfo",
        lua.create_function(|_, _criteria_index: i32| {
            Ok((Value::Nil, Value::Nil, Value::Boolean(false), Value::Integer(0), Value::Integer(0)))
        })?,
    )?;
    c_scenario_info.set(
        "GetCriteriaInfoByStep",
        lua.create_function(|_, (_step, _criteria): (i32, i32)| {
            Ok((Value::Nil, Value::Nil, Value::Boolean(false), Value::Integer(0), Value::Integer(0)))
        })?,
    )?;
    c_scenario_info.set(
        "IsInScenario",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_ScenarioInfo", c_scenario_info)?;

    // C_TooltipInfo namespace - tooltip data
    let c_tooltip_info = lua.create_table()?;
    c_tooltip_info.set(
        "GetItemByID",
        lua.create_function(|lua, _item_id: i32| {
            let info = lua.create_table()?;
            info.set("type", 1)?; // Item
            info.set("lines", lua.create_table()?)?;
            Ok(info)
        })?,
    )?;
    c_tooltip_info.set(
        "GetItemByGUID",
        lua.create_function(|lua, _item_guid: String| {
            let info = lua.create_table()?;
            info.set("type", 1)?;
            info.set("lines", lua.create_table()?)?;
            Ok(info)
        })?,
    )?;
    c_tooltip_info.set(
        "GetBagItem",
        lua.create_function(|lua, (_bag, _slot): (i32, i32)| {
            let info = lua.create_table()?;
            info.set("type", 1)?;
            info.set("lines", lua.create_table()?)?;
            Ok(info)
        })?,
    )?;
    c_tooltip_info.set(
        "GetSpellByID",
        lua.create_function(|lua, _spell_id: i32| {
            let info = lua.create_table()?;
            info.set("type", 2)?; // Spell
            info.set("lines", lua.create_table()?)?;
            Ok(info)
        })?,
    )?;
    c_tooltip_info.set(
        "GetUnit",
        lua.create_function(|lua, _unit: String| {
            let info = lua.create_table()?;
            info.set("type", 3)?; // Unit
            info.set("lines", lua.create_table()?)?;
            Ok(info)
        })?,
    )?;
    c_tooltip_info.set(
        "GetHyperlink",
        lua.create_function(|lua, _link: String| {
            let info = lua.create_table()?;
            info.set("type", 1)?;
            info.set("lines", lua.create_table()?)?;
            Ok(info)
        })?,
    )?;
    globals.set("C_TooltipInfo", c_tooltip_info)?;

    // TooltipDataProcessor - global for registering tooltip post-processing callbacks
    let tooltip_data_processor = lua.create_table()?;
    tooltip_data_processor.set(
        "AddTooltipPostCall",
        lua.create_function(|_, (_data_type, _callback): (Option<i32>, mlua::Function)| Ok(()))?,
    )?;
    globals.set("TooltipDataProcessor", tooltip_data_processor)?;

    // C_PetBattles namespace - pet battle system
    let c_pet_battles = lua.create_table()?;
    c_pet_battles.set(
        "IsInBattle",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_pet_battles.set(
        "IsWildBattle",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_pet_battles.set(
        "IsPlayerNPC",
        lua.create_function(|_, _owner_index: i32| Ok(false))?,
    )?;
    c_pet_battles.set(
        "GetNumAuras",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(0i32))?,
    )?;
    c_pet_battles.set(
        "GetActivePet",
        lua.create_function(|_, _owner: i32| Ok(1i32))?,
    )?;
    c_pet_battles.set(
        "GetNumPets",
        lua.create_function(|_, _owner: i32| Ok(0i32))?,
    )?;
    c_pet_battles.set(
        "GetHealth",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(0i32))?,
    )?;
    c_pet_battles.set(
        "GetMaxHealth",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(100i32))?,
    )?;
    c_pet_battles.set(
        "GetSpeed",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(0i32))?,
    )?;
    c_pet_battles.set(
        "GetPower",
        lua.create_function(|_, (_owner, _pet_index): (i32, i32)| Ok(0i32))?,
    )?;
    globals.set("C_PetBattles", c_pet_battles)?;

    // C_TradeSkillUI namespace - profession/tradeskill UI
    let c_trade_skill = lua.create_table()?;
    c_trade_skill.set(
        "GetTradeSkillLine",
        lua.create_function(|_, ()| {
            // Returns: skillLineID, skillLineName, skillLineRank, skillLineMaxRank, ...
            Ok((0i32, Value::Nil, 0i32, 0i32))
        })?,
    )?;
    c_trade_skill.set(
        "GetRecipeInfo",
        lua.create_function(|lua, _recipe_id: i32| {
            let info = lua.create_table()?;
            info.set("recipeID", 0)?;
            info.set("name", Value::Nil)?;
            info.set("craftable", false)?;
            Ok(info)
        })?,
    )?;
    c_trade_skill.set(
        "GetRecipeSchematic",
        lua.create_function(|lua, _recipe_id: i32| {
            let schematic = lua.create_table()?;
            schematic.set("recipeID", 0)?;
            Ok(schematic)
        })?,
    )?;
    c_trade_skill.set(
        "IsTradeSkillLinked",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_trade_skill.set(
        "IsNPCCrafting",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_trade_skill.set(
        "GetAllRecipeIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    globals.set("C_TradeSkillUI", c_trade_skill)?;

    // C_Heirloom namespace - heirloom collection
    let c_heirloom = lua.create_table()?;
    c_heirloom.set(
        "GetNumKnownHeirlooms",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_heirloom.set(
        "GetNumHeirlooms",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_heirloom.set(
        "PlayerHasHeirloom",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;
    c_heirloom.set(
        "GetHeirloomInfo",
        lua.create_function(|_, _item_id: i32| {
            // Returns: itemID, userFlags, sourceType, quality, name, icon, ...
            Ok((0i32, 0i32, 0i32, 0i32, Value::Nil, Value::Nil))
        })?,
    )?;
    c_heirloom.set(
        "GetHeirloomMaxUpgradeLevel",
        lua.create_function(|_, _item_id: i32| Ok(0i32))?,
    )?;
    c_heirloom.set(
        "CanHeirloomUpgradeFromPending",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;
    c_heirloom.set(
        "IsPendingHeirloomUpgrade",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_Heirloom", c_heirloom)?;

    // C_MythicPlus namespace - Mythic+ dungeon info
    let c_mythic_plus = lua.create_table()?;
    c_mythic_plus.set(
        "GetRunHistory",
        lua.create_function(|lua, (_include_prev_weeks, _include_incomplete): (Option<bool>, Option<bool>)| {
            lua.create_table()
        })?,
    )?;
    c_mythic_plus.set(
        "GetOwnedKeystoneLevel",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_mythic_plus.set(
        "GetOwnedKeystoneChallengeMapID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_mythic_plus.set(
        "GetOwnedKeystoneMapID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_mythic_plus.set(
        "GetCurrentAffixes",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_mythic_plus.set(
        "GetSeasonInfo",
        lua.create_function(|_, ()| {
            // Returns: season, seasonStart, seasonEnd
            Ok((1i32, 0i32, 0i32))
        })?,
    )?;
    c_mythic_plus.set(
        "GetCurrentSeason",
        lua.create_function(|_, ()| Ok(1i32))?,
    )?;
    c_mythic_plus.set(
        "GetRewardLevelFromKeystoneLevel",
        lua.create_function(|_, _keystone_level: i32| Ok(0i32))?,
    )?;
    c_mythic_plus.set(
        "GetWeeklyBestForMap",
        lua.create_function(|_, _map_challenge_mode_id: i32| {
            // Returns: durationSec, level, completionDate, affixIDs, members
            Ok(Value::Nil)
        })?,
    )?;
    c_mythic_plus.set(
        "GetOverallDungeonScore",
        lua.create_function(|_, ()| Ok(0.0_f64))?,
    )?;
    c_mythic_plus.set(
        "IsMythicPlusActive",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_MythicPlus", c_mythic_plus)?;

    // C_LFGInfo namespace - Looking for Group information
    let c_lfg_info = lua.create_table()?;
    c_lfg_info.set(
        "GetRoleCheckDifficultyDetails",
        lua.create_function(|_, ()| Ok((false, false, false)))?,
    )?;
    c_lfg_info.set(
        "GetDungeonInfo",
        lua.create_function(|lua, _dungeon_id: i32| {
            let info = lua.create_table()?;
            Ok(info)
        })?,
    )?;
    c_lfg_info.set(
        "GetLFDLockStates",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_lfg_info.set(
        "CanPartyLFGBackfill",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_lfg_info.set(
        "GetAllEntriesForCategory",
        lua.create_function(|lua, _category: i32| lua.create_table())?,
    )?;
    c_lfg_info.set(
        "HideNameFromUI",
        lua.create_function(|_, _dungeon_id: i32| Ok(false))?,
    )?;
    globals.set("C_LFGInfo", c_lfg_info)?;

    // C_NamePlate namespace - Nameplate management
    let c_nameplate = lua.create_table()?;
    c_nameplate.set(
        "GetNamePlateForUnit",
        lua.create_function(|_, _unit: String| Ok(Value::Nil))?,
    )?;
    c_nameplate.set(
        "GetNamePlates",
        lua.create_function(|lua, _include_forbidden: Option<bool>| lua.create_table())?,
    )?;
    c_nameplate.set(
        "SetNamePlateEnemySize",
        lua.create_function(|_, (_width, _height): (f32, f32)| Ok(()))?,
    )?;
    c_nameplate.set(
        "SetNamePlateFriendlySize",
        lua.create_function(|_, (_width, _height): (f32, f32)| Ok(()))?,
    )?;
    c_nameplate.set(
        "SetNamePlateSelfSize",
        lua.create_function(|_, (_width, _height): (f32, f32)| Ok(()))?,
    )?;
    c_nameplate.set(
        "GetNamePlateEnemySize",
        lua.create_function(|_, ()| Ok((110.0_f64, 45.0_f64)))?,
    )?;
    c_nameplate.set(
        "GetNamePlateFriendlySize",
        lua.create_function(|_, ()| Ok((110.0_f64, 45.0_f64)))?,
    )?;
    c_nameplate.set(
        "GetNamePlateSelfSize",
        lua.create_function(|_, ()| Ok((110.0_f64, 45.0_f64)))?,
    )?;
    c_nameplate.set(
        "SetNamePlateSelfClickThrough",
        lua.create_function(|_, _click_through: bool| Ok(()))?,
    )?;
    c_nameplate.set(
        "SetNamePlateEnemyClickThrough",
        lua.create_function(|_, _click_through: bool| Ok(()))?,
    )?;
    c_nameplate.set(
        "SetNamePlateFriendlyClickThrough",
        lua.create_function(|_, _click_through: bool| Ok(()))?,
    )?;
    globals.set("C_NamePlate", c_nameplate)?;

    // C_PlayerInfo namespace - Player information
    let c_player_info = lua.create_table()?;
    c_player_info.set(
        "GetPlayerMythicPlusRatingSummary",
        lua.create_function(|lua, _unit: String| {
            let summary = lua.create_table()?;
            summary.set("currentSeasonScore", 0.0_f64)?;
            summary.set("runs", lua.create_table()?)?;
            Ok(summary)
        })?,
    )?;
    c_player_info.set(
        "GetContentDifficultyCreatureForPlayer",
        lua.create_function(|_, _unit: String| Ok(0i32))?,
    )?;
    c_player_info.set(
        "GetContentDifficultyQualityForPlayer",
        lua.create_function(|_, _unit: String| Ok(0i32))?,
    )?;
    c_player_info.set(
        "CanPlayerUseMountEquipment",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    c_player_info.set(
        "IsPlayerNPERestricted",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_player_info.set(
        "GetGlidingInfo",
        lua.create_function(|_, ()| {
            // Returns: isGliding, canGlide, forwardSpeed
            Ok((false, false, 0.0_f64))
        })?,
    )?;
    c_player_info.set(
        "IsPlayerInChromieTime",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_PlayerInfo", c_player_info)?;

    // C_PartyInfo namespace - party/group information
    let c_party_info = lua.create_table()?;
    c_party_info.set(
        "GetActiveCategories",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_party_info.set(
        "GetInviteConfirmationInfo",
        lua.create_function(|_, _invite_guid: String| {
            // Returns: confirmationType, name, guid, zone, ...
            Ok(Value::Nil)
        })?,
    )?;
    c_party_info.set(
        "GetInviteReferralInfo",
        lua.create_function(|_, _invite_guid: String| {
            // Returns referral info
            Ok(Value::Nil)
        })?,
    )?;
    c_party_info.set(
        "ConfirmInviteUnit",
        lua.create_function(|_, _invite_guid: String| Ok(()))?,
    )?;
    c_party_info.set(
        "DeclineInviteUnit",
        lua.create_function(|_, _invite_guid: String| Ok(()))?,
    )?;
    c_party_info.set(
        "IsPartyFull",
        lua.create_function(|_, _category: Option<i32>| Ok(false))?,
    )?;
    c_party_info.set(
        "AllowedToDoPartyConversion",
        lua.create_function(|_, _to_raid: bool| Ok(true))?,
    )?;
    c_party_info.set(
        "CanInvite",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    c_party_info.set(
        "InviteUnit",
        lua.create_function(|_, _name: String| Ok(()))?,
    )?;
    c_party_info.set(
        "LeaveParty",
        lua.create_function(|_, _category: Option<i32>| Ok(()))?,
    )?;
    c_party_info.set(
        "ConvertToParty",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    c_party_info.set(
        "ConvertToRaid",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    c_party_info.set(
        "GetMinLevel",
        lua.create_function(|_, _category: Option<i32>| Ok(1i32))?,
    )?;
    c_party_info.set(
        "GetGatheringRequestInfo",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    globals.set("C_PartyInfo", c_party_info)?;

    // GetServerExpansionLevel - returns current expansion
    globals.set(
        "GetServerExpansionLevel",
        lua.create_function(|_, ()| Ok(10i32))?, // The War Within = 10
    )?;

    // C_ChatInfo namespace
    let c_chat_info = lua.create_table()?;
    c_chat_info.set(
        "RegisterAddonMessagePrefix",
        lua.create_function(|_, _prefix: String| Ok(true))?,
    )?;
    c_chat_info.set(
        "IsAddonMessagePrefixRegistered",
        lua.create_function(|_, _prefix: String| Ok(false))?,
    )?;
    c_chat_info.set(
        "SendAddonMessage",
        lua.create_function(
            |_, (_prefix, _message, _channel, _target): (String, String, Option<String>, Option<String>)| {
                Ok(())
            },
        )?,
    )?;
    c_chat_info.set(
        "GetRegisteredAddonMessagePrefixes",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_chat_info.set(
        "SendChatMessage",
        lua.create_function(
            |_, (_message, _channel, _language, _target): (String, Option<String>, Option<Value>, Option<String>)| {
                // No-op in simulation
                Ok(())
            },
        )?,
    )?;
    globals.set("C_ChatInfo", c_chat_info)?;

    // Legacy global version
    let register_addon_message_prefix = lua.create_function(|_, _prefix: String| Ok(true))?;
    globals.set("RegisterAddonMessagePrefix", register_addon_message_prefix)?;

    // GetChannelList() - returns list of chat channels (id, name) pairs
    // Returns: id1, name1, id2, name2, ... (repeating pairs)
    let get_channel_list = lua.create_function(|_, ()| {
        // Return empty list in simulation (no channels)
        Ok(mlua::MultiValue::new())
    })?;
    globals.set("GetChannelList", get_channel_list)?;

    // GetChannelName(channelID) - returns channel name and info
    let get_channel_name = lua.create_function(|_, _id: Value| {
        // Return nil for unknown channels in simulation
        Ok(mlua::MultiValue::from_vec(vec![Value::Nil, Value::Nil, Value::Nil]))
    })?;
    globals.set("GetChannelName", get_channel_name)?;

    // GetNumDisplayChannels() - returns number of displayed channels
    let get_num_display_channels = lua.create_function(|_, ()| Ok(0i32))?;
    globals.set("GetNumDisplayChannels", get_num_display_channels)?;

    // C_EventUtils namespace
    let c_event_utils = lua.create_table()?;
    c_event_utils.set(
        "IsEventValid",
        lua.create_function(|_, event: String| {
            // WoW events are UPPERCASE_WITH_UNDERSCORES
            // Must be at least 3 chars, all uppercase letters/digits/underscores
            // Must contain at least one underscore (most events do)
            // Must start with a letter
            if event.len() < 3 {
                return Ok(false);
            }
            let chars: Vec<char> = event.chars().collect();
            if !chars[0].is_ascii_uppercase() {
                return Ok(false);
            }
            let has_underscore = event.contains('_');
            let all_valid = chars
                .iter()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_');
            // Events like "L", "OnLoad" etc. should return false
            Ok(has_underscore && all_valid)
        })?,
    )?;
    globals.set("C_EventUtils", c_event_utils)?;

    // C_AddOnProfiler namespace - addon performance profiling
    let c_addon_profiler = lua.create_table()?;
    c_addon_profiler.set(
        "GetAddOnMetric",
        lua.create_function(|_, (_addon_id, _metric): (Value, i32)| Ok(0.0f64))?,
    )?;
    c_addon_profiler.set(
        "GetOverallMetric",
        lua.create_function(|_, _metric: i32| Ok(0.0f64))?,
    )?;
    c_addon_profiler.set(
        "IsEnabled",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_addon_profiler.set(
        "SetEnabled",
        lua.create_function(|_, _enabled: bool| Ok(()))?,
    )?;
    globals.set("C_AddOnProfiler", c_addon_profiler)?;

    // C_CurveUtil namespace - curve/animation utilities
    let c_curve_util = lua.create_table()?;
    c_curve_util.set(
        "CreateCurve",
        lua.create_function(|lua, ()| {
            let curve = lua.create_table()?;
            curve.set("AddPoint", lua.create_function(|_, (_self, _x, _y): (Value, f64, f64)| Ok(()))?)?;
            curve.set("GetValue", lua.create_function(|_, (_self, _x): (Value, f64)| Ok(0.0f64))?)?;
            curve.set("Clear", lua.create_function(|_, _self: Value| Ok(()))?)?;
            Ok(curve)
        })?,
    )?;
    c_curve_util.set(
        "CreateColorCurve",
        lua.create_function(|lua, ()| {
            let curve = lua.create_table()?;
            curve.set("AddPoint", lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(()))?)?;
            curve.set("GetColor", lua.create_function(|_, (_self, _x): (Value, f64)| Ok((1.0f64, 1.0f64, 1.0f64, 1.0f64)))?)?;
            curve.set("Clear", lua.create_function(|_, _self: Value| Ok(()))?)?;
            Ok(curve)
        })?,
    )?;
    globals.set("C_CurveUtil", c_curve_util)?;

    // C_CVar namespace - console variables
    let c_cvar = lua.create_table()?;
    let state_for_getcvar = Rc::clone(&state);
    c_cvar.set(
        "GetCVar",
        lua.create_function(move |lua, cvar: String| {
            let state = state_for_getcvar.borrow();
            match state.cvars.get(&cvar) {
                Some(value) => Ok(Value::String(lua.create_string(&value)?)),
                None => Ok(Value::Nil),
            }
        })?,
    )?;
    let state_for_setcvar = Rc::clone(&state);
    c_cvar.set(
        "SetCVar",
        lua.create_function(move |_, (cvar, value): (String, String)| {
            let state = state_for_setcvar.borrow();
            Ok(state.cvars.set(&cvar, &value))
        })?,
    )?;
    let state_for_getcvarbool = Rc::clone(&state);
    c_cvar.set(
        "GetCVarBool",
        lua.create_function(move |_, cvar: String| {
            let state = state_for_getcvarbool.borrow();
            Ok(state.cvars.get_bool(&cvar))
        })?,
    )?;
    let state_for_registercvar = Rc::clone(&state);
    c_cvar.set(
        "RegisterCVar",
        lua.create_function(move |_, (cvar, default): (String, Option<String>)| {
            let state = state_for_registercvar.borrow();
            state.cvars.register(&cvar, default.as_deref());
            Ok(())
        })?,
    )?;
    let state_for_getcvardefault = Rc::clone(&state);
    c_cvar.set(
        "GetCVarDefault",
        lua.create_function(move |lua, cvar: String| {
            let state = state_for_getcvardefault.borrow();
            match state.cvars.get_default(&cvar) {
                Some(value) => Ok(Value::String(lua.create_string(&value)?)),
                None => Ok(Value::Nil),
            }
        })?,
    )?;
    globals.set("C_CVar", c_cvar)?;

    // C_SpellBook namespace - spell book functions
    let c_spell_book = lua.create_table()?;
    c_spell_book.set(
        "GetSpellBookItemName",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    c_spell_book.set(
        "GetNumSpellBookSkillLines",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_spell_book.set(
        "GetSpellBookSkillLineInfo",
        lua.create_function(|_, _tab: i32| Ok(Value::Nil))?,
    )?;
    c_spell_book.set(
        "GetSpellBookItemInfo",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    c_spell_book.set(
        "HasPetSpells",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_spell_book.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;
    c_spell_book.set(
        "IsSpellKnown",
        lua.create_function(|_, (_spell_id, _is_pet): (i32, Option<bool>)| Ok(false))?,
    )?;
    globals.set("C_SpellBook", c_spell_book)?;

    // C_Spell namespace - spell information
    let c_spell = lua.create_table()?;
    // C_Spell.GetSpellInfo(spellID) - returns a SpellInfo table in modern API
    c_spell.set(
        "GetSpellInfo",
        lua.create_function(|lua, spell_id: i32| {
            // Return a spell info table with common fields
            let info = lua.create_table()?;
            info.set("name", format!("Spell {}", spell_id))?;
            info.set("spellID", spell_id)?;
            info.set("iconID", 136243)?; // INV_Misc_QuestionMark
            info.set("castTime", 0)?;
            info.set("minRange", 0)?;
            info.set("maxRange", 0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    // C_Spell.GetSpellCharges(spellID) - returns charges info
    c_spell.set(
        "GetSpellCharges",
        lua.create_function(|lua, _spell_id: i32| {
            // Return a charges table (most spells don't have charges)
            let info = lua.create_table()?;
            info.set("currentCharges", 0)?;
            info.set("maxCharges", 0)?;
            info.set("cooldownStartTime", 0)?;
            info.set("cooldownDuration", 0)?;
            info.set("chargeModRate", 1.0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_spell.set(
        "IsSpellPassive",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;
    c_spell.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;
    c_spell.set(
        "GetSchoolString",
        lua.create_function(|lua, school_mask: i32| {
            // WoW spell school bitmask to name
            let name = match school_mask {
                1 => "Physical",
                2 => "Holy",
                4 => "Fire",
                8 => "Nature",
                16 => "Frost",
                32 => "Shadow",
                64 => "Arcane",
                _ => "Unknown",
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    c_spell.set(
        "GetSpellTexture",
        lua.create_function(|_, _spell_id: i32| {
            // Return a generic spell icon ID
            Ok(136243) // INV_Misc_QuestionMark
        })?,
    )?;
    c_spell.set(
        "GetSpellLink",
        lua.create_function(|lua, spell_id: i32| {
            let link = format!("|cff71d5ff|Hspell:{}|h[Spell {}]|h|r", spell_id, spell_id);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;
    c_spell.set(
        "GetSpellName",
        lua.create_function(|lua, spell_id: i32| {
            // Return spell name (just "Spell {id}" for simulation)
            Ok(Value::String(lua.create_string(&format!("Spell {}", spell_id))?))
        })?,
    )?;
    c_spell.set(
        "GetSpellCooldown",
        lua.create_function(|lua, _spell_id: i32| {
            // Returns cooldown info table: { startTime, duration, isEnabled, modRate }
            let info = lua.create_table()?;
            info.set("startTime", 0.0)?;
            info.set("duration", 0.0)?;
            info.set("isEnabled", true)?;
            info.set("modRate", 1.0)?;
            Ok(info)
        })?,
    )?;
    c_spell.set(
        "DoesSpellExist",
        lua.create_function(|_, spell_id: i32| {
            // In simulation, assume all positive spell IDs exist
            Ok(spell_id > 0)
        })?,
    )?;
    globals.set("C_Spell", c_spell)?;

    // C_Traits namespace - talent/loadout system (Dragonflight+)
    let c_traits = lua.create_table()?;
    c_traits.set(
        "GenerateImportString",
        lua.create_function(|_, _config_id: i32| {
            // Return a dummy talent string
            Ok("dummy_talent_string".to_string())
        })?,
    )?;
    c_traits.set(
        "GetConfigIDBySystemID",
        lua.create_function(|_, _system_id: i32| Ok(0))?,
    )?;
    c_traits.set(
        "GetConfigIDByTreeID",
        lua.create_function(|_, _tree_id: i32| Ok(0))?,
    )?;
    c_traits.set(
        "GetConfigInfo",
        lua.create_function(|lua, _config_id: i32| {
            // Return a stub config info table with empty treeIDs
            let info = lua.create_table()?;
            info.set("treeIDs", lua.create_table()?)?; // Empty array
            info.set("ID", 0)?;
            info.set("type", 1)?; // Enum.TraitConfigType.Combat
            info.set("name", "")?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_traits.set(
        "GetNodeInfo",
        lua.create_function(|_, (_config_id, _node_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetEntryInfo",
        lua.create_function(|_, (_config_id, _entry_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetDefinitionInfo",
        lua.create_function(|_, _def_id: i32| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "InitializeViewLoadout",
        lua.create_function(|_, (_config_id, _tree_id): (i32, i32)| Ok(true))?,
    )?;
    c_traits.set(
        "GetTreeInfo",
        lua.create_function(|_, _config_id: i32| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetTreeNodes",
        lua.create_function(|lua, _tree_id: i32| lua.create_table())?,
    )?;
    c_traits.set(
        "GetTreeCurrencyInfo",
        lua.create_function(|_, (_tree_id, _currency_type): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetAllTreeIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_traits.set(
        "GetTraitSystemFlags",
        lua.create_function(|_, _system_id: i32| Ok(0))?,
    )?;
    c_traits.set(
        "GetEntryInfo",
        lua.create_function(|_, (_config_id, _entry_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetNodeInfo",
        lua.create_function(|_, (_config_id, _node_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "CanPurchaseRank",
        lua.create_function(|_, (_config_id, _node_id, _entry_id): (i32, i32, i32)| Ok(false))?,
    )?;
    c_traits.set(
        "GetLoadoutSerializationVersion",
        lua.create_function(|_, ()| Ok(2i32))?,
    )?;
    globals.set("C_Traits", c_traits)?;

    // C_ItemUpgrade namespace - item upgrade system
    let c_item_upgrade = lua.create_table()?;
    c_item_upgrade.set(
        "CanUpgradeItem",
        lua.create_function(|_, _item_location: Value| Ok(false))?,
    )?;
    c_item_upgrade.set(
        "GetItemUpgradeInfo",
        lua.create_function(|_, _item_location: Value| Ok(Value::Nil))?,
    )?;
    globals.set("C_ItemUpgrade", c_item_upgrade)?;

    // C_ProfSpecs namespace - profession specializations
    let c_prof_specs = lua.create_table()?;
    c_prof_specs.set(
        "GetSpecTabIDsForSkillLine",
        lua.create_function(|lua, _skill_line_id: i32| lua.create_table())?,
    )?;
    c_prof_specs.set(
        "GetConfigIDForSkillLine",
        lua.create_function(|_, _skill_line_id: i32| Ok(0i32))?,
    )?;
    c_prof_specs.set(
        "GetTabInfo",
        lua.create_function(|_, _tab_id: i32| Ok(Value::Nil))?,
    )?;
    c_prof_specs.set(
        "GetSpendCurrencyForPath",
        lua.create_function(|_, _path_id: i32| Ok(0i32))?,
    )?;
    c_prof_specs.set(
        "GetUnlockEntryForPath",
        lua.create_function(|_, _path_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_ProfSpecs", c_prof_specs)?;

    // C_QuestInfoSystem namespace - quest classification info
    let c_quest_info_system = lua.create_table()?;
    c_quest_info_system.set(
        "GetQuestClassification",
        lua.create_function(|_, _quest_id: i32| Ok(0i32))?, // Normal classification
    )?;
    c_quest_info_system.set(
        "HasQuestClassification",
        lua.create_function(|_, (_quest_id, _classification): (i32, i32)| Ok(false))?,
    )?;
    globals.set("C_QuestInfoSystem", c_quest_info_system)?;

    // C_QuestLine namespace - questline information
    let c_quest_line = lua.create_table()?;
    c_quest_line.set(
        "GetQuestLineInfo",
        lua.create_function(|_, (_quest_id, _ui_map_id, _displayable_only): (i32, Option<i32>, Option<bool>)| {
            Ok(Value::Nil)
        })?,
    )?;
    c_quest_line.set(
        "GetQuestLineQuests",
        lua.create_function(|lua, _quest_line_id: i32| lua.create_table())?,
    )?;
    c_quest_line.set(
        "IsComplete",
        lua.create_function(|_, _quest_line_id: i32| Ok(false))?,
    )?;
    c_quest_line.set(
        "RequestQuestLinesForMap",
        lua.create_function(|_, _ui_map_id: i32| Ok(()))?,
    )?;
    globals.set("C_QuestLine", c_quest_line)?;

    // C_CampaignInfo namespace - campaign/war campaign info
    let c_campaign_info = lua.create_table()?;
    c_campaign_info.set(
        "GetCampaignInfo",
        lua.create_function(|lua, campaign_id: i32| {
            let info = lua.create_table()?;
            // Return basic campaign info stub
            let name = match campaign_id {
                290 => "Legionfall",  // Broken Shore
                119 => "War Campaign",
                _ => "Campaign",
            };
            info.set("name", name)?;
            info.set("campaignID", campaign_id)?;
            info.set("isComplete", false)?;
            info.set("numChapters", 0)?;
            Ok(info)
        })?,
    )?;
    c_campaign_info.set(
        "IsCampaignQuest",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    c_campaign_info.set(
        "GetAvailableCampaigns",
        lua.create_function(|lua, ()| {
            let campaigns = lua.create_table()?;
            Ok(campaigns)
        })?,
    )?;
    globals.set("C_CampaignInfo", c_campaign_info)?;

    // C_RaidLocks namespace - raid lockout info
    let c_raid_locks = lua.create_table()?;
    c_raid_locks.set(
        "IsEncounterComplete",
        lua.create_function(|_, (_map_id, _boss_id, _difficulty): (i32, i32, Option<i32>)| Ok(false))?,
    )?;
    c_raid_locks.set(
        "GetRaidLockInfo",
        lua.create_function(|_, _map_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_RaidLocks", c_raid_locks)?;

    // C_ClassTalents namespace - class talent functions
    let c_class_talents = lua.create_table()?;
    c_class_talents.set(
        "GetActiveConfigID",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_class_talents.set(
        "GetConfigIDsBySpecID",
        lua.create_function(|lua, _spec_id: i32| {
            // Return empty table
            lua.create_table()
        })?,
    )?;
    c_class_talents.set(
        "GetStarterBuildActive",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_class_talents.set(
        "GetHasStarterBuild",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_class_talents.set(
        "GetTraitTreeForSpec",
        lua.create_function(|_, _spec_id: i32| Ok(0))?,
    )?;
    c_class_talents.set(
        "UpdateLastSelectedSavedConfigID",
        lua.create_function(|_, (_spec_id, _config_id): (i32, i32)| Ok(()))?,
    )?;
    c_class_talents.set(
        "InitializeViewLoadout",
        lua.create_function(|_, (_spec_id, _class_id): (i32, i32)| Ok(true))?,
    )?;
    c_class_talents.set(
        "ViewLoadout",
        lua.create_function(|_, _entries: mlua::Table| Ok(true))?,
    )?;
    c_class_talents.set(
        "GetHeroTalentSpecsForClassSpec",
        lua.create_function(|lua, (_config_id, _spec_id): (i32, i32)| lua.create_table())?,
    )?;
    c_class_talents.set(
        "GetLoadoutSerializationVersion",
        lua.create_function(|_, ()| Ok(1i32))?,
    )?;
    globals.set("C_ClassTalents", c_class_talents)?;

    // C_GuildInfo namespace - guild information
    let c_guild_info = lua.create_table()?;
    c_guild_info.set(
        "GetGuildNewsInfo",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_guild_info.set(
        "GetGuildRankOrder",
        lua.create_function(|_, _guild_member_guid: String| Ok(0i32))?,
    )?;
    c_guild_info.set(
        "GuildRoster",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    c_guild_info.set(
        "IsGuildOfficer",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_guild_info.set(
        "CanEditOfficerNote",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_guild_info.set(
        "CanViewOfficerNote",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_guild_info.set(
        "RemoveFromGuild",
        lua.create_function(|_, _guid: String| Ok(()))?,
    )?;
    globals.set("C_GuildInfo", c_guild_info)?;

    // C_GuildBank namespace - guild bank
    let c_guild_bank = lua.create_table()?;
    c_guild_bank.set(
        "GetNumTabs",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    globals.set("C_GuildBank", c_guild_bank)?;

    // C_AlliedRaces namespace - allied race unlocks
    let c_allied_races = lua.create_table()?;
    c_allied_races.set(
        "GetAllRacialAbilitiesFromID",
        lua.create_function(|lua, _race_id: i32| lua.create_table())?,
    )?;
    globals.set("C_AlliedRaces", c_allied_races)?;

    // C_AzeriteEssence namespace - BfA Azerite essence system
    let c_azerite_essence = lua.create_table()?;
    c_azerite_essence.set(
        "GetEssenceHyperlink",
        lua.create_function(|lua, (essence_id, rank): (i32, i32)| {
            // Returns hyperlink for an Azerite essence
            let link = format!(
                "|cff00ccff|Hazessence:{}:{}|h[Essence {}]|h|r",
                essence_id, rank, essence_id
            );
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;
    c_azerite_essence.set(
        "GetEssenceInfo",
        lua.create_function(|lua, _essence_id: i32| {
            let info = lua.create_table()?;
            info.set("ID", 0)?;
            info.set("name", "Unknown Essence")?;
            info.set("icon", 0)?;
            info.set("valid", false)?;
            info.set("unlocked", false)?;
            info.set("rank", 0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_azerite_essence.set(
        "GetMilestoneEssence",
        lua.create_function(|_, _milestone_id: i32| Ok(Value::Nil))?,
    )?;
    c_azerite_essence.set(
        "GetNumUnlockedEssences",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_azerite_essence.set(
        "GetNumUnlockedSlots",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_azerite_essence.set(
        "CanOpenUI",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_AzeriteEssence", c_azerite_essence)?;

    // C_PvP namespace - PvP information
    let c_pvp = lua.create_table()?;
    c_pvp.set(
        "GetZonePVPInfo",
        lua.create_function(|_, ()| {
            // Returns: pvpType, isFFA, faction
            // pvpType: "sanctuary", "friendly", "hostile", "contested", "combat", nil
            Ok((Value::Nil, false, Value::Nil))
        })?,
    )?;
    c_pvp.set(
        "GetScoreInfo",
        lua.create_function(|_, _index: i32| {
            // Returns nil when not in PvP
            Ok(Value::Nil)
        })?,
    )?;
    c_pvp.set(
        "IsWarModeDesired",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_pvp.set(
        "IsWarModeActive",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_pvp.set(
        "IsPVPMap",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_pvp.set(
        "IsRatedMap",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_pvp.set(
        "IsInBrawl",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_PvP", c_pvp)?;

    // C_FriendList namespace - friend list
    let c_friend_list = lua.create_table()?;
    c_friend_list.set(
        "GetNumFriends",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_friend_list.set(
        "GetNumOnlineFriends",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_friend_list.set(
        "GetFriendInfoByIndex",
        lua.create_function(|_, _index: i32| {
            // Returns nil when no friend at index
            Ok(Value::Nil)
        })?,
    )?;
    c_friend_list.set(
        "GetFriendInfoByName",
        lua.create_function(|_, _name: String| Ok(Value::Nil))?,
    )?;
    c_friend_list.set(
        "IsFriend",
        lua.create_function(|_, _guid: String| Ok(false))?,
    )?;
    globals.set("C_FriendList", c_friend_list)?;

    // C_AuctionHouse namespace - auction house
    let c_auction_house = lua.create_table()?;
    c_auction_house.set(
        "GetNumReplicateItems",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    globals.set("C_AuctionHouse", c_auction_house)?;

    // C_Bank namespace - personal bank
    let c_bank = lua.create_table()?;
    c_bank.set(
        "FetchDepositedMoney",
        lua.create_function(|_, _bank_type: i32| Ok(0i64))?,
    )?;
    globals.set("C_Bank", c_bank)?;

    // C_EncounterJournal namespace - dungeon/raid journal
    let c_encounter_journal = lua.create_table()?;
    c_encounter_journal.set(
        "GetEncounterInfo",
        lua.create_function(|_, _encounter_id: i32| Ok(Value::Nil))?,
    )?;
    c_encounter_journal.set(
        "GetSectionInfo",
        lua.create_function(|_, _section_id: i32| Ok(Value::Nil))?,
    )?;
    c_encounter_journal.set(
        "GetLootInfoByIndex",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_encounter_journal.set(
        "GetInstanceInfo",
        lua.create_function(|_, _instance_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_EncounterJournal", c_encounter_journal)?;

    // C_GMTicketInfo namespace - GM ticket system
    let c_gm_ticket_info = lua.create_table()?;
    c_gm_ticket_info.set(
        "HasGMTicket",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_GMTicketInfo", c_gm_ticket_info)?;

    // Legacy global spell functions
    // GetSpellInfo(spellId) - Returns: name, rank, icon, castTime, minRange, maxRange, spellId, originalIcon
    globals.set(
        "GetSpellInfo",
        lua.create_function(|lua, spell_id: i32| {
            // Return mock spell info - name, rank, icon, castTime, minRange, maxRange, spellId, originalIcon
            let name = lua.create_string(&format!("Spell {}", spell_id))?;
            let rank = lua.create_string("")?; // Empty string for rank
            let icon = lua.create_string("Interface\\Icons\\INV_Misc_QuestionMark")?;
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(name),   // name
                Value::String(rank),   // rank
                Value::String(icon),   // icon
                Value::Integer(0),     // castTime
                Value::Integer(0),     // minRange
                Value::Integer(0),     // maxRange
                Value::Integer(spell_id as i64), // spellId
                Value::String(lua.create_string("Interface\\Icons\\INV_Misc_QuestionMark")?), // originalIcon
            ]))
        })?,
    )?;
    globals.set(
        "GetSpellBookItemName",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetNumSpellTabs",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    globals.set(
        "GetSpellTabInfo",
        lua.create_function(|_, _tab: i32| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetSpellBookItemInfo",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "IsPassiveSpell",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;
    globals.set(
        "HasPetSpells",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;

    // C_Item namespace - item information
    let c_item = lua.create_table()?;
    c_item.set(
        "GetItemInfo",
        lua.create_function(|_, _item_id: Value| {
            // Return nil - no item info in simulation
            Ok(Value::Nil)
        })?,
    )?;
    c_item.set(
        "GetItemInfoInstant",
        lua.create_function(|lua, item_id: Value| {
            // GetItemInfoInstant returns: itemID, itemType, itemSubType, itemEquipLoc, icon, classID, subClassID
            // We only have item ID, so return that with stub values
            let id = match item_id {
                Value::Integer(n) => n as i32,
                Value::Number(n) => n as i32,
                Value::String(s) => {
                    // Could be item link or name
                    if let Ok(s) = s.to_str() {
                        if let Some(start) = s.find("|Hitem:") {
                            let rest = &s[start + 7..];
                            if let Some(end) = rest.find(':') {
                                rest[..end].parse().unwrap_or(0)
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                _ => return Ok(mlua::MultiValue::new()),
            };
            if id == 0 {
                return Ok(mlua::MultiValue::new());
            }
            // Return: itemID, itemType, itemSubType, itemEquipLoc, icon, classID, subClassID
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(id as i64),       // itemID
                Value::String(lua.create_string("Miscellaneous")?), // itemType
                Value::String(lua.create_string("Junk")?), // itemSubType
                Value::String(lua.create_string("")?), // itemEquipLoc
                Value::Integer(134400),          // icon (INV_Misc_Bag_07)
                Value::Integer(15),              // classID (Miscellaneous)
                Value::Integer(0),               // subClassID
            ]))
        })?,
    )?;
    c_item.set(
        "GetItemIDForItemInfo",
        lua.create_function(|_, item_id: Value| {
            // GetItemIDForItemInfo extracts item ID from itemID, name, or link
            let id = match item_id {
                Value::Integer(n) => n as i32,
                Value::Number(n) => n as i32,
                Value::String(s) => {
                    if let Ok(s) = s.to_str() {
                        if let Some(start) = s.find("|Hitem:") {
                            let rest = &s[start + 7..];
                            if let Some(end) = rest.find(':') {
                                rest[..end].parse().unwrap_or(0)
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                _ => 0,
            };
            if id == 0 {
                Ok(Value::Nil)
            } else {
                Ok(Value::Integer(id as i64))
            }
        })?,
    )?;
    c_item.set(
        "GetItemIconByID",
        lua.create_function(|_, _item_id: i32| Ok(134400i32))?, // INV_Misc_Bag_07
    )?;
    c_item.set(
        "GetItemSubClassInfo",
        lua.create_function(|lua, (class_id, subclass_id): (i32, i32)| {
            // Return item subclass name based on class/subclass IDs
            let name = match (class_id, subclass_id) {
                // Weapons (class 2)
                (2, 0) => "One-Handed Axes",
                (2, 1) => "Two-Handed Axes",
                (2, 2) => "Bows",
                (2, 3) => "Guns",
                (2, 4) => "One-Handed Maces",
                (2, 5) => "Two-Handed Maces",
                (2, 6) => "Polearms",
                (2, 7) => "One-Handed Swords",
                (2, 8) => "Two-Handed Swords",
                (2, 9) => "Warglaives",
                (2, 10) => "Staves",
                (2, 13) => "Fist Weapons",
                (2, 14) => "Miscellaneous",
                (2, 15) => "Daggers",
                (2, 16) => "Thrown",
                (2, 18) => "Crossbows",
                (2, 19) => "Wands",
                (2, 20) => "Fishing Poles",
                // Armor (class 4)
                (4, 0) => "Miscellaneous",
                (4, 1) => "Cloth",
                (4, 2) => "Leather",
                (4, 3) => "Mail",
                (4, 4) => "Plate",
                (4, 6) => "Shield",
                _ => "Unknown",
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    c_item.set(
        "GetItemCount",
        lua.create_function(|_, (_item_id, _include_bank, _include_charges, _include_reagent_bank): (Value, Option<bool>, Option<bool>, Option<bool>)| {
            // No items in simulation
            Ok(0)
        })?,
    )?;
    c_item.set(
        "GetItemClassInfo",
        lua.create_function(|lua, class_id: i32| {
            let name = match class_id {
                0 => "Consumable",
                1 => "Container",
                2 => "Weapon",
                3 => "Gem",
                4 => "Armor",
                5 => "Reagent",
                6 => "Projectile",
                7 => "Tradeskill",
                8 => "Item Enhancement",
                9 => "Recipe",
                10 => "Currency (Obsolete)",
                11 => "Quiver",
                12 => "Quest",
                13 => "Key",
                14 => "Permanent (Obsolete)",
                15 => "Miscellaneous",
                16 => "Glyph",
                17 => "Battle Pets",
                18 => "WoW Token",
                _ => "Unknown",
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    c_item.set(
        "GetItemSpecInfo",
        lua.create_function(|lua, _item_id: Value| {
            // Returns a table of spec IDs that can use this item, or nil if all specs can
            lua.create_table()
        })?,
    )?;
    c_item.set(
        "GetItemNameByID",
        lua.create_function(|lua, item_id: i32| {
            Ok(Value::String(lua.create_string(&format!("Item {}", item_id))?))
        })?,
    )?;
    c_item.set(
        "GetDetailedItemLevelInfo",
        lua.create_function(|_, _item_link: Value| {
            // Returns: actualItemLevel, previewLevel, sparseItemLevel
            Ok((0i32, 0i32, 0i32))
        })?,
    )?;
    c_item.set(
        "IsItemBindToAccountUntilEquip",
        lua.create_function(|_, _item_link: Value| Ok(false))?,
    )?;
    c_item.set(
        "GetItemLink",
        lua.create_function(|lua, item_id: i32| {
            let link = format!("|cffffffff|Hitem:{}::::::::80:::::|h[Item {}]|h|r", item_id, item_id);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;
    c_item.set(
        "GetItemQualityByID",
        lua.create_function(|_, _item_id: i32| Ok(1i32))?, // Common quality
    )?;
    c_item.set(
        "GetItemLearnTransmogSet",
        lua.create_function(|_, _item_id: i32| {
            // Returns nil if item doesn't teach a transmog set
            Ok(Value::Nil)
        })?,
    )?;
    c_item.set(
        "RequestLoadItemDataByID",
        lua.create_function(|_, _item_id: i32| {
            // Request asynchronous item data loading - stub that does nothing
            Ok(())
        })?,
    )?;
    globals.set("C_Item", c_item)?;

    // Legacy global GetItemInfo
    globals.set(
        "GetItemInfo",
        lua.create_function(|_, _item_id: Value| Ok(Value::Nil))?,
    )?;

    // GetItemID(itemLink) - Extract item ID from item link
    globals.set(
        "GetItemID",
        lua.create_function(|_, item_link: Option<String>| {
            // Parse item link format: |Hitem:12345:...| and extract 12345
            if let Some(link) = item_link {
                if let Some(start) = link.find("|Hitem:") {
                    let rest = &link[start + 7..];
                    if let Some(end) = rest.find(':') {
                        if let Ok(id) = rest[..end].parse::<i32>() {
                            return Ok(Some(id));
                        }
                    }
                }
            }
            Ok(None)
        })?,
    )?;

    // GetItemCount(itemID, includeBankItems, includeCharges) - Get count of item in inventory
    globals.set(
        "GetItemCount",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(0))?,
    )?;

    // GetItemClassInfo(classID) - Get item class name
    globals.set(
        "GetItemClassInfo",
        lua.create_function(|lua, class_id: i32| {
            let name = match class_id {
                0 => "Consumable",
                1 => "Container",
                2 => "Weapon",
                3 => "Gem",
                4 => "Armor",
                5 => "Reagent",
                6 => "Projectile",
                7 => "Tradeskill",
                8 => "Item Enhancement",
                9 => "Recipe",
                10 => "Currency (deprecated)",
                11 => "Quiver",
                12 => "Quest",
                13 => "Key",
                14 => "Permanent (deprecated)",
                15 => "Miscellaneous",
                16 => "Glyph",
                17 => "Battle Pets",
                18 => "WoW Token",
                19 => "Profession",
                _ => "",
            };
            if name.is_empty() {
                Ok(Value::Nil)
            } else {
                Ok(Value::String(lua.create_string(name)?))
            }
        })?,
    )?;

    // GetItemSpecInfo(itemID) - Get spec info for item
    globals.set(
        "GetItemSpecInfo",
        lua.create_function(|_, _item_id: i32| Ok(Value::Nil))?,
    )?;

    // IsArtifactRelicItem(itemID) - Check if item is artifact relic
    globals.set(
        "IsArtifactRelicItem",
        lua.create_function(|_, _item_id: i32| Ok(false))?,
    )?;

    // GetTradeSkillTexture(index) - Get tradeskill icon
    globals.set(
        "GetTradeSkillTexture",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;

    // GetSpellLink(spellID) - Get spell link
    globals.set(
        "GetSpellLink",
        lua.create_function(|lua, spell_id: i32| {
            // Return a basic spell link format
            let link = format!("|Hspell:{}|h[Spell {}]|h", spell_id, spell_id);
            Ok(Value::String(lua.create_string(&link)?))
        })?,
    )?;

    // GetSpellIcon(spellID) - Get spell icon texture
    globals.set(
        "GetSpellIcon",
        lua.create_function(|_, _spell_id: i32| Ok(136243))?, // INV_Misc_QuestionMark
    )?;

    // GetSpellTexture(spellID) - Get spell icon texture (alternative API)
    globals.set(
        "GetSpellTexture",
        lua.create_function(|_, _spell_id: i32| Ok(136243))?, // INV_Misc_QuestionMark
    )?;

    // GetSpellCooldown(spellID) - Get spell cooldown info
    globals.set(
        "GetSpellCooldown",
        lua.create_function(|_, _spell_id: Value| {
            // Return: start, duration, enabled, modRate
            Ok((0.0_f64, 0.0_f64, 1, 1.0_f64))
        })?,
    )?;

    // IsSpellKnown(spellID, isPetSpell) - Check if spell is known
    globals.set(
        "IsSpellKnown",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?,
    )?;

    // IsPlayerSpell(spellID) - Check if spell is a player spell
    globals.set(
        "IsPlayerSpell",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;

    // IsSpellKnownOrOverridesKnown(spellID) - Check if spell or override is known
    globals.set(
        "IsSpellKnownOrOverridesKnown",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;

    // SendChatMessage(msg, chatType, languageID, target) - Send chat message
    globals.set(
        "SendChatMessage",
        lua.create_function(|_, _args: mlua::MultiValue| {
            // No-op in simulation
            Ok(())
        })?,
    )?;

    // C_Container namespace - bag/container functions
    let c_container = lua.create_table()?;
    c_container.set(
        "GetContainerNumSlots",
        lua.create_function(|_, bag: i32| {
            // Return bag slot counts (0 = backpack has 16 slots, bags 1-4 vary)
            Ok(if bag == 0 { 16 } else { 0 })
        })?,
    )?;
    c_container.set(
        "GetContainerItemID",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_container.set(
        "GetContainerItemLink",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_container.set(
        "GetContainerItemInfo",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    globals.set("C_Container", c_container)?;

    // C_EncodingUtil namespace - string encoding/compression utilities
    // These are stubs - actual compression/encoding not implemented
    let c_encoding_util = lua.create_table()?;
    c_encoding_util.set(
        "CompressString",
        lua.create_function(|lua, (data, _method): (String, Option<i32>)| {
            // Return the data as-is (no actual compression in simulator)
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    c_encoding_util.set(
        "DecompressString",
        lua.create_function(|lua, (data, _method): (String, Option<i32>)| {
            // Return the data as-is (no actual decompression in simulator)
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    c_encoding_util.set(
        "EncodeBase64",
        lua.create_function(|lua, data: String| {
            // Return the data as-is (no actual base64 encoding)
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    c_encoding_util.set(
        "DecodeBase64",
        lua.create_function(|lua, data: String| {
            // Return the data as-is (no actual base64 decoding)
            Ok(Value::String(lua.create_string(&data)?))
        })?,
    )?;
    globals.set("C_EncodingUtil", c_encoding_util)?;

    // Legacy global container functions
    let get_container_num_slots = lua.create_function(|_, bag: i32| {
        Ok(if bag == 0 { 16 } else { 0 })
    })?;
    globals.set("GetContainerNumSlots", get_container_num_slots)?;
    globals.set(
        "GetContainerItemID",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetContainerItemLink",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;

    // Inventory slot functions
    globals.set(
        "GetInventorySlotInfo",
        lua.create_function(|_, slot_name: String| {
            // Return slot ID for known slot names
            let slot_id = match slot_name.as_str() {
                "HeadSlot" => 1,
                "NeckSlot" => 2,
                "ShoulderSlot" => 3,
                "BackSlot" => 15,
                "ChestSlot" => 5,
                "ShirtSlot" => 4,
                "TabardSlot" => 19,
                "WristSlot" => 9,
                "HandsSlot" => 10,
                "WaistSlot" => 6,
                "LegsSlot" => 7,
                "FeetSlot" => 8,
                "Finger0Slot" => 11,
                "Finger1Slot" => 12,
                "Trinket0Slot" => 13,
                "Trinket1Slot" => 14,
                "MainHandSlot" => 16,
                "SecondaryHandSlot" => 17,
                "RangedSlot" => 18,
                "AmmoSlot" => 0,
                _ => 0,
            };
            Ok(slot_id)
        })?,
    )?;

    // Pet action functions
    globals.set(
        "GetPetActionInfo",
        lua.create_function(|_, _slot: i32| {
            // Return: name, texture, isToken, isActive, autoCastAllowed, autoCastEnabled, spellID, checksRange, inRange
            // Return nil for no action in slot
            Ok(Value::Nil)
        })?,
    )?;
    globals.set(
        "GetPetActionCooldown",
        lua.create_function(|_, _slot: i32| {
            // Return: start, duration, enable
            Ok((0.0, 0.0, 0))
        })?,
    )?;

    // Legacy global CVar functions (call through to C_CVar)
    let state_for_legacy_getcvar = Rc::clone(&state);
    let get_cvar = lua.create_function(move |lua, cvar: String| {
        let state = state_for_legacy_getcvar.borrow();
        match state.cvars.get(&cvar) {
            Some(value) => Ok(Value::String(lua.create_string(&value)?)),
            None => Ok(Value::Nil),
        }
    })?;
    globals.set("GetCVar", get_cvar)?;

    let state_for_legacy_setcvar = Rc::clone(&state);
    let set_cvar = lua.create_function(move |_, (cvar, value): (String, String)| {
        let state = state_for_legacy_setcvar.borrow();
        Ok(state.cvars.set(&cvar, &value))
    })?;
    globals.set("SetCVar", set_cvar)?;

    // GetFramerate() - returns the current frame rate
    globals.set(
        "GetFramerate",
        lua.create_function(|_, ()| Ok(60.0_f64))?,
    )?;

    // GetCameraZoom() - returns the current camera zoom level
    globals.set(
        "GetCameraZoom",
        lua.create_function(|_, ()| Ok(8.0_f64))?,
    )?;

    // CameraZoomIn(increment) - zoom camera in
    globals.set(
        "CameraZoomIn",
        lua.create_function(|_, _increment: Option<f64>| Ok(()))?,
    )?;

    // CameraZoomOut(increment) - zoom camera out
    globals.set(
        "CameraZoomOut",
        lua.create_function(|_, _increment: Option<f64>| Ok(()))?,
    )?;

    // C_AddOns namespace - addon management
    let c_addons = lua.create_table()?;
    c_addons.set(
        "GetAddOnMetadata",
        lua.create_function(|lua, (addon, field): (String, String)| {
            // Return stub metadata - WeakAuras checks Version and X-Flavor
            let value = match field.as_str() {
                "Version" => "@project-version@",
                "X-Flavor" => "Mainline",
                "Title" => addon.as_str(),
                "Notes" => "Addon description",
                "Author" => "Unknown Author",
                "Group" => addon.as_str(), // Default group is the addon's name
                "Category" => "", // No default category
                _ => "",
            };
            if value.is_empty() {
                Ok(Value::Nil)
            } else {
                Ok(Value::String(lua.create_string(value)?))
            }
        })?,
    )?;
    c_addons.set(
        "EnableAddOn",
        lua.create_function(|_, _addon: String| Ok(()))?,
    )?;
    c_addons.set(
        "DisableAddOn",
        lua.create_function(|_, _addon: String| Ok(()))?,
    )?;
    // GetNumAddOns - return actual addon count
    let state_for_num = Rc::clone(&state);
    c_addons.set(
        "GetNumAddOns",
        lua.create_function(move |_, ()| {
            let state = state_for_num.borrow();
            Ok(state.addons.len() as i32)
        })?,
    )?;
    // GetAddOnInfo - return actual addon info
    let state_for_info = Rc::clone(&state);
    c_addons.set(
        "GetAddOnInfo",
        lua.create_function(move |lua, index_or_name: Value| {
            // Return: name, title, notes, loadable, reason, security, newVersion
            let state = state_for_info.borrow();
            let addon = match index_or_name {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize; // Lua is 1-indexed
                    state.addons.get(idx)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state.addons.iter().find(|a| a.folder_name == &*name)
                }
                _ => None,
            };

            if let Some(addon) = addon {
                // Mark all addons as loadable so they show with gold text
                let loadable = true;
                Ok((
                    Value::String(lua.create_string(&addon.folder_name)?),
                    Value::String(lua.create_string(&addon.title)?),
                    Value::String(lua.create_string(&addon.notes)?),
                    Value::Boolean(loadable),
                    Value::Nil, // reason
                    Value::String(lua.create_string("INSECURE")?),
                    Value::Boolean(false), // newVersion
                ))
            } else {
                Ok((
                    Value::Nil,
                    Value::Nil,
                    Value::Nil,
                    Value::Boolean(false),
                    Value::Nil,
                    Value::Nil,
                    Value::Boolean(false),
                ))
            }
        })?,
    )?;
    // IsAddOnLoaded - check if addon is actually loaded
    let state_for_loaded = Rc::clone(&state);
    c_addons.set(
        "IsAddOnLoaded",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_loaded.borrow();
            let found = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state.addons.get(idx).map(|a| a.loaded).unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state.addons.iter().any(|a| a.folder_name == &*name && a.loaded)
                }
                _ => false,
            };
            Ok(found)
        })?,
    )?;
    c_addons.set(
        "IsAddOnLoadable",
        lua.create_function(|_, _addon: String| Ok(true))?,
    )?;
    // IsAddOnLoadOnDemand - check actual LOD flag
    let state_for_lod = Rc::clone(&state);
    c_addons.set(
        "IsAddOnLoadOnDemand",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_lod.borrow();
            let lod = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state.addons.get(idx).map(|a| a.load_on_demand).unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state.addons.iter().find(|a| a.folder_name == &*name)
                        .map(|a| a.load_on_demand).unwrap_or(false)
                }
                _ => false,
            };
            Ok(lod)
        })?,
    )?;
    c_addons.set(
        "GetAddOnOptionalDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;
    c_addons.set(
        "GetAddOnDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;
    c_addons.set(
        "LoadAddOn",
        lua.create_function(|_, _addon: String| Ok((true, Value::Nil)))?,
    )?;
    // DoesAddOnExist - check if addon is in the registry
    let state_for_exists = Rc::clone(&state);
    c_addons.set(
        "DoesAddOnExist",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_exists.borrow();
            let exists = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    idx < state.addons.len()
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state.addons.iter().any(|a| a.folder_name == &*name)
                }
                _ => false,
            };
            Ok(exists)
        })?,
    )?;
    // GetAddOnEnableState - check actual enabled state
    let state_for_enable = Rc::clone(&state);
    c_addons.set(
        "GetAddOnEnableState",
        lua.create_function(move |_, (addon, _character): (Value, Option<String>)| {
            // Returns: enabled state (0 = disabled, 1 = enabled for some, 2 = enabled for all)
            let state = state_for_enable.borrow();
            let enabled = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state.addons.get(idx).map(|a| a.enabled).unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state.addons.iter().find(|a| a.folder_name == &*name)
                        .map(|a| a.enabled).unwrap_or(false)
                }
                _ => false,
            };
            Ok(if enabled { 2i32 } else { 0i32 })
        })?,
    )?;
    // GetAddOnName - return folder name
    let state_for_name = Rc::clone(&state);
    c_addons.set(
        "GetAddOnName",
        lua.create_function(move |lua, index: i64| {
            let state = state_for_name.borrow();
            let idx = (index - 1) as usize;
            if let Some(addon) = state.addons.get(idx) {
                Ok(Value::String(lua.create_string(&addon.folder_name)?))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;
    // GetAddOnTitle - return display title
    let state_for_title = Rc::clone(&state);
    c_addons.set(
        "GetAddOnTitle",
        lua.create_function(move |lua, index: i64| {
            let state = state_for_title.borrow();
            let idx = (index - 1) as usize;
            if let Some(addon) = state.addons.get(idx) {
                Ok(Value::String(lua.create_string(&addon.title)?))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;
    // GetAddOnNotes - return addon description/notes
    let state_for_notes = Rc::clone(&state);
    c_addons.set(
        "GetAddOnNotes",
        lua.create_function(move |lua, index: i64| {
            let state = state_for_notes.borrow();
            let idx = (index - 1) as usize;
            if let Some(addon) = state.addons.get(idx) {
                if addon.notes.is_empty() {
                    Ok(Value::Nil)
                } else {
                    Ok(Value::String(lua.create_string(&addon.notes)?))
                }
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;
    // GetAddOnSecurity - return security level (always INSECURE for addons)
    c_addons.set(
        "GetAddOnSecurity",
        lua.create_function(|lua, _index: i64| {
            // Security levels: SECURE, INSECURE, BANNED
            Ok(Value::String(lua.create_string("INSECURE")?))
        })?,
    )?;
    // IsAddonVersionCheckEnabled - check if addon version validation is enabled
    c_addons.set(
        "IsAddonVersionCheckEnabled",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    // SetAddonVersionCheck - toggle addon version validation
    c_addons.set(
        "SetAddonVersionCheck",
        lua.create_function(|_, _enabled: bool| Ok(()))?,
    )?;
    globals.set("C_AddOns", c_addons)?;

    // ADDON_ACTIONS_BLOCKED - table of addon names that have blocked actions (used by AddonList)
    // This is normally populated by the game when addons use protected functions
    globals.set("ADDON_ACTIONS_BLOCKED", lua.create_table()?)?;

    // AddOnPerformance - addon performance monitoring (nil when not loaded)
    // This is populated by Blizzard_AddOnPerformanceWarning addon
    globals.set("AddOnPerformance", Value::Nil)?;

    // C_XMLUtil namespace - XML template utilities
    let c_xml_util = lua.create_table()?;

    // GetTemplateInfo(templateName) - returns template type, width, height
    c_xml_util.set(
        "GetTemplateInfo",
        lua.create_function(|lua, template_name: String| {
            if let Some(info) = get_template_info(&template_name) {
                let result = lua.create_table()?;
                // WoW uses lowercase "type" for frame type
                result.set("type", info.frame_type)?;
                result.set("width", info.width)?;
                result.set("height", info.height)?;
                Ok(Value::Table(result))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;
    globals.set("C_XMLUtil", c_xml_util)?;

    // Legacy global function version
    globals.set(
        "GetAddOnEnableState",
        lua.create_function(|_, (_addon, _character): (Value, Option<String>)| {
            Ok(2i32)
        })?,
    )?;

    // C_Console namespace - console command system
    let c_console = lua.create_table()?;
    c_console.set(
        "GetAllCommands",
        lua.create_function(|lua, ()| {
            // Return empty table of console commands
            lua.create_table()
        })?,
    )?;
    c_console.set(
        "GetColorFromType",
        lua.create_function(|lua, _command_type: i32| {
            let color = lua.create_table()?;
            color.set("r", 1.0)?;
            color.set("g", 1.0)?;
            color.set("b", 1.0)?;
            Ok(color)
        })?,
    )?;
    globals.set("C_Console", c_console)?;

    // C_VoiceChat namespace - voice chat and TTS
    let c_voice_chat = lua.create_table()?;
    c_voice_chat.set(
        "SpeakText",
        lua.create_function(|_, (_voice_id, _text, _dest, _rate, _volume): (i32, String, i32, i32, i32)| {
            // Stub - would play TTS in real game
            Ok(())
        })?,
    )?;
    c_voice_chat.set(
        "StopSpeakingText",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    c_voice_chat.set(
        "IsSpeakingText",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_voice_chat.set(
        "GetTtsVoices",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    globals.set("C_VoiceChat", c_voice_chat)?;

    // C_TTSSettings namespace - TTS settings
    let c_tts_settings = lua.create_table()?;
    c_tts_settings.set(
        "GetSpeechRate",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_tts_settings.set(
        "SetSpeechRate",
        lua.create_function(|_, _rate: i32| Ok(()))?,
    )?;
    c_tts_settings.set(
        "GetSpeechVolume",
        lua.create_function(|_, ()| Ok(100))?,
    )?;
    c_tts_settings.set(
        "SetSpeechVolume",
        lua.create_function(|_, _volume: i32| Ok(()))?,
    )?;
    c_tts_settings.set(
        "GetVoiceOptionID",
        lua.create_function(|_, _option: i32| Ok(0))?,
    )?;
    c_tts_settings.set(
        "SetVoiceOption",
        lua.create_function(|_, (_option, _voice_id): (i32, i32)| Ok(()))?,
    )?;
    globals.set("C_TTSSettings", c_tts_settings)?;

    // Legacy console functions
    globals.set(
        "ConsoleGetAllCommands",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;

    // C_Reputation namespace - faction reputation system
    let c_reputation = lua.create_table()?;
    c_reputation.set(
        "GetFactionDataByID",
        lua.create_function(|_, _faction_id: i32| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "IsFactionParagon",
        lua.create_function(|_, _faction_id: i32| Ok(false))?,
    )?;
    c_reputation.set(
        "GetFactionParagonInfo",
        lua.create_function(|_, _faction_id: i32| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "GetNumFactions",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_reputation.set(
        "GetFactionInfo",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "GetWatchedFactionData",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "SetWatchedFactionByID",
        lua.create_function(|_, _faction_id: i32| Ok(()))?,
    )?;
    globals.set("C_Reputation", c_reputation)?;

    // C_Texture namespace - texture handling
    let c_texture = lua.create_table()?;
    c_texture.set(
        "GetAtlasInfo",
        lua.create_function(|lua, atlas_name: String| {
            // Look up atlas in our database
            if let Some(atlas_info) = crate::atlas::get_atlas_info(&atlas_name) {
                let info = lua.create_table()?;
                info.set("width", atlas_info.width)?;
                info.set("height", atlas_info.height)?;
                info.set("leftTexCoord", atlas_info.left_tex_coord)?;
                info.set("rightTexCoord", atlas_info.right_tex_coord)?;
                info.set("topTexCoord", atlas_info.top_tex_coord)?;
                info.set("bottomTexCoord", atlas_info.bottom_tex_coord)?;
                info.set("file", atlas_info.file)?;
                info.set("tilesHorizontally", atlas_info.tiles_horizontally)?;
                info.set("tilesVertically", atlas_info.tiles_vertically)?;
                Ok(Value::Table(info))
            } else {
                // Return nil for unknown atlases
                Ok(Value::Nil)
            }
        })?,
    )?;
    c_texture.set(
        "GetFilenameFromFileDataID",
        lua.create_function(|_, _file_data_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_Texture", c_texture)?;

    // C_CreatureInfo namespace - NPC/creature information
    let c_creature_info = lua.create_table()?;
    c_creature_info.set(
        "GetClassInfo",
        lua.create_function(|lua, class_id: i32| {
            // Return class info table
            let info = lua.create_table()?;
            let class_name = match class_id {
                1 => "WARRIOR",
                2 => "PALADIN",
                3 => "HUNTER",
                4 => "ROGUE",
                5 => "PRIEST",
                6 => "DEATHKNIGHT",
                7 => "SHAMAN",
                8 => "MAGE",
                9 => "WARLOCK",
                10 => "MONK",
                11 => "DRUID",
                12 => "DEMONHUNTER",
                13 => "EVOKER",
                _ => "UNKNOWN",
            };
            info.set("className", class_name)?;
            info.set("classFile", class_name)?;
            info.set("classID", class_id)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_creature_info.set(
        "GetRaceInfo",
        lua.create_function(|lua, race_id: i32| {
            let info = lua.create_table()?;
            // WoW race data: (name, clientFileString)
            let (race_name, client_file) = match race_id {
                1 => ("Human", "Human"),
                2 => ("Orc", "Orc"),
                3 => ("Dwarf", "Dwarf"),
                4 => ("Night Elf", "NightElf"),
                5 => ("Undead", "Scourge"),
                6 => ("Tauren", "Tauren"),
                7 => ("Gnome", "Gnome"),
                8 => ("Troll", "Troll"),
                9 => ("Goblin", "Goblin"),
                10 => ("Blood Elf", "BloodElf"),
                11 => ("Draenei", "Draenei"),
                22 => ("Worgen", "Worgen"),
                24 => ("Pandaren", "Pandaren"),
                25 => ("Pandaren", "Pandaren"),
                26 => ("Pandaren", "Pandaren"),
                27 => ("Nightborne", "Nightborne"),
                28 => ("Highmountain Tauren", "HighmountainTauren"),
                29 => ("Void Elf", "VoidElf"),
                30 => ("Lightforged Draenei", "LightforgedDraenei"),
                31 => ("Zandalari Troll", "ZandalariTroll"),
                32 => ("Kul Tiran", "KulTiran"),
                34 => ("Dark Iron Dwarf", "DarkIronDwarf"),
                35 => ("Vulpera", "Vulpera"),
                36 => ("Mag'har Orc", "MagharOrc"),
                37 => ("Mechagnome", "Mechagnome"),
                52 | 70 => ("Dracthyr", "Dracthyr"),
                84 | 85 => ("Earthen", "Earthen"),
                _ => ("Unknown", "Unknown"),
            };
            info.set("raceName", race_name)?;
            info.set("raceID", race_id)?;
            info.set("clientFileString", client_file)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_creature_info.set(
        "GetCreatureTypeIDs",
        lua.create_function(|lua, ()| {
            // WoW creature types: Beast, Dragonkin, Demon, Elemental, Giant, Undead, Humanoid, Critter, Mechanical, etc.
            let ids = lua.create_table()?;
            for (i, id) in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].iter().enumerate() {
                ids.set(i + 1, *id)?;
            }
            Ok(ids)
        })?,
    )?;
    c_creature_info.set(
        "GetCreatureTypeInfo",
        lua.create_function(|lua, creature_type_id: i32| {
            let info = lua.create_table()?;
            let name = match creature_type_id {
                1 => "Beast",
                2 => "Dragonkin",
                3 => "Demon",
                4 => "Elemental",
                5 => "Giant",
                6 => "Undead",
                7 => "Humanoid",
                8 => "Critter",
                9 => "Mechanical",
                10 => "Not specified",
                _ => "Unknown",
            };
            info.set("name", name)?;
            info.set("creatureTypeID", creature_type_id)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_creature_info.set(
        "GetCreatureFamilyIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_creature_info.set(
        "GetCreatureFamilyInfo",
        lua.create_function(|_, _family_id: i32| Ok(Value::Nil))?,
    )?;
    c_creature_info.set(
        "GetFactionInfo",
        lua.create_function(|lua, race_id: i32| {
            // Return faction info for a race
            let info = lua.create_table()?;
            // Map races to factions: Alliance (0) or Horde (1)
            let (name, group_tag) = match race_id {
                // Alliance races
                1 | 3 | 4 | 7 | 11 | 22 | 29 | 30 | 32 | 34 | 37 => ("Alliance", "Alliance"),
                // Horde races
                2 | 5 | 6 | 8 | 9 | 10 | 27 | 28 | 31 | 35 | 36 => ("Horde", "Horde"),
                // Neutral (Pandaren)
                24 | 25 | 26 => ("Neutral", "Neutral"),
                // Dracthyr - can be either, default to neutral
                52 | 70 => ("Neutral", "Neutral"),
                // Earthen - can be either, default to neutral
                84 | 85 => ("Neutral", "Neutral"),
                _ => return Ok(Value::Nil),
            };
            info.set("name", name)?;
            info.set("groupTag", group_tag)?;
            Ok(Value::Table(info))
        })?,
    )?;
    globals.set("C_CreatureInfo", c_creature_info)?;

    // C_Covenants namespace - Shadowlands covenant system
    let c_covenants = lua.create_table()?;
    c_covenants.set(
        "GetCovenantData",
        lua.create_function(|lua, covenant_id: i32| {
            let data = lua.create_table()?;
            let name = match covenant_id {
                1 => "Kyrian",
                2 => "Venthyr",
                3 => "Night Fae",
                4 => "Necrolord",
                _ => "None",
            };
            data.set("ID", covenant_id)?;
            data.set("name", name)?;
            data.set("textureKit", "")?;
            Ok(Value::Table(data))
        })?,
    )?;
    c_covenants.set(
        "GetActiveCovenantID",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_covenants.set(
        "GetCovenantIDs",
        lua.create_function(|lua, ()| {
            let ids = lua.create_table()?;
            ids.set(1, 1)?;
            ids.set(2, 2)?;
            ids.set(3, 3)?;
            ids.set(4, 4)?;
            Ok(ids)
        })?,
    )?;
    globals.set("C_Covenants", c_covenants)?;

    // C_Soulbinds namespace - Shadowlands soulbind system
    let c_soulbinds = lua.create_table()?;
    c_soulbinds.set(
        "GetActiveSoulbindID",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_soulbinds.set(
        "GetSoulbindData",
        lua.create_function(|lua, _soulbind_id: i32| {
            let data = lua.create_table()?;
            data.set("ID", 0)?;
            data.set("name", "")?;
            data.set("covenantID", 0)?;
            Ok(Value::Table(data))
        })?,
    )?;
    c_soulbinds.set(
        "GetConduitCollection",
        lua.create_function(|lua, _conduit_type: i32| lua.create_table())?,
    )?;
    c_soulbinds.set(
        "GetConduitCollectionData",
        lua.create_function(|_, _conduit_id: i32| Ok(Value::Nil))?,
    )?;
    c_soulbinds.set(
        "IsConduitInstalled",
        lua.create_function(|_, (_soulbind_id, _conduit_id): (i32, i32)| Ok(false))?,
    )?;
    globals.set("C_Soulbinds", c_soulbinds)?;

    // C_UnitAuras namespace - unit aura information (modern API)
    let c_unit_auras = lua.create_table()?;
    // GetAuraDataBySpellName(unit, spellName, filter) - Get aura data by spell name
    c_unit_auras.set(
        "GetAuraDataBySpellName",
        lua.create_function(|_, (_unit, _spell_name, _filter): (String, String, Option<String>)| {
            // Return nil (no aura found) - in real impl would return AuraData table
            Ok(Value::Nil)
        })?,
    )?;
    // GetAuraDataByIndex(unit, index, filter) - Get aura data by index
    c_unit_auras.set(
        "GetAuraDataByIndex",
        lua.create_function(|_, (_unit, _index, _filter): (String, i32, Option<String>)| {
            Ok(Value::Nil)
        })?,
    )?;
    // GetAuraDataByAuraInstanceID(unit, auraInstanceID) - Get aura data by instance ID
    c_unit_auras.set(
        "GetAuraDataByAuraInstanceID",
        lua.create_function(|_, (_unit, _instance_id): (String, i64)| Ok(Value::Nil))?,
    )?;
    // GetAuraDataBySlot(unit, slot) - Get aura data by slot
    c_unit_auras.set(
        "GetAuraDataBySlot",
        lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(Value::Nil))?,
    )?;
    // GetBuffDataByIndex(unit, index, filter) - Get buff data by index
    c_unit_auras.set(
        "GetBuffDataByIndex",
        lua.create_function(|_, (_unit, _index, _filter): (String, i32, Option<String>)| {
            Ok(Value::Nil)
        })?,
    )?;
    // GetDebuffDataByIndex(unit, index, filter) - Get debuff data by index
    c_unit_auras.set(
        "GetDebuffDataByIndex",
        lua.create_function(|_, (_unit, _index, _filter): (String, i32, Option<String>)| {
            Ok(Value::Nil)
        })?,
    )?;
    // GetPlayerAuraBySpellID(spellID) - Get player's aura by spell ID
    c_unit_auras.set(
        "GetPlayerAuraBySpellID",
        lua.create_function(|_, _spell_id: i32| Ok(Value::Nil))?,
    )?;
    // GetCooldownAuraBySpellID(spellID) - Get cooldown aura by spell ID
    c_unit_auras.set(
        "GetCooldownAuraBySpellID",
        lua.create_function(|_, _spell_id: i32| Ok(Value::Nil))?,
    )?;
    // GetAuraSlots(unit, filter) - Get aura slots
    c_unit_auras.set(
        "GetAuraSlots",
        lua.create_function(|_lua, (_unit, _filter): (String, Option<String>)| {
            // Return empty continuationToken and no slots
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Nil, // continuationToken
            ]))
        })?,
    )?;
    // AuraIsPrivate(unit, auraInstanceID) - Check if aura is private
    c_unit_auras.set(
        "AuraIsPrivate",
        lua.create_function(|_, (_unit, _instance_id): (String, i64)| Ok(false))?,
    )?;
    // IsAuraFilteredOutByInstanceID(unit, auraInstanceID, ...) - Check if filtered
    c_unit_auras.set(
        "IsAuraFilteredOutByInstanceID",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?,
    )?;
    // WantsAlteredForm() - Check if unit wants altered form display
    c_unit_auras.set(
        "WantsAlteredForm",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    // Private aura functions (stubs)
    c_unit_auras.set(
        "AddPrivateAuraAnchor",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    c_unit_auras.set(
        "RemovePrivateAuraAnchor",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    c_unit_auras.set(
        "AddPrivateAuraAppliedSound",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    c_unit_auras.set(
        "RemovePrivateAuraAppliedSound",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    c_unit_auras.set(
        "SetPrivateWarningTextAnchor",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    globals.set("C_UnitAuras", c_unit_auras)?;

    // C_CurrencyInfo namespace - currency information
    let c_currency_info = lua.create_table()?;
    c_currency_info.set(
        "GetCurrencyInfo",
        lua.create_function(|lua, currency_id: i32| {
            // Return stub currency info with basic fields
            let info = lua.create_table()?;
            info.set("name", format!("Currency {}", currency_id))?;
            info.set("currencyID", currency_id)?;
            info.set("quantity", 0)?;
            info.set("maxQuantity", 0)?;
            info.set("quality", 1)?;
            info.set("iconFileID", 0)?;
            info.set("discovered", false)?;
            info.set("isAccountWide", false)?;
            info.set("isAccountTransferable", false)?;
            info.set("transferPercentage", 0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_currency_info.set(
        "GetCurrencyInfoFromLink",
        lua.create_function(|_, _link: String| Ok(Value::Nil))?,
    )?;
    c_currency_info.set(
        "GetCurrencyListSize",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_currency_info.set(
        "GetCurrencyListInfo",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_currency_info.set(
        "GetWarResourcesCurrencyID",
        lua.create_function(|_, ()| Ok(1560))?, // War Resources currency ID
    )?;
    c_currency_info.set(
        "GetAzeriteCurrencyID",
        lua.create_function(|_, ()| Ok(1553))?, // Azerite currency ID (BfA)
    )?;
    c_currency_info.set(
        "GetBasicCurrencyInfo",
        lua.create_function(|lua, (currency_id, _quantity): (i32, Option<i32>)| {
            let info = lua.create_table()?;
            info.set("name", format!("Currency {}", currency_id))?;
            info.set("currencyID", currency_id)?;
            info.set("quantity", 0)?;
            info.set("iconFileID", 0)?;
            info.set("displayAmount", 0)?;
            Ok(Value::Table(info))
        })?,
    )?;
    globals.set("C_CurrencyInfo", c_currency_info)?;

    // C_VignetteInfo namespace - map vignette (special marker) info
    let c_vignette_info = lua.create_table()?;
    c_vignette_info.set(
        "GetVignettes",
        lua.create_function(|lua, ()| {
            // Return empty table (no vignettes in simulation)
            Ok(lua.create_table()?)
        })?,
    )?;
    c_vignette_info.set(
        "GetVignetteInfo",
        lua.create_function(|_, _vignette_guid: String| {
            // Return nil (no vignette info)
            Ok(Value::Nil)
        })?,
    )?;
    c_vignette_info.set(
        "GetVignettePosition",
        lua.create_function(|_, (_vignette_guid, _ui_map_id): (String, Option<i32>)| {
            // Return nil (no position)
            Ok(Value::Nil)
        })?,
    )?;
    c_vignette_info.set(
        "GetVignetteGUID",
        lua.create_function(|_, _object_guid: String| {
            Ok(Value::Nil)
        })?,
    )?;
    globals.set("C_VignetteInfo", c_vignette_info)?;

    // C_AreaPoiInfo namespace - area point of interest info
    let c_area_poi = lua.create_table()?;
    c_area_poi.set(
        "GetAreaPOIInfo",
        lua.create_function(|_, (_ui_map_id, _area_poi_id): (i32, i32)| {
            // Return nil (no POI info in simulation)
            Ok(Value::Nil)
        })?,
    )?;
    c_area_poi.set(
        "GetAreaPOISecondsLeft",
        lua.create_function(|_, _area_poi_id: i32| Ok(0i32))?,
    )?;
    c_area_poi.set(
        "IsAreaPOITimed",
        lua.create_function(|_, _area_poi_id: i32| Ok(false))?,
    )?;
    c_area_poi.set(
        "GetAreaPOIForMap",
        lua.create_function(|lua, _ui_map_id: i32| {
            // Return empty table
            Ok(lua.create_table()?)
        })?,
    )?;
    globals.set("C_AreaPoiInfo", c_area_poi)?;

    // C_PlayerChoice namespace - player choice (quest popup) system
    let c_player_choice = lua.create_table()?;
    c_player_choice.set(
        "GetCurrentPlayerChoiceInfo",
        lua.create_function(|_, ()| {
            // Return nil (no active player choice)
            Ok(Value::Nil)
        })?,
    )?;
    c_player_choice.set(
        "GetNumPlayerChoices",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_player_choice.set(
        "GetPlayerChoiceInfo",
        lua.create_function(|_, _choice_id: i32| Ok(Value::Nil))?,
    )?;
    c_player_choice.set(
        "GetPlayerChoiceOptionInfo",
        lua.create_function(|_, (_choice_id, _option_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_player_choice.set(
        "SendPlayerChoiceResponse",
        lua.create_function(|_, _response_id: i32| Ok(()))?,
    )?;
    c_player_choice.set(
        "IsWaitingForPlayerChoiceResponse",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_PlayerChoice", c_player_choice)?;

    // C_MajorFactions namespace - Major Factions/Renown system (DF+)
    let c_major_factions = lua.create_table()?;
    c_major_factions.set(
        "GetMajorFactionData",
        lua.create_function(|_, _faction_id: i32| {
            // Return nil (no major faction data)
            Ok(Value::Nil)
        })?,
    )?;
    c_major_factions.set(
        "GetMajorFactionIDs",
        lua.create_function(|lua, _expansion_id: Option<i32>| {
            // Return empty table
            Ok(lua.create_table()?)
        })?,
    )?;
    c_major_factions.set(
        "GetRenownLevels",
        lua.create_function(|lua, _faction_id: i32| {
            // Return empty table
            Ok(lua.create_table()?)
        })?,
    )?;
    c_major_factions.set(
        "GetCurrentRenownLevel",
        lua.create_function(|_, _faction_id: i32| Ok(0i32))?,
    )?;
    c_major_factions.set(
        "HasMaximumRenown",
        lua.create_function(|_, _faction_id: i32| Ok(false))?,
    )?;
    c_major_factions.set(
        "GetRenownRewardsForLevel",
        lua.create_function(|lua, (_faction_id, _renown_level): (i32, i32)| {
            Ok(lua.create_table()?)
        })?,
    )?;
    globals.set("C_MajorFactions", c_major_factions)?;

    // C_UIWidgetManager namespace - UI widgets (quest objectives, dungeon info, etc.)
    let c_ui_widget = lua.create_table()?;
    c_ui_widget.set(
        "GetAllWidgetsBySetID",
        lua.create_function(|lua, _set_id: i32| {
            // Return empty table
            Ok(lua.create_table()?)
        })?,
    )?;
    c_ui_widget.set(
        "GetStatusBarWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetTextWithStateWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetIconAndTextWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetCaptureBarWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetDoubleStatusBarWidgetVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetSpellDisplayVisualizationInfo",
        lua.create_function(|_, _widget_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetWidgetSetInfo",
        lua.create_function(|_, _set_id: i32| Ok(Value::Nil))?,
    )?;
    c_ui_widget.set(
        "GetTopCenterWidgetSetID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_ui_widget.set(
        "GetBelowMinimapWidgetSetID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_ui_widget.set(
        "GetObjectiveTrackerWidgetSetID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    globals.set("C_UIWidgetManager", c_ui_widget)?;

    // C_GossipInfo namespace - NPC gossip/dialog system
    let c_gossip_info = lua.create_table()?;
    c_gossip_info.set(
        "GetNumOptions",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_gossip_info.set(
        "GetOptions",
        lua.create_function(|lua, ()| Ok(lua.create_table()?))?,
    )?;
    c_gossip_info.set(
        "GetText",
        lua.create_function(|_, ()| Ok(""))?,
    )?;
    c_gossip_info.set(
        "SelectOption",
        lua.create_function(|_, (_option_id, _text, _confirmed): (i32, Option<String>, Option<bool>)| Ok(()))?,
    )?;
    c_gossip_info.set(
        "CloseGossip",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    c_gossip_info.set(
        "GetNumActiveQuests",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_gossip_info.set(
        "GetNumAvailableQuests",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_gossip_info.set(
        "GetActiveQuests",
        lua.create_function(|lua, ()| Ok(lua.create_table()?))?,
    )?;
    c_gossip_info.set(
        "GetAvailableQuests",
        lua.create_function(|lua, ()| Ok(lua.create_table()?))?,
    )?;
    c_gossip_info.set(
        "SelectActiveQuest",
        lua.create_function(|_, _index: i32| Ok(()))?,
    )?;
    c_gossip_info.set(
        "SelectAvailableQuest",
        lua.create_function(|_, _index: i32| Ok(()))?,
    )?;
    c_gossip_info.set(
        "GetFriendshipReputation",
        lua.create_function(|lua, _faction_id: Option<i32>| {
            // Return friendship reputation info table
            let info = lua.create_table()?;
            info.set("friendshipFactionID", 0)?;
            info.set("standing", 0)?;
            info.set("maxRep", 0)?;
            info.set("name", Value::Nil)?;
            info.set("text", Value::Nil)?;
            info.set("texture", Value::Nil)?;
            info.set("reaction", Value::Nil)?;
            info.set("reactionThreshold", 0)?;
            info.set("nextThreshold", Value::Nil)?;
            Ok(info)
        })?,
    )?;
    c_gossip_info.set(
        "GetFriendshipReputationRanks",
        lua.create_function(|lua, _faction_id: Option<i32>| {
            // Return friendship reputation ranks info table
            let info = lua.create_table()?;
            info.set("currentLevel", 0)?;
            info.set("maxLevel", 0)?;
            Ok(info)
        })?,
    )?;
    c_gossip_info.set(
        "ForceGossip",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_GossipInfo", c_gossip_info)?;

    // C_Calendar namespace - in-game calendar system
    let c_calendar = lua.create_table()?;
    c_calendar.set("GetDate", lua.create_function(|_, ()| {
        // Return: weekday, month, day, year
        Ok((1i32, 1i32, 1i32, 2024i32))
    })?)?;
    c_calendar.set("GetMonthInfo", lua.create_function(|lua, _offset: Option<i32>| {
        let info = lua.create_table()?;
        info.set("month", 1)?;
        info.set("year", 2024)?;
        info.set("numDays", 31)?;
        info.set("firstWeekday", 1)?;
        Ok(info)
    })?)?;
    c_calendar.set("GetNumDayEvents", lua.create_function(|_, (_offset, _day): (i32, i32)| Ok(0i32))?)?;
    c_calendar.set("GetDayEvent", lua.create_function(|_, (_offset, _day, _index): (i32, i32, i32)| Ok(Value::Nil))?)?;
    c_calendar.set("OpenCalendar", lua.create_function(|_, ()| Ok(()))?)?;
    c_calendar.set("CloseCalendar", lua.create_function(|_, ()| Ok(()))?)?;
    c_calendar.set("SetMonth", lua.create_function(|_, _offset: i32| Ok(()))?)?;
    c_calendar.set("SetAbsMonth", lua.create_function(|_, (_month, _year): (i32, i32)| Ok(()))?)?;
    c_calendar.set("GetMinDate", lua.create_function(|_, ()| Ok((1i32, 1i32, 2004i32)))?)?;
    c_calendar.set("GetMaxDate", lua.create_function(|_, ()| Ok((12i32, 31i32, 2030i32)))?)?;
    globals.set("C_Calendar", c_calendar)?;

    // C_CovenantCallings namespace - Shadowlands covenant callings
    let c_covenant_callings = lua.create_table()?;
    c_covenant_callings.set("AreCallingsUnlocked", lua.create_function(|_, ()| Ok(false))?)?;
    c_covenant_callings.set("RequestCallings", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("C_CovenantCallings", c_covenant_callings)?;

    // C_WeeklyRewards namespace - Great Vault rewards
    let c_weekly_rewards = lua.create_table()?;
    c_weekly_rewards.set("HasAvailableRewards", lua.create_function(|_, ()| Ok(false))?)?;
    c_weekly_rewards.set("CanClaimRewards", lua.create_function(|_, ()| Ok(false))?)?;
    c_weekly_rewards.set("GetActivities", lua.create_function(|lua, _type: Option<i32>| {
        lua.create_table()
    })?)?;
    c_weekly_rewards.set("GetNumCompletedDungeonRuns", lua.create_function(|_, ()| Ok(0i32))?)?;
    globals.set("C_WeeklyRewards", c_weekly_rewards)?;

    // DifficultyUtil - Helper for difficulty info
    let difficulty_util = lua.create_table()?;
    difficulty_util.set("GetDifficultyName", lua.create_function(|lua, diff_id: i32| {
        let name = match diff_id {
            1 => "Normal",
            2 => "Heroic",
            3 => "10 Player",
            4 => "25 Player",
            5 => "10 Player (Heroic)",
            6 => "25 Player (Heroic)",
            7 => "LFR",
            8 => "Mythic Keystone",
            14 => "Normal",
            15 => "Heroic",
            16 => "Mythic",
            17 => "LFR",
            23 => "Mythic",
            _ => "Unknown",
        };
        Ok(Value::String(lua.create_string(name)?))
    })?)?;
    let difficulty_id = lua.create_table()?;
    // Dungeon difficulties
    difficulty_id.set("DungeonNormal", 1)?;
    difficulty_id.set("DungeonHeroic", 2)?;
    difficulty_id.set("DungeonMythic", 23)?;
    difficulty_id.set("DungeonChallenge", 8)?;  // Mythic Keystone
    difficulty_id.set("DungeonTimewalker", 24)?;
    // Raid difficulties
    difficulty_id.set("Raid10Normal", 3)?;
    difficulty_id.set("Raid25Normal", 4)?;
    difficulty_id.set("Raid10Heroic", 5)?;
    difficulty_id.set("Raid25Heroic", 6)?;
    difficulty_id.set("Raid40", 9)?;
    difficulty_id.set("RaidLFR", 17)?;
    difficulty_id.set("RaidTimewalker", 33)?;
    difficulty_id.set("PrimaryRaidNormal", 14)?;
    difficulty_id.set("PrimaryRaidHeroic", 15)?;
    difficulty_id.set("PrimaryRaidMythic", 16)?;
    difficulty_id.set("PrimaryRaidLFR", 7)?;
    difficulty_util.set("ID", difficulty_id)?;
    globals.set("DifficultyUtil", difficulty_util)?;

    // ItemLocation - utility for creating item location objects
    let item_location = lua.create_table()?;
    item_location.set("CreateFromEquipmentSlot", lua.create_function(|lua, slot_id: i32| {
        let loc = lua.create_table()?;
        loc.set("slotID", slot_id)?;
        loc.set("bagID", Value::Nil)?;
        loc.set("IsEquipmentSlot", lua.create_function(|_, ()| Ok(true))?)?;
        loc.set("IsBagAndSlot", lua.create_function(|_, ()| Ok(false))?)?;
        loc.set("GetEquipmentSlot", lua.create_function(move |_, ()| Ok(slot_id))?)?;
        loc.set("IsValid", lua.create_function(|_, ()| Ok(true))?)?;
        loc.set("IsEqualTo", lua.create_function(|_, (_self, _other): (Value, Value)| Ok(false))?)?;
        loc.set("Clear", lua.create_function(|_, _self: Value| Ok(()))?)?;
        Ok(loc)
    })?)?;
    item_location.set("CreateFromBagAndSlot", lua.create_function(|lua, (bag_id, slot_id): (i32, i32)| {
        let loc = lua.create_table()?;
        loc.set("bagID", bag_id)?;
        loc.set("slotID", slot_id)?;
        loc.set("IsEquipmentSlot", lua.create_function(|_, ()| Ok(false))?)?;
        loc.set("IsBagAndSlot", lua.create_function(|_, ()| Ok(true))?)?;
        loc.set("GetBagAndSlot", lua.create_function(move |_, ()| Ok((bag_id, slot_id)))?)?;
        loc.set("IsValid", lua.create_function(|_, ()| Ok(true))?)?;
        loc.set("IsEqualTo", lua.create_function(|_, (_self, _other): (Value, Value)| Ok(false))?)?;
        loc.set("Clear", lua.create_function(|_, _self: Value| Ok(()))?)?;
        Ok(loc)
    })?)?;
    item_location.set("CreateEmpty", lua.create_function(|lua, ()| {
        let loc = lua.create_table()?;
        loc.set("IsValid", lua.create_function(|_, ()| Ok(false))?)?;
        loc.set("IsEquipmentSlot", lua.create_function(|_, ()| Ok(false))?)?;
        loc.set("IsBagAndSlot", lua.create_function(|_, ()| Ok(false))?)?;
        Ok(loc)
    })?)?;
    globals.set("ItemLocation", item_location)?;

    // WeeklyRewardsUtil - Helper for weekly rewards
    let weekly_rewards_util = lua.create_table()?;
    weekly_rewards_util.set("GetSlotMythicPlusLevel", lua.create_function(|_, _slot: i32| Ok(0i32))?)?;
    weekly_rewards_util.set("HasUnlockedRewards", lua.create_function(|_, _type: Option<i32>| Ok(false))?)?;
    globals.set("WeeklyRewardsUtil", weekly_rewards_util)?;

    // C_ContributionCollector namespace - Warfront contributions
    let c_contribution_collector = lua.create_table()?;
    c_contribution_collector.set("GetState", lua.create_function(|_, _id: i32| Ok(0i32))?)?;
    c_contribution_collector.set("GetContributionCollector", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    c_contribution_collector.set("GetManagedContributionsForCreatureID", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    c_contribution_collector.set("GetContributionResult", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    c_contribution_collector.set("IsAwaitingRewardQuestData", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("C_ContributionCollector", c_contribution_collector)?;

    // Encounter Journal functions
    globals.set("EJ_GetCreatureInfo", lua.create_function(|lua, (_index, section_id): (i32, Option<i32>)| {
        // Return: id, name, description, displayInfo, iconImage
        // Return stub values so callers don't crash on nil
        let id = section_id.unwrap_or(0);
        Ok((
            id, // creature id
            Value::String(lua.create_string(format!("Creature {}", id))?), // name
            Value::String(lua.create_string("")?), // description
            0i32, // displayInfo
            0i32, // iconImage
        ))
    })?)?;
    globals.set("EJ_GetEncounterInfo", lua.create_function(|_, _encounter_id: i32| {
        // Return: name, description, journalEncounterID, rootSectionID, link, journalInstanceID, dungeonEncounterID, instanceID
        Ok((Value::Nil, Value::Nil, 0i32, 0i32, Value::Nil, 0i32, 0i32, 0i32))
    })?)?;
    globals.set("EJ_GetInstanceInfo", lua.create_function(|_, _instance_id: i32| {
        // Return: name, description, bgImage, buttonImage, loreImage, buttonImage2, dungeonAreaMapID, link, shouldDisplayDifficulty
        Ok((Value::Nil, Value::Nil, Value::Nil, Value::Nil, Value::Nil, Value::Nil, 0i32, Value::Nil, false))
    })?)?;

    // C_Scenario namespace - scenario/dungeon scenario system
    let c_scenario = lua.create_table()?;
    c_scenario.set(
        "GetInfo",
        lua.create_function(|_, ()| {
            // Return: name, currentStage, numStages, flags, hasBonusStep, isBonusStepComplete, ...
            Ok((Value::Nil, 0i32, 0i32, 0i32, false, false))
        })?,
    )?;
    c_scenario.set(
        "GetStepInfo",
        lua.create_function(|_, _step: Option<i32>| {
            // Return: title, description, numCriteria, stepFailed, isBonusStep, ...
            Ok((Value::Nil, Value::Nil, 0i32, false, false))
        })?,
    )?;
    c_scenario.set(
        "GetCriteriaInfo",
        lua.create_function(|_, _criteria_index: i32| {
            Ok(Value::Nil)
        })?,
    )?;
    c_scenario.set(
        "IsInScenario",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_scenario.set(
        "ShouldShowCriteria",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_scenario.set(
        "GetBonusSteps",
        lua.create_function(|lua, ()| Ok(lua.create_table()?))?,
    )?;
    c_scenario.set(
        "GetProvingGroundsInfo",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    globals.set("C_Scenario", c_scenario)?;

    // Legacy addon functions
    globals.set(
        "GetAddOnMetadata",
        lua.create_function(|lua, (addon, field): (String, String)| {
            let value = match field.as_str() {
                "Version" => "@project-version@",
                "X-Flavor" => "Mainline",
                "Title" => addon.as_str(),
                "Group" => addon.as_str(), // Default group is addon name
                _ => "",
            };
            if value.is_empty() {
                Ok(Value::Nil)
            } else {
                Ok(Value::String(lua.create_string(value)?))
            }
        })?,
    )?;
    // Legacy global addon functions - delegate to state
    let state_for_legacy_num = Rc::clone(&state);
    globals.set(
        "GetNumAddOns",
        lua.create_function(move |_, ()| {
            let state = state_for_legacy_num.borrow();
            Ok(state.addons.len() as i32)
        })?,
    )?;
    let state_for_legacy_loaded = Rc::clone(&state);
    globals.set(
        "IsAddOnLoaded",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_legacy_loaded.borrow();
            let found = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state.addons.get(idx).map(|a| a.loaded).unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state.addons.iter().any(|a| a.folder_name == &*name && a.loaded)
                }
                _ => false,
            };
            Ok(found)
        })?,
    )?;
    globals.set(
        "LoadAddOn",
        lua.create_function(|_, _addon: String| Ok((true, Value::Nil)))?,
    )?;
    let state_for_legacy_lod = Rc::clone(&state);
    globals.set(
        "IsAddOnLoadOnDemand",
        lua.create_function(move |_, addon: Value| {
            let state = state_for_legacy_lod.borrow();
            let lod = match addon {
                Value::Integer(idx) => {
                    let idx = (idx - 1) as usize;
                    state.addons.get(idx).map(|a| a.load_on_demand).unwrap_or(false)
                }
                Value::String(ref s) => {
                    let name = s.to_string_lossy();
                    state.addons.iter().find(|a| a.folder_name == &*name)
                        .map(|a| a.load_on_demand).unwrap_or(false)
                }
                _ => false,
            };
            Ok(lod)
        })?,
    )?;
    globals.set(
        "GetAddOnOptionalDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;
    globals.set(
        "GetAddOnDependencies",
        lua.create_function(|_, _addon: String| Ok(mlua::MultiValue::new()))?,
    )?;

    // CreateColor(r, g, b, a) - creates a color object with proper methods
    lua.load(r#"
        function CreateColor(r, g, b, a)
            local color = { r = r or 0, g = g or 0, b = b or 0, a = a or 1 }

            function color:GetRGB()
                return self.r, self.g, self.b
            end

            function color:GetRGBA()
                return self.r, self.g, self.b, self.a
            end

            function color:GetRGBAsBytes()
                return math.floor(self.r * 255), math.floor(self.g * 255), math.floor(self.b * 255)
            end

            function color:SetRGB(r, g, b)
                self.r = r or 0
                self.g = g or 0
                self.b = b or 0
            end

            function color:SetRGBA(r, g, b, a)
                self.r = r or 0
                self.g = g or 0
                self.b = b or 0
                self.a = a or 1
            end

            function color:SetColorRGB(r, g, b)
                self.r = r or 0
                self.g = g or 0
                self.b = b or 0
            end

            function color:GenerateHexColor()
                return string.format("%02x%02x%02x", math.floor(self.r * 255), math.floor(self.g * 255), math.floor(self.b * 255))
            end

            function color:GenerateHexColorMarkup()
                return "|c" .. string.format("ff%02x%02x%02x", math.floor(self.r * 255), math.floor(self.g * 255), math.floor(self.b * 255))
            end

            function color:WrapTextInColorCode(text)
                return self:GenerateHexColorMarkup() .. text .. "|r"
            end

            function color:IsEqualTo(otherColor)
                return self.r == otherColor.r and self.g == otherColor.g and self.b == otherColor.b and self.a == otherColor.a
            end

            return color
        end

        -- CreateVector2D(x, y) - creates a 2D vector object
        function CreateVector2D(x, y)
            local vec = { x = x or 0, y = y or 0 }

            function vec:GetXY()
                return self.x, self.y
            end

            function vec:SetXY(x, y)
                self.x = x or 0
                self.y = y or 0
            end

            function vec:IsEqualTo(other)
                return self.x == other.x and self.y == other.y
            end

            function vec:Add(other)
                return CreateVector2D(self.x + other.x, self.y + other.y)
            end

            function vec:Subtract(other)
                return CreateVector2D(self.x - other.x, self.y - other.y)
            end

            function vec:ScaleBy(scalar)
                self.x = self.x * scalar
                self.y = self.y * scalar
            end

            function vec:GetLength()
                return math.sqrt(self.x * self.x + self.y * self.y)
            end

            function vec:Normalize()
                local len = self:GetLength()
                if len > 0 then
                    self.x = self.x / len
                    self.y = self.y / len
                end
            end

            return vec
        end
    "#).exec()?;

    // WrapTextInColorCode(text, colorStr) - wrap text in color escape codes
    globals.set(
        "WrapTextInColorCode",
        lua.create_function(|lua, (text, color_str): (String, String)| {
            let wrapped = format!("|c{}{}|r", color_str, text);
            Ok(Value::String(lua.create_string(&wrapped)?))
        })?,
    )?;

    // Faction color globals (now that CreateColor exists)
    lua.load(r#"
        PLAYER_FACTION_COLOR_HORDE = CreateColor(1.0, 0.1, 0.1)
        PLAYER_FACTION_COLOR_ALLIANCE = CreateColor(0.1, 0.1, 1.0)
        FACTION_HORDE = "Horde"
        FACTION_ALLIANCE = "Alliance"
    "#).exec()?;

    // Error message strings used by addons
    globals.set("ERR_CHAT_PLAYER_NOT_FOUND_S", "%s is not online")?;
    globals.set("ERR_NOT_IN_COMBAT", "You can't do that while in combat")?;
    globals.set("ERR_GENERIC_NO_TARGET", "You have no target")?;
    globals.set("ERR_FRIEND_OFFLINE_S", "%s is offline.")?;
    globals.set("ERR_FRIEND_ONLINE_SS", "|Hplayer:%s|h[%s]|h has come online.")?;
    globals.set("ERR_FRIEND_NOT_FOUND", "That player is not on your friends list.")?;
    globals.set("ERR_FRIEND_ADDED_S", "%s added to friends.")?;
    globals.set("ERR_FRIEND_REMOVED_S", "%s removed from friends.")?;
    globals.set("ERR_IGNORE_ADDED_S", "%s added to ignore list.")?;
    globals.set("ERR_IGNORE_REMOVED_S", "%s removed from ignore list.")?;

    // Game constants
    globals.set("NUM_PET_ACTION_SLOTS", 10)?;
    globals.set("NUM_ACTIONBAR_BUTTONS", 12)?;
    globals.set("NUM_BAG_SLOTS", 5)?;
    globals.set("BAGSLOTTEXT", "Bag Slot")?;
    globals.set("MAX_SKILLLINE_TABS", 8)?;
    globals.set("MAX_PLAYER_LEVEL", 80)?;
    globals.set("MAX_NUM_TALENTS", 20)?;
    globals.set("MAX_BOSS_FRAMES", 8)?;
    globals.set("MAX_PARTY_MEMBERS", 4)?;
    globals.set("MAX_RAID_MEMBERS", 40)?;
    globals.set("BOOKTYPE_SPELL", "spell")?;
    globals.set("BOOKTYPE_PET", "pet")?;

    // Expansion level constants (LE_EXPANSION_*)
    globals.set("LE_EXPANSION_CLASSIC", 0)?;
    globals.set("LE_EXPANSION_BURNING_CRUSADE", 1)?;
    globals.set("LE_EXPANSION_WRATH_OF_THE_LICH_KING", 2)?;
    globals.set("LE_EXPANSION_CATACLYSM", 3)?;
    globals.set("LE_EXPANSION_MISTS_OF_PANDARIA", 4)?;
    globals.set("LE_EXPANSION_WARLORDS_OF_DRAENOR", 5)?;
    globals.set("LE_EXPANSION_LEGION", 6)?;
    globals.set("LE_EXPANSION_BATTLE_FOR_AZEROTH", 7)?;
    globals.set("LE_EXPANSION_SHADOWLANDS", 8)?;
    globals.set("LE_EXPANSION_DRAGONFLIGHT", 9)?;
    globals.set("LE_EXPANSION_WAR_WITHIN", 10)?;
    globals.set("LE_EXPANSION_LEVEL_CURRENT", 10)?; // Current expansion (The War Within)

    // Inventory slot constants
    globals.set("INVSLOT_AMMO", 0)?;
    globals.set("INVSLOT_HEAD", 1)?;
    globals.set("INVSLOT_NECK", 2)?;
    globals.set("INVSLOT_SHOULDER", 3)?;
    globals.set("INVSLOT_BODY", 4)?;  // Shirt
    globals.set("INVSLOT_CHEST", 5)?;
    globals.set("INVSLOT_WAIST", 6)?;
    globals.set("INVSLOT_LEGS", 7)?;
    globals.set("INVSLOT_FEET", 8)?;
    globals.set("INVSLOT_WRIST", 9)?;
    globals.set("INVSLOT_HAND", 10)?;
    globals.set("INVSLOT_FINGER1", 11)?;
    globals.set("INVSLOT_FINGER2", 12)?;
    globals.set("INVSLOT_TRINKET1", 13)?;
    globals.set("INVSLOT_TRINKET2", 14)?;
    globals.set("INVSLOT_BACK", 15)?;
    globals.set("INVSLOT_MAINHAND", 16)?;
    globals.set("INVSLOT_OFFHAND", 17)?;
    globals.set("INVSLOT_RANGED", 18)?;
    globals.set("INVSLOT_TABARD", 19)?;
    globals.set("INVSLOT_FIRST_EQUIPPED", 1)?;
    globals.set("INVSLOT_LAST_EQUIPPED", 19)?;

    // Raid target marker names
    globals.set("RAID_TARGET_1", "Star")?;
    globals.set("RAID_TARGET_2", "Circle")?;
    globals.set("RAID_TARGET_3", "Diamond")?;
    globals.set("RAID_TARGET_4", "Triangle")?;
    globals.set("RAID_TARGET_5", "Moon")?;
    globals.set("RAID_TARGET_6", "Square")?;
    globals.set("RAID_TARGET_7", "Cross")?;
    globals.set("RAID_TARGET_8", "Skull")?;

    // Taxi/flight path constants
    globals.set("TAXIROUTE_LINEFACTOR", 128.0 / 126.0)?;
    globals.set("TAXIROUTE_LINEFACTOR_2", 1.0)?;

    // Keyboard modifier text
    globals.set("SHIFT_KEY_TEXT", "Shift")?;
    globals.set("ALT_KEY_TEXT", "Alt")?;
    globals.set("CTRL_KEY_TEXT", "Ctrl")?;

    // Keybinding functions
    globals.set(
        "GetBindingKey",
        lua.create_function(|_, _action: String| {
            // Returns the key(s) bound to an action, nil if none
            Ok(Value::Nil)
        })?,
    )?;
    globals.set(
        "GetBinding",
        lua.create_function(|_lua, index: i32| {
            // Returns: action, key1, key2 for binding at index
            // Return nil if no binding at index
            if index < 1 {
                return Ok(mlua::MultiValue::new());
            }
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Nil,
                Value::Nil,
                Value::Nil,
            ]))
        })?,
    )?;
    globals.set(
        "GetNumBindings",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    globals.set(
        "SetBinding",
        lua.create_function(|_, (_key, _action): (String, Option<String>)| {
            // Set a key binding (no-op in simulation)
            Ok(true)
        })?,
    )?;
    globals.set(
        "SetBindingClick",
        lua.create_function(|_, (_key, _button, _mouse_button): (String, String, Option<String>)| {
            Ok(true)
        })?,
    )?;
    globals.set(
        "SetBindingSpell",
        lua.create_function(|_, (_key, _spell): (String, String)| {
            Ok(true)
        })?,
    )?;
    globals.set(
        "SetBindingItem",
        lua.create_function(|_, (_key, _item): (String, String)| {
            Ok(true)
        })?,
    )?;
    globals.set(
        "SetBindingMacro",
        lua.create_function(|_, (_key, _macro): (String, String)| {
            Ok(true)
        })?,
    )?;
    globals.set(
        "GetCurrentBindingSet",
        lua.create_function(|_, ()| {
            // Returns 1 for character-specific, 2 for account
            Ok(1)
        })?,
    )?;
    globals.set(
        "SaveBindings",
        lua.create_function(|_, _which: i32| Ok(()))?,
    )?;
    globals.set(
        "LoadBindings",
        lua.create_function(|_, _which: i32| Ok(()))?,
    )?;
    globals.set(
        "GetBindingAction",
        lua.create_function(|_, (_key, _check_override): (String, Option<bool>)| {
            // Returns the action bound to a key
            Ok(Value::Nil)
        })?,
    )?;
    globals.set(
        "GetBindingText",
        lua.create_function(|lua, (key, _prefix, _abbrev): (String, Option<String>, Option<bool>)| {
            // Returns display text for a key binding
            Ok(Value::String(lua.create_string(&key)?))
        })?,
    )?;

    // GetCurrentKeyBoardFocus() - Returns the frame that currently has keyboard focus
    {
        let state_ref = state.clone();
        globals.set(
            "GetCurrentKeyBoardFocus",
            lua.create_function(move |lua, ()| {
                if let Ok(s) = state_ref.try_borrow() {
                    if let Some(frame_id) = s.focused_frame_id {
                        // Return a FrameHandle userdata for the focused frame
                        let handle = FrameHandle {
                            id: frame_id,
                            state: state_ref.clone(),
                        };
                        return Ok(Value::UserData(lua.create_userdata(handle)?));
                    }
                }
                Ok(Value::Nil)
            })?,
        )?;
    }

    // UI strings
    globals.set("SPECIALIZATION", "Specialization")?;
    globals.set("TALENT", "Talent")?;
    globals.set("ITEMS", "Items")?;
    globals.set("SPELLS", "Spells")?;
    globals.set("MOUNTS", "Mounts")?;
    globals.set("TOYS", "Toys")?;
    globals.set("PETS", "Pets")?;
    globals.set("HEIRLOOMS", "Heirlooms")?;
    globals.set("APPEARANCES", "Appearances")?;
    globals.set("TRANSMOG", "Transmog")?;
    globals.set("WARDROBE", "Wardrobe")?;
    globals.set("COLLECTIONS", "Collections")?;
    globals.set("ACHIEVEMENTS", "Achievements")?;
    globals.set("DUNGEONS", "Dungeons")?;
    globals.set("RAIDS", "Raids")?;
    globals.set("SCENARIO", "Scenario")?;
    globals.set("PVP", "PvP")?;
    globals.set("ARENA", "Arena")?;
    globals.set("BATTLEGROUND", "Battleground")?;
    globals.set("VENDOR", "Vendor")?;
    globals.set("MERCHANT", "Merchant")?;
    globals.set("TRAINER", "Trainer")?;
    globals.set("AUCTION_HOUSE", "Auction House")?;
    globals.set("GUILD_BANK", "Guild Bank")?;
    globals.set("MAIL", "Mail")?;
    globals.set("BANK", "Bank")?;
    globals.set("LOOT", "Loot")?;
    globals.set("TRADE", "Trade")?;
    globals.set("QUESTS", "Quests")?;
    globals.set("REPUTATION", "Reputation")?;
    globals.set("CURRENCY", "Currency")?;
    globals.set("PROFESSIONS", "Professions")?;
    globals.set("RECIPES", "Recipes")?;
    globals.set("NONE", "None")?;
    globals.set("DEFAULT", "Default")?;
    globals.set("UNKNOWN", "Unknown")?;
    globals.set("RETRIEVING_ITEM_INFO", "Retrieving item information")?;
    globals.set("RETRIEVING_DATA", "Retrieving data...")?;
    globals.set("YES", "Yes")?;
    globals.set("NO", "No")?;
    globals.set("OKAY", "Okay")?;
    globals.set("CANCEL", "Cancel")?;
    globals.set("ACCEPT", "Accept")?;
    globals.set("DECLINE", "Decline")?;
    globals.set("ENABLE", "Enable")?;
    globals.set("DISABLE", "Disable")?;
    globals.set("ADDON_LIST", "Addons")?;
    globals.set("ENABLE_ALL_ADDONS", "Enable All")?;
    globals.set("DISABLE_ALL_ADDONS", "Disable All")?;
    globals.set("ADDON_LOADED", "Loaded")?;
    globals.set("ADDON_DEPENDENCIES", "Dependencies")?;
    globals.set("ADDON_DEP_DISABLED", "Dependency Disabled")?;
    globals.set("HIGHLIGHTING", "Highlighting:")?;
    globals.set("READY", "Ready")?;
    globals.set("NOT_READY", "Not Ready")?;
    globals.set("BUSY", "Busy")?;
    globals.set("AFK", "Away")?;
    globals.set("DND", "Do Not Disturb")?;
    globals.set("SELL_PRICE", "Sell Price")?;
    globals.set("BUY_PRICE", "Buy Price")?;
    globals.set("PVP_ITEM_LEVEL_TOOLTIP", "PvP Item Level %d")?;
    globals.set("ITEM_UNIQUE_MULTIPLE", "Unique (%d)")?;
    globals.set("ITEM_UNIQUE", "Unique")?;
    globals.set("ITEM_UNIQUE_EQUIPPABLE", "Unique-Equipped")?;
    globals.set("ITEM_ACCOUNTBOUND", "Warbound")?;
    globals.set("ITEM_ACCOUNTBOUND_UNTIL_EQUIP", "Warbound until equipped")?;
    globals.set("ITEM_BNETACCOUNTBOUND", "Battle.net Account Bound")?;
    globals.set("ITEM_SOULBOUND", "Soulbound")?;
    globals.set("ITEM_BIND_ON_EQUIP", "Binds when equipped")?;
    globals.set("ITEM_BIND_ON_PICKUP", "Binds when picked up")?;
    globals.set("ITEM_BIND_ON_USE", "Binds when used")?;
    // Socket strings (lowercase matches what WoW uses for enUS)
    globals.set("EMPTY_SOCKET_BLUE", "blue socket")?;
    globals.set("EMPTY_SOCKET_RED", "red socket")?;
    globals.set("EMPTY_SOCKET_YELLOW", "yellow socket")?;
    globals.set("EMPTY_SOCKET_META", "meta socket")?;
    globals.set("EMPTY_SOCKET_PRISMATIC", "prismatic socket")?;
    globals.set("EMPTY_SOCKET_NO_COLOR", "prismatic socket")?;
    globals.set("EMPTY_SOCKET_COGWHEEL", "cogwheel socket")?;
    globals.set("EMPTY_SOCKET_HYDRAULIC", "sha-touched")?;
    globals.set("EMPTY_SOCKET_CYPHER", "crystallic socket")?;
    globals.set("EMPTY_SOCKET_DOMINATION", "domination socket")?;
    globals.set("EMPTY_SOCKET_PRIMORDIAL", "primordial socket")?;
    globals.set("EMPTY_SOCKET_PUNCHCARDBLUE", "blue punchcard socket")?;
    globals.set("EMPTY_SOCKET_PUNCHCARDRED", "red punchcard socket")?;
    globals.set("EMPTY_SOCKET_PUNCHCARDYELLOW", "yellow punchcard socket")?;
    globals.set("EMPTY_SOCKET_TINKER", "tinker socket")?;
    globals.set("EMPTY_SOCKET_SINGINGSEA", "singing sea socket")?;
    globals.set("EMPTY_SOCKET_SINGINGTHUNDER", "singing thunder socket")?;
    globals.set("EMPTY_SOCKET_SINGINGWIND", "singing wind socket")?;
    globals.set("BINDING_HEADER_RAID_TARGET", "Raid Target")?;
    globals.set("BINDING_HEADER_ACTIONBAR", "Action Bar")?;
    globals.set("BINDING_HEADER_MULTIACTIONBAR", "Multi-Action Bar")?;
    globals.set("BINDING_HEADER_MOVEMENT", "Movement")?;
    globals.set("BINDING_HEADER_CHAT", "Chat")?;
    globals.set("BINDING_HEADER_TARGETING", "Targeting")?;
    globals.set("BINDING_HEADER_INTERFACE", "Interface")?;
    globals.set("BINDING_HEADER_MISC", "Miscellaneous")?;
    globals.set("NOT_BOUND", "Not bound")?;
    globals.set("KEY_BOUND", "Key bound")?;
    globals.set("SOURCE", "Source:")?;
    globals.set("APPEARANCE_LABEL", "Appearance")?;
    globals.set("COLOR", "Color")?;
    globals.set("COMPACT_UNIT_FRAME_PROFILE_SORTBY_ALPHABETICAL", "Alphabetical")?;
    globals.set("ITEM_QUALITY6_DESC", "Artifact")?;
    globals.set("ITEM_COOLDOWN_TIME", "%s Cooldown")?;
    globals.set("TOY", "Toy")?;
    globals.set("MOUNT", "Mount")?;
    globals.set("PET", "Pet")?;
    globals.set("EQUIPMENT", "Equipment")?;
    globals.set("REAGENT", "Reagent")?;
    globals.set("APPEARANCE", "Appearance")?;
    globals.set("TRANSMOG_SOURCE_LABEL", "Source:")?;
    globals.set("TRANSMOGRIFY", "Transmogrify")?;
    globals.set("WORLD_QUEST_REWARD_FILTERS_ANIMA", "Anima")?;
    globals.set("WORLD_QUEST_REWARD_FILTERS_EQUIPMENT", "Equipment")?;
    globals.set("WORLD_QUEST_REWARD_FILTERS_GOLD", "Gold")?;
    globals.set("WORLD_QUEST_REWARD_FILTERS_RESOURCES", "Resources")?;

    // Duel strings (used by DeathNote)
    globals.set("DUEL_WINNER_KNOCKOUT", "%1$s has defeated %2$s in a duel")?;
    globals.set("DUEL_WINNER_RETREAT", "%1$s has defeated %2$s in a duel (retreat)")?;

    // Loot message strings (used by Chattynator)
    globals.set("LOOT_ITEM_PUSHED_SELF", "You receive loot: %s.")?;
    globals.set("LOOT_ITEM_SELF", "You receive loot: %s.")?;
    globals.set("LOOT_ITEM_PUSHED_SELF_MULTIPLE", "You receive loot: %sx%d.")?;
    globals.set("LOOT_ITEM_SELF_MULTIPLE", "You receive loot: %sx%d.")?;
    globals.set("CHANGED_OWN_ITEM", "Changed %s to %s.")?;
    globals.set("LOOT_ITEM", "%s receives loot: %s.")?;
    globals.set("LOOT_ITEM_MULTIPLE", "%s receives loot: %sx%d.")?;

    // Currency strings
    globals.set("CURRENCY_GAINED", "You receive currency: %s.")?;
    globals.set("CURRENCY_GAINED_MULTIPLE", "You receive currency: %s x%d.")?;
    globals.set("YOU_LOOT_MONEY", "You loot %s")?;

    // XP gain strings
    globals.set("COMBATLOG_XPGAIN_EXHAUSTION1", "%s dies, you gain %d experience (+%d exp Rested bonus).")?;
    globals.set("COMBATLOG_XPGAIN_QUEST", "You gain %d experience (+%d exp bonus).")?;
    globals.set("COMBATLOG_XPGAIN_FIRSTPERSON", "%s dies, you gain %d experience.")?;
    globals.set("COMBATLOG_XPGAIN_FIRSTPERSON_UNNAMED", "You gain %d experience.")?;

    // Quest reward strings
    globals.set("ERR_QUEST_REWARD_EXP_I", "Experience gained: %d.")?;
    globals.set("ERR_QUEST_REWARD_MONEY_S", "Received: %s")?;

    // Chat format strings
    globals.set("CHAT_MONSTER_SAY_GET", "%s says: ")?;
    globals.set("CHAT_MONSTER_YELL_GET", "%s yells: ")?;
    globals.set("CHAT_MONSTER_WHISPER_GET", "%s whispers: ")?;
    globals.set("CHAT_SAY_GET", "%s says: ")?;
    globals.set("CHAT_WHISPER_GET", "%s whispers: ")?;
    globals.set("CHAT_WHISPER_INFORM_GET", "To %s: ")?;
    globals.set("CHAT_BN_WHISPER_GET", "%s whispers: ")?;
    globals.set("CHAT_BN_WHISPER_INFORM_GET", "To %s: ")?;

    // Achievement broadcast
    globals.set("ACHIEVEMENT_BROADCAST", "%s has earned the achievement %s!")?;

    // Who list format strings
    globals.set("WHO_LIST_FORMAT", "%s - Level %d %s %s")?;
    globals.set("WHO_LIST_GUILD_FORMAT", "%s - Level %d %s %s <%s>")?;

    // Guild news constants
    globals.set("NEWS_ITEM_LOOTED", 0)?;
    globals.set("NEWS_LEGENDARY_LOOTED", 1)?;
    globals.set("NEWS_GUILD_ACHIEVEMENT", 2)?;
    globals.set("NEWS_PLAYER_ACHIEVEMENT", 3)?;
    globals.set("NEWS_DUNGEON_ENCOUNTER", 4)?;
    globals.set("NEWS_GUILD_LEVEL", 5)?;
    globals.set("NEWS_GUILD_CREATE", 6)?;
    globals.set("NEWS_ITEM_CRAFTED", 7)?;
    globals.set("NEWS_ITEM_PURCHASED", 8)?;
    globals.set("NEWS_GUILD_MOTD", 9)?;

    // Duration format strings
    globals.set("SPELL_DURATION_SEC", "%.1f sec")?;
    globals.set("SPELL_DURATION_MIN", "%.1f min")?;
    globals.set("SECONDS_ABBR", "%d sec")?;
    globals.set("MINUTES_ABBR", "%d min")?;
    globals.set("HOURS_ABBR", "%d hr")?;

    // Combat text and healing strings
    globals.set("SHOW_COMBAT_HEALING", "Healing")?;
    globals.set("SHOW_COMBAT_HEALING_TEXT", "Show Healing")?;
    globals.set("SHOW_COMBAT_HEALING_ABSORB_SELF", "Self Absorbs")?;
    globals.set("SHOW_COMBAT_HEALING_ABSORB_TARGET", "Target Absorbs")?;
    globals.set("OPTION_TOOLTIP_SHOW_COMBAT_HEALING", "Show combat healing numbers")?;
    globals.set("OPTION_TOOLTIP_SHOW_COMBAT_HEALING_ABSORB_SELF", "Show self absorbs")?;
    globals.set("OPTION_TOOLTIP_SHOW_COMBAT_HEALING_ABSORB_TARGET", "Show target absorbs")?;

    // Edit Mode HUD strings
    globals.set("HUD_EDIT_MODE_CAST_BAR_LABEL", "Cast Bar")?;
    globals.set("HUD_EDIT_MODE_PLAYER_FRAME_LABEL", "Player Frame")?;
    globals.set("HUD_EDIT_MODE_TARGET_FRAME_LABEL", "Target Frame")?;
    globals.set("HUD_EDIT_MODE_FOCUS_FRAME_LABEL", "Focus Frame")?;
    globals.set("HUD_EDIT_MODE_MINIMAP_LABEL", "Minimap")?;
    globals.set("HUD_EDIT_MODE_ACTION_BAR_LABEL", "Action Bar %d")?;
    globals.set("HUD_EDIT_MODE_STANCE_BAR_LABEL", "Stance Bar")?;
    globals.set("HUD_EDIT_MODE_PET_ACTION_BAR_LABEL", "Pet Action Bar")?;
    globals.set("HUD_EDIT_MODE_POSSESS_ACTION_BAR_LABEL", "Possess Bar")?;
    globals.set("HUD_EDIT_MODE_CHAT_FRAME_LABEL", "Chat Frame")?;
    globals.set("HUD_EDIT_MODE_BUFFS_LABEL", "Buffs")?;
    globals.set("HUD_EDIT_MODE_DEBUFFS_LABEL", "Debuffs")?;
    globals.set("HUD_EDIT_MODE_OBJECTIVE_TRACKER_LABEL", "Objectives")?;
    globals.set("HUD_EDIT_MODE_BOSS_FRAMES_LABEL", "Boss Frames")?;
    globals.set("HUD_EDIT_MODE_ARENA_FRAMES_LABEL", "Arena Frames")?;
    globals.set("HUD_EDIT_MODE_PARTY_FRAMES_LABEL", "Party Frames")?;
    globals.set("HUD_EDIT_MODE_RAID_FRAMES_LABEL", "Raid Frames")?;
    globals.set("HUD_EDIT_MODE_VEHICLE_LEAVE_BUTTON_LABEL", "Vehicle Exit")?;
    globals.set("HUD_EDIT_MODE_ENCOUNTER_BAR_LABEL", "Encounter Bar")?;
    globals.set("HUD_EDIT_MODE_EXTRA_ACTION_BUTTON_LABEL", "Extra Action Button")?;
    globals.set("HUD_EDIT_MODE_ZONE_ABILITY_FRAME_LABEL", "Zone Ability")?;
    globals.set("HUD_EDIT_MODE_BAGS_LABEL", "Bags")?;
    globals.set("HUD_EDIT_MODE_MICRO_MENU_LABEL", "Micro Menu")?;
    globals.set("HUD_EDIT_MODE_TALKING_HEAD_FRAME_LABEL", "Talking Head")?;
    globals.set("HUD_EDIT_MODE_DURABILITY_FRAME_LABEL", "Durability")?;
    globals.set("HUD_EDIT_MODE_STATUS_TRACKING_BAR_LABEL", "Status Bars")?;
    globals.set("HUD_EDIT_MODE_EXPERIENCE_BAR_LABEL", "Experience Bar")?;
    globals.set("HUD_EDIT_MODE_HUD_TOOLTIP_LABEL", "HUD Tooltip")?;
    globals.set("HUD_EDIT_MODE_TIMER_BARS_LABEL", "Timer Bars")?;
    globals.set("BAG_NAME_BACKPACK", "Backpack")?;
    globals.set("LOSS_OF_CONTROL", "Loss of Control")?;
    globals.set("COOLDOWN_VIEWER_LABEL", "Cooldown Viewer")?;
    globals.set("BINDING_HEADER_HOUSING_SYSTEM", "Housing System")?;

    // Unit frame strings
    globals.set("FOCUS", "Focus")?;
    globals.set("TARGET", "Target")?;
    globals.set("PLAYER", "Player")?;
    globals.set("PET", "Pet")?;
    globals.set("PARTY", "Party")?;
    globals.set("RAID", "Raid")?;
    globals.set("BOSS", "Boss")?;
    globals.set("ARENA", "Arena")?;
    globals.set("SHOW_TARGET_OF_TARGET_TEXT", "Target of Target")?;
    globals.set("TARGET_OF_TARGET", "Target of Target")?;
    globals.set("FOCUS_FRAME_LABEL", "Focus Frame")?;
    globals.set("HEALTH", "Health")?;
    globals.set("MANA", "Mana")?;
    globals.set("RAGE", "Rage")?;
    globals.set("ENERGY", "Energy")?;
    globals.set("POWER_TYPE_FOCUS", "Focus")?;
    globals.set("RUNIC_POWER", "Runic Power")?;
    globals.set("SOUL_SHARDS", "Soul Shards")?;
    globals.set("SOUL_SHARDS_POWER", "Soul Shards")?;
    globals.set("HOLY_POWER", "Holy Power")?;
    globals.set("CHI", "Chi")?;
    globals.set("CHI_POWER", "Chi")?;
    globals.set("INSANITY", "Insanity")?;
    globals.set("MAELSTROM", "Maelstrom")?;
    globals.set("FURY", "Fury")?;
    globals.set("PAIN", "Pain")?;
    globals.set("LUNAR_POWER", "Astral Power")?;
    globals.set("COMBO_POINTS", "Combo Points")?;
    globals.set("COMBO_POINTS_POWER", "Combo Points")?;
    globals.set("ARCANE_CHARGES", "Arcane Charges")?;
    globals.set("POWER_TYPE_ARCANE_CHARGES", "Arcane Charges")?;
    globals.set("POWER_TYPE_ESSENCE", "Essence")?;
    globals.set("RUNES", "Runes")?;
    globals.set("CLEAR_ALL", "Clear All")?;
    globals.set("SHARE_QUEST_ABBREV", "Share")?;
    globals.set("BUFFOPTIONS_LABEL", "Buffs and Debuffs")?;
    globals.set("DEBUFFOPTIONS_LABEL", "Debuffs")?;
    globals.set("BUFFFRAME_LABEL", "Buff Frame")?;
    globals.set("DEBUFFFRAME_LABEL", "Debuff Frame")?;
    globals.set("UNIT_NAME_FRIENDLY_TOTEMS", "Friendly Totems")?;

    // Tooltip default color (used by LibUIDropDownMenu)
    let tooltip_default_color = lua.create_table()?;
    tooltip_default_color.set("r", 1.0)?;
    tooltip_default_color.set("g", 1.0)?;
    tooltip_default_color.set("b", 1.0)?;
    tooltip_default_color.set("a", 1.0)?;
    globals.set("TOOLTIP_DEFAULT_COLOR", tooltip_default_color)?;

    // Tooltip background color
    let tooltip_default_bg_color = lua.create_table()?;
    tooltip_default_bg_color.set("r", 0.0)?;
    tooltip_default_bg_color.set("g", 0.0)?;
    tooltip_default_bg_color.set("b", 0.0)?;
    tooltip_default_bg_color.set("a", 1.0)?;
    globals.set("TOOLTIP_DEFAULT_BACKGROUND_COLOR", tooltip_default_bg_color)?;

    // Item quality colors (indexed by quality 0-8)
    // 0=Poor, 1=Common, 2=Uncommon, 3=Rare, 4=Epic, 5=Legendary, 6=Artifact, 7=Heirloom, 8=WoW Token
    let item_quality_colors = lua.create_table()?;
    let quality_values: [(i32, f64, f64, f64, &str); 9] = [
        (0, 0.62, 0.62, 0.62, "ff9d9d9d"),  // Poor (gray)
        (1, 1.00, 1.00, 1.00, "ffffffff"),  // Common (white)
        (2, 0.12, 1.00, 0.00, "ff1eff00"),  // Uncommon (green)
        (3, 0.00, 0.44, 0.87, "ff0070dd"),  // Rare (blue)
        (4, 0.64, 0.21, 0.93, "ffa335ee"),  // Epic (purple)
        (5, 1.00, 0.50, 0.00, "ffff8000"),  // Legendary (orange)
        (6, 0.90, 0.80, 0.50, "ffe6cc80"),  // Artifact (light gold)
        (7, 0.00, 0.80, 1.00, "ff00ccff"),  // Heirloom (light blue)
        (8, 0.00, 0.80, 1.00, "ff00ccff"),  // WoW Token
    ];
    for (idx, r, g, b, hex) in quality_values {
        let color = lua.create_table()?;
        color.set("r", r)?;
        color.set("g", g)?;
        color.set("b", b)?;
        color.set("hex", hex)?;
        color.set("color", format!("|c{}|r", hex))?;
        item_quality_colors.set(idx, color)?;
    }
    globals.set("ITEM_QUALITY_COLORS", item_quality_colors)?;

    // Combat text strings
    globals.set("COMBAT_TEXT_SHOW_COMBO_POINTS_TEXT", "Combo Points")?;
    globals.set("COMBAT_TEXT_SHOW_FRIENDLY_NAMES_TEXT", "Friendly Names")?;
    globals.set("COMBAT_TEXT_SHOW_DODGE_PARRY_MISS_TEXT", "Dodge/Parry/Miss")?;
    globals.set("COMBAT_TEXT_SHOW_MANA_TEXT", "Show Mana")?;
    globals.set("COMBAT_TEXT_SHOW_HONOR_GAINED_TEXT", "Honor Gained")?;
    globals.set("COMBAT_TEXT_SHOW_REACTIVES_TEXT", "Reactives")?;
    globals.set("COMBAT_TEXT_SHOW_RESISTANCES_TEXT", "Resistances")?;
    globals.set("COMBAT_TEXT_SHOW_ENERGIZE_TEXT", "Energize")?;

    // Duel strings (used by DeathNote)
    globals.set("DUEL_WINNER_KNOCKOUT", "%1$s has defeated %2$s in a duel")?;
    globals.set("DUEL_WINNER_RETREAT", "%1$s has defeated %2$s in a duel (retreat)")?;

    // Combat log object raid target constants (used by DeathNote)
    globals.set("COMBATLOG_OBJECT_RAIDTARGET1", 0x00100000)?;
    globals.set("COMBATLOG_OBJECT_RAIDTARGET2", 0x00200000)?;
    globals.set("COMBATLOG_OBJECT_RAIDTARGET3", 0x00400000)?;
    globals.set("COMBATLOG_OBJECT_RAIDTARGET4", 0x00800000)?;
    globals.set("COMBATLOG_OBJECT_RAIDTARGET5", 0x01000000)?;
    globals.set("COMBATLOG_OBJECT_RAIDTARGET6", 0x02000000)?;
    globals.set("COMBATLOG_OBJECT_RAIDTARGET7", 0x04000000)?;
    globals.set("COMBATLOG_OBJECT_RAIDTARGET8", 0x08000000)?;

    // TEXT_MODE_A_STRING_* constants for combat text formatting
    globals.set("TEXT_MODE_A_STRING_RESULT_OVERKILLING", "(Overkill)")?;
    globals.set("TEXT_MODE_A_STRING_RESULT_RESIST", "(Resisted)")?;
    globals.set("TEXT_MODE_A_STRING_RESULT_BLOCK", "(Blocked)")?;
    globals.set("TEXT_MODE_A_STRING_RESULT_ABSORB", "(Absorbed)")?;
    globals.set("TEXT_MODE_A_STRING_RESULT_CRITICAL", "(Critical)")?;

    // Class name lookup tables
    let class_names_male = lua.create_table()?;
    let class_names_female = lua.create_table()?;
    for (key, name) in [
        ("WARRIOR", "Warrior"),
        ("PALADIN", "Paladin"),
        ("HUNTER", "Hunter"),
        ("ROGUE", "Rogue"),
        ("PRIEST", "Priest"),
        ("DEATHKNIGHT", "Death Knight"),
        ("SHAMAN", "Shaman"),
        ("MAGE", "Mage"),
        ("WARLOCK", "Warlock"),
        ("MONK", "Monk"),
        ("DRUID", "Druid"),
        ("DEMONHUNTER", "Demon Hunter"),
        ("EVOKER", "Evoker"),
    ] {
        class_names_male.set(key, name)?;
        class_names_female.set(key, name)?;
    }
    globals.set("LOCALIZED_CLASS_NAMES_MALE", class_names_male)?;
    globals.set("LOCALIZED_CLASS_NAMES_FEMALE", class_names_female)?;

    // Font path globals (used by SetFont and CreateFont)
    globals.set(
        "STANDARD_TEXT_FONT",
        "Fonts\\FRIZQT__.TTF",
    )?;
    globals.set(
        "UNIT_NAME_FONT",
        "Fonts\\FRIZQT__.TTF",
    )?;
    globals.set(
        "UNIT_NAME_FONT_CHINESE",
        "Fonts\\ARKai_T.TTF",
    )?;
    globals.set(
        "UNIT_NAME_FONT_CYRILLIC",
        "Fonts\\FRIZQT___CYR.TTF",
    )?;
    globals.set(
        "UNIT_NAME_FONT_KOREAN",
        "Fonts\\2002.TTF",
    )?;
    globals.set(
        "DAMAGE_TEXT_FONT",
        "Fonts\\FRIZQT__.TTF",
    )?;
    globals.set(
        "NAMEPLATE_FONT",
        "Fonts\\FRIZQT__.TTF",
    )?;

    // LFG/Group Finder strings
    globals.set("GROUP_FINDER", "Group Finder")?;
    globals.set("STAT_CATEGORY_PVP", "PvP")?;

    // Stat strings
    globals.set("STAT_ARMOR", "Armor")?;
    globals.set("STAT_STRENGTH", "Strength")?;
    globals.set("STAT_AGILITY", "Agility")?;
    globals.set("STAT_STAMINA", "Stamina")?;
    globals.set("STAT_INTELLECT", "Intellect")?;
    globals.set("STAT_SPIRIT", "Spirit")?;

    // ITEM_MOD strings - primary stats
    globals.set("ITEM_MOD_STRENGTH", "Strength")?;
    globals.set("ITEM_MOD_STRENGTH_SHORT", "Strength")?;
    globals.set("ITEM_MOD_AGILITY", "Agility")?;
    globals.set("ITEM_MOD_AGILITY_SHORT", "Agility")?;
    globals.set("ITEM_MOD_STAMINA", "Stamina")?;
    globals.set("ITEM_MOD_STAMINA_SHORT", "Stamina")?;
    globals.set("ITEM_MOD_INTELLECT", "Intellect")?;
    globals.set("ITEM_MOD_INTELLECT_SHORT", "Intellect")?;
    globals.set("ITEM_MOD_SPIRIT", "Spirit")?;
    globals.set("ITEM_MOD_SPIRIT_SHORT", "Spirit")?;

    // ITEM_MOD strings - secondary stats
    globals.set("ITEM_MOD_CRIT_RATING", "Critical Strike")?;
    globals.set("ITEM_MOD_CRIT_RATING_SHORT", "Critical Strike")?;
    globals.set("ITEM_MOD_HASTE_RATING", "Haste")?;
    globals.set("ITEM_MOD_HASTE_RATING_SHORT", "Haste")?;
    globals.set("ITEM_MOD_MASTERY_RATING", "Mastery")?;
    globals.set("ITEM_MOD_MASTERY_RATING_SHORT", "Mastery")?;
    globals.set("ITEM_MOD_VERSATILITY", "Versatility")?;

    // ITEM_MOD strings - tertiary and other stats
    globals.set("ITEM_MOD_CR_AVOIDANCE_SHORT", "Avoidance")?;
    globals.set("ITEM_MOD_CR_LIFESTEAL_SHORT", "Leech")?;
    globals.set("ITEM_MOD_CR_SPEED_SHORT", "Speed")?;
    globals.set("ITEM_MOD_CR_STURDINESS_SHORT", "Indestructible")?;
    globals.set("ITEM_MOD_ATTACK_POWER_SHORT", "Attack Power")?;
    globals.set("ITEM_MOD_SPELL_POWER_SHORT", "Spell Power")?;
    globals.set("ITEM_MOD_BLOCK_RATING_SHORT", "Block")?;
    globals.set("ITEM_MOD_DODGE_RATING_SHORT", "Dodge")?;
    globals.set("ITEM_MOD_PARRY_RATING_SHORT", "Parry")?;
    globals.set("ITEM_MOD_HIT_RATING_SHORT", "Hit")?;
    globals.set("ITEM_MOD_EXTRA_ARMOR_SHORT", "Bonus Armor")?;
    globals.set("ITEM_MOD_PVP_POWER_SHORT", "PvP Power")?;
    globals.set("ITEM_MOD_RESILIENCE_RATING_SHORT", "PvP Resilience")?;
    globals.set("ITEM_MOD_MANA_SHORT", "Mana")?;
    globals.set("ITEM_MOD_MANA_REGENERATION_SHORT", "Mana Regeneration")?;
    globals.set("ITEM_MOD_HEALTH_REGENERATION_SHORT", "Health Regeneration")?;
    globals.set("ITEM_MOD_DAMAGE_PER_SECOND_SHORT", "Damage Per Second")?;
    globals.set("ITEM_MOD_CRAFTING_SPEED_SHORT", "Crafting Speed")?;
    globals.set("ITEM_MOD_MULTICRAFT_SHORT", "Multicraft")?;
    globals.set("ITEM_MOD_RESOURCEFULNESS_SHORT", "Resourcefulness")?;
    globals.set("ITEM_MOD_PERCEPTION_SHORT", "Perception")?;
    globals.set("ITEM_MOD_DEFTNESS_SHORT", "Deftness")?;
    globals.set("ITEM_MOD_FINESSE_SHORT", "Finesse")?;
    globals.set("LFG_TYPE_ZONE", "Zone")?;
    globals.set("LFG_TYPE_DUNGEON", "Dungeon")?;
    globals.set("LFG_TYPE_RAID", "Raid")?;
    globals.set("LFG_TYPE_HEROIC_DUNGEON", "Heroic Dungeon")?;
    globals.set("DUNGEONS_BUTTON", "Dungeons")?;
    globals.set("RAIDS_BUTTON", "Raids")?;
    globals.set("SCENARIOS_BUTTON", "Scenarios")?;
    globals.set("PLAYER_V_PLAYER", "Player vs. Player")?;
    globals.set("LFG_LIST_LOADING", "Loading...")?;
    globals.set("LFG_LIST_SEARCH_PLACEHOLDER", "Enter search...")?;

    // Slash command globals (used by macro addons)
    globals.set("SLASH_CAST1", "/cast")?;
    globals.set("SLASH_CAST2", "/spell")?;
    globals.set("SLASH_CAST3", "/use")?;
    globals.set("SLASH_CAST4", "/castrandom")?;
    globals.set("SLASH_CASTSEQUENCE1", "/castsequence")?;
    globals.set("SLASH_CASTRANDOM1", "/castrandom")?;
    globals.set("SLASH_CLICK1", "/click")?;
    globals.set("SLASH_TARGET1", "/target")?;
    globals.set("SLASH_TARGET2", "/tar")?;
    globals.set("SLASH_FOCUS1", "/focus")?;
    globals.set("SLASH_ASSIST1", "/assist")?;

    // Binding name globals (used by !KalielsTracker, etc)
    globals.set("BINDING_NAME_EXTRAACTIONBUTTON1", "Extra Action Button")?;
    globals.set("BINDING_NAME_BONUSACTIONBUTTON1", "Bonus Action Button")?;
    globals.set("BINDING_NAME_ACTIONBUTTON1", "Action Button 1")?;
    globals.set("BINDING_NAME_ACTIONBUTTON2", "Action Button 2")?;
    globals.set("BINDING_NAME_ACTIONBUTTON3", "Action Button 3")?;
    globals.set("BINDING_NAME_ACTIONBUTTON4", "Action Button 4")?;
    globals.set("BINDING_NAME_ACTIONBUTTON5", "Action Button 5")?;
    globals.set("BINDING_NAME_ACTIONBUTTON6", "Action Button 6")?;
    globals.set("BINDING_NAME_ACTIONBUTTON7", "Action Button 7")?;
    globals.set("BINDING_NAME_ACTIONBUTTON8", "Action Button 8")?;
    globals.set("BINDING_NAME_ACTIONBUTTON9", "Action Button 9")?;
    globals.set("BINDING_NAME_ACTIONBUTTON10", "Action Button 10")?;
    globals.set("BINDING_NAME_ACTIONBUTTON11", "Action Button 11")?;
    globals.set("BINDING_NAME_ACTIONBUTTON12", "Action Button 12")?;
    globals.set("SLASH_FOLLOW1", "/follow")?;
    globals.set("SLASH_FOLLOW2", "/fol")?;
    globals.set("SLASH_PET_ATTACK1", "/petattack")?;
    globals.set("SLASH_PET_FOLLOW1", "/petfollow")?;
    globals.set("SLASH_PET_PASSIVE1", "/petpassive")?;
    globals.set("SLASH_PET_DEFENSIVE1", "/petdefensive")?;
    globals.set("SLASH_PET_AGGRESSIVE1", "/petaggressive")?;
    globals.set("SLASH_PET_STAY1", "/petstay")?;
    globals.set("SLASH_EQUIP1", "/equip")?;
    globals.set("SLASH_EQUIPSLOT1", "/equipslot")?;
    globals.set("SLASH_USETALENTS1", "/usetalents")?;
    globals.set("SLASH_STOPCASTING1", "/stopcasting")?;
    globals.set("SLASH_STOPATTACK1", "/stopattack")?;
    globals.set("SLASH_CANCELAURA1", "/cancelaura")?;
    globals.set("SLASH_CANCELFORM1", "/cancelform")?;
    globals.set("SLASH_DISMOUNT1", "/dismount")?;
    globals.set("SLASH_STARTATTACK1", "/startattack")?;

    // LFG error strings
    globals.set("ERR_LFG_PROPOSAL_FAILED", "The dungeon finder proposal failed.")?;
    globals.set("ERR_LFG_PROPOSAL_DECLINED", "A player declined the dungeon finder proposal.")?;
    globals.set("ERR_LFG_ROLE_CHECK_FAILED", "The role check failed.")?;
    globals.set("ERR_LFG_NO_SLOTS_PLAYER", "You are not in a valid slot.")?;
    globals.set("ERR_LFG_NO_SLOTS_PARTY", "Your party is not in a valid slot.")?;
    globals.set("ERR_LFG_MISMATCHED_SLOTS", "You do not meet the requirements for that dungeon.")?;
    globals.set("ERR_LFG_DESERTER_PLAYER", "You cannot queue because you have the Deserter debuff.")?;

    // Loot error strings
    globals.set("ERR_LOOT_GONE", "Item is no longer available (already looted)")?;
    globals.set("ERR_LOOT_NOTILE", "You are too far away to loot that corpse.")?;
    globals.set("ERR_LOOT_DIDNT_KILL", "You didn't kill that creature.")?;
    globals.set("ERR_LOOT_ROLL_PENDING", "You cannot loot while the roll is pending.")?;
    globals.set("ERR_LOOT_WHILE_INVULNERABLE", "You can't loot while invulnerable.")?;

    // Instance strings
    globals.set("INSTANCE_SAVED", "You are now saved to this instance.")?;
    globals.set("TRANSFER_ABORT_TOO_MANY_INSTANCES", "You have entered too many instances recently.")?;
    globals.set("NO_RAID_INSTANCES_SAVED", "You are not saved to any raid instances.")?;

    // Objective watch/tracker strings
    globals.set("OBJECTIVES_WATCH_TOO_MANY", "You are tracking too many quests.")?;
    globals.set("OBJECTIVES_TRACKER_LABEL", "Objectives")?;

    // Tracker/Quest log strings
    globals.set("TRACKER_HEADER_WORLD_QUESTS", "World Quests")?;
    globals.set("TRACKER_HEADER_BONUS_OBJECTIVES", "Bonus Objectives")?;
    globals.set("TRACKER_HEADER_SCENARIO", "Scenario")?;
    globals.set("TRACKER_HEADER_OBJECTIVE", "Objective")?;
    globals.set("TRACKER_HEADER_PROVINGGROUNDS", "Proving Grounds")?;
    globals.set("TRACKER_HEADER_DUNGEON", "Dungeon")?;
    globals.set("TRACKER_HEADER_DELVES", "Delves")?;
    globals.set("TRACKER_HEADER_CAMPAIGN_QUESTS", "Campaign")?;
    globals.set("TRACKER_HEADER_QUESTS", "Quests")?;

    // Character strings
    globals.set("CLASS", "Class")?;
    globals.set("RACE", "Race")?;
    globals.set("LEVEL", "Level")?;
    globals.set("GUILD", "Guild")?;
    globals.set("REALM", "Realm")?;
    globals.set("OFFLINE", "Offline")?;
    globals.set("ONLINE", "Online")?;

    // Tooltip strings (use %s to allow both numbers and "??" level strings)
    globals.set("TOOLTIP_UNIT_LEVEL", "Level %s")?;
    globals.set("TOOLTIP_UNIT_LEVEL_TYPE", "Level %s %s")?;
    globals.set("TOOLTIP_UNIT_LEVEL_CLASS", "Level %s %s")?;
    globals.set("TOOLTIP_UNIT_LEVEL_RACE_CLASS", "Level %s %s %s")?;
    globals.set("ELITE", "Elite")?;
    globals.set("RARE", "Rare")?;
    globals.set("RAREELITE", "Rare Elite")?;
    globals.set("WORLDBOSS", "Boss")?;

    // Item requirement strings
    globals.set("ITEM_REQ_SKILL", "Requires %s")?;
    globals.set("ITEM_REQ_REPUTATION", "Requires %s - %s")?;
    globals.set("ITEM_REQ_ALLIANCE", "Alliance")?;
    globals.set("ITEM_REQ_HORDE", "Horde")?;
    globals.set("ITEM_MIN_LEVEL", "Requires Level %d")?;
    globals.set("ITEM_LEVEL", "Item Level %d")?;
    globals.set("ITEM_CLASSES_ALLOWED", "Classes: %s")?;
    globals.set("ITEM_RACES_ALLOWED", "Races: %s")?;

    // Achievement/Collection strings
    globals.set("ACHIEVEMENTS", "Achievements")?;
    globals.set("ACHIEVEMENT_UNLOCKED", "Achievement Unlocked")?;
    globals.set("ACHIEVEMENT_POINTS", "Achievement Points")?;
    globals.set("HEIRLOOMS", "Heirlooms")?;
    globals.set("TOYS", "Toys")?;
    globals.set("MOUNTS", "Mounts")?;
    globals.set("PETS", "Pets")?;
    globals.set("TITLES", "Titles")?;
    globals.set("APPEARANCES", "Appearances")?;
    globals.set("TRANSMOG_SETS", "Transmog Sets")?;

    // Currency strings
    globals.set("CURRENCY_GAINED", "You receive currency: %s x%d.")?;
    globals.set("CURRENCY_GAINED_MULTIPLE", "You receive currency: %s x%d.")?;
    globals.set("CURRENCY_GAINED_MULTIPLE_BONUS", "You receive currency: %s x%d (Bonus Roll).")?;
    globals.set("CURRENCY_TOTAL", "Total: %s")?;

    // Spell error strings
    globals.set("SPELL_FAILED_CUSTOM_ERROR_1029", "Requires Skyriding")?;
    globals.set("SPELL_FAILED_NOT_READY", "Spell is not ready")?;
    globals.set("SPELL_FAILED_BAD_TARGETS", "Invalid target")?;
    globals.set("SPELL_FAILED_NO_VALID_TARGETS", "No valid targets")?;

    // Item upgrade strings
    globals.set("UPGRADE", "Upgrade")?;
    globals.set("UPGRADE_ITEM", "Upgrade Item")?;
    globals.set("UPGRADE_LEVEL", "Upgrade Level")?;

    // Spellbook/Encounter strings
    globals.set("SPELLBOOK_AVAILABLE_AT", "Available at level %d")?;
    globals.set("ENCOUNTER_JOURNAL", "Adventure Guide")?;
    globals.set("ENCOUNTER_JOURNAL_ENCOUNTER", "Encounter")?;
    globals.set("ENCOUNTER_JOURNAL_DUNGEON", "Dungeon")?;
    globals.set("ENCOUNTER_JOURNAL_RAID", "Raid")?;
    globals.set("BOSS", "Boss")?;
    globals.set("GARRISON_LOCATION_TOOLTIP", "Garrison")?;
    globals.set("GARRISON_SHIPYARD", "Shipyard")?;
    globals.set("GARRISON_MISSION_COMPLETE", "Mission Complete")?;
    globals.set("GARRISON_FOLLOWER", "Follower")?;
    globals.set("RAID_BOSSES", "Raid Bosses")?;
    globals.set("RAID_INSTANCES", "Raid Instances")?;
    globals.set("DUNGEON_BOSSES", "Dungeon Bosses")?;
    globals.set("DUNGEON_INSTANCES", "Dungeon Instances")?;
    globals.set("DUNGEONS", "Dungeons")?;
    globals.set("RAIDS", "Raids")?;
    globals.set("WORLD", "World")?;
    globals.set("ZONE", "Zone")?;
    globals.set("SPECIAL", "Special")?;
    globals.set("TUTORIAL_TITLE20", "Tutorial")?;
    globals.set("CALENDAR_FILTER_WEEKLY_HOLIDAYS", "Weekly Holidays")?;
    globals.set("CHALLENGE_MODE", "Challenge Mode")?;
    globals.set("PLAYER_DIFFICULTY_MYTHIC_PLUS", "Mythic+")?;
    globals.set("PLAYER_DIFFICULTY1", "Normal")?;
    globals.set("PLAYER_DIFFICULTY2", "Heroic")?;
    globals.set("PLAYER_DIFFICULTY3", "Mythic")?;
    globals.set("PLAYER_DIFFICULTY4", "LFR")?;
    globals.set("PLAYER_DIFFICULTY5", "Challenge")?;
    globals.set("PLAYER_DIFFICULTY6", "Timewalking")?;

    // Dungeon difficulty strings
    globals.set("DUNGEON_DIFFICULTY1", "Normal")?;
    globals.set("DUNGEON_DIFFICULTY2", "Heroic")?;
    globals.set("DUNGEON_DIFFICULTY_NORMAL", "Normal")?;
    globals.set("DUNGEON_DIFFICULTY_HEROIC", "Heroic")?;
    globals.set("DUNGEON_DIFFICULTY_MYTHIC", "Mythic")?;
    globals.set("RAID_DIFFICULTY1", "10 Player")?;
    globals.set("RAID_DIFFICULTY2", "25 Player")?;
    globals.set("RAID_DIFFICULTY3", "10 Player (Heroic)")?;
    globals.set("RAID_DIFFICULTY4", "25 Player (Heroic)")?;

    // Instance reset strings
    globals.set("INSTANCE_RESET_SUCCESS", "%s has been reset.")?;
    globals.set("INSTANCE_RESET_FAILED", "Cannot reset %s. There are players still inside the instance.")?;
    globals.set("INSTANCE_RESET_FAILED_OFFLINE", "Cannot reset %s. There are players offline in your party.")?;

    // Raid difficulty changed strings
    globals.set("ERR_RAID_DIFFICULTY_CHANGED_S", "Raid difficulty changed to %s.")?;
    globals.set("ERR_DUNGEON_DIFFICULTY_CHANGED_S", "Dungeon difficulty changed to %s.")?;

    // Raid marker icon list (texture prefixes)
    let icon_list = lua.create_table()?;
    icon_list.set(1, "|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_1:")?;  // Star
    icon_list.set(2, "|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_2:")?;  // Circle
    icon_list.set(3, "|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_3:")?;  // Diamond
    icon_list.set(4, "|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_4:")?;  // Triangle
    icon_list.set(5, "|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_5:")?;  // Moon
    icon_list.set(6, "|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_6:")?;  // Square
    icon_list.set(7, "|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_7:")?;  // Cross
    icon_list.set(8, "|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_8:")?;  // Skull
    globals.set("ICON_LIST", icon_list)?;

    // Font color codes (escape sequences for colored text)
    globals.set("NORMAL_FONT_COLOR_CODE", "|cffffd100")?;
    globals.set("HIGHLIGHT_FONT_COLOR_CODE", "|cffffffff")?;
    globals.set("RED_FONT_COLOR_CODE", "|cffff2020")?;
    globals.set("GREEN_FONT_COLOR_CODE", "|cff20ff20")?;
    globals.set("GRAY_FONT_COLOR_CODE", "|cff808080")?;
    globals.set("YELLOW_FONT_COLOR_CODE", "|cffffff00")?;
    globals.set("LIGHTYELLOW_FONT_COLOR_CODE", "|cffffff9a")?;
    globals.set("ORANGE_FONT_COLOR_CODE", "|cffff8040")?;
    globals.set("ACHIEVEMENT_COLOR_CODE", "|cffffff00")?;
    globals.set("BATTLENET_FONT_COLOR_CODE", "|cff82c5ff")?;
    globals.set("DISABLED_FONT_COLOR_CODE", "|cff808080")?;
    globals.set("FONT_COLOR_CODE_CLOSE", "|r")?;
    globals.set("LINK_FONT_COLOR_CODE", "|cff00ccff")?;

    // Item binding strings
    globals.set("BIND_TRADE_TIME_REMAINING", "You may trade this item with players that were also eligible to loot this item for the next %s.")?;
    globals.set("BIND_ON_PICKUP", "Binds when picked up")?;
    globals.set("BIND_ON_EQUIP", "Binds when equipped")?;
    globals.set("BIND_ON_USE", "Binds when used")?;
    globals.set("BIND_TO_ACCOUNT", "Binds to Blizzard account")?;
    globals.set("BIND_TO_BNETACCOUNT", "Binds to Battle.net account")?;

    // Time abbreviation strings (for time formatting)
    globals.set("DAY_ONELETTER_ABBR", "%dd")?;
    globals.set("HOUR_ONELETTER_ABBR", "%dh")?;
    globals.set("MINUTE_ONELETTER_ABBR", "%dm")?;
    globals.set("SECOND_ONELETTER_ABBR", "%ds")?;
    globals.set("DAYS_ABBR", "%d Days")?;
    globals.set("HOURS_ABBR", "%d Hours")?;
    globals.set("MINUTES_ABBR", "%d Min")?;
    globals.set("SECONDS_ABBR", "%d Sec")?;
    globals.set("DAYS", "Days")?;
    globals.set("HOURS", "Hours")?;
    globals.set("MINUTES", "Minutes")?;
    globals.set("SECONDS", "Seconds")?;

    // CreateColor function for ColorMixin objects
    let create_color = lua.create_function(|lua, (r, g, b, a): (f64, f64, f64, Option<f64>)| {
        let color = lua.create_table()?;
        color.set("r", r)?;
        color.set("g", g)?;
        color.set("b", b)?;
        color.set("a", a.unwrap_or(1.0))?;
        // WrapTextInColorCode method
        let r_byte = (r * 255.0) as u8;
        let g_byte = (g * 255.0) as u8;
        let b_byte = (b * 255.0) as u8;
        let color_code = format!("|cff{:02x}{:02x}{:02x}", r_byte, g_byte, b_byte);
        color.set(
            "WrapTextInColorCode",
            lua.create_function(move |lua, text: Value| {
                let text_str = match text {
                    Value::String(s) => s.to_str().map(|s| s.to_string()).unwrap_or_default(),
                    Value::Integer(i) => i.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Nil => "nil".to_string(),
                    Value::Boolean(b) => b.to_string(),
                    _ => lua.coerce_string(text)
                        .ok()
                        .flatten()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default(),
                };
                Ok(format!("{}{}|r", color_code, text_str))
            })?,
        )?;
        color.set(
            "GetRGB",
            lua.create_function(move |_, ()| Ok((r, g, b)))?,
        )?;
        color.set(
            "GetRGBA",
            lua.create_function(move |_, ()| Ok((r, g, b, a.unwrap_or(1.0))))?,
        )?;
        color.set(
            "GenerateHexColor",
            lua.create_function(move |_, ()| {
                Ok(format!("{:02x}{:02x}{:02x}", r_byte, g_byte, b_byte))
            })?,
        )?;
        Ok(color)
    })?;
    globals.set("CreateColor", create_color.clone())?;

    // Color objects with WrapTextInColorCode
    let red_color = create_color.call::<mlua::Table>((1.0, 0.125, 0.125, 1.0))?;
    globals.set("RED_FONT_COLOR", red_color)?;
    let green_color = create_color.call::<mlua::Table>((0.125, 1.0, 0.125, 1.0))?;
    globals.set("GREEN_FONT_COLOR", green_color)?;
    let yellow_color = create_color.call::<mlua::Table>((1.0, 1.0, 0.0, 1.0))?;
    globals.set("YELLOW_FONT_COLOR", yellow_color)?;
    let normal_color = create_color.call::<mlua::Table>((1.0, 0.82, 0.0, 1.0))?;
    globals.set("NORMAL_FONT_COLOR", normal_color)?;
    let highlight_color = create_color.call::<mlua::Table>((1.0, 1.0, 1.0, 1.0))?;
    globals.set("HIGHLIGHT_FONT_COLOR", highlight_color)?;
    let gray_color = create_color.call::<mlua::Table>((0.5, 0.5, 0.5, 1.0))?;
    globals.set("GRAY_FONT_COLOR", gray_color)?;
    let lightgray_color = create_color.call::<mlua::Table>((0.75, 0.75, 0.75, 1.0))?;
    globals.set("LIGHTGRAY_FONT_COLOR", lightgray_color)?;
    let white_color = create_color.call::<mlua::Table>((1.0, 1.0, 1.0, 1.0))?;
    globals.set("WHITE_FONT_COLOR", white_color)?;
    let dim_red = create_color.call::<mlua::Table>((0.8, 0.1, 0.1, 1.0))?;
    globals.set("DIM_RED_FONT_COLOR", dim_red)?;
    let dim_green = create_color.call::<mlua::Table>((0.1, 0.8, 0.1, 1.0))?;
    globals.set("DIM_GREEN_FONT_COLOR", dim_green)?;
    let disabled = create_color.call::<mlua::Table>((0.5, 0.5, 0.5, 1.0))?;
    globals.set("DISABLED_FONT_COLOR", disabled)?;
    let link_color = create_color.call::<mlua::Table>((0.0, 0.8, 1.0, 1.0))?;
    globals.set("LINK_FONT_COLOR", link_color)?;
    let blue_color = create_color.call::<mlua::Table>((0.0, 0.44, 0.87, 1.0))?;
    globals.set("BLUE_FONT_COLOR", blue_color)?;
    let epic_purple = create_color.call::<mlua::Table>((0.639, 0.208, 0.933, 1.0))?;
    globals.set("EPIC_PURPLE_COLOR", epic_purple)?;
    let mixed_text_color = create_color.call::<mlua::Table>((0.75, 0.75, 0.75, 1.0))?;
    globals.set("MIXED_TEXT_COLOR", mixed_text_color)?;
    let tooltip_bg_color = create_color.call::<mlua::Table>((0.0, 0.0, 0.0, 0.9))?;
    globals.set("TOOLTIP_DEFAULT_BACKGROUND_COLOR", tooltip_bg_color)?;
    let tooltip_border_color = create_color.call::<mlua::Table>((0.3, 0.3, 0.3, 1.0))?;
    globals.set("TOOLTIP_DEFAULT_COLOR", tooltip_border_color)?;
    // Objective tracker colors
    let objective_header_color = create_color.call::<mlua::Table>((0.75, 0.61, 0.0, 1.0))?;
    globals.set("OBJECTIVE_TRACKER_BLOCK_HEADER_COLOR", objective_header_color)?;
    let quest_objective_color = create_color.call::<mlua::Table>((0.8, 0.8, 0.8, 1.0))?;
    globals.set("QUEST_OBJECTIVE_FONT_COLOR", quest_objective_color)?;

    // Battle pet sources
    globals.set("BATTLE_PET_SOURCE_1", "Drop")?;
    globals.set("BATTLE_PET_SOURCE_2", "Quest")?;
    globals.set("BATTLE_PET_SOURCE_3", "Vendor")?;
    globals.set("BATTLE_PET_SOURCE_4", "Profession")?;
    globals.set("BATTLE_PET_SOURCE_5", "Pet Battle")?;
    globals.set("BATTLE_PET_SOURCE_6", "Achievement")?;
    globals.set("BATTLE_PET_SOURCE_7", "World Event")?;
    globals.set("BATTLE_PET_SOURCE_8", "Promotion")?;
    globals.set("BATTLE_PET_SOURCE_9", "Trading Card Game")?;
    globals.set("BATTLE_PET_SOURCE_10", "In-Game Shop")?;
    globals.set("BATTLE_PET_SOURCE_11", "Discovery")?;

    // Dungeon floor names
    globals.set("DUNGEON_FLOOR_NAXXRAMAS1", "The Construct Quarter")?;
    globals.set("DUNGEON_FLOOR_NAXXRAMAS2", "The Arachnid Quarter")?;
    globals.set("DUNGEON_FLOOR_NAXXRAMAS3", "The Military Quarter")?;
    globals.set("DUNGEON_FLOOR_NAXXRAMAS4", "The Plague Quarter")?;
    globals.set("DUNGEON_FLOOR_NAXXRAMAS6", "Frostwyrm Lair")?;
    globals.set("DUNGEON_FLOOR_BLACKROCKDEPTHS1", "Detention Block")?;
    globals.set("DUNGEON_FLOOR_BLACKROCKDEPTHS2", "Shadowforge City")?;
    globals.set("DUNGEON_FLOOR_UPPERBLACKROCKSPIRE1", "Dragonspire Hall")?;
    globals.set("DUNGEON_FLOOR_DIREMAUL1", "Gordok Commons")?;
    globals.set("DUNGEON_FLOOR_DIREMAUL2", "Capital Gardens")?;
    globals.set("DUNGEON_FLOOR_DIREMAUL5", "Warpwood Quarter")?;
    globals.set("DUNGEON_FLOOR_DESOLACE21", "Maraudon")?;
    globals.set("DUNGEON_FLOOR_DESOLACE22", "Maraudon")?;
    globals.set("DUNGEON_FLOOR_NIGHTMARERAID7", "Rift of Aln")?;
    globals.set("DUNGEON_FLOOR_NIGHTMARERAID8", "The Emerald Nightmare")?;
    globals.set("DUNGEON_FLOOR_NIGHTMARERAID9", "Core of the Nightmare")?;
    globals.set("BROKENSHORE_BUILDING_NETHERDISRUPTOR", "Nether Disruptor")?;

    // Inventory type strings
    globals.set("ARMOR", "Armor")?;
    globals.set("INVTYPE_CLOAK", "Back")?;
    globals.set("INVTYPE_CHEST", "Chest")?;
    globals.set("INVTYPE_FEET", "Feet")?;
    globals.set("INVTYPE_FINGER", "Finger")?;
    globals.set("INVTYPE_HAND", "Hands")?;
    globals.set("INVTYPE_HEAD", "Head")?;
    globals.set("INVTYPE_LEGS", "Legs")?;
    globals.set("INVTYPE_NECK", "Neck")?;
    globals.set("INVTYPE_SHOULDER", "Shoulder")?;
    globals.set("INVTYPE_TRINKET", "Trinket")?;
    globals.set("INVTYPE_WAIST", "Waist")?;
    globals.set("INVTYPE_WRIST", "Wrist")?;

    // Garrison/Mission strings
    globals.set("GARRISON_MISSIONS", "Missions")?;
    globals.set("CAPACITANCE_WORK_ORDERS", "Work Orders")?;
    globals.set("BROKENSHORE_BUILDING_MAGETOWER", "Mage Tower")?;
    globals.set("SPLASH_BATTLEFORAZEROTH_8_2_0_FEATURE2_TITLE", "Nazjatar")?;
    globals.set("ISLANDS_HEADER", "Island Expeditions")?;
    globals.set("WORLD_MAP_THREATS", "Threats")?;
    globals.set("COVENANT_MISSIONS_TITLE", "Adventures")?;
    globals.set("ANIMA_DIVERSION_ORIGIN_TOOLTIP", "Anima Conductor")?;
    globals.set("GARRISON_CURRENT_LEVEL", "Tier")?;
    globals.set("COVENANT_SANCTUM_FEATURE_KYRIAN", "Path of Ascension")?;
    globals.set("COVENANT_SANCTUM_FEATURE_NECROLORDS", "Abomination Factory")?;
    globals.set("COVENANT_SANCTUM_FEATURE_NIGHTFAE", "Queen's Conservatory")?;
    globals.set("COVENANT_SANCTUM_FEATURE_VENTHYR", "Ember Court")?;

    // Item quality description strings
    globals.set("ITEM_QUALITY0_DESC", "Poor")?;
    globals.set("ITEM_QUALITY1_DESC", "Common")?;
    globals.set("ITEM_QUALITY2_DESC", "Uncommon")?;
    globals.set("ITEM_QUALITY3_DESC", "Rare")?;
    globals.set("ITEM_QUALITY4_DESC", "Epic")?;
    globals.set("ITEM_QUALITY5_DESC", "Legendary")?;
    globals.set("ITEM_QUALITY6_DESC", "Artifact")?;
    globals.set("ITEM_QUALITY7_DESC", "Heirloom")?;
    globals.set("LOOT_JOURNAL_LEGENDARIES", "Legendaries")?;

    // PowerBarColor - power bar colors indexed by power type
    let power_bar_color = lua.create_table()?;
    // MANA (0)
    let mana_color = lua.create_table()?;
    mana_color.set("r", 0.0)?;
    mana_color.set("g", 0.0)?;
    mana_color.set("b", 1.0)?;
    power_bar_color.set("MANA", mana_color)?;
    // RAGE (1)
    let rage_color = lua.create_table()?;
    rage_color.set("r", 1.0)?;
    rage_color.set("g", 0.0)?;
    rage_color.set("b", 0.0)?;
    power_bar_color.set("RAGE", rage_color)?;
    // ENERGY (3)
    let energy_color = lua.create_table()?;
    energy_color.set("r", 1.0)?;
    energy_color.set("g", 1.0)?;
    energy_color.set("b", 0.0)?;
    power_bar_color.set("ENERGY", energy_color)?;
    // RUNIC_POWER (6)
    let runic_power_color = lua.create_table()?;
    runic_power_color.set("r", 0.0)?;
    runic_power_color.set("g", 0.82)?;
    runic_power_color.set("b", 1.0)?;
    power_bar_color.set("RUNIC_POWER", runic_power_color)?;
    // Add numeric keys as aliases
    power_bar_color.set(0, power_bar_color.get::<mlua::Table>("MANA")?)?;
    power_bar_color.set(1, power_bar_color.get::<mlua::Table>("RAGE")?)?;
    power_bar_color.set(3, power_bar_color.get::<mlua::Table>("ENERGY")?)?;
    power_bar_color.set(6, power_bar_color.get::<mlua::Table>("RUNIC_POWER")?)?;
    globals.set("PowerBarColor", power_bar_color)?;

    // OBJECTIVE_TRACKER_COLOR - colors for objective tracker UI
    let objective_tracker_color = lua.create_table()?;
    // Header color
    let header_color = lua.create_table()?;
    header_color.set("r", 1.0)?;
    header_color.set("g", 0.82)?;
    header_color.set("b", 0.0)?;
    objective_tracker_color.set("Header", header_color)?;
    // HeaderHighlight color
    let header_highlight = lua.create_table()?;
    header_highlight.set("r", 1.0)?;
    header_highlight.set("g", 1.0)?;
    header_highlight.set("b", 0.0)?;
    objective_tracker_color.set("HeaderHighlight", header_highlight)?;
    // Normal color
    let normal_color = lua.create_table()?;
    normal_color.set("r", 0.8)?;
    normal_color.set("g", 0.8)?;
    normal_color.set("b", 0.8)?;
    objective_tracker_color.set("Normal", normal_color)?;
    // NormalHighlight color
    let normal_highlight = lua.create_table()?;
    normal_highlight.set("r", 1.0)?;
    normal_highlight.set("g", 1.0)?;
    normal_highlight.set("b", 1.0)?;
    objective_tracker_color.set("NormalHighlight", normal_highlight)?;
    // Complete color (green)
    let complete_color = lua.create_table()?;
    complete_color.set("r", 0.0)?;
    complete_color.set("g", 1.0)?;
    complete_color.set("b", 0.0)?;
    objective_tracker_color.set("Complete", complete_color)?;
    // Failed color (red)
    let failed_color = lua.create_table()?;
    failed_color.set("r", 1.0)?;
    failed_color.set("g", 0.0)?;
    failed_color.set("b", 0.0)?;
    objective_tracker_color.set("Failed", failed_color)?;
    globals.set("OBJECTIVE_TRACKER_COLOR", objective_tracker_color)?;

    // Item spell/charge strings
    globals.set("ITEM_SPELL_KNOWN", "Already known")?;
    globals.set("ITEM_SPELL_CHARGES", "%d |4Charge:Charges;")?;
    globals.set("ITEM_SPELL_CHARGES_NONE", "No Charges")?;
    globals.set("ITEM_SPELL_CHARGES_P1", "%d |4Charge:Charges;")?;

    // Wardrobe/Transmog strings
    globals.set("WARDROBE_SETS", "Sets")?;
    globals.set("WARDROBE_TOOLTIP_APPEARANCE_KNOWN", "Collected")?;
    globals.set("WARDROBE_TOOLTIP_APPEARANCE_UNKNOWN", "Not Collected")?;

    // Transmog source strings
    globals.set("TRANSMOG_SOURCE_1", "Boss Drop")?;
    globals.set("TRANSMOG_SOURCE_2", "Quest")?;
    globals.set("TRANSMOG_SOURCE_3", "Vendor")?;
    globals.set("TRANSMOG_SOURCE_4", "World Drop")?;
    globals.set("TRANSMOG_SOURCE_5", "Achievement")?;
    globals.set("TRANSMOG_SOURCE_6", "Profession")?;

    // Unit name title strings (for pet/minion ownership display)
    globals.set("UNITNAME_TITLE_PET", "%s's Pet")?;
    globals.set("UNITNAME_TITLE_COMPANION", "%s's Companion")?;
    globals.set("UNITNAME_TITLE_GUARDIAN", "%s's Guardian")?;
    globals.set("UNITNAME_TITLE_MINION", "%s's Minion")?;
    globals.set("UNITNAME_TITLE_CHARM", "%s's Charmed")?;
    globals.set("UNITNAME_TITLE_CREATION", "%s's Creation")?;
    globals.set("UNITNAME_TITLE_SQUIRE", "%s's Squire")?;
    globals.set("UNITNAME_TITLE_NAME", "%s's %s")?;

    // Pet type strings
    globals.set("PET_TYPE_PET", "Pet")?;
    globals.set("PET_TYPE_DEMON", "Demon")?;
    globals.set("PET_TYPE_GHOUL", "Ghoul")?;
    globals.set("PET_TYPE_GUARDIAN", "Guardian")?;
    globals.set("PET_TYPE_TOTEM", "Totem")?;
    globals.set("PET_TYPE_TREANT", "Treant")?;

    // Battle pet quality strings
    globals.set("BATTLE_PET_BREED_QUALITY0", "Poor")?;
    globals.set("BATTLE_PET_BREED_QUALITY1", "Common")?;
    globals.set("BATTLE_PET_BREED_QUALITY2", "Uncommon")?;
    globals.set("BATTLE_PET_BREED_QUALITY3", "Rare")?;
    globals.set("BATTLE_PET_BREED_QUALITY4", "Epic")?;
    globals.set("BATTLE_PET_BREED_QUALITY5", "Legendary")?;

    // Expansion filter text
    globals.set("EXPANSION_FILTER_TEXT", "Expansion: %s")?;
    globals.set("EXPANSION_NAME0", "Classic")?;
    globals.set("EXPANSION_NAME1", "The Burning Crusade")?;
    globals.set("EXPANSION_NAME2", "Wrath of the Lich King")?;
    globals.set("EXPANSION_NAME3", "Cataclysm")?;
    globals.set("EXPANSION_NAME4", "Mists of Pandaria")?;
    globals.set("EXPANSION_NAME5", "Warlords of Draenor")?;
    globals.set("EXPANSION_NAME6", "Legion")?;
    globals.set("EXPANSION_NAME7", "Battle for Azeroth")?;
    globals.set("EXPANSION_NAME8", "Shadowlands")?;
    globals.set("EXPANSION_NAME9", "Dragonflight")?;
    globals.set("EXPANSION_NAME10", "The War Within")?;

    // Item upgrade tooltip format
    globals.set("ITEM_UPGRADE_TOOLTIP_FORMAT_STRING", "Upgradeable: %s")?;

    // Spell school strings (damage types)
    globals.set("STRING_SCHOOL_PHYSICAL", "Physical")?;
    globals.set("STRING_SCHOOL_HOLY", "Holy")?;
    globals.set("STRING_SCHOOL_FIRE", "Fire")?;
    globals.set("STRING_SCHOOL_NATURE", "Nature")?;
    globals.set("STRING_SCHOOL_FROST", "Frost")?;
    globals.set("STRING_SCHOOL_SHADOW", "Shadow")?;
    globals.set("STRING_SCHOOL_ARCANE", "Arcane")?;
    globals.set("STRING_SCHOOL_HOLYSTRIKE", "Holystrike")?;
    globals.set("STRING_SCHOOL_FLAMESTRIKE", "Flamestrike")?;
    globals.set("STRING_SCHOOL_HOLYFIRE", "Holyfire")?;
    globals.set("STRING_SCHOOL_STORMSTRIKE", "Stormstrike")?;
    globals.set("STRING_SCHOOL_HOLYSTORM", "Holystorm")?;
    globals.set("STRING_SCHOOL_FIRESTORM", "Firestorm")?;
    globals.set("STRING_SCHOOL_VOLCANIC", "Volcanic")?;
    globals.set("STRING_SCHOOL_FROSTSTRIKE", "Froststrike")?;
    globals.set("STRING_SCHOOL_HOLYFROST", "Holyfrost")?;
    globals.set("STRING_SCHOOL_FROSTFIRE", "Frostfire")?;
    globals.set("STRING_SCHOOL_FROSTSTORM", "Froststorm")?;
    globals.set("STRING_SCHOOL_SHADOWSTRIKE", "Shadowstrike")?;
    globals.set("STRING_SCHOOL_SHADOWLIGHT", "Shadowlight")?;
    globals.set("STRING_SCHOOL_SHADOWFLAME", "Shadowflame")?;
    globals.set("STRING_SCHOOL_SHADOWSTORM", "Shadowstorm")?;
    globals.set("STRING_SCHOOL_SHADOWFROST", "Shadowfrost")?;
    globals.set("STRING_SCHOOL_SHADOWHOLY", "Shadowholy")?;
    globals.set("STRING_SCHOOL_HOLYFIRE", "Holyfire")?;
    globals.set("STRING_SCHOOL_HOLYSTORM", "Holystorm")?;
    globals.set("STRING_SCHOOL_HOLYFROST", "Holyfrost")?;
    globals.set("STRING_SCHOOL_HOLYNATURE", "Holynature")?;
    globals.set("STRING_SCHOOL_SPELLSTRIKE", "Spellstrike")?;
    globals.set("STRING_SCHOOL_DIVINE", "Divine")?;
    globals.set("STRING_SCHOOL_SPELLFIRE", "Spellfire")?;
    globals.set("STRING_SCHOOL_SPELLSTORM", "Spellstorm")?;
    globals.set("STRING_SCHOOL_SPELLFROST", "Spellfrost")?;
    globals.set("STRING_SCHOOL_SPELLSHADOW", "Spellshadow")?;
    globals.set("STRING_SCHOOL_ELEMENTAL", "Elemental")?;
    globals.set("STRING_SCHOOL_CHROMATIC", "Chromatic")?;
    globals.set("STRING_SCHOOL_COSMIC", "Cosmic")?;
    globals.set("STRING_SCHOOL_CHAOS", "Chaos")?;
    globals.set("STRING_SCHOOL_MAGIC", "Magic")?;
    globals.set("STRING_SCHOOL_UNKNOWN", "Unknown")?;

    // Role strings
    globals.set("TANK", "Tank")?;
    globals.set("HEALER", "Healer")?;
    globals.set("DAMAGER", "Damage")?;
    globals.set("COMBATLOG_FILTER_MELEE", "Melee")?;
    globals.set("COMBATLOG_FILTER_RANGED", "Ranged")?;
    globals.set("RANGED_ABILITY", "Ranged")?;
    globals.set("MELEE", "Melee")?;

    // Auction House categories
    globals.set("AUCTION_CATEGORY_WEAPONS", "Weapons")?;
    globals.set("AUCTION_CATEGORY_ARMOR", "Armor")?;
    globals.set("AUCTION_CATEGORY_CONTAINERS", "Containers")?;
    globals.set("AUCTION_CATEGORY_CONSUMABLES", "Consumables")?;
    globals.set("AUCTION_CATEGORY_GEMS", "Gems")?;
    globals.set("AUCTION_CATEGORY_ITEM_ENHANCEMENT", "Item Enhancement")?;
    globals.set("AUCTION_CATEGORY_RECIPES", "Recipes")?;
    globals.set("AUCTION_CATEGORY_TRADE_GOODS", "Trade Goods")?;
    globals.set("AUCTION_CATEGORY_QUEST_ITEMS", "Quest Items")?;
    globals.set("AUCTION_CATEGORY_BATTLE_PETS", "Battle Pets")?;
    globals.set("AUCTION_CATEGORY_MISCELLANEOUS", "Miscellaneous")?;
    globals.set("AUCTION_CATEGORY_WOW_TOKEN", "WoW Token")?;
    globals.set("ARMOR", "Armor")?;

    // PvP/Battlegrounds strings
    globals.set("BATTLEGROUNDS", "Battlegrounds")?;
    globals.set("BATTLEGROUND", "Battleground")?;
    globals.set("BATTLEGROUND_INSTANCE", "BG Instance")?;
    globals.set("PVP_RATED_BATTLEGROUNDS", "Rated Battlegrounds")?;
    globals.set("ARENA", "Arena")?;
    globals.set("PVP", "PvP")?;
    globals.set("PVE", "PvE")?;
    globals.set("HONOR", "Honor")?;
    globals.set("PAPERDOLL_SIDEBAR_TITLES", "Titles")?;
    globals.set("BATTLEFIELD_LEVEL", "Level ")?;
    globals.set("PVP_PRESTIGE_RANK_UP_TITLE", "Prestige")?;
    globals.set("PVP_RATED_BATTLEGROUND", "Rated Battleground")?;
    globals.set("PVP_TAB_CONQUEST", "Conquest")?;
    globals.set("AZERITE_ESSENCE_RANK", "Rank %d")?;
    globals.set("COVENANT_SANCTUM_TIER", "Tier %d")?;

    // Month names
    globals.set("MONTH_JANUARY", "January")?;
    globals.set("MONTH_FEBRUARY", "February")?;
    globals.set("MONTH_MARCH", "March")?;
    globals.set("MONTH_APRIL", "April")?;
    globals.set("MONTH_MAY", "May")?;
    globals.set("MONTH_JUNE", "June")?;
    globals.set("MONTH_JULY", "July")?;
    globals.set("MONTH_AUGUST", "August")?;
    globals.set("MONTH_SEPTEMBER", "September")?;
    globals.set("MONTH_OCTOBER", "October")?;
    globals.set("MONTH_NOVEMBER", "November")?;
    globals.set("MONTH_DECEMBER", "December")?;

    // Black Market Auction House
    globals.set("BLACK_MARKET_AUCTION_HOUSE", "Black Market Auction House")?;

    // Calendar strings
    globals.set("CALENDAR_REPEAT_WEEKLY", "Weekly")?;
    globals.set("CALENDAR_REPEAT_MONTHLY", "Monthly")?;
    globals.set("CALENDAR_REPEAT_DAILY", "Daily")?;

    // Combat log functions
    globals.set(
        "CombatLogGetCurrentEventInfo",
        lua.create_function(|_, ()| {
            // Returns: timestamp, subevent, hideCaster, sourceGUID, sourceName, sourceFlags, sourceRaidFlags,
            // destGUID, destName, destFlags, destRaidFlags, ...
            // Return empty in simulation (no combat)
            Ok(mlua::MultiValue::new())
        })?,
    )?;
    globals.set(
        "CombatLogClearEntries",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    globals.set(
        "CombatLogSetCurrentEntry",
        lua.create_function(|_, (_entry, _is_previous): (Value, Option<bool>)| Ok(()))?,
    )?;
    globals.set(
        "CombatLogResetFilter",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    globals.set(
        "CombatLogAddFilter",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    globals.set(
        "LoggingCombat",
        lua.create_function(|_, _enable: Option<bool>| {
            // Returns current combat log state, or sets it
            Ok(false)
        })?,
    )?;
    globals.set(
        "LoggingChat",
        lua.create_function(|_, _enable: Option<bool>| Ok(false))?,
    )?;

    // Role icon markup strings
    globals.set("INLINE_TANK_ICON", "|A:groupfinder-icon-role-large-tank:16:16:0:0|a")?;
    globals.set("INLINE_HEALER_ICON", "|A:groupfinder-icon-role-large-healer:16:16:0:0|a")?;
    globals.set("INLINE_DAMAGER_ICON", "|A:groupfinder-icon-role-large-dps:16:16:0:0|a")?;

    // Interface options (legacy and modern APIs)
    globals.set(
        "InterfaceOptions_AddCategory",
        lua.create_function(|_, _frame: Value| {
            // No-op - we don't have interface options panel
            Ok(())
        })?,
    )?;
    globals.set(
        "InterfaceAddOnsList_Update",
        lua.create_function(|_, ()| Ok(()))?,
    )?;

    // Settings API (modern replacement for InterfaceOptions)
    let settings = lua.create_table()?;
    settings.set(
        "RegisterCanvasLayoutCategory",
        lua.create_function(|lua, (_frame, _name, _group): (Value, Option<String>, Option<String>)| {
            // Return a dummy category object
            let category = lua.create_table()?;
            category.set("ID", "CustomCategory")?;
            Ok(category)
        })?,
    )?;
    settings.set(
        "RegisterCanvasLayoutSubcategory",
        lua.create_function(|lua, (_parent, _frame, _name): (Value, Value, Option<String>)| {
            let category = lua.create_table()?;
            category.set("ID", "CustomSubcategory")?;
            Ok(category)
        })?,
    )?;
    settings.set(
        "RegisterAddOnCategory",
        lua.create_function(|_, _category: Value| Ok(()))?,
    )?;
    settings.set(
        "OpenToCategory",
        lua.create_function(|_, _category_id: String| Ok(()))?,
    )?;
    settings.set(
        "RegisterVerticalLayoutCategory",
        lua.create_function(|lua, _name: String| {
            let category = lua.create_table()?;
            category.set("ID", "VerticalCategory")?;
            Ok(category)
        })?,
    )?;
    settings.set(
        "RegisterVerticalLayoutSubcategory",
        lua.create_function(|lua, (_parent, _name): (Value, String)| {
            let category = lua.create_table()?;
            category.set("ID", "VerticalSubcategory")?;
            Ok(category)
        })?,
    )?;
    // GetCategory(categoryID) - returns category by ID
    settings.set(
        "GetCategory",
        lua.create_function(|lua, _category_id: String| {
            let category = lua.create_table()?;
            category.set("ID", _category_id.clone())?;
            category.set("name", _category_id)?;
            Ok(category)
        })?,
    )?;
    globals.set("Settings", settings)?;

    // Achievement category info (legacy API)
    globals.set(
        "GetCategoryInfo",
        lua.create_function(|_, category_id: i32| {
            // Returns: title, parentCategoryID, flags
            Ok((
                format!("Category {}", category_id), // title
                -1i32,                                // parentCategoryID (-1 = root)
                0i32,                                 // flags
            ))
        })?,
    )?;
    globals.set(
        "GetCategoryList",
        lua.create_function(|lua, ()| {
            // Return empty table (no categories in simulation)
            lua.create_table()
        })?,
    )?;
    globals.set(
        "GetCategoryNumAchievements",
        lua.create_function(|_, _category_id: i32| {
            // Returns: total, completed
            Ok((0i32, 0i32))
        })?,
    )?;
    globals.set(
        "GetAchievementInfo",
        lua.create_function(|_, achievement_id: i32| {
            // Returns: id, name, points, completed, month, day, year, description, flags, icon, rewardText, isGuild, wasEarnedByMe, earnedBy, isStatistic
            Ok((
                achievement_id,                          // id
                format!("Achievement {}", achievement_id), // name
                10i32,                                   // points
                false,                                   // completed
                1i32,                                    // month
                1i32,                                    // day
                2020i32,                                 // year
                "Achievement description",               // description
                0i32,                                    // flags
                132486i32,                               // icon (default icon ID)
                "",                                      // rewardText
                false,                                   // isGuild
                false,                                   // wasEarnedByMe
                "",                                      // earnedBy
                false,                                   // isStatistic
            ))
        })?,
    )?;

    // RAID_CLASS_COLORS - color table for each class
    lua.load(
        r##"
        RAID_CLASS_COLORS = {
            ["WARRIOR"] = { r = 0.78, g = 0.61, b = 0.43, colorStr = "ffc79c6e" },
            ["PALADIN"] = { r = 0.96, g = 0.55, b = 0.73, colorStr = "fff58cba" },
            ["HUNTER"] = { r = 0.67, g = 0.83, b = 0.45, colorStr = "ffabd473" },
            ["ROGUE"] = { r = 1.00, g = 0.96, b = 0.41, colorStr = "fffff569" },
            ["PRIEST"] = { r = 1.00, g = 1.00, b = 1.00, colorStr = "ffffffff" },
            ["DEATHKNIGHT"] = { r = 0.77, g = 0.12, b = 0.23, colorStr = "ffc41f3b" },
            ["SHAMAN"] = { r = 0.00, g = 0.44, b = 0.87, colorStr = "ff0070de" },
            ["MAGE"] = { r = 0.41, g = 0.80, b = 0.94, colorStr = "ff69ccf0" },
            ["WARLOCK"] = { r = 0.58, g = 0.51, b = 0.79, colorStr = "ff9482c9" },
            ["MONK"] = { r = 0.00, g = 1.00, b = 0.59, colorStr = "ff00ff96" },
            ["DRUID"] = { r = 1.00, g = 0.49, b = 0.04, colorStr = "ffff7d0a" },
            ["DEMONHUNTER"] = { r = 0.64, g = 0.19, b = 0.79, colorStr = "ffa330c9" },
            ["EVOKER"] = { r = 0.20, g = 0.58, b = 0.50, colorStr = "ff33937f" },
        }
        -- Add colorStr getter method to each color
        for class, color in pairs(RAID_CLASS_COLORS) do
            setmetatable(color, {
                __index = {
                    GenerateHexColor = function(self) return self.colorStr end,
                    GenerateHexColorMarkup = function(self) return "|c" .. self.colorStr end,
                    WrapTextInColorCode = function(self, text) return "|c" .. self.colorStr .. text .. "|r" end,
                }
            })
        end
    "##,
    )
    .exec()?;

    // QuestDifficultyColors - color table for quest difficulty levels
    lua.load(
        r##"
        QuestDifficultyColors = {
            ["impossible"] = { r = 1.00, g = 0.10, b = 0.10, font = "QuestDifficulty_Impossible", hex = "" },
            ["verydifficult"] = { r = 1.00, g = 0.50, b = 0.25, font = "QuestDifficulty_VeryDifficult", hex = "" },
            ["difficult"] = { r = 1.00, g = 1.00, b = 0.00, font = "QuestDifficulty_Difficult", hex = "" },
            ["standard"] = { r = 0.25, g = 0.75, b = 0.25, font = "QuestDifficulty_Standard", hex = "" },
            ["trivial"] = { r = 0.50, g = 0.50, b = 0.50, font = "QuestDifficulty_Trivial", hex = "" },
            ["header"] = { r = 0.70, g = 0.70, b = 0.70, font = "QuestDifficulty_Header", hex = "" },
        }
        -- Build hex codes
        for _, color in pairs(QuestDifficultyColors) do
            color.hex = string.format("%02x%02x%02x", color.r * 255, color.g * 255, color.b * 255)
        end

        -- QuestDifficultyHighlightColors - same as above but lighter (for highlighting)
        QuestDifficultyHighlightColors = {
            ["impossible"] = { r = 1.00, g = 0.40, b = 0.40, font = "QuestDifficulty_Impossible", hex = "" },
            ["verydifficult"] = { r = 1.00, g = 0.70, b = 0.45, font = "QuestDifficulty_VeryDifficult", hex = "" },
            ["difficult"] = { r = 1.00, g = 1.00, b = 0.40, font = "QuestDifficulty_Difficult", hex = "" },
            ["standard"] = { r = 0.45, g = 0.85, b = 0.45, font = "QuestDifficulty_Standard", hex = "" },
            ["trivial"] = { r = 0.70, g = 0.70, b = 0.70, font = "QuestDifficulty_Trivial", hex = "" },
            ["header"] = { r = 0.85, g = 0.85, b = 0.85, font = "QuestDifficulty_Header", hex = "" },
        }
        for _, color in pairs(QuestDifficultyHighlightColors) do
            color.hex = string.format("%02x%02x%02x", color.r * 255, color.g * 255, color.b * 255)
        end
    "##,
    )
    .exec()?;

    // C_SpecializationInfo namespace
    let c_spec_info = lua.create_table()?;
    c_spec_info.set(
        "GetSpellsDisplay",
        lua.create_function(|lua, _spec_id: i32| lua.create_table())?,
    )?;
    c_spec_info.set(
        "GetInspectSelectedSpecialization",
        lua.create_function(|_, _unit: Option<String>| Ok(0))?,
    )?;
    c_spec_info.set(
        "CanPlayerUseTalentSpecUI",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    c_spec_info.set(
        "IsInitialized",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    // GetSpecialization() - returns current spec index (1-4)
    c_spec_info.set(
        "GetSpecialization",
        lua.create_function(|_, ()| Ok(1i32))?,
    )?;
    // GetSpecializationInfo(specIndex) - returns specID, name, description, icon, role, primaryStat
    c_spec_info.set(
        "GetSpecializationInfo",
        lua.create_function(|lua, spec_index: i32| {
            // Return dummy spec info for spec index 1 (Warrior Arms as example)
            let spec_id = match spec_index {
                1 => 71,  // Arms Warrior
                2 => 72,  // Fury Warrior
                3 => 73,  // Protection Warrior
                _ => 71,
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(spec_id),
                Value::String(lua.create_string("Arms")?),
                Value::String(lua.create_string("A battle-hardened master of weapons.")?),
                Value::Integer(132355), // icon ID
                Value::String(lua.create_string("DAMAGER")?),
                Value::Integer(1), // primary stat (Strength)
            ]))
        })?,
    )?;
    // GetAllSelectedPvpTalentIDs() - returns array of selected PvP talent IDs
    c_spec_info.set(
        "GetAllSelectedPvpTalentIDs",
        lua.create_function(|lua, ()| {
            // Return empty array (no PvP talents selected)
            lua.create_table()
        })?,
    )?;
    // GetNumSpecializationsForClassID(classID, sex) - number of specs for class
    c_spec_info.set(
        "GetNumSpecializationsForClassID",
        lua.create_function(|_, (_class_id, _sex): (Option<i32>, Option<i32>)| {
            // Return 3 specs for most classes (or 0 if classID is nil)
            Ok(_class_id.map_or(0, |_| 3i32))
        })?,
    )?;
    globals.set("C_SpecializationInfo", c_spec_info)?;

    // C_ChallengeMode namespace - Mythic+ dungeons
    let c_challenge_mode = lua.create_table()?;
    c_challenge_mode.set(
        "GetMapUIInfo",
        lua.create_function(|lua, _map_id: i32| {
            // Return: name, id, timeLimit, texture, backgroundTexture
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string("Unknown Dungeon")?),
                Value::Integer(0),
                Value::Integer(0),
                Value::Nil,
                Value::Nil,
            ]))
        })?,
    )?;
    c_challenge_mode.set(
        "GetMapTable",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_challenge_mode.set(
        "GetActiveKeystoneInfo",
        lua.create_function(|_, ()| {
            // Return: activeKeystoneLevel, activeAffixIDs, wasActiveKeystoneCharged
            Ok((0, Value::Nil, false))
        })?,
    )?;
    c_challenge_mode.set(
        "GetAffixInfo",
        lua.create_function(|lua, _affix_id: i32| {
            // Return: name, description, filedataid
            Ok((
                Value::String(lua.create_string("Unknown Affix")?),
                Value::String(lua.create_string("")?),
                Value::Integer(0),
            ))
        })?,
    )?;
    c_challenge_mode.set(
        "IsChallengeModeActive",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_ChallengeMode", c_challenge_mode)?;

    // GetSpecializationInfoByID(specID) - Get spec info
    globals.set(
        "GetSpecializationInfoByID",
        lua.create_function(|lua, spec_id: i32| {
            // Return: specID, specName, description, icon, role, isRecommended, isAllowed
            // Stub - return some default values
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(spec_id as i64),
                Value::String(lua.create_string("Unknown Spec")?),
                Value::String(lua.create_string("")?),
                Value::Integer(0), // icon
                Value::String(lua.create_string("DAMAGER")?),
                Value::Boolean(false),
                Value::Boolean(true),
            ]))
        })?,
    )?;

    // GetSpecialization() - Get current player spec index
    globals.set(
        "GetSpecialization",
        lua.create_function(|_, ()| Ok(1))?,
    )?;

    // GetNumSpecializations() - Get number of specs for player class
    globals.set(
        "GetNumSpecializations",
        lua.create_function(|_, ()| Ok(3))?,
    )?;

    // GetSpecializationInfo(specIndex) - Get spec info by index
    globals.set(
        "GetSpecializationInfo",
        lua.create_function(|lua, _spec_index: i32| {
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(0), // specID
                Value::String(lua.create_string("Unknown")?),
                Value::String(lua.create_string("")?),
                Value::Integer(0), // icon
                Value::String(lua.create_string("DAMAGER")?),
                Value::Boolean(false),
                Value::Boolean(true),
            ]))
        })?,
    )?;

    // Create GameTooltip - a built-in tooltip frame with special methods
    // GameTooltip is used by almost all addons for displaying tooltips
    {
        // Create the GameTooltip frame in the widget registry
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some("GameTooltip".to_string()),
            ui_parent_id,
        );
        tooltip_frame.visible = false; // Hidden by default
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;

        // Create NineSlice child frame (required by SharedTooltipTemplates)
        let nine_slice = Frame::new(WidgetType::Frame, None, Some(tooltip_id));
        let nine_slice_id = nine_slice.id;
        tooltip_frame.children.push(nine_slice_id);
        tooltip_frame
            .children_keys
            .insert("NineSlice".to_string(), nine_slice_id);

        {
            let mut s = state.borrow_mut();
            s.widgets.register(nine_slice);
            s.widgets.register(tooltip_frame);
        }

        // Create the FrameHandle for GameTooltip
        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        // Store in globals
        globals.set("GameTooltip", tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create ItemRefTooltip - tooltip for item links clicked in chat
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some("ItemRefTooltip".to_string()),
            ui_parent_id,
        );
        tooltip_frame.visible = false;
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;
        state.borrow_mut().widgets.register(tooltip_frame);

        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        globals.set("ItemRefTooltip", tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create ItemRefShoppingTooltip1/2 - comparison tooltips for item links
    for i in 1..=2 {
        let tooltip_name = format!("ItemRefShoppingTooltip{}", i);
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some(tooltip_name.clone()),
            ui_parent_id,
        );
        tooltip_frame.visible = false;
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;
        state.borrow_mut().widgets.register(tooltip_frame);

        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        globals.set(tooltip_name.as_str(), tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create ShoppingTooltip1/2 - comparison tooltips for GameTooltip
    for i in 1..=2 {
        let tooltip_name = format!("ShoppingTooltip{}", i);
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some(tooltip_name.clone()),
            ui_parent_id,
        );
        tooltip_frame.visible = false;
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;
        state.borrow_mut().widgets.register(tooltip_frame);

        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        globals.set(tooltip_name.as_str(), tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create FriendsTooltip - tooltip for friends list
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some("FriendsTooltip".to_string()),
            ui_parent_id,
        );
        tooltip_frame.visible = false;
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;
        state.borrow_mut().widgets.register(tooltip_frame);

        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        globals.set("FriendsTooltip", tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create FriendsListFrame - friends list UI frame
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut friends_frame = Frame::new(
            WidgetType::Frame,
            Some("FriendsListFrame".to_string()),
            ui_parent_id,
        );
        friends_frame.visible = false;
        friends_frame.width = 300.0;
        friends_frame.height = 400.0;
        let friends_id = friends_frame.id;
        state.borrow_mut().widgets.register(friends_frame);

        // Create ScrollBox child (used by addons to iterate friend buttons)
        let scrollbox = Frame::new(WidgetType::Frame, None, Some(friends_id));
        let scrollbox_id = scrollbox.id;
        state.borrow_mut().widgets.register(scrollbox);
        state.borrow_mut().widgets.add_child(friends_id, scrollbox_id);
        if let Some(f) = state.borrow_mut().widgets.get_mut(friends_id) {
            f.children_keys.insert("ScrollBox".to_string(), scrollbox_id);
        }

        let handle = FrameHandle {
            id: friends_id,
            state: Rc::clone(&state),
        };
        let friends_ud = lua.create_userdata(handle)?;

        globals.set("FriendsListFrame", friends_ud.clone())?;
        let frame_key = format!("__frame_{}", friends_id);
        globals.set(frame_key.as_str(), friends_ud)?;
    }

    // Create QuestFrame - quest UI frame
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut quest_frame = Frame::new(
            WidgetType::Frame,
            Some("QuestFrame".to_string()),
            ui_parent_id,
        );
        quest_frame.visible = false;
        quest_frame.width = 350.0;
        quest_frame.height = 450.0;
        let quest_id = quest_frame.id;
        state.borrow_mut().widgets.register(quest_frame);

        let handle = FrameHandle {
            id: quest_id,
            state: Rc::clone(&state),
        };
        let quest_ud = lua.create_userdata(handle)?;

        globals.set("QuestFrame", quest_ud.clone())?;
        let frame_key = format!("__frame_{}", quest_id);
        globals.set(frame_key.as_str(), quest_ud)?;

        // Create QuestFrameRewardPanel (used by WorldQuestTracker)
        let mut reward_panel = Frame::new(
            WidgetType::Frame,
            Some("QuestFrameRewardPanel".to_string()),
            Some(quest_id),
        );
        reward_panel.visible = false;
        let reward_id = reward_panel.id;
        state.borrow_mut().widgets.register(reward_panel);
        state.borrow_mut().widgets.add_child(quest_id, reward_id);

        let reward_handle = FrameHandle {
            id: reward_id,
            state: Rc::clone(&state),
        };
        let reward_ud = lua.create_userdata(reward_handle)?;
        globals.set("QuestFrameRewardPanel", reward_ud.clone())?;
        globals.set(format!("__frame_{}", reward_id).as_str(), reward_ud)?;

        // Create QuestFrameCompleteQuestButton (used by WorldQuestTracker)
        let mut complete_btn = Frame::new(
            WidgetType::Button,
            Some("QuestFrameCompleteQuestButton".to_string()),
            Some(quest_id),
        );
        complete_btn.visible = false;
        let btn_id = complete_btn.id;
        state.borrow_mut().widgets.register(complete_btn);
        state.borrow_mut().widgets.add_child(quest_id, btn_id);

        let btn_handle = FrameHandle {
            id: btn_id,
            state: Rc::clone(&state),
        };
        let btn_ud = lua.create_userdata(btn_handle)?;
        globals.set("QuestFrameCompleteQuestButton", btn_ud.clone())?;
        globals.set(format!("__frame_{}", btn_id).as_str(), btn_ud)?;

        // Create QuestFrameDetailPanel (used by WorldQuestTracker)
        let mut detail_panel = Frame::new(
            WidgetType::Frame,
            Some("QuestFrameDetailPanel".to_string()),
            Some(quest_id),
        );
        detail_panel.visible = false;
        let detail_id = detail_panel.id;
        state.borrow_mut().widgets.register(detail_panel);
        state.borrow_mut().widgets.add_child(quest_id, detail_id);

        let detail_handle = FrameHandle {
            id: detail_id,
            state: Rc::clone(&state),
        };
        let detail_ud = lua.create_userdata(detail_handle)?;
        globals.set("QuestFrameDetailPanel", detail_ud.clone())?;
        globals.set(format!("__frame_{}", detail_id).as_str(), detail_ud)?;

        // Create QuestFrameProgressPanel (used by WorldQuestTracker)
        let mut progress_panel = Frame::new(
            WidgetType::Frame,
            Some("QuestFrameProgressPanel".to_string()),
            Some(quest_id),
        );
        progress_panel.visible = false;
        let progress_id = progress_panel.id;
        state.borrow_mut().widgets.register(progress_panel);
        state.borrow_mut().widgets.add_child(quest_id, progress_id);

        let progress_handle = FrameHandle {
            id: progress_id,
            state: Rc::clone(&state),
        };
        let progress_ud = lua.create_userdata(progress_handle)?;
        globals.set("QuestFrameProgressPanel", progress_ud.clone())?;
        globals.set(format!("__frame_{}", progress_id).as_str(), progress_ud)?;

        // Create QuestFrameAcceptButton (used by WorldQuestTracker)
        let mut accept_btn = Frame::new(
            WidgetType::Button,
            Some("QuestFrameAcceptButton".to_string()),
            Some(quest_id),
        );
        accept_btn.visible = false;
        let accept_id = accept_btn.id;
        state.borrow_mut().widgets.register(accept_btn);
        state.borrow_mut().widgets.add_child(quest_id, accept_id);

        let accept_handle = FrameHandle {
            id: accept_id,
            state: Rc::clone(&state),
        };
        let accept_ud = lua.create_userdata(accept_handle)?;
        globals.set("QuestFrameAcceptButton", accept_ud.clone())?;
        globals.set(format!("__frame_{}", accept_id).as_str(), accept_ud)?;

        // Create QuestFrameCompleteButton (used by WorldQuestTracker)
        let mut quest_complete_btn = Frame::new(
            WidgetType::Button,
            Some("QuestFrameCompleteButton".to_string()),
            Some(quest_id),
        );
        quest_complete_btn.visible = false;
        let quest_complete_id = quest_complete_btn.id;
        state.borrow_mut().widgets.register(quest_complete_btn);
        state.borrow_mut().widgets.add_child(quest_id, quest_complete_id);

        let quest_complete_handle = FrameHandle {
            id: quest_complete_id,
            state: Rc::clone(&state),
        };
        let quest_complete_ud = lua.create_userdata(quest_complete_handle)?;
        globals.set("QuestFrameCompleteButton", quest_complete_ud.clone())?;
        globals.set(format!("__frame_{}", quest_complete_id).as_str(), quest_complete_ud)?;
    }

    // Create MainMenuMicroButton - main menu micro button (used by EditModeExpanded)
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut micro_btn = Frame::new(
            WidgetType::Button,
            Some("MainMenuMicroButton".to_string()),
            ui_parent_id,
        );
        micro_btn.visible = true;
        micro_btn.width = 28.0;
        micro_btn.height = 36.0;
        let micro_id = micro_btn.id;
        state.borrow_mut().widgets.register(micro_btn);

        let handle = FrameHandle {
            id: micro_id,
            state: Rc::clone(&state),
        };
        let micro_ud = lua.create_userdata(handle)?;

        globals.set("MainMenuMicroButton", micro_ud.clone())?;
        let frame_key = format!("__frame_{}", micro_id);
        globals.set(frame_key.as_str(), micro_ud)?;
    }

    // Create other MicroButtons - UI micro menu buttons (used by EditModeExpanded and others)
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let micro_buttons = [
            "CharacterMicroButton",
            "ProfessionMicroButton",
            "PlayerSpellsMicroButton",
            "AchievementMicroButton",
            "QuestLogMicroButton",
            "GuildMicroButton",
            "LFDMicroButton",
            "CollectionsMicroButton",
            "EJMicroButton",
            "StoreMicroButton",
            "HelpMicroButton",
            "HousingMicroButton",
        ];

        for name in micro_buttons {
            let mut btn = Frame::new(
                WidgetType::Button,
                Some(name.to_string()),
                ui_parent_id,
            );
            btn.visible = true;
            btn.width = 28.0;
            btn.height = 36.0;
            let btn_id = btn.id;
            state.borrow_mut().widgets.register(btn);

            let handle = FrameHandle {
                id: btn_id,
                state: Rc::clone(&state),
            };
            let btn_ud = lua.create_userdata(handle)?;
            globals.set(name, btn_ud.clone())?;
            let frame_key = format!("__frame_{}", btn_id);
            globals.set(frame_key.as_str(), btn_ud)?;
        }
    }

    // Create MerchantFrame and MerchantItem1-12 - merchant UI frames
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut merchant_frame = Frame::new(
            WidgetType::Frame,
            Some("MerchantFrame".to_string()),
            ui_parent_id,
        );
        merchant_frame.visible = false;
        merchant_frame.width = 336.0;
        merchant_frame.height = 447.0;
        let merchant_id = merchant_frame.id;
        state.borrow_mut().widgets.register(merchant_frame);

        let handle = FrameHandle {
            id: merchant_id,
            state: Rc::clone(&state),
        };
        let merchant_ud = lua.create_userdata(handle)?;
        globals.set("MerchantFrame", merchant_ud.clone())?;
        let frame_key = format!("__frame_{}", merchant_id);
        globals.set(frame_key.as_str(), merchant_ud)?;

        // Create MerchantItem1-12 button frames
        for i in 1..=12 {
            let item_name = format!("MerchantItem{}", i);
            let mut item_frame = Frame::new(
                WidgetType::Button,
                Some(item_name.clone()),
                Some(merchant_id),
            );
            item_frame.visible = true;
            item_frame.width = 160.0;
            item_frame.height = 36.0;
            let item_id = item_frame.id;
            state.borrow_mut().widgets.register(item_frame);
            state.borrow_mut().widgets.add_child(merchant_id, item_id);

            let handle = FrameHandle {
                id: item_id,
                state: Rc::clone(&state),
            };
            let item_ud = lua.create_userdata(handle)?;
            globals.set(item_name.as_str(), item_ud.clone())?;
            let frame_key = format!("__frame_{}", item_id);
            globals.set(frame_key.as_str(), item_ud)?;
        }
    }

    // Create action bar frames - used by EditModeExpanded and other addons
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let action_bars = [
            "MainActionBar",
            "MultiBarBottomLeft",
            "MultiBarBottomRight",
            "MultiBarRight",
            "MultiBarLeft",
            "MultiBar5",
            "MultiBar6",
            "MultiBar7",
            "MultiBar8",
            "StanceBar",
            "PetActionBar",
            "PossessBar",
            "OverrideActionBar",
        ];

        for name in action_bars {
            let mut bar = Frame::new(
                WidgetType::Frame,
                Some(name.to_string()),
                ui_parent_id,
            );
            bar.visible = true;
            bar.width = 500.0;
            bar.height = 40.0;
            let bar_id = bar.id;
            state.borrow_mut().widgets.register(bar);

            let handle = FrameHandle {
                id: bar_id,
                state: Rc::clone(&state),
            };
            let bar_ud = lua.create_userdata(handle)?;
            globals.set(name, bar_ud.clone())?;
            let frame_key = format!("__frame_{}", bar_id);
            globals.set(frame_key.as_str(), bar_ud)?;
        }
    }

    // Create StatusTrackingBarManager - manages XP/rep/honor bars (used by DynamicCam)
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut bar_mgr = Frame::new(
            WidgetType::Frame,
            Some("StatusTrackingBarManager".to_string()),
            ui_parent_id,
        );
        bar_mgr.visible = true;
        bar_mgr.width = 800.0;
        bar_mgr.height = 14.0;
        let bar_mgr_id = bar_mgr.id;
        state.borrow_mut().widgets.register(bar_mgr);

        let handle = FrameHandle {
            id: bar_mgr_id,
            state: Rc::clone(&state),
        };
        let bar_mgr_ud = lua.create_userdata(handle)?;
        globals.set("StatusTrackingBarManager", bar_mgr_ud.clone())?;

        // Add bars table for DynamicCam - set via Lua so it's accessible
        let bars_table = lua.create_table()?;
        lua.load(r#"
            StatusTrackingBarManager.bars = ...
        "#).call::<()>(bars_table)?;
    }

    // Create MainStatusTrackingBarContainer and SecondaryStatusTrackingBarContainer (used by DynamicCam)
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let containers = [
            "MainStatusTrackingBarContainer",
            "SecondaryStatusTrackingBarContainer",
        ];

        for name in containers {
            let mut container = Frame::new(
                WidgetType::Frame,
                Some(name.to_string()),
                ui_parent_id,
            );
            container.visible = true;
            container.width = 800.0;
            container.height = 14.0;
            let container_id = container.id;
            state.borrow_mut().widgets.register(container);

            let handle = FrameHandle {
                id: container_id,
                state: Rc::clone(&state),
            };
            let container_ud = lua.create_userdata(handle)?;
            globals.set(name, container_ud.clone())?;

            // Add bars table for DynamicCam
            let bars_table = lua.create_table()?;
            lua.load(format!(r#"
                {}.bars = ...
            "#, name).as_str()).call::<()>(bars_table)?;
        }
    }

    // Create CompactRaidFrameContainer (used by DynamicCam for raid frame hiding)
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut container = Frame::new(
            WidgetType::Frame,
            Some("CompactRaidFrameContainer".to_string()),
            ui_parent_id,
        );
        container.visible = true;
        container.width = 200.0;
        container.height = 200.0;
        let container_id = container.id;
        state.borrow_mut().widgets.register(container);

        let handle = FrameHandle {
            id: container_id,
            state: Rc::clone(&state),
        };
        let container_ud = lua.create_userdata(handle)?;
        globals.set("CompactRaidFrameContainer", container_ud)?;
    }

    // Create GameTooltipTextLeft1..N and GameTooltipTextRight1..N font strings
    // Many addons access these directly to read tooltip text
    for i in 1..=16 {
        let left_name = format!("GameTooltipTextLeft{}", i);
        let right_name = format!("GameTooltipTextRight{}", i);

        // Create stub font string tables with GetText method
        let left_fontstring = lua.create_table()?;
        left_fontstring.set("GetText", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
        left_fontstring.set("SetText", lua.create_function(|_, _text: Option<String>| Ok(()))?)?;
        left_fontstring.set("GetFontObject", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
        left_fontstring.set("GetFont", lua.create_function(|_, ()| Ok(("Fonts/FRIZQT__.TTF", 12.0, "")))?)?;
        globals.set(left_name.as_str(), left_fontstring)?;

        let right_fontstring = lua.create_table()?;
        right_fontstring.set("GetText", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
        right_fontstring.set("SetText", lua.create_function(|_, _text: Option<String>| Ok(()))?)?;
        right_fontstring.set("GetFontObject", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
        right_fontstring.set("GetFont", lua.create_function(|_, ()| Ok(("Fonts/FRIZQT__.TTF", 12.0, "")))?)?;
        globals.set(right_name.as_str(), right_fontstring)?;
    }

    // ====================================================================
    // UIDropDownMenu System
    // ====================================================================
    // WoW's dropdown menu system using global frames and functions

    // Global constants
    globals.set("UIDROPDOWNMENU_MAXBUTTONS", 1)?;
    globals.set("UIDROPDOWNMENU_MAXLEVELS", 3)?;
    globals.set("UIDROPDOWNMENU_BUTTON_HEIGHT", 16)?;
    globals.set("UIDROPDOWNMENU_BORDER_HEIGHT", 15)?;
    globals.set("UIDROPDOWNMENU_OPEN_MENU", Value::Nil)?;
    globals.set("UIDROPDOWNMENU_INIT_MENU", Value::Nil)?;
    globals.set("UIDROPDOWNMENU_MENU_LEVEL", 1)?;
    globals.set("UIDROPDOWNMENU_MENU_VALUE", Value::Nil)?;
    globals.set("UIDROPDOWNMENU_SHOW_TIME", 2)?;
    globals.set("UIDROPDOWNMENU_DEFAULT_TEXT_HEIGHT", Value::Nil)?;
    globals.set("UIDROPDOWNMENU_DEFAULT_WIDTH_PADDING", 25)?;
    globals.set("OPEN_DROPDOWNMENUS", lua.create_table()?)?;

    // Create DropDownList frames (DropDownList1, DropDownList2, DropDownList3)
    for level in 1..=3 {
        let list_name = format!("DropDownList{}", level);
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut list_frame = Frame::new(
            WidgetType::Button,
            Some(list_name.clone()),
            ui_parent_id,
        );
        list_frame.visible = false;
        list_frame.width = 180.0;
        list_frame.height = 32.0;
        list_frame.frame_strata = FrameStrata::FullscreenDialog;
        list_frame.clamped_to_screen = true;
        let list_id = list_frame.id;
        state.borrow_mut().widgets.register(list_frame);

        let handle = FrameHandle {
            id: list_id,
            state: Rc::clone(&state),
        };
        let list_ud = lua.create_userdata(handle)?;

        // Set numButtons and maxWidth fields
        {
            let fields_table: mlua::Table =
                globals.get::<mlua::Table>("__frame_fields").unwrap_or_else(|_| {
                    let t = lua.create_table().unwrap();
                    globals.set("__frame_fields", t.clone()).unwrap();
                    t
                });
            let frame_fields = lua.create_table()?;
            frame_fields.set("numButtons", 0)?;
            frame_fields.set("maxWidth", 0)?;
            fields_table.set(list_id, frame_fields)?;
        }

        globals.set(list_name.as_str(), list_ud.clone())?;
        let frame_key = format!("__frame_{}", list_id);
        globals.set(frame_key.as_str(), list_ud)?;

        // Create buttons for each dropdown list (DropDownList1Button1, etc.)
        for btn_idx in 1..=8 {
            let btn_name = format!("DropDownList{}Button{}", level, btn_idx);
            let mut btn_frame = Frame::new(
                WidgetType::Button,
                Some(btn_name.clone()),
                Some(list_id),
            );
            btn_frame.visible = false;
            btn_frame.width = 100.0;
            btn_frame.height = 16.0;
            let btn_id = btn_frame.id;
            state.borrow_mut().widgets.register(btn_frame);

            let btn_handle = FrameHandle {
                id: btn_id,
                state: Rc::clone(&state),
            };
            let btn_ud = lua.create_userdata(btn_handle)?;
            globals.set(btn_name.as_str(), btn_ud.clone())?;
            let btn_frame_key = format!("__frame_{}", btn_id);
            globals.set(btn_frame_key.as_str(), btn_ud)?;

            // Create child elements for buttons (NormalText, Icon, etc.)
            let text_name = format!("DropDownList{}Button{}NormalText", level, btn_idx);
            let mut text_frame = Frame::new(
                WidgetType::FontString,
                Some(text_name.clone()),
                Some(btn_id),
            );
            text_frame.visible = true;
            let text_id = text_frame.id;
            state.borrow_mut().widgets.register(text_frame);

            let text_handle = FrameHandle {
                id: text_id,
                state: Rc::clone(&state),
            };
            let text_ud = lua.create_userdata(text_handle)?;
            globals.set(text_name.as_str(), text_ud)?;
        }
    }

    // UIDropDownMenu_CreateInfo() - returns empty table for info structure
    let create_info = lua.create_function(|lua, ()| {
        lua.create_table().map(Value::Table)
    })?;
    globals.set("UIDropDownMenu_CreateInfo", create_info)?;

    // UIDropDownMenu_Initialize(frame, initFunction, displayMode, level, menuList)
    let _state_init = Rc::clone(&state);
    let init_func = lua.create_function(
        move |lua,
              (frame, init_fn, _display_mode, _level, _menu_list): (
                  Value,
                  Option<mlua::Function>,
                  Option<String>,
                  Option<i32>,
                  Option<mlua::Table>,
              )| {
            // Store the initialize function on the frame
            if let Value::UserData(ud) = &frame {
                if let Ok(handle) = ud.borrow::<FrameHandle>() {
                    let frame_id = handle.id;
                    let fields_table: mlua::Table = lua
                        .globals()
                        .get::<mlua::Table>("__frame_fields")
                        .unwrap_or_else(|_| {
                            let t = lua.create_table().unwrap();
                            lua.globals().set("__frame_fields", t.clone()).unwrap();
                            t
                        });
                    let frame_fields: mlua::Table =
                        fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                            let t = lua.create_table().unwrap();
                            fields_table.set(frame_id, t.clone()).unwrap();
                            t
                        });
                    if let Some(ref func) = init_fn {
                        frame_fields.set("initialize", func.clone())?;
                    }
                }
            }

            // Set UIDROPDOWNMENU_INIT_MENU
            lua.globals().set("UIDROPDOWNMENU_INIT_MENU", frame.clone())?;

            // Call the init function if provided
            if let Some(func) = init_fn {
                let level = _level.unwrap_or(1);
                let _ = func.call::<()>((frame.clone(), level, _menu_list));
            }

            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_Initialize", init_func)?;

    // UIDropDownMenu_AddButton(info, level) - adds a button to the menu
    let state_add = Rc::clone(&state);
    let add_button = lua.create_function(move |lua, (info, level): (mlua::Table, Option<i32>)| {
        let level = level.unwrap_or(1);
        let list_name = format!("DropDownList{}", level);

        // Get the list frame and increment numButtons
        if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>(list_name.as_str()) {
            if let Ok(handle) = list_ud.borrow::<FrameHandle>() {
                let frame_id = handle.id;
                let fields_table: mlua::Table = lua
                    .globals()
                    .get::<mlua::Table>("__frame_fields")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_fields", t.clone()).unwrap();
                        t
                    });
                let frame_fields: mlua::Table =
                    fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        fields_table.set(frame_id, t.clone()).unwrap();
                        t
                    });
                let num_buttons: i32 = frame_fields.get("numButtons").unwrap_or(0);
                let new_index = num_buttons + 1;
                frame_fields.set("numButtons", new_index)?;

                // Get the button and configure it
                let btn_name = format!("DropDownList{}Button{}", level, new_index);
                if let Ok(btn_ud) = lua.globals().get::<mlua::AnyUserData>(btn_name.as_str()) {
                    if let Ok(btn_handle) = btn_ud.borrow::<FrameHandle>() {
                        // Store button properties from info table
                        let btn_id = btn_handle.id;
                        let btn_fields: mlua::Table =
                            fields_table.get::<mlua::Table>(btn_id).unwrap_or_else(|_| {
                                let t = lua.create_table().unwrap();
                                fields_table.set(btn_id, t.clone()).unwrap();
                                t
                            });

                        // Copy info properties to button fields
                        for pair in info.pairs::<String, Value>() {
                            if let Ok((k, v)) = pair {
                                btn_fields.set(k, v)?;
                            }
                        }

                        // Set the text if provided
                        if let Ok(text) = info.get::<mlua::String>("text") {
                            let mut s = state_add.borrow_mut();
                            if let Some(btn_frame) = s.widgets.get_mut(btn_id) {
                                btn_frame.text = Some(text.to_string_lossy().to_string());
                                btn_frame.visible = true;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    })?;
    globals.set("UIDropDownMenu_AddButton", add_button)?;

    // UIDropDownMenu_SetWidth(frame, width, padding)
    let state_width = Rc::clone(&state);
    let set_width = lua.create_function(
        move |_lua, (frame, width, _padding): (mlua::AnyUserData, f32, Option<f32>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_width.borrow_mut();
                if let Some(f) = s.widgets.get_mut(handle.id) {
                    f.width = width;
                }
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetWidth", set_width)?;

    // UIDropDownMenu_SetText(frame, text)
    let state_text = Rc::clone(&state);
    let set_text = lua.create_function(
        move |_lua, (frame, text): (mlua::AnyUserData, Option<String>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_text.borrow_mut();
                if let Some(f) = s.widgets.get_mut(handle.id) {
                    f.text = text;
                }
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetText", set_text)?;

    // UIDropDownMenu_GetText(frame) -> string
    let state_get_text = Rc::clone(&state);
    let get_text_fn = lua.create_function(move |lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let s = state_get_text.borrow();
            if let Some(f) = s.widgets.get(handle.id) {
                if let Some(ref text) = f.text {
                    let lua_str = lua.create_string(text)?;
                    return Ok(Value::String(lua_str));
                }
            }
        }
        Ok(Value::Nil)
    })?;
    globals.set("UIDropDownMenu_GetText", get_text_fn)?;

    // UIDropDownMenu_SetSelectedID(frame, id, useValue)
    let set_selected_id = lua.create_function(
        |lua, (frame, id, _use_value): (mlua::AnyUserData, i32, Option<bool>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let frame_id = handle.id;
                let fields_table: mlua::Table = lua
                    .globals()
                    .get::<mlua::Table>("__frame_fields")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_fields", t.clone()).unwrap();
                        t
                    });
                let frame_fields: mlua::Table =
                    fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        fields_table.set(frame_id, t.clone()).unwrap();
                        t
                    });
                frame_fields.set("selectedID", id)?;
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetSelectedID", set_selected_id)?;

    // UIDropDownMenu_GetSelectedID(frame) -> number
    let get_selected_id = lua.create_function(|lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let frame_id = handle.id;
            if let Ok(fields_table) = lua.globals().get::<mlua::Table>("__frame_fields") {
                if let Ok(frame_fields) = fields_table.get::<mlua::Table>(frame_id) {
                    if let Ok(id) = frame_fields.get::<i32>("selectedID") {
                        return Ok(Value::Integer(id as i64));
                    }
                }
            }
        }
        Ok(Value::Nil)
    })?;
    globals.set("UIDropDownMenu_GetSelectedID", get_selected_id)?;

    // UIDropDownMenu_SetSelectedValue(frame, value, useValue)
    let set_selected_value = lua.create_function(
        |lua, (frame, value, _use_value): (mlua::AnyUserData, Value, Option<bool>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let frame_id = handle.id;
                let fields_table: mlua::Table = lua
                    .globals()
                    .get::<mlua::Table>("__frame_fields")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_fields", t.clone()).unwrap();
                        t
                    });
                let frame_fields: mlua::Table =
                    fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        fields_table.set(frame_id, t.clone()).unwrap();
                        t
                    });
                frame_fields.set("selectedValue", value)?;
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetSelectedValue", set_selected_value)?;

    // UIDropDownMenu_GetSelectedValue(frame) -> value
    let get_selected_value = lua.create_function(|lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let frame_id = handle.id;
            if let Ok(fields_table) = lua.globals().get::<mlua::Table>("__frame_fields") {
                if let Ok(frame_fields) = fields_table.get::<mlua::Table>(frame_id) {
                    if let Ok(value) = frame_fields.get::<Value>("selectedValue") {
                        return Ok(value);
                    }
                }
            }
        }
        Ok(Value::Nil)
    })?;
    globals.set("UIDropDownMenu_GetSelectedValue", get_selected_value)?;

    // UIDropDownMenu_SetSelectedName(frame, name, useValue)
    let state_sel_name = Rc::clone(&state);
    let set_selected_name = lua.create_function(
        move |_lua, (frame, name, _use_value): (mlua::AnyUserData, String, Option<bool>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_sel_name.borrow_mut();
                if let Some(f) = s.widgets.get_mut(handle.id) {
                    f.text = Some(name);
                }
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetSelectedName", set_selected_name)?;

    // UIDropDownMenu_EnableDropDown(frame)
    let state_enable = Rc::clone(&state);
    let enable_dropdown = lua.create_function(move |_lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let mut s = state_enable.borrow_mut();
            if let Some(f) = s.widgets.get_mut(handle.id) {
                f.mouse_enabled = true;
            }
        }
        Ok(())
    })?;
    globals.set("UIDropDownMenu_EnableDropDown", enable_dropdown)?;

    // UIDropDownMenu_DisableDropDown(frame)
    let state_disable = Rc::clone(&state);
    let disable_dropdown = lua.create_function(move |_lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let mut s = state_disable.borrow_mut();
            if let Some(f) = s.widgets.get_mut(handle.id) {
                f.mouse_enabled = false;
            }
        }
        Ok(())
    })?;
    globals.set("UIDropDownMenu_DisableDropDown", disable_dropdown)?;

    // UIDropDownMenu_Refresh(frame, useValue, dropdownLevel)
    let refresh_dropdown = lua.create_function(
        |_lua, (_frame, _use_value, _level): (Value, Option<bool>, Option<i32>)| {
            // Stub - refresh would re-run init function
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_Refresh", refresh_dropdown)?;

    // UIDropDownMenu_SetAnchor(dropdown, xOffset, yOffset, point, relativeTo, relativePoint)
    let set_anchor = lua.create_function(
        |_lua,
         (_dropdown, _x, _y, _point, _relative_to, _relative_point): (
            Value,
            Option<f32>,
            Option<f32>,
            Option<String>,
            Option<Value>,
            Option<String>,
        )| {
            // Stub - anchor settings for menu display
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetAnchor", set_anchor)?;

    // ToggleDropDownMenu(level, value, dropDownFrame, anchorName, xOffset, yOffset, menuList, button, autoHideDelay, displayMode)
    let state_toggle = Rc::clone(&state);
    let toggle_dropdown = lua.create_function(
        move |lua,
              (level, _value, dropdown_frame, _anchor, _x_off, _y_off, _menu_list, _button, _auto_hide, _display_mode): (
                  Option<i32>,
                  Option<Value>,
                  Option<Value>,
                  Option<String>,
                  Option<f32>,
                  Option<f32>,
                  Option<mlua::Table>,
                  Option<Value>,
                  Option<f32>,
                  Option<String>,
              )| {
            let level = level.unwrap_or(1);
            let list_name = format!("DropDownList{}", level);

            // Toggle visibility of the dropdown list
            if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>(list_name.as_str()) {
                if let Ok(handle) = list_ud.borrow::<FrameHandle>() {
                    let mut s = state_toggle.borrow_mut();
                    if let Some(f) = s.widgets.get_mut(handle.id) {
                        f.visible = !f.visible;
                    }
                }
            }

            // Set UIDROPDOWNMENU_OPEN_MENU
            if let Some(frame) = dropdown_frame {
                lua.globals().set("UIDROPDOWNMENU_OPEN_MENU", frame)?;
            }

            lua.globals().set("UIDROPDOWNMENU_MENU_LEVEL", level)?;

            Ok(())
        },
    )?;
    globals.set("ToggleDropDownMenu", toggle_dropdown)?;

    // CloseDropDownMenus(level) - close menus at specified level and above
    let state_close = Rc::clone(&state);
    let close_menus = lua.create_function(move |lua, level: Option<i32>| {
        let start_level = level.unwrap_or(1);
        for lvl in start_level..=3 {
            let list_name = format!("DropDownList{}", lvl);
            if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>(list_name.as_str()) {
                if let Ok(handle) = list_ud.borrow::<FrameHandle>() {
                    let mut s = state_close.borrow_mut();
                    if let Some(f) = s.widgets.get_mut(handle.id) {
                        f.visible = false;
                    }
                }
            }
        }
        lua.globals().set("UIDROPDOWNMENU_OPEN_MENU", Value::Nil)?;
        Ok(())
    })?;
    globals.set("CloseDropDownMenus", close_menus)?;

    // UIDropDownMenu_HandleGlobalMouseEvent(button, event) - handles clicks outside to close
    let handle_global_mouse = lua.create_function(|_lua, (_button, _event): (Option<String>, Option<String>)| {
        // Stub - would check if click is outside menu and close
        Ok(())
    })?;
    globals.set("UIDropDownMenu_HandleGlobalMouseEvent", handle_global_mouse)?;

    // UIDropDownMenu_SetInitializeFunction(frame, initFunction)
    let set_init_func = lua.create_function(|lua, (frame, init_fn): (mlua::AnyUserData, mlua::Function)| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let frame_id = handle.id;
            let fields_table: mlua::Table = lua
                .globals()
                .get::<mlua::Table>("__frame_fields")
                .unwrap_or_else(|_| {
                    let t = lua.create_table().unwrap();
                    lua.globals().set("__frame_fields", t.clone()).unwrap();
                    t
                });
            let frame_fields: mlua::Table =
                fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                    let t = lua.create_table().unwrap();
                    fields_table.set(frame_id, t.clone()).unwrap();
                    t
                });
            frame_fields.set("initialize", init_fn)?;
        }
        Ok(())
    })?;
    globals.set("UIDropDownMenu_SetInitializeFunction", set_init_func)?;

    // UIDropDownMenu_JustifyText(frame, justification)
    let justify_text = lua.create_function(|_lua, (_frame, _justify): (Value, Option<String>)| {
        // Stub - text justification
        Ok(())
    })?;
    globals.set("UIDropDownMenu_JustifyText", justify_text)?;

    // UIDropDownMenu_SetFrameStrata(frame, strata)
    let state_strata = Rc::clone(&state);
    let set_frame_strata = lua.create_function(
        move |_lua, (frame, strata): (mlua::AnyUserData, String)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_strata.borrow_mut();
                if let Some(f) = s.widgets.get_mut(handle.id) {
                    if let Some(fs) = FrameStrata::from_str(&strata) {
                        f.frame_strata = fs;
                    }
                }
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetFrameStrata", set_frame_strata)?;

    // UIDropDownMenu_AddSeparator(level)
    let add_separator = lua.create_function(|lua, level: Option<i32>| {
        // Add a separator (empty button acting as divider)
        let info = lua.create_table()?;
        info.set("disabled", true)?;
        info.set("notCheckable", true)?;
        let add_btn: mlua::Function = lua.globals().get("UIDropDownMenu_AddButton")?;
        add_btn.call::<()>((info, level))?;
        Ok(())
    })?;
    globals.set("UIDropDownMenu_AddSeparator", add_separator)?;

    // UIDropDownMenu_AddSpace(level)
    let add_space = lua.create_function(|lua, level: Option<i32>| {
        let info = lua.create_table()?;
        info.set("disabled", true)?;
        info.set("notCheckable", true)?;
        info.set("isTitle", true)?;
        let add_btn: mlua::Function = lua.globals().get("UIDropDownMenu_AddButton")?;
        add_btn.call::<()>((info, level))?;
        Ok(())
    })?;
    globals.set("UIDropDownMenu_AddSpace", add_space)?;

    // UIDropDownMenu_GetCurrentDropDown() -> frame
    let get_current = lua.create_function(|lua, ()| {
        lua.globals().get::<Value>("UIDROPDOWNMENU_INIT_MENU")
    })?;
    globals.set("UIDropDownMenu_GetCurrentDropDown", get_current)?;

    // UIDropDownMenu_IsOpen(frame) -> boolean
    let state_is_open = Rc::clone(&state);
    let is_open = lua.create_function(move |lua, frame: Option<mlua::AnyUserData>| {
        if let Some(f) = frame {
            if let Ok(handle) = f.borrow::<FrameHandle>() {
                // Check if this frame is the open menu
                if let Ok(open_menu) = lua.globals().get::<mlua::AnyUserData>("UIDROPDOWNMENU_OPEN_MENU") {
                    if let Ok(open_handle) = open_menu.borrow::<FrameHandle>() {
                        if open_handle.id == handle.id {
                            // Check if DropDownList1 is visible
                            if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>("DropDownList1") {
                                if let Ok(list_handle) = list_ud.borrow::<FrameHandle>() {
                                    let s = state_is_open.borrow();
                                    if let Some(list) = s.widgets.get(list_handle.id) {
                                        return Ok(list.visible);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(false)
    })?;
    globals.set("UIDropDownMenu_IsOpen", is_open)?;

    // Add more Enum values to the existing Enum table (created earlier in this function)
    let enum_table: mlua::Table = globals.get("Enum")?;

    // TooltipDataLineType - types of tooltip lines
    let tooltip_data_line_type = lua.create_table()?;
    tooltip_data_line_type.set("None", 0)?;
    tooltip_data_line_type.set("QuestTitle", 1)?;
    tooltip_data_line_type.set("QuestTimeRemaining", 2)?;
    tooltip_data_line_type.set("QuestObjective", 3)?;
    tooltip_data_line_type.set("QuestEnding", 4)?;
    tooltip_data_line_type.set("QuestReward", 5)?;
    tooltip_data_line_type.set("ItemName", 6)?;
    tooltip_data_line_type.set("ItemLevel", 7)?;
    tooltip_data_line_type.set("ItemBinding", 8)?;
    tooltip_data_line_type.set("ItemUnique", 9)?;
    tooltip_data_line_type.set("SpellName", 10)?;
    tooltip_data_line_type.set("SpellRange", 11)?;
    tooltip_data_line_type.set("SpellCastTime", 12)?;
    tooltip_data_line_type.set("SpellCooldown", 13)?;
    tooltip_data_line_type.set("SpellDescription", 14)?;
    tooltip_data_line_type.set("UnitName", 15)?;
    tooltip_data_line_type.set("UnitLevel", 16)?;
    tooltip_data_line_type.set("UnitClass", 17)?;
    tooltip_data_line_type.set("UnitFaction", 18)?;
    tooltip_data_line_type.set("UnitPVPFaction", 19)?;
    enum_table.set("TooltipDataLineType", tooltip_data_line_type)?;

    // TooltipDataType - types of tooltip data (matches first definition earlier in file)
    let tooltip_data_type = lua.create_table()?;
    tooltip_data_type.set("Item", 0)?;
    tooltip_data_type.set("Spell", 1)?;
    tooltip_data_type.set("Unit", 2)?;
    tooltip_data_type.set("Corpse", 3)?;
    tooltip_data_type.set("Object", 4)?;
    tooltip_data_type.set("Currency", 5)?;
    tooltip_data_type.set("Achievement", 6)?;
    tooltip_data_type.set("Quest", 7)?;
    tooltip_data_type.set("QuestItem", 8)?;
    tooltip_data_type.set("BattlePet", 9)?;
    tooltip_data_type.set("CompanionPet", 10)?;
    tooltip_data_type.set("Mount", 11)?;
    tooltip_data_type.set("Macro", 12)?;
    tooltip_data_type.set("EquipmentSet", 13)?;
    tooltip_data_type.set("Hyperlink", 14)?;
    tooltip_data_type.set("Toy", 15)?;
    tooltip_data_type.set("RecipeRankInfo", 16)?;
    tooltip_data_type.set("Totem", 17)?;
    tooltip_data_type.set("UnitAura", 18)?;
    tooltip_data_type.set("QuestPartyProgress", 19)?;
    tooltip_data_type.set("InstanceLock", 20)?;
    tooltip_data_type.set("MinimapMouseover", 21)?;
    tooltip_data_type.set("CorruptionReplacementEffect", 22)?;
    enum_table.set("TooltipDataType", tooltip_data_type)?;

    // LootSlotType enums - types of loot slots
    let loot_slot_type = lua.create_table()?;
    loot_slot_type.set("None", 0)?;
    loot_slot_type.set("Item", 1)?;
    loot_slot_type.set("Money", 2)?;
    loot_slot_type.set("Currency", 3)?;
    enum_table.set("LootSlotType", loot_slot_type)?;

    // ItemQuality enums
    let item_quality = lua.create_table()?;
    item_quality.set("Poor", 0)?;
    item_quality.set("Common", 1)?;
    item_quality.set("Uncommon", 2)?;
    item_quality.set("Good", 2)?; // Alias for Uncommon (used by Auctionator)
    item_quality.set("Rare", 3)?;
    item_quality.set("Epic", 4)?;
    item_quality.set("Legendary", 5)?;
    item_quality.set("Artifact", 6)?;
    item_quality.set("Heirloom", 7)?;
    item_quality.set("WoWToken", 8)?;
    enum_table.set("ItemQuality", item_quality)?;

    // ItemClass enums - item types
    let item_class = lua.create_table()?;
    item_class.set("Consumable", 0)?;
    item_class.set("Container", 1)?;
    item_class.set("Weapon", 2)?;
    item_class.set("Gem", 3)?;
    item_class.set("Armor", 4)?;
    item_class.set("Reagent", 5)?;
    item_class.set("Projectile", 6)?;
    item_class.set("Tradegoods", 7)?;
    item_class.set("ItemEnhancement", 8)?;
    item_class.set("Recipe", 9)?;
    item_class.set("Quiver", 11)?;
    item_class.set("Quest", 12)?;
    item_class.set("Questitem", 12)?; // Alias for Quest (used by Auctionator)
    item_class.set("Key", 13)?;
    item_class.set("Miscellaneous", 15)?;
    item_class.set("Glyph", 16)?;
    item_class.set("Battlepet", 17)?;
    item_class.set("WoWToken", 18)?;
    item_class.set("Profession", 19)?;
    enum_table.set("ItemClass", item_class)?;

    // UIWidgetVisualizationType enums
    let ui_widget_viz_type = lua.create_table()?;
    ui_widget_viz_type.set("IconAndText", 0)?;
    ui_widget_viz_type.set("CaptureBar", 1)?;
    ui_widget_viz_type.set("StatusBar", 2)?;
    ui_widget_viz_type.set("DoubleStatusBar", 3)?;
    ui_widget_viz_type.set("IconTextAndBackground", 4)?;
    ui_widget_viz_type.set("DoubleIconAndText", 5)?;
    ui_widget_viz_type.set("StackedResourceTracker", 6)?;
    ui_widget_viz_type.set("IconTextAndCurrencies", 7)?;
    ui_widget_viz_type.set("TextWithState", 8)?;
    ui_widget_viz_type.set("HorizontalCurrencies", 9)?;
    ui_widget_viz_type.set("BulletTextList", 10)?;
    ui_widget_viz_type.set("ScenarioHeaderCurrenciesAndBackground", 11)?;
    ui_widget_viz_type.set("TextureAndText", 12)?;
    ui_widget_viz_type.set("SpellDisplay", 13)?;
    ui_widget_viz_type.set("DoubleStateIconRow", 14)?;
    ui_widget_viz_type.set("TextureAndTextRow", 15)?;
    ui_widget_viz_type.set("ZoneControl", 16)?;
    ui_widget_viz_type.set("CaptureZone", 17)?;
    ui_widget_viz_type.set("TextureWithAnimation", 18)?;
    ui_widget_viz_type.set("DiscreteProgressSteps", 19)?;
    ui_widget_viz_type.set("ScenarioHeaderTimer", 20)?;
    ui_widget_viz_type.set("TextColumnRow", 21)?;
    ui_widget_viz_type.set("Spacer", 22)?;
    ui_widget_viz_type.set("UnitPowerBar", 23)?;
    ui_widget_viz_type.set("FillUpFrames", 24)?;
    ui_widget_viz_type.set("TextWithSubtext", 25)?;
    ui_widget_viz_type.set("MapPinAnimation", 26)?;
    ui_widget_viz_type.set("ItemDisplay", 27)?;
    ui_widget_viz_type.set("TugOfWar", 28)?;
    enum_table.set("UIWidgetVisualizationType", ui_widget_viz_type)?;

    // TransmogType enums - types of transmog appearances
    let transmog_type = lua.create_table()?;
    transmog_type.set("Appearance", 0)?;
    transmog_type.set("Illusion", 1)?;
    enum_table.set("TransmogType", transmog_type)?;

    // TransmogModification enums
    let transmog_mod = lua.create_table()?;
    transmog_mod.set("Main", 0)?;
    transmog_mod.set("Secondary", 1)?;
    transmog_mod.set("None", 2)?;
    enum_table.set("TransmogModification", transmog_mod)?;

    // TransmogSource enums - sources of transmog appearances
    let transmog_source = lua.create_table()?;
    transmog_source.set("None", 0)?;
    transmog_source.set("JournalEncounter", 1)?;
    transmog_source.set("Quest", 2)?;
    transmog_source.set("Vendor", 3)?;
    transmog_source.set("WorldDrop", 4)?;
    transmog_source.set("Achievement", 5)?;
    transmog_source.set("Profession", 6)?;
    transmog_source.set("TradingPost", 7)?;
    transmog_source.set("NotValidForTransmog", 8)?;
    enum_table.set("TransmogSource", transmog_source)?;

    // AuctionHouseSortOrder - auction house sorting options
    let auction_sort_order = lua.create_table()?;
    auction_sort_order.set("Price", 0)?;
    auction_sort_order.set("Name", 1)?;
    auction_sort_order.set("Level", 2)?;
    auction_sort_order.set("Bid", 3)?;
    auction_sort_order.set("Buyout", 4)?;
    auction_sort_order.set("Quality", 5)?;
    auction_sort_order.set("TimeRemaining", 6)?;
    auction_sort_order.set("Seller", 7)?;
    auction_sort_order.set("UnitPrice", 8)?;
    enum_table.set("AuctionHouseSortOrder", auction_sort_order)?;

    // AuctionHouseTimeLeftBand - time remaining bands for auctions
    let auction_time_left = lua.create_table()?;
    auction_time_left.set("Short", 0)?;
    auction_time_left.set("Medium", 1)?;
    auction_time_left.set("Long", 2)?;
    auction_time_left.set("VeryLong", 3)?;
    enum_table.set("AuctionHouseTimeLeftBand", auction_time_left)?;

    // ItemRecipeSubclass - recipe/profession item subclasses
    let recipe_subclass = lua.create_table()?;
    recipe_subclass.set("Book", 0)?;
    recipe_subclass.set("Leatherworking", 1)?;
    recipe_subclass.set("Tailoring", 2)?;
    recipe_subclass.set("Engineering", 3)?;
    recipe_subclass.set("Blacksmithing", 4)?;
    recipe_subclass.set("Cooking", 5)?;
    recipe_subclass.set("Alchemy", 6)?;
    recipe_subclass.set("FirstAid", 7)?;
    recipe_subclass.set("Enchanting", 8)?;
    recipe_subclass.set("Fishing", 9)?;
    recipe_subclass.set("Jewelcrafting", 10)?;
    recipe_subclass.set("Inscription", 11)?;
    enum_table.set("ItemRecipeSubclass", recipe_subclass)?;

    // ItemBind - item binding types
    let item_bind = lua.create_table()?;
    item_bind.set("None", 0)?;
    item_bind.set("OnEquip", 1)?;
    item_bind.set("OnAcquire", 2)?;  // Bind on pickup
    item_bind.set("OnUse", 3)?;
    item_bind.set("Quest", 4)?;
    item_bind.set("ToAccount", 5)?;  // BoA
    item_bind.set("ToWoWAccount", 6)?;
    item_bind.set("ToBnetAccount", 7)?;
    enum_table.set("ItemBind", item_bind)?;

    // AddOnProfilerMetric - addon profiling metric types
    let addon_profiler_metric = lua.create_table()?;
    addon_profiler_metric.set("SessionAverageTime", 0)?;
    addon_profiler_metric.set("RecentAverageTime", 1)?;
    addon_profiler_metric.set("EncounterAverageTime", 2)?;
    addon_profiler_metric.set("LastTime", 3)?;
    addon_profiler_metric.set("PeakTime", 4)?;
    addon_profiler_metric.set("CountTimeOver1Ms", 5)?;
    addon_profiler_metric.set("CountTimeOver5Ms", 6)?;
    addon_profiler_metric.set("CountTimeOver10Ms", 7)?;
    addon_profiler_metric.set("CountTimeOver50Ms", 8)?;
    addon_profiler_metric.set("CountTimeOver100Ms", 9)?;
    addon_profiler_metric.set("CountTimeOver500Ms", 10)?;
    enum_table.set("AddOnProfilerMetric", addon_profiler_metric)?;

    // AuctionHouseFilter - auction house quality/type filters
    let ah_filter = lua.create_table()?;
    ah_filter.set("PoorQuality", 0)?;
    ah_filter.set("CommonQuality", 1)?;
    ah_filter.set("UncommonQuality", 2)?;
    ah_filter.set("RareQuality", 3)?;
    ah_filter.set("EpicQuality", 4)?;
    ah_filter.set("LegendaryQuality", 5)?;
    ah_filter.set("ArtifactQuality", 6)?;
    ah_filter.set("HeirloomQuality", 7)?;
    ah_filter.set("UncollectedOnly", 8)?;
    ah_filter.set("CanUseOnly", 9)?;
    ah_filter.set("UpgradesOnly", 10)?;
    ah_filter.set("ExactMatch", 11)?;
    enum_table.set("AuctionHouseFilter", ah_filter)?;

    // Damageclass - damage school bit masks used by combat log
    let damageclass = lua.create_table()?;
    damageclass.set("MaskPhysical", 1)?;  // SCHOOL_MASK_PHYSICAL
    damageclass.set("MaskHoly", 2)?;      // SCHOOL_MASK_HOLY
    damageclass.set("MaskFire", 4)?;      // SCHOOL_MASK_FIRE
    damageclass.set("MaskNature", 8)?;    // SCHOOL_MASK_NATURE
    damageclass.set("MaskFrost", 16)?;    // SCHOOL_MASK_FROST
    damageclass.set("MaskShadow", 32)?;   // SCHOOL_MASK_SHADOW
    damageclass.set("MaskArcane", 64)?;   // SCHOOL_MASK_ARCANE
    enum_table.set("Damageclass", damageclass)?;

    // EditModeSystem - Edit Mode UI frame indices (used by EditModeExpanded)
    let edit_mode_system = lua.create_table()?;
    edit_mode_system.set("ActionBar1", 0)?;
    edit_mode_system.set("ActionBar2", 1)?;
    edit_mode_system.set("ActionBar3", 2)?;
    edit_mode_system.set("ActionBar4", 3)?;
    edit_mode_system.set("ActionBar5", 4)?;
    edit_mode_system.set("ActionBar6", 5)?;
    edit_mode_system.set("ActionBar7", 6)?;
    edit_mode_system.set("ActionBar8", 7)?;
    edit_mode_system.set("MainMenuBar", 8)?;
    edit_mode_system.set("MultiBarBottomLeft", 9)?;
    edit_mode_system.set("MultiBarBottomRight", 10)?;
    edit_mode_system.set("MultiBarRight", 11)?;
    edit_mode_system.set("MultiBarLeft", 12)?;
    edit_mode_system.set("EncounterBar", 13)?;
    edit_mode_system.set("StanceBar", 14)?;
    edit_mode_system.set("PetActionBar", 15)?;
    edit_mode_system.set("PossessActionBar", 16)?;
    edit_mode_system.set("ExtraAbilityContainer", 17)?;
    edit_mode_system.set("MicroMenu", 18)?;
    edit_mode_system.set("BagsBar", 19)?;
    enum_table.set("EditModeSystem", edit_mode_system)?;

    // (No need to set Enum again - we modified the existing table)

    // Item class - for async item loading (used by LegionRemixHelper, etc)
    let item_class = lua.create_table()?;
    item_class.set(
        "CreateFromItemID",
        lua.create_function(|lua, (_self, item_id): (Value, i32)| {
            // Create an item object with callback methods
            let item = lua.create_table()?;
            item.set("itemID", item_id)?;

            // ContinueOnItemLoad - calls callback immediately in simulation
            item.set(
                "ContinueOnItemLoad",
                lua.create_function(|_, (this, callback): (mlua::Table, mlua::Function)| {
                    // In simulation, immediately call the callback
                    callback.call::<()>(())?;
                    let _ = this; // Silence unused warning
                    Ok(())
                })?,
            )?;

            // GetItemID
            item.set(
                "GetItemID",
                lua.create_function(|_, this: mlua::Table| {
                    this.get::<i32>("itemID")
                })?,
            )?;

            // GetItemName - return placeholder name
            item.set(
                "GetItemName",
                lua.create_function(|lua, this: mlua::Table| {
                    let id: i32 = this.get("itemID")?;
                    Ok(Value::String(lua.create_string(&format!("Item {}", id))?))
                })?,
            )?;

            // GetItemLink
            item.set(
                "GetItemLink",
                lua.create_function(|lua, this: mlua::Table| {
                    let id: i32 = this.get("itemID")?;
                    let link = format!("|cff1eff00|Hitem:{}::::::::60:::::|h[Item {}]|h|r", id, id);
                    Ok(Value::String(lua.create_string(&link)?))
                })?,
            )?;

            // GetItemIcon
            item.set(
                "GetItemIcon",
                lua.create_function(|_, _this: mlua::Table| {
                    Ok(134400i32) // INV_Misc_QuestionMark
                })?,
            )?;

            // GetItemQuality
            item.set(
                "GetItemQuality",
                lua.create_function(|_, _this: mlua::Table| {
                    Ok(1i32) // Common quality
                })?,
            )?;

            // IsItemDataCached - always true in simulation
            item.set(
                "IsItemDataCached",
                lua.create_function(|_, _this: mlua::Table| Ok(true))?,
            )?;

            Ok(item)
        })?,
    )?;
    item_class.set(
        "CreateFromItemLink",
        lua.create_function(|lua, (_self, item_link): (Value, String)| {
            // Extract item ID from link if possible, otherwise use 0
            let item_id = item_link
                .split("item:")
                .nth(1)
                .and_then(|s| s.split(':').next())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            // Reuse CreateFromItemID logic
            let item = lua.create_table()?;
            item.set("itemID", item_id)?;

            item.set(
                "ContinueOnItemLoad",
                lua.create_function(|_, (this, callback): (mlua::Table, mlua::Function)| {
                    callback.call::<()>(())?;
                    let _ = this;
                    Ok(())
                })?,
            )?;

            item.set(
                "GetItemID",
                lua.create_function(|_, this: mlua::Table| {
                    this.get::<i32>("itemID")
                })?,
            )?;

            item.set(
                "GetItemName",
                lua.create_function(|lua, this: mlua::Table| {
                    let id: i32 = this.get("itemID")?;
                    Ok(Value::String(lua.create_string(&format!("Item {}", id))?))
                })?,
            )?;

            item.set(
                "GetItemLink",
                lua.create_function(|lua, this: mlua::Table| {
                    let id: i32 = this.get("itemID")?;
                    let link = format!("|cff1eff00|Hitem:{}::::::::60:::::|h[Item {}]|h|r", id, id);
                    Ok(Value::String(lua.create_string(&link)?))
                })?,
            )?;

            item.set(
                "GetItemIcon",
                lua.create_function(|_, _this: mlua::Table| Ok(134400i32))?,
            )?;

            item.set(
                "GetItemQuality",
                lua.create_function(|_, _this: mlua::Table| Ok(1i32))?,
            )?;

            item.set(
                "IsItemDataCached",
                lua.create_function(|_, _this: mlua::Table| Ok(true))?,
            )?;

            Ok(item)
        })?,
    )?;
    globals.set("Item", item_class)?;

    // Create standard WoW font objects
    // These are font objects that many addons expect to exist
    create_standard_font_objects(lua)?;

    Ok(())
}

/// Create standard WoW font objects that addons expect to exist
fn create_standard_font_objects(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // Helper to create a font object with specific properties
    let create_font_obj = |name: &str, height: f64, flags: &str, r: f64, g: f64, b: f64| -> Result<mlua::Table> {
        let font = lua.create_table()?;

        // Internal state
        font.set("__fontPath", "Fonts\\FRIZQT__.TTF")?;
        font.set("__fontHeight", height)?;
        font.set("__fontFlags", flags)?;
        font.set("__textColorR", r)?;
        font.set("__textColorG", g)?;
        font.set("__textColorB", b)?;
        font.set("__textColorA", 1.0)?;
        font.set("__shadowColorR", 0.0)?;
        font.set("__shadowColorG", 0.0)?;
        font.set("__shadowColorB", 0.0)?;
        font.set("__shadowColorA", 0.0)?;
        font.set("__shadowOffsetX", 0.0)?;
        font.set("__shadowOffsetY", 0.0)?;
        font.set("__justifyH", "CENTER")?;
        font.set("__justifyV", "MIDDLE")?;
        font.set("__name", name)?;

        // Add methods
        font.set(
            "SetFont",
            lua.create_function(|_, (this, path, height, flags): (mlua::Table, String, f64, Option<String>)| {
                this.set("__fontPath", path)?;
                this.set("__fontHeight", height)?;
                this.set("__fontFlags", flags.unwrap_or_default())?;
                Ok(())
            })?,
        )?;
        font.set(
            "GetFont",
            lua.create_function(|_, this: mlua::Table| {
                let path: String = this.get("__fontPath")?;
                let height: f64 = this.get("__fontHeight")?;
                let flags: String = this.get("__fontFlags")?;
                Ok((path, height, flags))
            })?,
        )?;
        font.set(
            "SetTextColor",
            lua.create_function(|_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                this.set("__textColorR", r)?;
                this.set("__textColorG", g)?;
                this.set("__textColorB", b)?;
                this.set("__textColorA", a.unwrap_or(1.0))?;
                Ok(())
            })?,
        )?;
        font.set(
            "GetTextColor",
            lua.create_function(|_, this: mlua::Table| {
                let r: f64 = this.get("__textColorR")?;
                let g: f64 = this.get("__textColorG")?;
                let b: f64 = this.get("__textColorB")?;
                let a: f64 = this.get("__textColorA")?;
                Ok((r, g, b, a))
            })?,
        )?;
        font.set(
            "SetShadowColor",
            lua.create_function(|_, (this, r, g, b, a): (mlua::Table, f64, f64, f64, Option<f64>)| {
                this.set("__shadowColorR", r)?;
                this.set("__shadowColorG", g)?;
                this.set("__shadowColorB", b)?;
                this.set("__shadowColorA", a.unwrap_or(1.0))?;
                Ok(())
            })?,
        )?;
        font.set(
            "GetShadowColor",
            lua.create_function(|_, this: mlua::Table| {
                let r: f64 = this.get("__shadowColorR")?;
                let g: f64 = this.get("__shadowColorG")?;
                let b: f64 = this.get("__shadowColorB")?;
                let a: f64 = this.get("__shadowColorA")?;
                Ok((r, g, b, a))
            })?,
        )?;
        font.set(
            "SetShadowOffset",
            lua.create_function(|_, (this, x, y): (mlua::Table, f64, f64)| {
                this.set("__shadowOffsetX", x)?;
                this.set("__shadowOffsetY", y)?;
                Ok(())
            })?,
        )?;
        font.set(
            "GetShadowOffset",
            lua.create_function(|_, this: mlua::Table| {
                let x: f64 = this.get("__shadowOffsetX")?;
                let y: f64 = this.get("__shadowOffsetY")?;
                Ok((x, y))
            })?,
        )?;
        font.set(
            "SetJustifyH",
            lua.create_function(|_, (this, justify): (mlua::Table, String)| {
                this.set("__justifyH", justify)?;
                Ok(())
            })?,
        )?;
        font.set(
            "GetJustifyH",
            lua.create_function(|_, this: mlua::Table| {
                let justify: String = this.get("__justifyH")?;
                Ok(justify)
            })?,
        )?;
        font.set(
            "SetJustifyV",
            lua.create_function(|_, (this, justify): (mlua::Table, String)| {
                this.set("__justifyV", justify)?;
                Ok(())
            })?,
        )?;
        font.set(
            "GetJustifyV",
            lua.create_function(|_, this: mlua::Table| {
                let justify: String = this.get("__justifyV")?;
                Ok(justify)
            })?,
        )?;
        font.set(
            "SetSpacing",
            lua.create_function(|_, (this, spacing): (mlua::Table, f64)| {
                this.set("__spacing", spacing)?;
                Ok(())
            })?,
        )?;
        font.set(
            "GetSpacing",
            lua.create_function(|_, this: mlua::Table| {
                let spacing: f64 = this.get("__spacing").unwrap_or(0.0);
                Ok(spacing)
            })?,
        )?;
        font.set(
            "CopyFontObject",
            lua.create_function(|_, (this, src): (mlua::Table, mlua::Table)| {
                if let Ok(v) = src.get::<String>("__fontPath") { this.set("__fontPath", v)?; }
                if let Ok(v) = src.get::<f64>("__fontHeight") { this.set("__fontHeight", v)?; }
                if let Ok(v) = src.get::<String>("__fontFlags") { this.set("__fontFlags", v)?; }
                for key in &["__textColorR", "__textColorG", "__textColorB", "__textColorA",
                             "__shadowColorR", "__shadowColorG", "__shadowColorB", "__shadowColorA",
                             "__shadowOffsetX", "__shadowOffsetY"] {
                    if let Ok(v) = src.get::<f64>(*key) { this.set(*key, v)?; }
                }
                if let Ok(v) = src.get::<String>("__justifyH") { this.set("__justifyH", v)?; }
                if let Ok(v) = src.get::<String>("__justifyV") { this.set("__justifyV", v)?; }
                Ok(())
            })?,
        )?;
        font.set(
            "GetName",
            lua.create_function(|_, this: mlua::Table| {
                let name: Option<String> = this.get("__name").ok();
                Ok(name)
            })?,
        )?;
        // GetFontObjectForAlphabet(alphabet) - returns self for font localization
        font.set(
            "GetFontObjectForAlphabet",
            lua.create_function(|_, this: mlua::Table| Ok(this))?,
        )?;

        globals.set(name, font.clone())?;
        Ok(font)
    };

    // Standard font objects - white text
    create_font_obj("GameFontNormal", 12.0, "", 1.0, 0.82, 0.0)?;  // Gold text
    create_font_obj("GameFontNormalSmall", 10.0, "", 1.0, 0.82, 0.0)?;
    create_font_obj("GameFontNormalLarge", 16.0, "", 1.0, 0.82, 0.0)?;
    create_font_obj("GameFontNormalHuge", 20.0, "", 1.0, 0.82, 0.0)?;

    // Highlighted (white) fonts
    create_font_obj("GameFontHighlight", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightSmall", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightSmallOutline", 10.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightLarge", 16.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightHuge", 20.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontHighlightOutline", 12.0, "OUTLINE", 1.0, 1.0, 1.0)?;

    // Disabled fonts (gray)
    create_font_obj("GameFontDisable", 12.0, "", 0.5, 0.5, 0.5)?;
    create_font_obj("GameFontDisableSmall", 10.0, "", 0.5, 0.5, 0.5)?;
    create_font_obj("GameFontDisableLarge", 16.0, "", 0.5, 0.5, 0.5)?;

    // Red/error fonts
    create_font_obj("GameFontRed", 12.0, "", 1.0, 0.1, 0.1)?;
    create_font_obj("GameFontRedSmall", 10.0, "", 1.0, 0.1, 0.1)?;
    create_font_obj("GameFontRedLarge", 16.0, "", 1.0, 0.1, 0.1)?;

    // Green fonts
    create_font_obj("GameFontGreen", 12.0, "", 0.1, 1.0, 0.1)?;
    create_font_obj("GameFontGreenSmall", 10.0, "", 0.1, 1.0, 0.1)?;
    create_font_obj("GameFontGreenLarge", 16.0, "", 0.1, 1.0, 0.1)?;

    // White fonts
    create_font_obj("GameFontWhite", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontWhiteSmall", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameFontWhiteTiny", 9.0, "", 1.0, 1.0, 1.0)?;

    // Black fonts
    create_font_obj("GameFontBlack", 12.0, "", 0.0, 0.0, 0.0)?;
    create_font_obj("GameFontBlackSmall", 10.0, "", 0.0, 0.0, 0.0)?;

    // Number fonts
    create_font_obj("NumberFontNormal", 14.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("NumberFontNormalSmall", 12.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("NumberFontNormalLarge", 16.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("NumberFontNormalHuge", 24.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("NumberFontNormalRightRed", 14.0, "OUTLINE", 1.0, 0.1, 0.1)?;
    create_font_obj("NumberFontNormalRightYellow", 14.0, "OUTLINE", 1.0, 1.0, 0.0)?;

    // Chat fonts
    create_font_obj("ChatFontNormal", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("ChatFontSmall", 12.0, "", 1.0, 1.0, 1.0)?;

    // System fonts
    create_font_obj("SystemFont_Small", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Med1", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Med2", 13.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Med3", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Large", 16.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Huge1", 20.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Huge2", 24.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Outline", 12.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_OutlineThick_Huge2", 24.0, "OUTLINE, THICKOUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_OutlineThick_Huge4", 32.0, "OUTLINE, THICKOUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_OutlineThick_WTF", 64.0, "OUTLINE, THICKOUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Small", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Med1", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Med2", 13.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Med3", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Large", 16.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Large_Outline", 16.0, "OUTLINE", 1.0, 1.0, 1.0)?;
    create_font_obj("SystemFont_Shadow_Huge1", 20.0, "", 1.0, 1.0, 1.0)?;

    // Tooltip fonts
    create_font_obj("GameTooltipHeader", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameTooltipText", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("GameTooltipTextSmall", 10.0, "", 1.0, 1.0, 1.0)?;

    // Subzone fonts
    create_font_obj("SubZoneTextFont", 26.0, "OUTLINE", 1.0, 0.82, 0.0)?;
    create_font_obj("PVPInfoTextFont", 20.0, "OUTLINE", 1.0, 0.1, 0.1)?;

    // Misc fonts
    create_font_obj("FriendsFont_Normal", 12.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("FriendsFont_Small", 10.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("FriendsFont_Large", 14.0, "", 1.0, 1.0, 1.0)?;
    create_font_obj("FriendsFont_UserText", 11.0, "", 1.0, 1.0, 1.0)?;

    // C_HouseEditor namespace - player housing editor
    let c_house_editor = lua.create_table()?;
    c_house_editor.set(
        "IsHouseEditorActive",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_house_editor.set(
        "GetActiveHouseEditorMode",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_house_editor.set(
        "ActivateHouseEditorMode",
        lua.create_function(|_, _mode: i32| Ok(()))?,
    )?;
    c_house_editor.set(
        "GetHouseEditorModeAvailability",
        lua.create_function(|_, _mode: i32| Ok(false))?,
    )?;
    c_house_editor.set(
        "IsHouseEditorModeActive",
        lua.create_function(|_, _mode: i32| Ok(false))?,
    )?;
    globals.set("C_HouseEditor", c_house_editor)?;

    // C_QuestOffer namespace - quest offer/reward info
    let c_quest_offer = lua.create_table()?;
    c_quest_offer.set(
        "GetQuestRewardCurrencyInfo",
        lua.create_function(|lua, (_quest_id, _index): (i32, i32)| {
            // Returns: name, texture, amount, quality
            let info = lua.create_table()?;
            Ok(info)
        })?,
    )?;
    c_quest_offer.set(
        "GetNumQuestRewardCurrencies",
        lua.create_function(|_, _quest_id: i32| Ok(0i32))?,
    )?;
    globals.set("C_QuestOffer", c_quest_offer)?;

    // C_ArtifactUI namespace - artifact weapon interface
    let c_artifact_ui = lua.create_table()?;
    c_artifact_ui.set(
        "GetArtifactItemID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_artifact_ui.set(
        "GetArtifactTier",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_artifact_ui.set(
        "IsAtForge",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_artifact_ui.set(
        "GetEquippedArtifactInfo",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    globals.set("C_ArtifactUI", c_artifact_ui)?;

    // C_SuperTrack namespace - map pin super tracking
    let c_super_track = lua.create_table()?;
    c_super_track.set(
        "GetSuperTrackedMapPin",
        lua.create_function(|_, ()| Ok((Value::Nil, Value::Nil)))?,
    )?;
    c_super_track.set(
        "SetSuperTrackedMapPin",
        lua.create_function(|_, (_map_id, _x, _y): (i32, f32, f32)| Ok(()))?,
    )?;
    c_super_track.set(
        "ClearSuperTrackedMapPin",
        lua.create_function(|_, ()| Ok(()))?,
    )?;
    c_super_track.set(
        "GetSuperTrackedQuestID",
        lua.create_function(|_, ()| Ok(0i32))?,
    )?;
    c_super_track.set(
        "SetSuperTrackedQuestID",
        lua.create_function(|_, _quest_id: i32| Ok(()))?,
    )?;
    c_super_track.set(
        "IsSuperTrackingQuest",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_super_track.set(
        "IsSuperTrackingMapPin",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_SuperTrack", c_super_track)?;

    // C_PlayerInteractionManager namespace - NPC interaction management
    let c_player_interaction_manager = lua.create_table()?;
    c_player_interaction_manager.set(
        "IsInteractingWithNpcOfType",
        lua.create_function(|_, _npc_type: i32| Ok(false))?,
    )?;
    c_player_interaction_manager.set(
        "ClearInteraction",
        lua.create_function(|_, _interaction_type: Option<i32>| Ok(()))?,
    )?;
    c_player_interaction_manager.set(
        "GetCurrentInteraction",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    globals.set("C_PlayerInteractionManager", c_player_interaction_manager)?;

    // C_HousingDecor namespace - housing decoration management
    let c_housing_decor = lua.create_table()?;
    c_housing_decor.set(
        "GetHoveredDecorInfo",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_housing_decor.set(
        "IsHoveringDecor",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_housing_decor.set(
        "GetDecorInfo",
        lua.create_function(|_, _decor_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_HousingDecor", c_housing_decor)?;

    // C_HousingBasicMode namespace - housing basic edit mode
    let c_housing_basic_mode = lua.create_table()?;
    c_housing_basic_mode.set(
        "IsDecorSelected",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_housing_basic_mode.set(
        "GetSelectedDecorInfo",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    globals.set("C_HousingBasicMode", c_housing_basic_mode)?;

    // C_TransmogSets namespace - transmog set collection
    let c_transmog_sets = lua.create_table()?;
    c_transmog_sets.set(
        "GetBaseSetID",
        lua.create_function(|_, _set_id: i32| Ok(0i32))?,
    )?;
    c_transmog_sets.set(
        "GetVariantSets",
        lua.create_function(|lua, _set_id: i32| lua.create_table())?,
    )?;
    c_transmog_sets.set(
        "GetSetInfo",
        lua.create_function(|lua, _set_id: i32| {
            let info = lua.create_table()?;
            info.set("setID", 0)?;
            info.set("name", "")?;
            info.set("description", "")?;
            info.set("label", "")?;
            info.set("expansionID", 0)?;
            info.set("collected", false)?;
            Ok(info)
        })?,
    )?;
    c_transmog_sets.set(
        "GetSetPrimaryAppearances",
        lua.create_function(|lua, _set_id: i32| lua.create_table())?,
    )?;
    c_transmog_sets.set(
        "GetAllSets",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_transmog_sets.set(
        "GetUsableSets",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_transmog_sets.set(
        "IsBaseSetCollected",
        lua.create_function(|_, _set_id: i32| Ok(false))?,
    )?;
    c_transmog_sets.set(
        "GetSourcesForSlot",
        lua.create_function(|lua, (_set_id, _slot): (i32, i32)| lua.create_table())?,
    )?;
    globals.set("C_TransmogSets", c_transmog_sets)?;

    Ok(())
}

/// Calculate frame width from anchors or explicit size (recursive).
/// WoW behavior: anchors defining opposite edges override explicit size.
fn calculate_frame_width(widgets: &crate::widget::WidgetRegistry, id: u64) -> f32 {
    if let Some(frame) = widgets.get(id) {
        // Try to calculate from left+right anchors first (they override explicit size)
        use crate::widget::AnchorPoint::*;
        let left_anchors = [TopLeft, BottomLeft, Left];
        let right_anchors = [TopRight, BottomRight, Right];
        let left = frame.anchors.iter().find(|a| left_anchors.contains(&a.point));
        let right = frame.anchors.iter().find(|a| right_anchors.contains(&a.point));
        if let (Some(left_anchor), Some(right_anchor)) = (left, right) {
            // Both must anchor to same relative frame
            if left_anchor.relative_to_id == right_anchor.relative_to_id {
                let parent_id = left_anchor.relative_to_id.map(|id| id as u64).or(frame.parent_id);
                if let Some(pid) = parent_id {
                    // Recursively calculate parent width
                    let parent_width = calculate_frame_width(widgets, pid);
                    if parent_width > 0.0 {
                        return (parent_width - left_anchor.x_offset + right_anchor.x_offset).max(0.0);
                    }
                }
            }
        }
        // Fall back to explicit width
        frame.width
    } else {
        0.0
    }
}

/// Calculate frame height from anchors or explicit size (recursive).
/// WoW behavior: anchors defining opposite edges override explicit size.
fn calculate_frame_height(widgets: &crate::widget::WidgetRegistry, id: u64) -> f32 {
    if let Some(frame) = widgets.get(id) {
        // Try to calculate from top+bottom anchors first (they override explicit size)
        use crate::widget::AnchorPoint::*;
        let top_anchors = [TopLeft, TopRight, Top];
        let bottom_anchors = [BottomLeft, BottomRight, Bottom];
        let top = frame.anchors.iter().find(|a| top_anchors.contains(&a.point));
        let bottom = frame.anchors.iter().find(|a| bottom_anchors.contains(&a.point));
        if let (Some(top_anchor), Some(bottom_anchor)) = (top, bottom) {
            // Both must anchor to same relative frame
            if top_anchor.relative_to_id == bottom_anchor.relative_to_id {
                let parent_id = top_anchor.relative_to_id.map(|id| id as u64).or(frame.parent_id);
                if let Some(pid) = parent_id {
                    // Recursively calculate parent height
                    let parent_height = calculate_frame_height(widgets, pid);
                    if parent_height > 0.0 {
                        return (parent_height + top_anchor.y_offset - bottom_anchor.y_offset).max(0.0);
                    }
                }
            }
        }
        // Fall back to explicit height
        frame.height
    } else {
        0.0
    }
}

/// Userdata handle to a frame (passed to Lua).
#[derive(Clone)]
pub struct FrameHandle {
    pub id: u64,
    pub state: Rc<RefCell<SimState>>,
}

impl UserData for FrameHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Support custom field access via __index/__newindex
        // This allows addons to do: frame.customField = value

        methods.add_meta_function(MetaMethod::Index, |lua: &Lua, (ud, key): (mlua::AnyUserData, Value)| {
            // Try to get from the custom fields table
            let handle = ud.borrow::<FrameHandle>()?;
            let frame_id = handle.id;
            let state_rc = Rc::clone(&handle.state);
            drop(handle); // Release borrow before accessing state

            // Handle numeric indices - returns n-th child frame (1-indexed)
            if let Value::Integer(idx) = key {
                if idx > 0 {
                    let state = state_rc.borrow();
                    if let Some(frame) = state.widgets.get(frame_id) {
                        if let Some(&child_id) = frame.children.get((idx - 1) as usize) {
                            drop(state);
                            let child_handle = FrameHandle {
                                id: child_id,
                                state: Rc::clone(&state_rc),
                            };
                            return lua.create_userdata(child_handle).map(Value::UserData);
                        }
                    }
                }
                return Ok(Value::Nil);
            }

            // Convert key to string for named access
            let key = match &key {
                Value::String(s) => s.to_string_lossy().to_string(),
                _ => return Ok(Value::Nil),
            };

            // First check children_keys (for template-created children like "Text")
            {
                let state = state_rc.borrow();
                if let Some(frame) = state.widgets.get(frame_id) {
                    if let Some(&child_id) = frame.children_keys.get(&key) {
                        // Create userdata for the child frame
                        drop(state); // Release borrow before creating userdata
                        let child_handle = FrameHandle {
                            id: child_id,
                            state: Rc::clone(&state_rc),
                        };
                        return lua.create_userdata(child_handle).map(Value::UserData);
                    }
                }
            }

            let fields_table: Option<mlua::Table> = lua.globals().get("__frame_fields").ok();

            if let Some(table) = fields_table {
                let frame_fields: Option<mlua::Table> = table.get::<mlua::Table>(frame_id).ok();
                if let Some(fields) = frame_fields {
                    let value: Value = fields.get::<Value>(key.as_str()).unwrap_or(Value::Nil);
                    if value != Value::Nil {
                        return Ok(value);
                    }
                }
            }

            // Special handling for Cooldown:Clear() - only for Cooldown frame type
            // This avoids conflicts with addons that use frame.Clear as a field
            if key == "Clear" {
                let is_cooldown = {
                    let state = state_rc.borrow();
                    state.widgets.get(frame_id)
                        .map(|f| f.widget_type == WidgetType::Cooldown)
                        .unwrap_or(false)
                };
                if is_cooldown {
                    return Ok(Value::Function(lua.create_function(|_, _: mlua::MultiValue| Ok(()))?));
                }
            }

            // Fallback methods that might conflict with custom properties
            // These are only returned if no custom field was found above
            if key == "Lower" || key == "Raise" {
                // Lower() and Raise() adjust frame stacking order (no-op in simulator)
                return Ok(Value::Function(lua.create_function(|_, _: mlua::MultiValue| Ok(()))?));
            }

            // Not found in custom fields, return nil (methods are handled separately by mlua)
            Ok(Value::Nil)
        });

        methods.add_meta_function(MetaMethod::NewIndex, |lua: &Lua, (ud, key, value): (mlua::AnyUserData, String, Value)| {
            let handle = ud.borrow::<FrameHandle>()?;
            let frame_id: u64 = handle.id;
            let state_rc = Rc::clone(&handle.state);
            drop(handle);

            // If assigning a FrameHandle value, update children_keys in the Rust widget registry
            // This syncs parentKey relationships from Lua space to Rust
            if let Value::UserData(child_ud) = &value {
                if let Ok(child_handle) = child_ud.borrow::<FrameHandle>() {
                    let child_id = child_handle.id;
                    drop(child_handle);
                    let mut state = state_rc.borrow_mut();
                    if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                        parent_frame.children_keys.insert(key.clone(), child_id);
                    }
                }
            }

            // Get or create the fields table
            let fields_table: mlua::Table = lua.globals().get::<mlua::Table>("__frame_fields").unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                lua.globals().set("__frame_fields", t.clone()).unwrap();
                t
            });

            // Get or create the frame's field table
            let frame_fields: mlua::Table = fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                fields_table.set(frame_id, t.clone()).unwrap();
                t
            });

            frame_fields.set(key, value)?;
            Ok(())
        });

        // __len metamethod - returns number of children (for array-like iteration)
        methods.add_meta_function(MetaMethod::Len, |_lua: &Lua, ud: mlua::AnyUserData| {
            let handle = ud.borrow::<FrameHandle>()?;
            let state = handle.state.borrow();
            let len = state.widgets.get(handle.id)
                .map(|f| f.children.len())
                .unwrap_or(0);
            Ok(len)
        });

        // __eq metamethod - compare FrameHandles by id
        // This is needed because __index creates new userdata objects when accessing children_keys,
        // so ParentFrame.child and ChildFrame would be different userdata objects with the same id
        methods.add_meta_function(MetaMethod::Eq, |_lua: &Lua, (ud1, ud2): (mlua::AnyUserData, mlua::AnyUserData)| {
            let handle1 = ud1.borrow::<FrameHandle>()?;
            let handle2 = ud2.borrow::<FrameHandle>()?;
            Ok(handle1.id == handle2.id)
        });

        // GetName()
        methods.add_method("GetName", |_, this, ()| {
            let state = this.state.borrow();
            let name = state
                .widgets
                .get(this.id)
                .and_then(|f| f.name.clone())
                .unwrap_or_default();
            Ok(name)
        });

        // GetWidth() - returns explicit width or calculates from anchors
        methods.add_method("GetWidth", |_, this, ()| {
            let state = this.state.borrow();
            Ok(calculate_frame_width(&state.widgets, this.id))
        });

        // GetHeight() - returns explicit height or calculates from anchors
        methods.add_method("GetHeight", |_, this, ()| {
            let state = this.state.borrow();
            Ok(calculate_frame_height(&state.widgets, this.id))
        });

        // GetSize() -> width, height (with anchor calculation)
        methods.add_method("GetSize", |_, this, ()| {
            let state = this.state.borrow();
            let width = calculate_frame_width(&state.widgets, this.id);
            let height = calculate_frame_height(&state.widgets, this.id);
            Ok((width, height))
        });

        // GetZoom() - for Minimap frame
        methods.add_method("GetZoom", |_, _this, ()| Ok(0));

        // SetZoom(zoom) - for Minimap frame
        methods.add_method("SetZoom", |_, _this, _zoom: i32| Ok(()));

        // GetCanvas() - for WorldMapFrame (returns self as the canvas)
        methods.add_method("GetCanvas", |lua, this, ()| {
            let handle = FrameHandle {
                id: this.id,
                state: Rc::clone(&this.state),
            };
            lua.create_userdata(handle)
        });

        // SetTextCopyable(copyable) - for EditBox/ScrollingMessageFrame
        methods.add_method("SetTextCopyable", |_, _this, _copyable: bool| Ok(()));

        // SetInsertMode(mode) - for ScrollingMessageFrame
        methods.add_method("SetInsertMode", |_, _this, _mode: String| Ok(()));

        // SetFading(fading) - for ScrollingMessageFrame
        methods.add_method("SetFading", |_, _this, _fading: bool| Ok(()));

        // SetFadeDuration(duration) - for ScrollingMessageFrame
        methods.add_method("SetFadeDuration", |_, _this, _duration: f32| Ok(()));

        // SetTimeVisible(time) - for ScrollingMessageFrame
        methods.add_method("SetTimeVisible", |_, _this, _time: f32| Ok(()));

        // AddQueuedAlertFrameSubSystem(system) - for AlertFrame
        // Returns an alert subsystem object with methods like SetCanShowMoreConditionFunc
        methods.add_method("AddQueuedAlertFrameSubSystem", |lua, _this, _args: mlua::MultiValue| {
            let subsystem = lua.create_table()?;
            subsystem.set("SetCanShowMoreConditionFunc", lua.create_function(|_, (_self, _func): (Value, Value)| Ok(()))?)?;
            subsystem.set("AddAlert", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
            subsystem.set("RemoveAlert", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
            subsystem.set("ClearAllAlerts", lua.create_function(|_, _self: Value| Ok(()))?)?;
            Ok(Value::Table(subsystem))
        });

        // AddDataProvider(provider) - for WorldMapFrame (used by HereBeDragons)
        methods.add_method("AddDataProvider", |_, _this, _provider: mlua::Value| Ok(()));

        // RemoveDataProvider(provider) - for WorldMapFrame
        methods.add_method("RemoveDataProvider", |_, _this, _provider: mlua::Value| Ok(()));

        // SetSize(width, height)
        methods.add_method("SetSize", |_, this, (width, height): (f32, f32)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.set_size(width, height);
            }
            Ok(())
        });

        // SetWidth(width)
        methods.add_method("SetWidth", |_, this, width: f32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.width = width;
            }
            Ok(())
        });

        // SetHeight(height)
        methods.add_method("SetHeight", |_, this, height: f32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.height = height;
            }
            Ok(())
        });

        // SetPoint(point, relativeTo, relativePoint, xOfs, yOfs)
        methods.add_method("SetPoint", |_, this, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let point_str = args
                .first()
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "CENTER".to_string());

            let point =
                crate::widget::AnchorPoint::from_str(&point_str).unwrap_or_default();

            // Helper to extract numeric value from Value (handles both Number and Integer)
            fn get_number(v: &Value) -> Option<f32> {
                match v {
                    Value::Number(n) => Some(*n as f32),
                    Value::Integer(n) => Some(*n as f32),
                    _ => None,
                }
            }

            // Helper to extract frame ID from Value
            let get_frame_id = |v: &Value| -> Option<usize> {
                if let Value::UserData(ud) = v {
                    if let Ok(frame_handle) = ud.borrow::<FrameHandle>() {
                        return Some(frame_handle.id as usize);
                    }
                }
                None
            };

            // Parse the variable arguments
            let (relative_to, relative_point, x_ofs, y_ofs) = match args.len() {
                1 => (None, point, 0.0, 0.0),
                2 | 3 => {
                    // SetPoint("CENTER", x, y) or SetPoint("CENTER", relativeTo)
                    let x = args.get(1).and_then(get_number);
                    let y = args.get(2).and_then(get_number);
                    if let (Some(x), Some(y)) = (x, y) {
                        (None, point, x, y)
                    } else {
                        // Could be SetPoint("CENTER", relativeTo) - get frame ID
                        let rel_to = args.get(1).and_then(get_frame_id);
                        (rel_to, point, 0.0, 0.0)
                    }
                }
                _ => {
                    // Full form: SetPoint(point, relativeTo, relativePoint, x, y)
                    let rel_to = args.get(1).and_then(get_frame_id);

                    let rel_point_str = args.get(2).and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    });
                    let rel_point = rel_point_str
                        .and_then(|s| crate::widget::AnchorPoint::from_str(&s))
                        .unwrap_or(point);
                    let x = args.get(3).and_then(get_number).unwrap_or(0.0);
                    let y = args.get(4).and_then(get_number).unwrap_or(0.0);
                    (rel_to, rel_point, x, y)
                }
            };

            let mut state = this.state.borrow_mut();

            // Check for anchor cycles before setting point
            if let Some(rel_id) = relative_to {
                if state.widgets.would_create_anchor_cycle(this.id, rel_id as u64) {
                    // Silently ignore the anchor to prevent cycles (matches WoW behavior)
                    // WoW logs an error but doesn't crash
                    return Ok(());
                }
            }

            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.set_point(point, relative_to, relative_point, x_ofs, y_ofs);
            }
            Ok(())
        });

        // ClearAllPoints()
        methods.add_method("ClearAllPoints", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.clear_all_points();
            }
            Ok(())
        });

        // AdjustPointsOffset(x, y) - Adjusts the offsets of all anchor points
        methods.add_method(
            "AdjustPointsOffset",
            |_, this, (x_offset, y_offset): (f32, f32)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    for anchor in &mut frame.anchors {
                        anchor.x_offset += x_offset;
                        anchor.y_offset += y_offset;
                    }
                }
                Ok(())
            },
        );

        // Show()
        methods.add_method("Show", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = true;
            }
            Ok(())
        });

        // Hide()
        methods.add_method("Hide", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = false;
            }
            Ok(())
        });

        // IsVisible() / IsShown()
        methods.add_method("IsVisible", |_, this, ()| {
            let state = this.state.borrow();
            let visible = state.widgets.get(this.id).map(|f| f.visible).unwrap_or(false);
            Ok(visible)
        });

        methods.add_method("IsShown", |_, this, ()| {
            let state = this.state.borrow();
            let visible = state.widgets.get(this.id).map(|f| f.visible).unwrap_or(false);
            Ok(visible)
        });

        // RegisterEvent(event)
        methods.add_method("RegisterEvent", |_, this, event: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.register_event(&event);
            }
            Ok(())
        });

        // RegisterUnitEvent(event, unit1, unit2, ...) - register for unit-specific events
        // Some addons pass a callback function as the last argument (non-standard)
        methods.add_method(
            "RegisterUnitEvent",
            |_, this, (event, _args): (String, mlua::Variadic<Value>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.register_event(&event);
                }
                Ok(())
            },
        );

        // UnregisterEvent(event)
        methods.add_method("UnregisterEvent", |_, this, event: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.unregister_event(&event);
            }
            Ok(())
        });

        // UnregisterAllEvents()
        methods.add_method("UnregisterAllEvents", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.registered_events.clear();
            }
            Ok(())
        });

        // IsEventRegistered(event) -> bool
        methods.add_method("IsEventRegistered", |_, this, event: String| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                return Ok(frame.registered_events.contains(&event));
            }
            Ok(false)
        });

        // SetPropagateKeyboardInput(propagate) - keyboard input propagation
        methods.add_method("SetPropagateKeyboardInput", |_, _this, _propagate: bool| {
            // In the simulator, this is a no-op
            Ok(())
        });

        // GetPropagateKeyboardInput() -> bool
        methods.add_method("GetPropagateKeyboardInput", |_, _this, ()| {
            // Default to false in the simulator
            Ok(false)
        });

        // UseRaidStylePartyFrames() -> bool (for EditModeManagerFrame)
        methods.add_method("UseRaidStylePartyFrames", |_, _this, ()| {
            // Default to false (using party frames, not raid frames)
            Ok(false)
        });

        // SetScript(handler, func)
        methods.add_method("SetScript", |lua, this, (handler, func): (String, Value)| {
            let handler_type = crate::event::ScriptHandler::from_str(&handler);

            if let (Some(h), Value::Function(f)) = (handler_type, func) {
                // Store function in a global Lua table for later retrieval
                let scripts_table: mlua::Table =
                    lua.globals().get("__scripts").unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__scripts", t.clone()).unwrap();
                        t
                    });

                let frame_key = format!("{}_{}", this.id, handler);
                scripts_table.set(frame_key.as_str(), f)?;

                // Mark that this widget has this handler
                let mut state = this.state.borrow_mut();
                state.scripts.set(this.id, h, 1); // Just mark it exists
            }
            Ok(())
        });

        // GetScript(handler)
        methods.add_method("GetScript", |lua, this, handler: String| {
            let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();

            if let Some(table) = scripts_table {
                let frame_key = format!("{}_{}", this.id, handler);
                let func: Value = table.get(frame_key.as_str()).unwrap_or(Value::Nil);
                Ok(func)
            } else {
                Ok(Value::Nil)
            }
        });

        // SetOnClickHandler(func) - WoW 10.0+ method for setting OnClick handler (used by Edit Mode)
        methods.add_method("SetOnClickHandler", |lua, this, func: Value| {
            if let Value::Function(f) = func {
                let scripts_table: mlua::Table =
                    lua.globals().get("__scripts").unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__scripts", t.clone()).unwrap();
                        t
                    });

                let frame_key = format!("{}_OnClick", this.id);
                scripts_table.set(frame_key.as_str(), f)?;

                let mut state = this.state.borrow_mut();
                state.scripts.set(this.id, crate::event::ScriptHandler::OnClick, 1);
            }
            Ok(())
        });

        // HookScript(handler, func) - Hook into existing script handler
        methods.add_method("HookScript", |lua, this, (handler, func): (String, Value)| {
            if let Value::Function(f) = func {
                // Store hook in a global table
                let hooks_table: mlua::Table =
                    lua.globals().get("__script_hooks").unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__script_hooks", t.clone()).unwrap();
                        t
                    });

                let frame_key = format!("{}_{}", this.id, handler);
                // Get existing hooks array or create new
                let hooks_array: mlua::Table = hooks_table
                    .get::<mlua::Table>(frame_key.as_str())
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        hooks_table.set(frame_key.as_str(), t.clone()).unwrap();
                        t
                    });
                // Append the new hook
                let len = hooks_array.len().unwrap_or(0);
                hooks_array.set(len + 1, f)?;
            }
            Ok(())
        });

        // WrapScript(frame, scriptType, preBody, postBody) - Wraps a secure script handler
        methods.add_method(
            "WrapScript",
            |_, _this, (_target, _script, _pre_body): (mlua::Value, String, String)| {
                // Stub for secure script wrapping - not implemented in simulator
                Ok(())
            },
        );

        // UnwrapScript(frame, scriptType) - Removes script wrapping
        methods.add_method("UnwrapScript", |_, _this, (_target, _script): (mlua::Value, String)| {
            Ok(())
        });

        // HasScript(scriptType) - Check if frame supports a script handler
        methods.add_method("HasScript", |_, _this, script_type: String| {
            // Most frames support common script types
            let common_scripts = [
                "OnClick", "OnEnter", "OnLeave", "OnShow", "OnHide",
                "OnMouseDown", "OnMouseUp", "OnMouseWheel", "OnDragStart",
                "OnDragStop", "OnUpdate", "OnEvent", "OnLoad", "OnSizeChanged",
                "OnAttributeChanged", "OnEnable", "OnDisable", "OnTooltipSetItem",
                "OnTooltipSetUnit", "OnTooltipSetSpell", "OnTooltipCleared",
                "PostClick", "PreClick", "OnValueChanged", "OnMinMaxChanged",
                "OnEditFocusGained", "OnEditFocusLost", "OnTextChanged",
                "OnEnterPressed", "OnEscapePressed", "OnKeyDown", "OnKeyUp",
                "OnChar", "OnTabPressed", "OnSpacePressed", "OnReceiveDrag",
            ];
            Ok(common_scripts.iter().any(|s| s.eq_ignore_ascii_case(&script_type)))
        });

        // GetParent()
        methods.add_method("GetParent", |lua, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(parent_id) = frame.parent_id {
                    let handle = FrameHandle {
                        id: parent_id,
                        state: Rc::clone(&this.state),
                    };
                    return Ok(Value::UserData(lua.create_userdata(handle)?));
                }
            }
            Ok(Value::Nil)
        });

        // SetParent(parent)
        methods.add_method("SetParent", |_, this, parent: Value| {
            let new_parent_id = match parent {
                Value::Nil => None,
                Value::UserData(ud) => {
                    ud.borrow::<FrameHandle>().ok().map(|h| h.id)
                }
                _ => None,
            };
            let mut state = this.state.borrow_mut();

            // Get parent's strata and level for inheritance
            let parent_props = new_parent_id.and_then(|pid| {
                state.widgets.get(pid).map(|p| (p.frame_strata, p.frame_level))
            });

            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.parent_id = new_parent_id;

                // Inherit strata and level from parent (like wowless does)
                if let Some((parent_strata, parent_level)) = parent_props {
                    if !frame.has_fixed_frame_strata {
                        frame.frame_strata = parent_strata;
                    }
                    if !frame.has_fixed_frame_level {
                        frame.frame_level = parent_level + 1;
                    }
                }
            }
            Ok(())
        });

        // GetObjectType()
        methods.add_method("GetObjectType", |_, this, ()| {
            let state = this.state.borrow();
            let obj_type = state
                .widgets
                .get(this.id)
                .map(|f| f.widget_type.as_str())
                .unwrap_or("Frame");
            Ok(obj_type.to_string())
        });

        // SetAlpha(alpha)
        methods.add_method("SetAlpha", |_, this, alpha: f32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.alpha = alpha.clamp(0.0, 1.0);
            }
            Ok(())
        });

        // GetAlpha()
        methods.add_method("GetAlpha", |_, this, ()| {
            let state = this.state.borrow();
            let alpha = state.widgets.get(this.id).map(|f| f.alpha).unwrap_or(1.0);
            Ok(alpha)
        });

        // SetFrameStrata(strata)
        methods.add_method("SetFrameStrata", |_, this, strata: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                if let Some(s) = crate::widget::FrameStrata::from_str(&strata) {
                    frame.frame_strata = s;
                    frame.has_fixed_frame_strata = true;
                }
            }
            Ok(())
        });

        // GetFrameStrata()
        methods.add_method("GetFrameStrata", |_, this, ()| {
            let state = this.state.borrow();
            let strata = state
                .widgets
                .get(this.id)
                .map(|f| f.frame_strata.as_str())
                .unwrap_or("MEDIUM");
            Ok(strata.to_string())
        });

        // SetFrameLevel(level)
        methods.add_method("SetFrameLevel", |_, this, level: i32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.frame_level = level;
                frame.has_fixed_frame_level = true;
            }
            Ok(())
        });

        // GetFrameLevel()
        methods.add_method("GetFrameLevel", |_, this, ()| {
            let state = this.state.borrow();
            let level = state.widgets.get(this.id).map(|f| f.frame_level).unwrap_or(0);
            Ok(level)
        });

        // SetFixedFrameStrata(fixed) - Controls if strata is inherited from parent
        methods.add_method("SetFixedFrameStrata", |_, this, fixed: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.has_fixed_frame_strata = fixed;
            }
            Ok(())
        });

        // SetFixedFrameLevel(fixed) - Controls if level is inherited from parent
        methods.add_method("SetFixedFrameLevel", |_, this, fixed: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.has_fixed_frame_level = fixed;
            }
            Ok(())
        });

        // SetToplevel(toplevel) - Mark frame as toplevel (raises on click)
        methods.add_method("SetToplevel", |_, _this, _toplevel: bool| {
            Ok(())
        });

        // IsToplevel()
        methods.add_method("IsToplevel", |_, _this, ()| {
            Ok(false)
        });

        // NOTE: Raise() and Lower() methods are handled in __index metamethod
        // to allow custom properties with these names to take precedence.
        // See the __index handler for "Raise" and "Lower" fallback.

        // SetBackdrop(backdropInfo) - WoW backdrop system for frame backgrounds
        methods.add_method("SetBackdrop", |_, this, backdrop: Option<mlua::Table>| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                if let Some(info) = backdrop {
                    frame.backdrop.enabled = true;
                    // Parse texture paths
                    if let Ok(bg_file) = info.get::<String>("bgFile") {
                        frame.backdrop.bg_file = Some(bg_file);
                    }
                    if let Ok(edge_file) = info.get::<String>("edgeFile") {
                        frame.backdrop.edge_file = Some(edge_file);
                    }
                    // Parse edge size if provided
                    if let Ok(edge_size) = info.get::<f32>("edgeSize") {
                        frame.backdrop.edge_size = edge_size;
                    }
                    // Parse insets if provided
                    if let Ok(insets) = info.get::<mlua::Table>("insets") {
                        if let Ok(left) = insets.get::<f32>("left") {
                            frame.backdrop.insets = left;
                        }
                    }
                } else {
                    frame.backdrop.enabled = false;
                    frame.backdrop.bg_file = None;
                    frame.backdrop.edge_file = None;
                }
            }
            Ok(())
        });

        // ApplyBackdrop() - Apply backdrop template (used by DBM and other addons)
        methods.add_method("ApplyBackdrop", |_, this, args: mlua::MultiValue| {
            // ApplyBackdrop can take optional r, g, b, a parameters for background color
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.backdrop.enabled = true;
                // Parse optional color arguments
                let args_vec: Vec<Value> = args.into_iter().collect();
                if args_vec.len() >= 3 {
                    let r = match &args_vec[0] {
                        Value::Number(n) => *n as f32,
                        Value::Integer(n) => *n as f32,
                        _ => 0.0,
                    };
                    let g = match &args_vec[1] {
                        Value::Number(n) => *n as f32,
                        Value::Integer(n) => *n as f32,
                        _ => 0.0,
                    };
                    let b = match &args_vec[2] {
                        Value::Number(n) => *n as f32,
                        Value::Integer(n) => *n as f32,
                        _ => 0.0,
                    };
                    let a = if args_vec.len() >= 4 {
                        match &args_vec[3] {
                            Value::Number(n) => *n as f32,
                            Value::Integer(n) => *n as f32,
                            _ => 1.0,
                        }
                    } else {
                        1.0
                    };
                    frame.backdrop.bg_color = crate::widget::Color::new(r, g, b, a);
                }
            }
            Ok(())
        });

        // SetBackdropColor(r, g, b, a) - Set backdrop background color
        methods.add_method(
            "SetBackdropColor",
            |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.backdrop.enabled = true;
                    frame.backdrop.bg_color =
                        crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
                }
                Ok(())
            },
        );

        // GetBackdropColor() - Get backdrop background color
        methods.add_method("GetBackdropColor", |_, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                Ok((
                    frame.backdrop.bg_color.r,
                    frame.backdrop.bg_color.g,
                    frame.backdrop.bg_color.b,
                    frame.backdrop.bg_color.a,
                ))
            } else {
                Ok((0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32))
            }
        });

        // SetBackdropBorderColor(r, g, b, a) - Set backdrop border color
        methods.add_method(
            "SetBackdropBorderColor",
            |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.backdrop.enabled = true;
                    frame.backdrop.border_color =
                        crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
                }
                Ok(())
            },
        );

        // GetBackdropBorderColor() - Get backdrop border color
        methods.add_method("GetBackdropBorderColor", |_, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                Ok((
                    frame.backdrop.border_color.r,
                    frame.backdrop.border_color.g,
                    frame.backdrop.border_color.b,
                    frame.backdrop.border_color.a,
                ))
            } else {
                Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32))
            }
        });

        // SetID(id) - Set frame ID (used for tab ordering, etc.)
        methods.add_method("SetID", |_, _this, _id: i32| {
            // Accept ID but don't use it for now
            Ok(())
        });

        // GetID() - Get frame ID
        methods.add_method("GetID", |_, _this, ()| {
            Ok(0) // Default ID
        });

        // EnableMouse(enable)
        methods.add_method("EnableMouse", |_, this, enable: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.mouse_enabled = enable;
            }
            Ok(())
        });

        // IsMouseEnabled()
        methods.add_method("IsMouseEnabled", |_, this, ()| {
            let state = this.state.borrow();
            let enabled = state.widgets.get(this.id).map(|f| f.mouse_enabled).unwrap_or(false);
            Ok(enabled)
        });

        // EnableMouseWheel(enable) - enable mouse wheel events
        methods.add_method("EnableMouseWheel", |_, _this, _enable: bool| {
            Ok(())
        });

        // IsMouseWheelEnabled()
        methods.add_method("IsMouseWheelEnabled", |_, _this, ()| {
            Ok(false)
        });

        // EnableKeyboard(enable) - enable keyboard input for frame
        methods.add_method("EnableKeyboard", |_, _this, _enable: bool| {
            Ok(())
        });

        // IsKeyboardEnabled()
        methods.add_method("IsKeyboardEnabled", |_, _this, ()| {
            Ok(false)
        });

        // SetAllPoints(relativeTo)
        // SetAllPoints accepts: nil, frame, or boolean (true = parent, false = no-op)
        // Sets TOPLEFT→TOPLEFT and BOTTOMRIGHT→BOTTOMRIGHT to the relative frame
        methods.add_method("SetAllPoints", |_, this, arg: Option<Value>| {
            // Handle boolean case: true means use parent, false is a no-op
            let (should_set, relative_to_id) = match &arg {
                Some(Value::Boolean(false)) => (false, None),
                Some(Value::UserData(ud)) => {
                    // Extract frame ID from userdata
                    if let Ok(handle) = ud.borrow::<FrameHandle>() {
                        (true, Some(handle.id as usize))
                    } else {
                        (true, None)
                    }
                }
                _ => (true, None), // nil, true => use parent (None)
            };

            if should_set {
                let mut state = this.state.borrow_mut();

                // Check for anchor cycles before setting points
                if let Some(rel_id) = relative_to_id {
                    if state.widgets.would_create_anchor_cycle(this.id, rel_id as u64) {
                        return Ok(());
                    }
                }

                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.clear_all_points();
                    // SetAllPoints makes the frame fill its relative frame
                    frame.set_point(
                        crate::widget::AnchorPoint::TopLeft,
                        relative_to_id,
                        crate::widget::AnchorPoint::TopLeft,
                        0.0,
                        0.0,
                    );
                    frame.set_point(
                        crate::widget::AnchorPoint::BottomRight,
                        relative_to_id,
                        crate::widget::AnchorPoint::BottomRight,
                        0.0,
                        0.0,
                    );
                }
            }
            Ok(())
        });

        // GetPoint(index) -> point, relativeTo, relativePoint, xOfs, yOfs
        methods.add_method("GetPoint", |lua, this, index: Option<i32>| {
            let idx = index.unwrap_or(1) - 1; // Lua is 1-indexed
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(anchor) = frame.anchors.get(idx as usize) {
                    // Get the relative frame reference if we have an ID
                    let relative_to: Value = if let Some(rel_id) = anchor.relative_to_id {
                        // Look up the frame reference from globals
                        let frame_ref_key = format!("__frame_{}", rel_id);
                        lua.globals().get(frame_ref_key.as_str()).unwrap_or(Value::Nil)
                    } else {
                        Value::Nil
                    };
                    return Ok(mlua::MultiValue::from_vec(vec![
                        Value::String(lua.create_string(anchor.point.as_str())?),
                        relative_to,
                        Value::String(lua.create_string(anchor.relative_point.as_str())?),
                        Value::Number(anchor.x_offset as f64),
                        Value::Number(anchor.y_offset as f64),
                    ]));
                }
            }
            Ok(mlua::MultiValue::new())
        });

        // GetNumPoints()
        methods.add_method("GetNumPoints", |_, this, ()| {
            let state = this.state.borrow();
            let count = state.widgets.get(this.id).map(|f| f.anchors.len()).unwrap_or(0);
            Ok(count as i32)
        });

        // GetPointByName(pointName) -> point, relativeTo, relativePoint, xOfs, yOfs
        // Finds an anchor by its point name (e.g., "TOPLEFT", "CENTER")
        methods.add_method("GetPointByName", |lua, this, point_name: String| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                let point_upper = point_name.to_uppercase();
                for anchor in &frame.anchors {
                    if anchor.point.as_str().to_uppercase() == point_upper {
                        return Ok(mlua::MultiValue::from_vec(vec![
                            Value::String(lua.create_string(anchor.point.as_str())?),
                            Value::Nil, // relativeTo (would need to return frame reference)
                            Value::String(lua.create_string(anchor.relative_point.as_str())?),
                            Value::Number(anchor.x_offset as f64),
                            Value::Number(anchor.y_offset as f64),
                        ]));
                    }
                }
            }
            Ok(mlua::MultiValue::new())
        });

        // GetNumChildren() - return count of child frames
        methods.add_method("GetNumChildren", |_, this, ()| {
            let state = this.state.borrow();
            let count = state.widgets.get(this.id).map(|f| f.children.len()).unwrap_or(0);
            Ok(count as i32)
        });

        // GetChildren() - return all child frames as multiple return values
        methods.add_method("GetChildren", |lua, this, ()| {
            let state = this.state.borrow();
            let mut result = mlua::MultiValue::new();
            if let Some(frame) = state.widgets.get(this.id) {
                let children = frame.children.clone();
                drop(state); // Release borrow before creating userdata

                for child_id in children {
                    let handle = FrameHandle {
                        id: child_id,
                        state: Rc::clone(&this.state),
                    };
                    if let Ok(ud) = lua.create_userdata(handle) {
                        result.push_back(Value::UserData(ud));
                    }
                }
            }
            Ok(result)
        });

        // CreateTexture(name, layer, inherits, subLevel)
        methods.add_method("CreateTexture", |lua, this, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let name_raw: Option<String> = args.first().and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            });

            // Handle $parent substitution
            let name: Option<String> = name_raw.map(|n| {
                if n.contains("$parent") || n.contains("$Parent") {
                    let state = this.state.borrow();
                    if let Some(parent_name) = state.widgets.get(this.id).and_then(|f| f.name.clone())
                    {
                        n.replace("$parent", &parent_name)
                            .replace("$Parent", &parent_name)
                    } else {
                        n.replace("$parent", "").replace("$Parent", "")
                    }
                } else {
                    n
                }
            });

            let texture = Frame::new(WidgetType::Texture, name.clone(), Some(this.id));
            let texture_id = texture.id;

            {
                let mut state = this.state.borrow_mut();
                state.widgets.register(texture);
                state.widgets.add_child(this.id, texture_id);
            }

            let handle = FrameHandle {
                id: texture_id,
                state: Rc::clone(&this.state),
            };

            let ud = lua.create_userdata(handle)?;

            if let Some(ref n) = name {
                lua.globals().set(n.as_str(), ud.clone())?;
            }

            let frame_key = format!("__frame_{}", texture_id);
            lua.globals().set(frame_key.as_str(), ud.clone())?;

            Ok(ud)
        });

        // CreateMaskTexture(layer, inherits, subLevel) - create a mask texture (stub)
        methods.add_method("CreateMaskTexture", |lua, this, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let name_raw: Option<String> = args.first().and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            });

            // Handle $parent substitution
            let name: Option<String> = name_raw.map(|n| {
                if n.contains("$parent") || n.contains("$Parent") {
                    let state = this.state.borrow();
                    if let Some(parent_name) = state.widgets.get(this.id).and_then(|f| f.name.clone())
                    {
                        n.replace("$parent", &parent_name)
                            .replace("$Parent", &parent_name)
                    } else {
                        n.replace("$parent", "").replace("$Parent", "")
                    }
                } else {
                    n
                }
            });

            // Create a texture and return it as a mask texture (they're essentially the same)
            let texture = Frame::new(WidgetType::Texture, name.clone(), Some(this.id));
            let texture_id = texture.id;

            {
                let mut state = this.state.borrow_mut();
                state.widgets.register(texture);
                state.widgets.add_child(this.id, texture_id);
            }

            let handle = FrameHandle {
                id: texture_id,
                state: Rc::clone(&this.state),
            };

            let ud = lua.create_userdata(handle)?;

            if let Some(ref n) = name {
                lua.globals().set(n.as_str(), ud.clone())?;
            }

            let frame_key = format!("__frame_{}", texture_id);
            lua.globals().set(frame_key.as_str(), ud.clone())?;

            Ok(ud)
        });

        // CreateFontString(name, layer, inherits)
        methods.add_method("CreateFontString", |lua, this, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let name_raw: Option<String> = args.first().and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            });

            // Handle $parent substitution
            let name: Option<String> = name_raw.map(|n| {
                if n.contains("$parent") || n.contains("$Parent") {
                    let state = this.state.borrow();
                    if let Some(parent_name) = state.widgets.get(this.id).and_then(|f| f.name.clone())
                    {
                        n.replace("$parent", &parent_name)
                            .replace("$Parent", &parent_name)
                    } else {
                        n.replace("$parent", "").replace("$Parent", "")
                    }
                } else {
                    n
                }
            });

            let fontstring = Frame::new(WidgetType::FontString, name.clone(), Some(this.id));
            let fontstring_id = fontstring.id;

            {
                let mut state = this.state.borrow_mut();
                state.widgets.register(fontstring);
                state.widgets.add_child(this.id, fontstring_id);
            }

            let handle = FrameHandle {
                id: fontstring_id,
                state: Rc::clone(&this.state),
            };

            let ud = lua.create_userdata(handle)?;

            if let Some(ref n) = name {
                lua.globals().set(n.as_str(), ud.clone())?;
            }

            let frame_key = format!("__frame_{}", fontstring_id);
            lua.globals().set(frame_key.as_str(), ud.clone())?;

            Ok(ud)
        });

        // CreateAnimationGroup(name, inherits) - create animation group (stub)
        methods.add_method("CreateAnimationGroup", |lua, _this, (_name, _inherits): (Option<String>, Option<String>)| {
            // Return a stub animation group table - all methods accept self for colon notation
            let anim_group = lua.create_table()?;
            anim_group.set("Play", lua.create_function(|_, _self: Value| Ok(()))?)?;
            anim_group.set("Stop", lua.create_function(|_, _self: Value| Ok(()))?)?;
            anim_group.set("Pause", lua.create_function(|_, _self: Value| Ok(()))?)?;
            anim_group.set("Finish", lua.create_function(|_, _self: Value| Ok(()))?)?;
            anim_group.set("IsPlaying", lua.create_function(|_, _self: Value| Ok(false))?)?;
            anim_group.set("IsPaused", lua.create_function(|_, _self: Value| Ok(false))?)?;
            anim_group.set("IsDone", lua.create_function(|_, _self: Value| Ok(true))?)?;
            anim_group.set("SetLooping", lua.create_function(|_, (_self, _looping): (Value, Option<String>)| Ok(()))?)?;
            anim_group.set("GetLooping", lua.create_function(|lua, _self: Value| Ok(Value::String(lua.create_string("NONE")?)))?)?;
            anim_group.set("GetParent", lua.create_function(|_, _self: Value| Ok(Value::Nil))?)?;
            anim_group.set("GetAnimations", lua.create_function(|_, _self: Value| Ok(mlua::MultiValue::new()))?)?;
            // Additional AnimationGroup methods
            anim_group.set("SetToFinalAlpha", lua.create_function(|_, (_self, _final): (Value, bool)| Ok(()))?)?;
            anim_group.set("GetToFinalAlpha", lua.create_function(|_, _self: Value| Ok(false))?)?;
            // Animation creation methods
            anim_group.set("CreateAnimation", lua.create_function(|lua, (_self, _anim_type, _name, _inherits): (Value, Option<String>, Option<String>, Option<String>)| {
                let anim = lua.create_table()?;
                // All animation methods take self as first param when called with colon notation
                anim.set("SetDuration", lua.create_function(|_, (_self, _dur): (Value, f64)| Ok(()))?)?;
                anim.set("GetDuration", lua.create_function(|_, _self: Value| Ok(0.0_f64))?)?;
                anim.set("SetStartDelay", lua.create_function(|_, (_self, _delay): (Value, f64)| Ok(()))?)?;
                anim.set("SetEndDelay", lua.create_function(|_, (_self, _delay): (Value, f64)| Ok(()))?)?;
                anim.set("SetOrder", lua.create_function(|_, (_self, _order): (Value, i32)| Ok(()))?)?;
                anim.set("SetSmoothing", lua.create_function(|_, (_self, _smooth): (Value, String)| Ok(()))?)?;
                // Type-specific methods
                anim.set("SetFromAlpha", lua.create_function(|_, (_self, _alpha): (Value, f64)| Ok(()))?)?;
                anim.set("SetToAlpha", lua.create_function(|_, (_self, _alpha): (Value, f64)| Ok(()))?)?;
                anim.set("SetChange", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                anim.set("SetScale", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                anim.set("SetScaleFrom", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                anim.set("SetScaleTo", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                anim.set("SetOffset", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                anim.set("SetDegrees", lua.create_function(|_, (_self, _degrees): (Value, f64)| Ok(()))?)?;
                anim.set("SetOrigin", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                // Additional animation methods
                anim.set("Play", lua.create_function(|_, _self: Value| Ok(()))?)?;
                anim.set("Stop", lua.create_function(|_, _self: Value| Ok(()))?)?;
                anim.set("Pause", lua.create_function(|_, _self: Value| Ok(()))?)?;
                anim.set("IsPlaying", lua.create_function(|_, _self: Value| Ok(false))?)?;
                anim.set("IsPaused", lua.create_function(|_, _self: Value| Ok(false))?)?;
                anim.set("IsDone", lua.create_function(|_, _self: Value| Ok(true))?)?;
                anim.set("GetParent", lua.create_function(|_, _self: Value| Ok(Value::Nil))?)?;
                anim.set("GetRegionParent", lua.create_function(|_, _self: Value| Ok(Value::Nil))?)?;
                anim.set("GetProgress", lua.create_function(|_, _self: Value| Ok(0.0_f64))?)?;
                anim.set("GetSmoothProgress", lua.create_function(|_, _self: Value| Ok(0.0_f64))?)?;
                // Animation script handlers (OnFinished, OnUpdate, OnPlay, OnStop, etc.)
                anim.set("SetScript", lua.create_function(|_, (_self, _event, _handler): (Value, String, Option<mlua::Function>)| Ok(()))?)?;
                anim.set("GetScript", lua.create_function(|_, (_self, _event): (Value, String)| Ok(Value::Nil))?)?;
                anim.set("HasScript", lua.create_function(|_, (_self, _event): (Value, String)| Ok(false))?)?;
                Ok(anim)
            })?)?;
            // Script handlers (accept self for colon notation)
            anim_group.set("SetScript", lua.create_function(|_, (_self, _event, _handler): (Value, String, Option<mlua::Function>)| Ok(()))?)?;
            anim_group.set("GetScript", lua.create_function(|_, (_self, _event): (Value, String)| Ok(Value::Nil))?)?;
            Ok(anim_group)
        });

        // SetTexture(path) - for Texture widgets
        methods.add_method("SetTexture", |_, this, path: Option<String>| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.texture = path;
            }
            Ok(())
        });

        // GetTexture() - for Texture widgets
        methods.add_method("GetTexture", |_, this, ()| {
            let state = this.state.borrow();
            let texture = state
                .widgets
                .get(this.id)
                .and_then(|f| f.texture.clone());
            Ok(texture)
        });

        // SetHorizTile(tile) - Enable/disable horizontal tiling
        methods.add_method("SetHorizTile", |_, this, tile: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.horiz_tile = tile;
            }
            Ok(())
        });

        // GetHorizTile() - Check if horizontal tiling is enabled
        methods.add_method("GetHorizTile", |_, this, ()| {
            let state = this.state.borrow();
            let tile = state
                .widgets
                .get(this.id)
                .map(|f| f.horiz_tile)
                .unwrap_or(false);
            Ok(tile)
        });

        // SetVertTile(tile) - Enable/disable vertical tiling
        methods.add_method("SetVertTile", |_, this, tile: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.vert_tile = tile;
            }
            Ok(())
        });

        // GetVertTile() - Check if vertical tiling is enabled
        methods.add_method("GetVertTile", |_, this, ()| {
            let state = this.state.borrow();
            let tile = state
                .widgets
                .get(this.id)
                .map(|f| f.vert_tile)
                .unwrap_or(false);
            Ok(tile)
        });

        // SetBlendMode(blendMode) - Set texture blend mode (ADD, ALPHAKEY, BLEND, DISABLE, MOD)
        methods.add_method("SetBlendMode", |_, _this, _mode: Option<String>| {
            // Stub - blend mode is a rendering hint
            Ok(())
        });

        // GetBlendMode() - Get texture blend mode
        methods.add_method("GetBlendMode", |_, _this, ()| {
            Ok("BLEND") // Default blend mode
        });

        // SetDesaturated(desaturation) - Set texture desaturation
        methods.add_method("SetDesaturated", |_, _this, _desaturated: bool| {
            // Stub - desaturation is a rendering effect
            Ok(())
        });

        // IsDesaturated() - Check if texture is desaturated
        methods.add_method("IsDesaturated", |_, _this, ()| {
            Ok(false)
        });

        // SetAtlas(atlasName, useAtlasSize, filterMode, resetTexCoords) - Set texture from atlas
        methods.add_method("SetAtlas", |_, this, args: mlua::MultiValue| {
            let args_vec: Vec<Value> = args.into_iter().collect();
            let atlas_name = args_vec.first().and_then(|v| match v {
                Value::String(s) => Some(s.to_string_lossy().to_string()),
                _ => None,
            });
            let use_atlas_size = args_vec.get(1).map(|v| matches!(v, Value::Boolean(true))).unwrap_or(false);

            if let Some(name) = atlas_name {
                // Look up atlas info
                if let Some(atlas_info) = crate::atlas::get_atlas_info(&name) {
                    let mut state = this.state.borrow_mut();
                    if let Some(frame) = state.widgets.get_mut(this.id) {
                        // Set texture file from atlas
                        frame.texture = Some(atlas_info.file.to_string());
                        // Set texture coordinates
                        frame.tex_coords = Some((
                            atlas_info.left_tex_coord,
                            atlas_info.right_tex_coord,
                            atlas_info.top_tex_coord,
                            atlas_info.bottom_tex_coord,
                        ));
                        // Set tiling flags
                        frame.horiz_tile = atlas_info.tiles_horizontally;
                        frame.vert_tile = atlas_info.tiles_vertically;
                        // Store atlas name
                        frame.atlas = Some(name);
                        // Optionally set size from atlas
                        if use_atlas_size {
                            frame.width = atlas_info.width as f32;
                            frame.height = atlas_info.height as f32;
                        }
                    }
                } else {
                    // Unknown atlas - just store the name
                    let mut state = this.state.borrow_mut();
                    if let Some(frame) = state.widgets.get_mut(this.id) {
                        frame.atlas = Some(name);
                    }
                }
            }
            Ok(())
        });

        // GetAtlas() - Get current atlas name
        methods.add_method("GetAtlas", |lua, this, ()| {
            let state = this.state.borrow();
            let atlas = state
                .widgets
                .get(this.id)
                .and_then(|f| f.atlas.clone());
            match atlas {
                Some(name) => Ok(Value::String(lua.create_string(&name)?)),
                None => Ok(Value::Nil),
            }
        });

        // SetSnapToPixelGrid(snap) - Set whether texture snaps to pixel grid
        methods.add_method("SetSnapToPixelGrid", |_, _this, _snap: bool| {
            // No-op for now, just store the state if needed
            Ok(())
        });

        // IsSnappingToPixelGrid() - Get whether texture snaps to pixel grid
        methods.add_method("IsSnappingToPixelGrid", |_, _this, ()| {
            Ok(false)
        });

        // SetTexelSnappingBias(bias) - Set texel snapping bias for pixel-perfect rendering
        methods.add_method("SetTexelSnappingBias", |_, _this, _bias: f32| {
            // No-op - this controls sub-pixel texture positioning
            Ok(())
        });

        // GetTexelSnappingBias() - Get texel snapping bias
        methods.add_method("GetTexelSnappingBias", |_, _this, ()| {
            Ok(0.0_f32)
        });

        // SetTextureSliceMargins(left, right, top, bottom) - Set 9-slice margins
        methods.add_method(
            "SetTextureSliceMargins",
            |_, _this, (_left, _right, _top, _bottom): (f32, f32, f32, f32)| Ok(()),
        );

        // GetTextureSliceMargins() - Get 9-slice margins
        methods.add_method("GetTextureSliceMargins", |_, _this, ()| {
            Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32))
        });

        // SetTextureSliceMode(mode) - Set 9-slice mode
        methods.add_method("SetTextureSliceMode", |_, _this, _mode: i32| Ok(()));

        // GetTextureSliceMode() - Get 9-slice mode
        methods.add_method("GetTextureSliceMode", |_, _this, ()| Ok(0i32));

        // ClearTextureSlice() - Clear 9-slice configuration
        methods.add_method("ClearTextureSlice", |_, _this, ()| Ok(()));

        // SetText(text) - for FontString widgets
        // Auto-sizes the FontString to fit the text content
        methods.add_method("SetText", |_, this, text: Option<String>| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                // Calculate auto-size dimensions if width/height is 0
                if let Some(ref txt) = text {
                    // Auto-size: ~7 pixels per character for width, font_size for height
                    if frame.width == 0.0 {
                        frame.width = txt.len() as f32 * 7.0;
                    }
                    if frame.height == 0.0 {
                        frame.height = frame.font_size.max(12.0);
                    }
                }
                frame.text = text;
            }
            Ok(())
        });

        // GetText() - for FontString widgets
        methods.add_method("GetText", |_, this, ()| {
            let state = this.state.borrow();
            let text = state
                .widgets
                .get(this.id)
                .and_then(|f| f.text.clone())
                .unwrap_or_default();
            Ok(text)
        });

        // SetFormattedText(format, ...) - for FontString widgets (like string.format + SetText)
        // Auto-sizes the FontString to fit the text content
        methods.add_method("SetFormattedText", |lua, this, args: mlua::MultiValue| {
            // Use Lua's string.format to format the text
            let string_table: mlua::Table = lua.globals().get("string")?;
            let format_func: mlua::Function = string_table.get("format")?;
            if let Ok(Value::String(result)) = format_func.call::<Value>(args) {
                let text = result.to_string_lossy().to_string();
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    // Auto-size: ~7 pixels per character for width, font_size for height
                    if frame.width == 0.0 {
                        frame.width = text.len() as f32 * 7.0;
                    }
                    if frame.height == 0.0 {
                        frame.height = frame.font_size.max(12.0);
                    }
                    frame.text = Some(text);
                }
            }
            Ok(())
        });

        // SetTitle(title) - for PortraitFrame/ButtonFrame templates
        // In WoW: self:GetTitleText():SetText(title) where GetTitleText returns self.TitleContainer.TitleText
        methods.add_method("SetTitle", |_, this, title: Option<String>| {
            let mut state = this.state.borrow_mut();

            // Find TitleContainer.TitleText and update its text
            let title_text_id = state.widgets.get(this.id)
                .and_then(|f| f.children_keys.get("TitleContainer").copied())
                .and_then(|tc_id| state.widgets.get(tc_id))
                .and_then(|tc| tc.children_keys.get("TitleText").copied());

            if let Some(tt_id) = title_text_id {
                if let Some(title_text) = state.widgets.get_mut(tt_id) {
                    title_text.text = title.clone();
                    // Auto-size height if not set (FontStrings need height for rendering)
                    if title_text.height == 0.0 {
                        title_text.height = title_text.font_size.max(12.0);
                    }
                }
            }

            // Also store on frame itself for GetTitle
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.title = title;
            }
            Ok(())
        });

        // GetTitle() - for DefaultPanelTemplate frames
        methods.add_method("GetTitle", |_, this, ()| {
            let state = this.state.borrow();
            let title = state
                .widgets
                .get(this.id)
                .and_then(|f| f.title.clone())
                .unwrap_or_default();
            Ok(title)
        });

        // SetTitleOffsets(x, y) - set title position offset
        methods.add_method("SetTitleOffsets", |_, _this, (_x, _y): (f64, f64)| {
            // Stub - title offsets are a rendering detail
            Ok(())
        });

        // SetBorder(layoutName) - set nine-slice border layout (for PortraitFrameMixin)
        // This is called by ButtonFrameTemplate_HidePortrait to switch to a non-portrait layout
        methods.add_method("SetBorder", |lua, this, layout_name: Option<String>| {
            if let Some(layout) = layout_name {
                // Get the frame name from state
                let state = this.state.borrow();
                let frame_name = state
                    .widgets
                    .get(this.id)
                    .and_then(|f| f.name.clone())
                    .unwrap_or_else(|| format!("__frame_{}", this.id));
                drop(state);

                // Try to call NineSliceUtil.ApplyLayout if it exists
                // This will update the NineSlice child's textures
                let code = format!(
                    r#"
                    local frame = {0}
                    local layoutName = "{1}"
                    if frame and frame.NineSlice then
                        if NineSliceUtil and NineSliceUtil.ApplyLayout and NineSliceUtil.GetLayout then
                            local layoutTable = NineSliceUtil.GetLayout(layoutName)
                            if layoutTable then
                                NineSliceUtil.ApplyLayout(frame.NineSlice, layoutTable)
                            end
                        else
                            -- Fallback: If layout is NoPortrait variant, update corners directly
                            local ns = frame.NineSlice
                            if layoutName:find("NoPortrait") and ns.TopLeftCorner then
                                -- Switch from portrait corner to regular corner
                                local atlas = ns.TopLeftCorner:GetAtlas()
                                if atlas and atlas:find("Portrait") then
                                    local newAtlas = atlas:gsub("Portrait", "")
                                    ns.TopLeftCorner:SetAtlas(newAtlas, true)
                                end
                            end
                        end
                    end
                    "#,
                    frame_name, layout
                );
                if let Err(e) = lua.load(&code).exec() {
                    eprintln!("SetBorder Lua error: {}", e);
                }
            }
            Ok(())
        });

        // SetBorderColor(r, g, b, a) - set border color
        methods.add_method("SetBorderColor", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetBorderInsets(left, right, top, bottom) - set border insets
        methods.add_method("SetBorderInsets", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetPortraitTextureSizeAndOffset(size, x, y) - for portrait frames
        methods.add_method("SetPortraitTextureSizeAndOffset", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetPortraitTextureRaw(tex) - set portrait texture
        methods.add_method("SetPortraitTextureRaw", |_, _this, _tex: Option<String>| {
            Ok(())
        });

        // SetPortraitToAsset(fileID) - set portrait from file ID
        methods.add_method("SetPortraitToAsset", |_, _this, _file_id: i32| {
            Ok(())
        });

        // SetPortraitToUnit(unit) - set portrait from unit
        methods.add_method("SetPortraitToUnit", |_, _this, _unit: String| {
            Ok(())
        });

        // SetPortraitShown(shown) - show/hide the portrait container
        // Called by ButtonFrameTemplate_HidePortrait to hide the portrait area
        methods.add_method("SetPortraitShown", |lua, this, shown: bool| {
            // Get the frame name from state
            let state = this.state.borrow();
            let frame_name = state
                .widgets
                .get(this.id)
                .and_then(|f| f.name.clone())
                .unwrap_or_else(|| format!("__frame_{}", this.id));
            drop(state);

            // Hide/show the PortraitContainer child frame
            let code = format!(
                r#"
                local frame = {}
                if frame and frame.PortraitContainer then
                    if {} then
                        frame.PortraitContainer:Show()
                    else
                        frame.PortraitContainer:Hide()
                    end
                end
                "#,
                frame_name,
                if shown { "true" } else { "false" }
            );
            let _ = lua.load(&code).exec();
            Ok(())
        });

        // SetShadowOffset(x, y) - set shadow offset for FontStrings
        methods.add_method("SetShadowOffset", |_, _this, (_x, _y): (f64, f64)| {
            Ok(())
        });

        // GetShadowOffset() - get shadow offset for FontStrings
        methods.add_method("GetShadowOffset", |_, _this, ()| {
            Ok((0.0_f64, 0.0_f64))
        });

        // SetShadowColor(r, g, b, a) - set shadow color for FontStrings
        methods.add_method("SetShadowColor", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // GetShadowColor() - get shadow color for FontStrings
        methods.add_method("GetShadowColor", |_, _this, ()| {
            Ok((0.0_f64, 0.0_f64, 0.0_f64, 1.0_f64))
        });

        // SetFont(font, size, flags) - for FontString widgets
        methods.add_method("SetFont", |_, this, (font, size, _flags): (String, Option<f32>, Option<String>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.font = Some(font);
                if let Some(s) = size {
                    frame.font_size = s;
                }
            }
            Ok(true) // Returns success
        });

        // SetVertexColor(r, g, b, a) - for Texture widgets
        methods.add_method(
            "SetVertexColor",
            |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.vertex_color =
                        Some(crate::widget::Color::new(r, g, b, a.unwrap_or(1.0)));
                }
                Ok(())
            },
        );

        // GetVertexColor() - get vertex color for Texture widgets
        methods.add_method("GetVertexColor", |_, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(color) = &frame.vertex_color {
                    return Ok((color.r, color.g, color.b, color.a));
                }
            }
            Ok((1.0f32, 1.0f32, 1.0f32, 1.0f32)) // Default white
        });

        // SetCenterColor(r, g, b, a) - for NineSlice frames (sets center fill color)
        methods.add_method(
            "SetCenterColor",
            |_, _this, _args: mlua::MultiValue| {
                // NineSlice center color - just stub for now
                Ok(())
            },
        );

        // SetTextColor(r, g, b, a) - for FontString widgets
        methods.add_method(
            "SetTextColor",
            |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.text_color = crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
                }
                Ok(())
            },
        );

        // GetTextColor() - for FontString widgets
        methods.add_method("GetTextColor", |_, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                Ok((
                    frame.text_color.r,
                    frame.text_color.g,
                    frame.text_color.b,
                    frame.text_color.a,
                ))
            } else {
                Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32))
            }
        });

        // SetTexCoord(left, right, top, bottom) - for Texture widgets
        // Can also be called with 8 values for corner-based coords
        methods.add_method("SetTexCoord", |_, this, args: mlua::MultiValue| {
            let args_vec: Vec<Value> = args.into_iter().collect();
            if args_vec.len() >= 4 {
                let left = match &args_vec[0] {
                    Value::Number(n) => *n as f32,
                    Value::Integer(n) => *n as f32,
                    _ => 0.0,
                };
                let right = match &args_vec[1] {
                    Value::Number(n) => *n as f32,
                    Value::Integer(n) => *n as f32,
                    _ => 1.0,
                };
                let top = match &args_vec[2] {
                    Value::Number(n) => *n as f32,
                    Value::Integer(n) => *n as f32,
                    _ => 0.0,
                };
                let bottom = match &args_vec[3] {
                    Value::Number(n) => *n as f32,
                    Value::Integer(n) => *n as f32,
                    _ => 1.0,
                };

                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.tex_coords = Some((left, right, top, bottom));
                }
            }
            Ok(())
        });

        // AddMaskTexture(mask) - add a mask texture to this texture
        methods.add_method("AddMaskTexture", |_, _this, _mask: Value| {
            // Mask textures control alpha blending on parent texture
            Ok(())
        });

        // RemoveMaskTexture(mask) - remove a mask texture from this texture
        methods.add_method("RemoveMaskTexture", |_, _this, _mask: Value| {
            Ok(())
        });

        // GetNumMaskTextures() - get number of mask textures
        methods.add_method("GetNumMaskTextures", |_, _this, ()| {
            Ok(0)
        });

        // GetMaskTexture(index) - get mask texture by index
        methods.add_method("GetMaskTexture", |_, _this, _index: i32| {
            Ok(Value::Nil)
        });

        // SetColorTexture(r, g, b, a) - for Texture widgets
        methods.add_method("SetColorTexture", |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.color_texture = Some(crate::widget::Color::new(r, g, b, a.unwrap_or(1.0)));
                // Clear file texture when setting color texture
                frame.texture = None;
            }
            Ok(())
        });

        // SetGradient(orientation, minColor, maxColor) - set gradient on texture
        methods.add_method("SetGradient", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetDrawLayer(layer, sublayer) - set draw layer for texture/fontstring
        methods.add_method("SetDrawLayer", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // GetDrawLayer() - get draw layer for texture/fontstring
        methods.add_method("GetDrawLayer", |_, _this, ()| {
            // Returns: layer, sublayer
            Ok(("ARTWORK", 0i32))
        });

        // SetFontObject(fontObject) - for FontString widgets
        methods.add_method("SetFontObject", |_, _this, _font_object: Value| {
            // Would copy font settings from another FontString
            Ok(())
        });

        // GetFont() - for FontString widgets, returns fontFile, fontHeight, fontFlags
        methods.add_method("GetFont", |lua, this, ()| {
            let state = this.state.borrow();
            let font_size = state.widgets.get(this.id).map(|f| f.font_size).unwrap_or(12.0);
            // Return: fontFile, fontHeight, fontFlags
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string("Fonts\\FRIZQT__.TTF")?),
                Value::Number(font_size as f64),
                Value::String(lua.create_string("")?), // flags
            ]))
        });

        // SetJustifyH(justify) - for FontString widgets
        methods.add_method("SetJustifyH", |_, this, justify: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.justify_h = crate::widget::TextJustify::from_wow_str(&justify);
            }
            Ok(())
        });

        // SetJustifyV(justify) - for FontString widgets
        methods.add_method("SetJustifyV", |_, this, justify: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.justify_v = crate::widget::TextJustify::from_wow_str(&justify);
            }
            Ok(())
        });

        // GetStringWidth() - for FontString widgets
        methods.add_method("GetStringWidth", |_, this, ()| {
            let state = this.state.borrow();
            // Approximate: 7 pixels per character
            let width = state
                .widgets
                .get(this.id)
                .and_then(|f| f.text.as_ref())
                .map(|t| t.len() as f32 * 7.0)
                .unwrap_or(0.0);
            Ok(width)
        });

        // GetTextWidth() - alias for GetStringWidth (EditBox uses this)
        methods.add_method("GetTextWidth", |_, this, ()| {
            let state = this.state.borrow();
            let width = state
                .widgets
                .get(this.id)
                .and_then(|f| f.text.as_ref())
                .map(|t| t.len() as f32 * 7.0)
                .unwrap_or(0.0);
            Ok(width)
        });

        // GetUnboundedStringWidth() - string width without word wrap constraints
        methods.add_method("GetUnboundedStringWidth", |_, this, ()| {
            let state = this.state.borrow();
            // Approximate: 7 pixels per character
            let width = state
                .widgets
                .get(this.id)
                .and_then(|f| f.text.as_ref())
                .map(|t| t.len() as f32 * 7.0)
                .unwrap_or(0.0);
            Ok(width)
        });

        // GetFontObjectForAlphabet(alphabet) - returns self for font localization
        // In WoW this returns different fonts for Latin/Cyrillic/etc.
        // For simulation, just return self
        methods.add_method("GetFontObjectForAlphabet", |lua, this, _alphabet: Option<String>| {
            // Return the frame itself (as a userdata) - it's a font object
            let ud = lua.create_userdata(this.clone())?;
            Ok(ud)
        });

        // GetStringHeight() - for FontString widgets
        methods.add_method("GetStringHeight", |_, this, ()| {
            let state = this.state.borrow();
            let height = state.widgets.get(this.id).map(|f| f.font_size).unwrap_or(12.0);
            Ok(height)
        });

        // SetWordWrap(wrap) - for FontString widgets
        methods.add_method("SetWordWrap", |_, this, wrap: bool| {
            if let Ok(mut s) = this.state.try_borrow_mut() {
                if let Some(frame) = s.widgets.get_mut(this.id) {
                    frame.word_wrap = wrap;
                }
            }
            Ok(())
        });

        // GetWordWrap() - check if word wrap is enabled
        methods.add_method("GetWordWrap", |_, this, ()| {
            if let Ok(s) = this.state.try_borrow() {
                if let Some(frame) = s.widgets.get(this.id) {
                    return Ok(frame.word_wrap);
                }
            }
            Ok(false)
        });

        // IsTruncated() - check if text is truncated (for FontString)
        methods.add_method("IsTruncated", |_, _this, ()| {
            // Return false since we don't actually render/measure text
            Ok(false)
        });

        // SetTextScale(scale) - set text scale factor
        methods.add_method("SetTextScale", |_, this, scale: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.text_scale = scale;
            }
            Ok(())
        });

        // GetTextScale() - get text scale factor
        methods.add_method("GetTextScale", |_, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                return Ok(frame.text_scale);
            }
            Ok(1.0_f64)
        });

        // CanWordWrap() - check if word wrap is supported
        methods.add_method("CanWordWrap", |_, _this, ()| {
            Ok(true)
        });

        // GetWrappedWidth() - get width when wrapped
        methods.add_method("GetWrappedWidth", |_, this, ()| {
            let state = this.state.borrow();
            let width = state.widgets.get(this.id).map(|f| f.width).unwrap_or(0.0);
            Ok(width)
        });

        // SetNonSpaceWrap(wrap) - for FontString widgets
        methods.add_method("SetNonSpaceWrap", |_, _this, _wrap: bool| {
            Ok(())
        });

        // CanNonSpaceWrap() - check if non-space wrap is supported
        methods.add_method("CanNonSpaceWrap", |_, _this, ()| {
            Ok(true)
        });

        // SetMaxLines(maxLines) - for FontString widgets
        methods.add_method("SetMaxLines", |_, _this, _max_lines: i32| {
            Ok(())
        });

        // GetMaxLines() - for FontString widgets
        methods.add_method("GetMaxLines", |_, _this, ()| {
            Ok(0i32) // 0 means unlimited
        });

        // SetIndentedWordWrap(indent) - for FontString widgets
        methods.add_method("SetIndentedWordWrap", |_, _this, _indent: bool| {
            Ok(())
        });

        // SetSpacing(spacing) - for FontString widgets
        methods.add_method("SetSpacing", |_, _this, _spacing: f64| {
            Ok(())
        });

        // GetSpacing() - for FontString widgets
        methods.add_method("GetSpacing", |_, _this, ()| {
            Ok(0.0_f64)
        });

        // SetForbidden() - marks frame as forbidden (security feature, no-op in simulation)
        methods.add_method("SetForbidden", |_, _this, _forbidden: Option<bool>| {
            Ok(())
        });

        // IsForbidden() - check if frame is forbidden
        methods.add_method("IsForbidden", |_, _this, ()| {
            Ok(false)
        });

        // CanChangeProtectedState() - check if we can change protected state
        methods.add_method("CanChangeProtectedState", |_, _this, ()| {
            Ok(true) // Always true in simulation
        });

        // SetPassThroughButtons(...) - set which mouse buttons pass through
        methods.add_method("SetPassThroughButtons", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetFlattensRenderLayers(flatten) - for render optimization
        methods.add_method("SetFlattensRenderLayers", |_, _this, _flatten: Option<bool>| {
            Ok(())
        });

        // SetClipsChildren(clips) - whether to clip child frames
        methods.add_method("SetClipsChildren", |_, _this, _clips: Option<bool>| {
            Ok(())
        });

        // SetMotionScriptsWhileDisabled(enabled) - enable/disable motion scripts when disabled
        methods.add_method("SetMotionScriptsWhileDisabled", |_, _this, _enabled: Option<bool>| {
            Ok(())
        });

        // GetMotionScriptsWhileDisabled() - check if motion scripts run when disabled
        methods.add_method("GetMotionScriptsWhileDisabled", |_, _this, ()| {
            Ok(false)
        });

        // SetHitRectInsets(left, right, top, bottom) - extend/contract clickable area
        methods.add_method("SetHitRectInsets", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // GetHitRectInsets() - get clickable area insets
        methods.add_method("GetHitRectInsets", |_, _this, ()| {
            Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64))
        });

        // SetShown(shown) - show/hide based on boolean
        methods.add_method("SetShown", |_, this, shown: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = shown;
            }
            Ok(())
        });

        // GetEffectiveScale() - get combined scale of frame and parents
        methods.add_method("GetEffectiveScale", |_, _this, ()| {
            Ok(1.0f32) // No scaling in simulation
        });

        // GetScale() - get frame's scale
        methods.add_method("GetScale", |_, _this, ()| {
            Ok(1.0f32)
        });

        // SetScale(scale) - set frame's scale
        methods.add_method("SetScale", |_, _this, _scale: f32| {
            Ok(())
        });

        // SetIgnoreParentScale(ignore) - set whether frame ignores parent scale
        methods.add_method("SetIgnoreParentScale", |_, _this, _ignore: bool| {
            // No-op in simulation
            Ok(())
        });

        // GetIgnoreParentScale() - get whether frame ignores parent scale
        methods.add_method("GetIgnoreParentScale", |_, _this, ()| {
            Ok(false)
        });

        // SetIgnoreParentAlpha(ignore) - set whether frame ignores parent alpha
        methods.add_method("SetIgnoreParentAlpha", |_, _this, _ignore: bool| {
            Ok(())
        });

        // GetIgnoreParentAlpha() - get whether frame ignores parent alpha
        methods.add_method("GetIgnoreParentAlpha", |_, _this, ()| {
            Ok(false)
        });

        // GetAttribute(name) - get a named attribute from the frame
        methods.add_method("GetAttribute", |lua, this, name: String| {
            // First check for table attributes stored in Lua
            let table_attrs: Option<mlua::Table> = lua.globals().get("__frame_table_attributes").ok();
            if let Some(attrs) = table_attrs {
                let key = format!("{}_{}", this.id, name);
                let table_val: Value = attrs.get(key.as_str()).unwrap_or(Value::Nil);
                if !matches!(table_val, Value::Nil) {
                    return Ok(table_val);
                }
            }

            // Fall back to non-table attributes stored in Rust
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(attr) = frame.attributes.get(&name) {
                    return match attr {
                        AttributeValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
                        AttributeValue::Number(n) => Ok(Value::Number(*n)),
                        AttributeValue::Boolean(b) => Ok(Value::Boolean(*b)),
                        AttributeValue::Nil => Ok(Value::Nil),
                    };
                }
            }
            Ok(Value::Nil)
        });

        // SetAttribute(name, value) - set a named attribute on the frame
        methods.add_method("SetAttribute", |lua, this, (name, value): (String, Value)| {
            // For table values, store in a Lua table to preserve the reference
            if matches!(&value, Value::Table(_)) {
                // Ensure __frame_table_attributes exists
                let table_attrs: mlua::Table = lua.globals().get("__frame_table_attributes")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_table_attributes", t.clone()).ok();
                        t
                    });
                let key = format!("{}_{}", this.id, name);
                table_attrs.set(key, value.clone())?;
            } else {
                // Store simple types in Rust
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    let attr = match &value {
                        Value::Nil => AttributeValue::Nil,
                        Value::Boolean(b) => AttributeValue::Boolean(*b),
                        Value::Integer(i) => AttributeValue::Number(*i as f64),
                        Value::Number(n) => AttributeValue::Number(*n),
                        Value::String(s) => AttributeValue::String(s.to_str().map(|s| s.to_string()).unwrap_or_default()),
                        _ => AttributeValue::Nil,
                    };
                    if matches!(attr, AttributeValue::Nil) && matches!(value, Value::Nil) {
                        frame.attributes.remove(&name);
                        // Also remove from table attributes if it exists there
                        if let Ok(table_attrs) = lua.globals().get::<mlua::Table>("__frame_table_attributes") {
                            let key = format!("{}_{}", this.id, name);
                            table_attrs.set(key, Value::Nil).ok();
                        }
                    } else {
                        frame.attributes.insert(name.clone(), attr);
                    }
                }
            }

            // Trigger OnAttributeChanged script if one exists
            let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();
            if let Some(table) = scripts_table {
                let frame_key = format!("{}_OnAttributeChanged", this.id);
                let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();
                if let Some(handler) = handler {
                    // Get frame userdata
                    let frame_ref_key = format!("__frame_{}", this.id);
                    let frame_ud: Value = lua.globals().get(frame_ref_key.as_str()).unwrap_or(Value::Nil);
                    // Call handler with (self, name, value)
                    let name_str = lua.create_string(&name)?;
                    let _ = handler.call::<()>((frame_ud, name_str, value));
                }
            }
            Ok(())
        });

        // ClearAttributes() - remove all attributes from the frame
        methods.add_method("ClearAttributes", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.attributes.clear();
            }
            Ok(())
        });

        // SetFrameRef(label, frame) - Store a reference to another frame
        methods.add_method("SetFrameRef", |_, _this, (_label, _frame): (String, Value)| {
            // Frame references are used for secure frame communication
            // Just a stub for simulation
            Ok(())
        });

        // GetFrameRef(label) - Get a stored frame reference
        methods.add_method("GetFrameRef", |_, _this, _label: String| {
            Ok(Value::Nil)
        });

        // ===== Button Methods =====

        // SetNormalFontObject(fontObject) - Set font for normal state
        methods.add_method("SetNormalFontObject", |lua, this, font_object: Value| {
            // Store in global table by frame ID
            let store: mlua::Table = lua.load("_G.__button_font_objects = _G.__button_font_objects or {}; return _G.__button_font_objects").eval()?;
            let key = format!("{}:normal", this.id);
            store.set(key, font_object)?;
            Ok(())
        });

        // GetNormalFontObject() - Get font for normal state
        methods.add_method("GetNormalFontObject", |lua, this, ()| {
            let store: mlua::Table = lua.load("return _G.__button_font_objects or {}").eval()?;
            let key = format!("{}:normal", this.id);
            let font: Value = store.get(key)?;
            Ok(font)
        });

        // SetHighlightFontObject(fontObject) - Set font for highlight state
        methods.add_method("SetHighlightFontObject", |lua, this, font_object: Value| {
            let store: mlua::Table = lua.load("_G.__button_font_objects = _G.__button_font_objects or {}; return _G.__button_font_objects").eval()?;
            let key = format!("{}:highlight", this.id);
            store.set(key, font_object)?;
            Ok(())
        });

        // GetHighlightFontObject() - Get font for highlight state
        methods.add_method("GetHighlightFontObject", |lua, this, ()| {
            let store: mlua::Table = lua.load("return _G.__button_font_objects or {}").eval()?;
            let key = format!("{}:highlight", this.id);
            let font: Value = store.get(key)?;
            Ok(font)
        });

        // SetDisabledFontObject(fontObject) - Set font for disabled state
        methods.add_method("SetDisabledFontObject", |lua, this, font_object: Value| {
            let store: mlua::Table = lua.load("_G.__button_font_objects = _G.__button_font_objects or {}; return _G.__button_font_objects").eval()?;
            let key = format!("{}:disabled", this.id);
            store.set(key, font_object)?;
            Ok(())
        });

        // GetDisabledFontObject() - Get font for disabled state
        methods.add_method("GetDisabledFontObject", |lua, this, ()| {
            let store: mlua::Table = lua.load("return _G.__button_font_objects or {}").eval()?;
            let key = format!("{}:disabled", this.id);
            let font: Value = store.get(key)?;
            Ok(font)
        });

        // SetPushedTextOffset(x, y) - Set text offset when button is pushed
        methods.add_method(
            "SetPushedTextOffset",
            |_, _this, (_x, _y): (f64, f64)| Ok(()),
        );

        // GetPushedTextOffset() - Get text offset when button is pushed
        methods.add_method("GetPushedTextOffset", |_, _this, ()| Ok((0.0_f64, 0.0_f64)));

        // GetNormalTexture() - Get or create the normal state texture
        // In WoW, this returns the texture object, creating it if necessary
        // Always calls get_or_create_button_texture to ensure anchors are set
        methods.add_method("GetNormalTexture", |lua, this, ()| {
            let tex_id = get_or_create_button_texture(&mut this.state.borrow_mut(), this.id, "NormalTexture");
            let handle = FrameHandle {
                id: tex_id,
                state: Rc::clone(&this.state),
            };
            lua.create_userdata(handle).map(Value::UserData)
        });

        // GetHighlightTexture() - Get or create the highlight state texture
        methods.add_method("GetHighlightTexture", |lua, this, ()| {
            let tex_id = get_or_create_button_texture(&mut this.state.borrow_mut(), this.id, "HighlightTexture");
            let handle = FrameHandle {
                id: tex_id,
                state: Rc::clone(&this.state),
            };
            lua.create_userdata(handle).map(Value::UserData)
        });

        // GetPushedTexture() - Get or create the pushed state texture
        methods.add_method("GetPushedTexture", |lua, this, ()| {
            let tex_id = get_or_create_button_texture(&mut this.state.borrow_mut(), this.id, "PushedTexture");
            let handle = FrameHandle {
                id: tex_id,
                state: Rc::clone(&this.state),
            };
            lua.create_userdata(handle).map(Value::UserData)
        });

        // GetDisabledTexture() - Get or create the disabled state texture
        methods.add_method("GetDisabledTexture", |lua, this, ()| {
            let tex_id = get_or_create_button_texture(&mut this.state.borrow_mut(), this.id, "DisabledTexture");
            let handle = FrameHandle {
                id: tex_id,
                state: Rc::clone(&this.state),
            };
            lua.create_userdata(handle).map(Value::UserData)
        });

        // Helper to create a button texture child if it doesn't exist
        // Also ensures existing textures have proper anchors to fill the button
        fn get_or_create_button_texture(
            state: &mut crate::lua_api::SimState,
            button_id: u64,
            key: &str,
        ) -> u64 {
            // Check if texture child already exists - copy the id to avoid borrow conflict
            let existing_tex_id = state.widgets.get(button_id)
                .and_then(|frame| frame.children_keys.get(key).copied());

            if let Some(tex_id) = existing_tex_id {
                // Ensure existing texture has anchors to fill parent
                if let Some(tex) = state.widgets.get_mut(tex_id) {
                    if tex.anchors.is_empty() {
                        // Add anchors to fill parent button
                        tex.anchors.push(crate::widget::Anchor {
                            point: crate::widget::AnchorPoint::TopLeft,
                            relative_to: None,
                            relative_to_id: Some(button_id as usize),
                            relative_point: crate::widget::AnchorPoint::TopLeft,
                            x_offset: 0.0,
                            y_offset: 0.0,
                        });
                        tex.anchors.push(crate::widget::Anchor {
                            point: crate::widget::AnchorPoint::BottomRight,
                            relative_to: None,
                            relative_to_id: Some(button_id as usize),
                            relative_point: crate::widget::AnchorPoint::BottomRight,
                            x_offset: 0.0,
                            y_offset: 0.0,
                        });
                    }
                }
                return tex_id;
            }

            // Create new texture child
            let mut texture = Frame::new(WidgetType::Texture, None, Some(button_id));
            // Set texture to fill parent (SetAllPoints behavior)
            texture.anchors.push(crate::widget::Anchor {
                point: crate::widget::AnchorPoint::TopLeft,
                relative_to: None,
                relative_to_id: Some(button_id as usize),
                relative_point: crate::widget::AnchorPoint::TopLeft,
                x_offset: 0.0,
                y_offset: 0.0,
            });
            texture.anchors.push(crate::widget::Anchor {
                point: crate::widget::AnchorPoint::BottomRight,
                relative_to: None,
                relative_to_id: Some(button_id as usize),
                relative_point: crate::widget::AnchorPoint::BottomRight,
                x_offset: 0.0,
                y_offset: 0.0,
            });
            let texture_id = texture.id;

            state.widgets.register(texture);
            state.widgets.add_child(button_id, texture_id);

            // Store in children_keys
            if let Some(frame) = state.widgets.get_mut(button_id) {
                frame.children_keys.insert(key.to_string(), texture_id);
            }

            texture_id
        }

        // SetNormalTexture(texture) - Set texture for normal state
        methods.add_method("SetNormalTexture", |_, this, texture: Value| {
            let path = match texture {
                Value::String(s) => Some(s.to_str()?.to_string()),
                Value::Nil => None,
                _ => None,
            };
            let mut state = this.state.borrow_mut();

            // Store path on button for renderer
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.normal_texture = path.clone();
            }

            // Create/get texture child and set its texture
            let tex_id = get_or_create_button_texture(&mut state, this.id, "NormalTexture");
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                tex.texture = path;
            }

            Ok(())
        });

        // SetHighlightTexture(texture) - Set texture for highlight state
        methods.add_method("SetHighlightTexture", |_, this, texture: Value| {
            let path = match texture {
                Value::String(s) => Some(s.to_str()?.to_string()),
                Value::Nil => None,
                _ => None,
            };
            let mut state = this.state.borrow_mut();

            // Store path on button for renderer
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.highlight_texture = path.clone();
            }

            // Create/get texture child and set its texture
            let tex_id = get_or_create_button_texture(&mut state, this.id, "HighlightTexture");
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                tex.texture = path;
            }

            Ok(())
        });

        // SetPushedTexture(texture) - Set texture for pushed state
        methods.add_method("SetPushedTexture", |_, this, texture: Value| {
            let path = match texture {
                Value::String(s) => Some(s.to_str()?.to_string()),
                Value::Nil => None,
                _ => None,
            };
            let mut state = this.state.borrow_mut();

            // Store path on button for renderer
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.pushed_texture = path.clone();
            }

            // Create/get texture child and set its texture
            let tex_id = get_or_create_button_texture(&mut state, this.id, "PushedTexture");
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                tex.texture = path;
            }

            Ok(())
        });

        // SetDisabledTexture(texture) - Set texture for disabled state
        methods.add_method("SetDisabledTexture", |_, this, texture: Value| {
            let path = match texture {
                Value::String(s) => Some(s.to_str()?.to_string()),
                Value::Nil => None,
                _ => None,
            };
            let mut state = this.state.borrow_mut();

            // Store path on button for renderer
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.disabled_texture = path.clone();
            }

            // Create/get texture child and set its texture
            let tex_id = get_or_create_button_texture(&mut state, this.id, "DisabledTexture");
            if let Some(tex) = state.widgets.get_mut(tex_id) {
                tex.texture = path;
            }

            Ok(())
        });

        // SetLeftTexture(texture) - Set left cap texture for 3-slice buttons
        methods.add_method("SetLeftTexture", |_, this, texture: Value| {
            let path = match texture {
                Value::String(s) => Some(s.to_str()?.to_string()),
                Value::Nil => None,
                _ => None,
            };
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.left_texture = path;
            }
            Ok(())
        });

        // SetMiddleTexture(texture) - Set middle (stretchable) texture for 3-slice buttons
        methods.add_method("SetMiddleTexture", |_, this, texture: Value| {
            let path = match texture {
                Value::String(s) => Some(s.to_str()?.to_string()),
                Value::Nil => None,
                _ => None,
            };
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.middle_texture = path;
            }
            Ok(())
        });

        // SetRightTexture(texture) - Set right cap texture for 3-slice buttons
        methods.add_method("SetRightTexture", |_, this, texture: Value| {
            let path = match texture {
                Value::String(s) => Some(s.to_str()?.to_string()),
                Value::Nil => None,
                _ => None,
            };
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.right_texture = path;
            }
            Ok(())
        });

        // GetFontString() - Get button's text font string
        methods.add_method("GetFontString", |lua, this, ()| {
            // Return stub fontstring - buttons have text layers
            // Note: All methods take (_self, ...) because Lua passes self when using colon syntax
            let state = this.state.borrow();
            if let Some(_frame) = state.widgets.get(this.id) {
                // Create a more complete stub fontstring with common methods
                let fs = lua.create_table()?;
                fs.set("SetText", lua.create_function(|_, (_self, _text): (Value, Option<String>)| Ok(()))?)?;
                fs.set("SetFormattedText", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                fs.set("GetText", lua.create_function(|_, _self: Value| Ok(""))?)?;
                fs.set("SetFont", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                fs.set("SetTextColor", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                fs.set("SetJustifyH", lua.create_function(|_, (_self, _justify): (Value, String)| Ok(()))?)?;
                fs.set("SetJustifyV", lua.create_function(|_, (_self, _justify): (Value, String)| Ok(()))?)?;
                fs.set("GetStringWidth", lua.create_function(|_, _self: Value| Ok(0.0_f64))?)?;
                fs.set("GetStringHeight", lua.create_function(|_, _self: Value| Ok(12.0_f64))?)?;
                fs.set("GetUnboundedStringWidth", lua.create_function(|_, _self: Value| Ok(0.0_f64))?)?;
                // Additional methods for Cell compatibility
                fs.set("SetWordWrap", lua.create_function(|_, (_self, _wrap): (Value, bool)| Ok(()))?)?;
                fs.set("GetWordWrap", lua.create_function(|_, _self: Value| Ok(false))?)?;
                fs.set("SetNonSpaceWrap", lua.create_function(|_, (_self, _wrap): (Value, bool)| Ok(()))?)?;
                fs.set("GetNonSpaceWrap", lua.create_function(|_, _self: Value| Ok(false))?)?;
                fs.set("SetPoint", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                fs.set("ClearAllPoints", lua.create_function(|_, _self: Value| Ok(()))?)?;
                fs.set("SetWidth", lua.create_function(|_, (_self, _w): (Value, f64)| Ok(()))?)?;
                fs.set("SetHeight", lua.create_function(|_, (_self, _h): (Value, f64)| Ok(()))?)?;
                fs.set("SetSize", lua.create_function(|_, (_self, _w, _h): (Value, f64, f64)| Ok(()))?)?;
                fs.set("GetWidth", lua.create_function(|_, _self: Value| Ok(0.0_f64))?)?;
                fs.set("GetHeight", lua.create_function(|_, _self: Value| Ok(12.0_f64))?)?;
                fs.set("SetAlpha", lua.create_function(|_, (_self, _alpha): (Value, f64)| Ok(()))?)?;
                fs.set("GetAlpha", lua.create_function(|_, _self: Value| Ok(1.0_f64))?)?;
                fs.set("Show", lua.create_function(|_, _self: Value| Ok(()))?)?;
                fs.set("Hide", lua.create_function(|_, _self: Value| Ok(()))?)?;
                fs.set("IsShown", lua.create_function(|_, _self: Value| Ok(true))?)?;
                fs.set("SetShadowColor", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
                fs.set("SetShadowOffset", lua.create_function(|_, (_self, _x, _y): (Value, f64, f64)| Ok(()))?)?;
                fs.set("SetSpacing", lua.create_function(|_, (_self, _spacing): (Value, f64)| Ok(()))?)?;
                fs.set("SetMaxLines", lua.create_function(|_, (_self, _lines): (Value, i32)| Ok(()))?)?;
                Ok(Value::Table(fs))
            } else {
                Ok(Value::Nil)
            }
        });

        // SetFontString(fontstring) - Set button's text font string
        methods.add_method("SetFontString", |_, _this, _fontstring: Value| {
            Ok(())
        });

        // SetEnabled(enabled) - Enable/disable button
        methods.add_method("SetEnabled", |_, _this, _enabled: bool| {
            Ok(())
        });

        // Enable() - Enable button
        methods.add_method("Enable", |_, _this, ()| {
            Ok(())
        });

        // Disable() - Disable button
        methods.add_method("Disable", |_, _this, ()| {
            Ok(())
        });

        // IsEnabled() - Check if button is enabled
        methods.add_method("IsEnabled", |_, _this, ()| {
            Ok(true)
        });

        // Click() - Simulate button click
        methods.add_method("Click", |_, _this, ()| {
            Ok(())
        });

        // RegisterForClicks(...) - Register which mouse buttons trigger clicks
        methods.add_method("RegisterForClicks", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetButtonState(state, locked) - Set button visual state
        methods.add_method("SetButtonState", |_, _this, (_state, _locked): (String, Option<bool>)| {
            Ok(())
        });

        // GetButtonState() - Get button visual state
        methods.add_method("GetButtonState", |_, _this, ()| {
            Ok("NORMAL".to_string())
        });

        // ===== GameTooltip-specific methods =====
        // These methods are used by GameTooltip and similar tooltip frames

        // SetOwner(owner, anchor, x, y) - Set the tooltip's owner and anchor
        methods.add_method("SetOwner", |_, _this, _args: mlua::MultiValue| {
            // args: owner frame, anchor string, optional x, y offsets
            Ok(())
        });

        // ClearLines() - Clear all text lines from the tooltip
        methods.add_method("ClearLines", |_, _this, ()| {
            Ok(())
        });

        // AddLine(text, r, g, b, wrap) - Add a line of text
        methods.add_method("AddLine", |_, _this, _args: mlua::MultiValue| {
            // args: text, r (0-1), g (0-1), b (0-1), wrap (bool)
            Ok(())
        });

        // AddDoubleLine(leftText, rightText, lR, lG, lB, rR, rG, rB) - Add two-column line
        methods.add_method("AddDoubleLine", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // AddMessage(text, r, g, b, id, holdTime) - Add message to a scrolling message frame
        methods.add_method("AddMessage", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // AddMsg(text, ...) - Alias for AddMessage (used by some addons like DBM)
        methods.add_method("AddMsg", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetSpellByID(spellID) - Set tooltip to show spell info
        methods.add_method("SetSpellByID", |_, _this, _spell_id: i32| {
            Ok(())
        });

        // SetItemByID(itemID) - Set tooltip to show item info
        methods.add_method("SetItemByID", |_, _this, _item_id: i32| {
            Ok(())
        });

        // SetHyperlink(link) - Set tooltip from a hyperlink
        methods.add_method("SetHyperlink", |_, _this, _link: String| {
            Ok(())
        });

        // SetUnitBuff(unit, index, filter) - Set tooltip to show unit buff
        methods.add_method("SetUnitBuff", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetUnitDebuff(unit, index, filter) - Set tooltip to show unit debuff
        methods.add_method("SetUnitDebuff", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetUnitAura(unit, index, filter) - Set tooltip to show unit aura
        methods.add_method("SetUnitAura", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetUnitBuffByAuraInstanceID(unit, auraInstanceID, filter)
        methods.add_method("SetUnitBuffByAuraInstanceID", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetUnitDebuffByAuraInstanceID(unit, auraInstanceID, filter)
        methods.add_method("SetUnitDebuffByAuraInstanceID", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // NumLines() - Get number of lines in tooltip
        methods.add_method("NumLines", |_, _this, ()| {
            Ok(0)
        });

        // GetUnit() - Get the unit this tooltip is showing info for
        methods.add_method("GetUnit", |_, _this, ()| -> Result<(Option<String>, Option<String>)> {
            Ok((None, None)) // Returns name, unit
        });

        // GetSpell() - Get the spell this tooltip is showing info for
        methods.add_method("GetSpell", |_, _this, ()| -> Result<(Option<String>, Option<i32>)> {
            Ok((None, None)) // Returns name, spellID
        });

        // GetItem() - Get the item this tooltip is showing info for
        methods.add_method("GetItem", |_, _this, ()| -> Result<(Option<String>, Option<String>)> {
            Ok((None, None)) // Returns name, link
        });

        // SetMinimumWidth(width) - Set minimum tooltip width
        methods.add_method("SetMinimumWidth", |_, _this, _width: f32| {
            Ok(())
        });

        // GetMinimumWidth() - Get minimum tooltip width
        methods.add_method("GetMinimumWidth", |_, _this, ()| {
            Ok(0.0_f32)
        });

        // SetPadding(width) - Set tooltip padding
        methods.add_method("SetPadding", |_, _this, _width: f32| {
            Ok(())
        });

        // AddTexture(texture) - Add a texture to the tooltip
        methods.add_method("AddTexture", |_, _this, _texture: String| {
            Ok(())
        });

        // SetText(text, r, g, b, wrap) - Set the tooltip's main text
        methods.add_method_mut("SetText", |_, this, args: mlua::MultiValue| {
            let mut args_iter = args.into_iter();
            if let Some(Value::String(text)) = args_iter.next() {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.text = Some(text.to_string_lossy().to_string());
                }
            }
            Ok(())
        });

        // AppendText(text) - Append text to the tooltip
        methods.add_method("AppendText", |_, _this, _text: String| {
            Ok(())
        });

        // IsOwned(frame) - Check if tooltip is owned by a frame
        methods.add_method("IsOwned", |_, _this, _frame: Value| {
            Ok(false)
        });

        // FadeOut() - Fade out the tooltip
        methods.add_method("FadeOut", |_, _this, ()| {
            Ok(())
        });

        // ===== EditBox methods =====
        methods.add_method("SetFocus", |_, this, ()| {
            if let Ok(mut s) = this.state.try_borrow_mut() {
                s.focused_frame_id = Some(this.id);
            }
            Ok(())
        });
        methods.add_method("ClearFocus", |_, this, ()| {
            if let Ok(mut s) = this.state.try_borrow_mut() {
                if s.focused_frame_id == Some(this.id) {
                    s.focused_frame_id = None;
                }
            }
            Ok(())
        });
        methods.add_method("HasFocus", |_, this, ()| {
            if let Ok(s) = this.state.try_borrow() {
                return Ok(s.focused_frame_id == Some(this.id));
            }
            Ok(false)
        });
        methods.add_method("SetCursorPosition", |_, _this, _pos: i32| Ok(()));
        methods.add_method("GetCursorPosition", |_, _this, ()| Ok(0));
        methods.add_method("HighlightText", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("Insert", |_, _this, _text: String| Ok(()));
        methods.add_method("SetMaxLetters", |_, _this, _max: i32| Ok(()));
        methods.add_method("GetMaxLetters", |_, _this, ()| Ok(0));
        methods.add_method("SetMaxBytes", |_, _this, _max: i32| Ok(()));
        methods.add_method("GetMaxBytes", |_, _this, ()| Ok(0));
        methods.add_method("SetNumber", |_, this, n: f64| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.text = Some(n.to_string());
            }
            Ok(())
        });
        methods.add_method("GetNumber", |_, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(text) = &frame.text {
                    return Ok(text.parse::<f64>().unwrap_or(0.0));
                }
            }
            Ok(0.0)
        });
        methods.add_method("SetMultiLine", |_, _this, _multi: bool| Ok(()));
        methods.add_method("IsMultiLine", |_, _this, ()| Ok(false));
        methods.add_method("SetAutoFocus", |_, _this, _auto: bool| Ok(()));
        methods.add_method("SetNumeric", |_, _this, _numeric: bool| Ok(()));
        methods.add_method("IsNumeric", |_, _this, ()| Ok(false));
        methods.add_method("SetPassword", |_, _this, _pw: bool| Ok(()));
        methods.add_method("IsPassword", |_, _this, ()| Ok(false));
        methods.add_method("SetBlinkSpeed", |_, _this, _speed: f64| Ok(()));
        methods.add_method("SetHistoryLines", |_, _this, _lines: i32| Ok(()));
        methods.add_method("AddHistoryLine", |_, _this, _text: String| Ok(()));
        methods.add_method("GetHistoryLines", |_, _this, ()| Ok(0));
        methods.add_method("SetTextInsets", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("GetTextInsets", |_, _this, ()| Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32)));

        // ===== Slider methods =====
        methods.add_method("SetMinMaxValues", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("GetMinMaxValues", |_, _this, ()| Ok((0.0_f64, 100.0_f64)));
        methods.add_method("SetValue", |_, _this, _value: f64| Ok(()));
        methods.add_method("GetValue", |_, _this, ()| Ok(0.0_f64));
        methods.add_method("SetValueStep", |_, _this, _step: f64| Ok(()));
        methods.add_method("GetValueStep", |_, _this, ()| Ok(1.0_f64));
        methods.add_method("SetOrientation", |_, _this, _orientation: String| Ok(()));
        methods.add_method("GetOrientation", |_, _this, ()| Ok("HORIZONTAL".to_string()));
        methods.add_method("SetThumbTexture", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("GetThumbTexture", |lua, this, ()| {
            // Create or return the thumb texture for slider
            let texture_key = format!("__frame_{}_ThumbTexture", this.id);
            if let Ok(existing) = lua.globals().get::<Value>(texture_key.as_str()) {
                if !matches!(existing, Value::Nil) {
                    return Ok(existing);
                }
            }

            let texture = Frame::new(WidgetType::Texture, None, Some(this.id));
            let texture_id = texture.id;

            {
                let mut state = this.state.borrow_mut();
                state.widgets.register(texture);
                state.widgets.add_child(this.id, texture_id);
            }

            let handle = FrameHandle {
                id: texture_id,
                state: Rc::clone(&this.state),
            };

            let ud = lua.create_userdata(handle)?;
            lua.globals().set(texture_key.as_str(), ud.clone())?;

            let frame_key = format!("__frame_{}", texture_id);
            lua.globals().set(frame_key.as_str(), ud.clone())?;

            Ok(Value::UserData(ud))
        });
        methods.add_method("SetObeyStepOnDrag", |_, _this, _obey: bool| Ok(()));
        methods.add_method("SetStepsPerPage", |_, _this, _steps: i32| Ok(()));
        methods.add_method("GetStepsPerPage", |_, _this, ()| Ok(1));

        // ===== StatusBar methods =====
        methods.add_method("SetStatusBarTexture", |_, _this, _texture: Value| Ok(()));
        methods.add_method("GetStatusBarTexture", |lua, this, ()| {
            // Create or return the status bar texture child
            // Check if we already have a __StatusBarTexture child
            let texture_key = format!("__frame_{}_StatusBarTexture", this.id);
            if let Ok(existing) = lua.globals().get::<Value>(texture_key.as_str()) {
                if !matches!(existing, Value::Nil) {
                    return Ok(existing);
                }
            }

            // Create a new texture for this status bar
            let texture = Frame::new(WidgetType::Texture, None, Some(this.id));
            let texture_id = texture.id;

            {
                let mut state = this.state.borrow_mut();
                state.widgets.register(texture);
                state.widgets.add_child(this.id, texture_id);
            }

            let handle = FrameHandle {
                id: texture_id,
                state: Rc::clone(&this.state),
            };

            let ud = lua.create_userdata(handle)?;
            lua.globals().set(texture_key.as_str(), ud.clone())?;

            // Also store as the generic frame key
            let frame_key = format!("__frame_{}", texture_id);
            lua.globals().set(frame_key.as_str(), ud.clone())?;

            Ok(Value::UserData(ud))
        });
        methods.add_method("SetStatusBarColor", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("GetStatusBarColor", |_, _this, ()| Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32)));
        methods.add_method("SetRotatesTexture", |_, _this, _rotates: bool| Ok(()));
        methods.add_method("SetReverseFill", |_, _this, _reverse: bool| Ok(()));
        methods.add_method("SetFillStyle", |_, _this, _style: String| Ok(()));

        // ===== CheckButton methods =====
        methods.add_method("SetChecked", |_, this, checked: bool| {
            // Store checked state in attributes
            {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.attributes.insert("__checked".to_string(), AttributeValue::Boolean(checked));
                }
                // Also toggle CheckedTexture visibility if it exists
                if let Some(frame) = state.widgets.get(this.id) {
                    if let Some(&checked_tex_id) = frame.children_keys.get("CheckedTexture") {
                        if let Some(tex) = state.widgets.get_mut(checked_tex_id) {
                            tex.visible = checked;
                        }
                    }
                }
            }
            Ok(())
        });
        methods.add_method("GetChecked", |_, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(AttributeValue::Boolean(checked)) = frame.attributes.get("__checked") {
                    return Ok(*checked);
                }
            }
            Ok(false)
        });
        methods.add_method("GetCheckedTexture", |lua, this, ()| {
            let texture_key = format!("__frame_{}_CheckedTexture", this.id);
            if let Ok(existing) = lua.globals().get::<Value>(texture_key.as_str()) {
                if !matches!(existing, Value::Nil) {
                    return Ok(existing);
                }
            }

            let texture = Frame::new(WidgetType::Texture, None, Some(this.id));
            let texture_id = texture.id;

            {
                let mut state = this.state.borrow_mut();
                state.widgets.register(texture);
                state.widgets.add_child(this.id, texture_id);
            }

            let handle = FrameHandle {
                id: texture_id,
                state: Rc::clone(&this.state),
            };

            let ud = lua.create_userdata(handle)?;
            lua.globals().set(texture_key.as_str(), ud.clone())?;

            let frame_key = format!("__frame_{}", texture_id);
            lua.globals().set(frame_key.as_str(), ud.clone())?;

            Ok(Value::UserData(ud))
        });
        methods.add_method("SetCheckedTexture", |_, _this, _texture: Value| Ok(()));

        // ===== Cooldown methods =====
        methods.add_method("SetCooldown", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("SetCooldownUNIX", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("GetCooldownTimes", |_, _this, ()| Ok((0.0_f64, 0.0_f64)));
        methods.add_method("SetSwipeColor", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("SetDrawSwipe", |_, _this, _draw: bool| Ok(()));
        methods.add_method("SetDrawEdge", |_, _this, _draw: bool| Ok(()));
        methods.add_method("SetDrawBling", |_, _this, _draw: bool| Ok(()));
        methods.add_method("SetReverse", |_, _this, _reverse: bool| Ok(()));
        methods.add_method("SetHideCountdownNumbers", |_, _this, _hide: bool| Ok(()));
        // Note: Clear() for Cooldown frames is handled in __index to avoid conflicts
        // with addons that use frame.Clear as a field

        // ===== ScrollFrame methods =====
        methods.add_method("SetScrollChild", |_, _this, _child: Value| Ok(()));
        methods.add_method("GetScrollChild", |_, _this, ()| Ok(Value::Nil));
        methods.add_method("SetHorizontalScroll", |_, _this, _offset: f64| Ok(()));
        methods.add_method("GetHorizontalScroll", |_, _this, ()| Ok(0.0_f64));
        methods.add_method("SetVerticalScroll", |_, _this, _offset: f64| Ok(()));
        methods.add_method("GetVerticalScroll", |_, _this, ()| Ok(0.0_f64));
        methods.add_method("GetHorizontalScrollRange", |_, _this, ()| Ok(0.0_f64));
        methods.add_method("GetVerticalScrollRange", |_, _this, ()| Ok(0.0_f64));
        methods.add_method("UpdateScrollChildRect", |_, _this, ()| Ok(()));

        // ===== Model methods =====
        methods.add_method("SetModel", |_, _this, _path: String| Ok(()));
        methods.add_method("GetModel", |_, _this, ()| -> Result<Option<String>> { Ok(None) });
        methods.add_method("SetModelScale", |_, _this, _scale: f64| Ok(()));
        methods.add_method("GetModelScale", |_, _this, ()| Ok(1.0_f64));
        methods.add_method("SetPosition", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("GetPosition", |_, _this, ()| Ok((0.0_f64, 0.0_f64, 0.0_f64)));
        methods.add_method("SetFacing", |_, _this, _radians: f64| Ok(()));
        methods.add_method("GetFacing", |_, _this, ()| Ok(0.0_f64));
        methods.add_method("SetUnit", |_, _this, _unit: Option<String>| Ok(()));
        methods.add_method("SetAutoDress", |_, _this, _auto_dress: bool| Ok(()));
        methods.add_method("SetDisplayInfo", |_, _this, _display_id: i32| Ok(()));
        methods.add_method("SetCreature", |_, _this, _creature_id: i32| Ok(()));
        methods.add_method("SetAnimation", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("SetCamDistanceScale", |_, _this, _scale: f64| Ok(()));
        methods.add_method("GetCamDistanceScale", |_, _this, ()| Ok(1.0_f64));
        methods.add_method("SetCamera", |_, _this, _camera_id: i32| Ok(()));
        methods.add_method("SetPortraitZoom", |_, _this, _zoom: f64| Ok(()));
        methods.add_method("SetDesaturation", |_, _this, _desat: f64| Ok(()));
        methods.add_method("SetRotation", |_, _this, _radians: f64| Ok(()));
        methods.add_method("SetLight", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("SetSequence", |_, _this, _sequence: i32| Ok(()));
        methods.add_method("SetSequenceTime", |_, _this, (_seq, _time): (i32, i32)| Ok(()));
        methods.add_method("ClearModel", |_, _this, ()| Ok(()));
        methods.add_method(
            "TransitionToModelSceneID",
            |_, _this, _args: mlua::MultiValue| Ok(()),
        );
        methods.add_method("SetFromModelSceneID", |_, _this, _scene_id: i32| Ok(()));
        methods.add_method("GetModelSceneID", |_, _this, ()| Ok(0i32));

        // ===== ColorSelect methods =====
        // SetColorRGB(r, g, b) - Set the RGB color
        methods.add_method("SetColorRGB", |_, this, (r, g, b): (f64, f64, f64)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.attributes.insert("colorR".to_string(), AttributeValue::Number(r));
                frame.attributes.insert("colorG".to_string(), AttributeValue::Number(g));
                frame.attributes.insert("colorB".to_string(), AttributeValue::Number(b));
            }
            Ok(())
        });

        // GetColorRGB() - Get the RGB color
        methods.add_method("GetColorRGB", |_, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                let get_num = |key: &str| -> f64 {
                    match frame.attributes.get(key) {
                        Some(AttributeValue::Number(n)) => *n,
                        _ => 1.0,
                    }
                };
                let r = get_num("colorR");
                let g = get_num("colorG");
                let b = get_num("colorB");
                return Ok((r, g, b));
            }
            Ok((1.0, 1.0, 1.0))
        });

        // SetColorHSV(h, s, v) - Set the HSV color
        methods.add_method("SetColorHSV", |_, this, (h, s, v): (f64, f64, f64)| {
            // Convert HSV to RGB for storage
            let h = h % 360.0;
            let c = v * s;
            let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
            let m = v - c;

            let (r1, g1, b1) = if h < 60.0 {
                (c, x, 0.0)
            } else if h < 120.0 {
                (x, c, 0.0)
            } else if h < 180.0 {
                (0.0, c, x)
            } else if h < 240.0 {
                (0.0, x, c)
            } else if h < 300.0 {
                (x, 0.0, c)
            } else {
                (c, 0.0, x)
            };

            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.attributes.insert("colorR".to_string(), AttributeValue::Number(r1 + m));
                frame.attributes.insert("colorG".to_string(), AttributeValue::Number(g1 + m));
                frame.attributes.insert("colorB".to_string(), AttributeValue::Number(b1 + m));
                // Also store HSV for GetColorHSV
                frame.attributes.insert("colorH".to_string(), AttributeValue::Number(h));
                frame.attributes.insert("colorS".to_string(), AttributeValue::Number(s));
                frame.attributes.insert("colorV".to_string(), AttributeValue::Number(v));
            }
            Ok(())
        });

        // GetColorHSV() - Get the HSV color
        methods.add_method("GetColorHSV", |_, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                let get_num = |key: &str| -> Option<f64> {
                    match frame.attributes.get(key) {
                        Some(AttributeValue::Number(n)) => Some(*n),
                        _ => None,
                    }
                };
                // Check if we have stored HSV values
                if let (Some(h), Some(s), Some(v)) = (
                    get_num("colorH"),
                    get_num("colorS"),
                    get_num("colorV"),
                ) {
                    return Ok((h, s, v));
                }
                // Otherwise convert from RGB
                let r: f64 = get_num("colorR").unwrap_or(1.0);
                let g: f64 = get_num("colorG").unwrap_or(1.0);
                let b: f64 = get_num("colorB").unwrap_or(1.0);

                let max = r.max(g).max(b);
                let min = r.min(g).min(b);
                let delta = max - min;

                let v = max;
                let s = if max == 0.0 { 0.0 } else { delta / max };
                let h = if delta == 0.0 {
                    0.0
                } else if max == r {
                    60.0 * (((g - b) / delta) % 6.0)
                } else if max == g {
                    60.0 * ((b - r) / delta + 2.0)
                } else {
                    60.0 * ((r - g) / delta + 4.0)
                };
                let h = if h < 0.0 { h + 360.0 } else { h };

                return Ok((h, s, v));
            }
            Ok((0.0, 0.0, 1.0))
        });

        // ===== Frame dragging/moving =====
        methods.add_method("StartMoving", |_, this, ()| {
            if let Ok(mut s) = this.state.try_borrow_mut() {
                if let Some(frame) = s.widgets.get_mut(this.id) {
                    if frame.movable {
                        frame.is_moving = true;
                    }
                }
            }
            Ok(())
        });
        methods.add_method("StopMovingOrSizing", |_, this, ()| {
            if let Ok(mut s) = this.state.try_borrow_mut() {
                if let Some(frame) = s.widgets.get_mut(this.id) {
                    frame.is_moving = false;
                }
            }
            Ok(())
        });
        methods.add_method("SetMovable", |_, this, movable: bool| {
            if let Ok(mut s) = this.state.try_borrow_mut() {
                if let Some(frame) = s.widgets.get_mut(this.id) {
                    frame.movable = movable;
                }
            }
            Ok(())
        });
        methods.add_method("IsMovable", |_, this, ()| {
            if let Ok(s) = this.state.try_borrow() {
                if let Some(frame) = s.widgets.get(this.id) {
                    return Ok(frame.movable);
                }
            }
            Ok(false)
        });
        methods.add_method("SetResizable", |_, this, resizable: bool| {
            if let Ok(mut s) = this.state.try_borrow_mut() {
                if let Some(frame) = s.widgets.get_mut(this.id) {
                    frame.resizable = resizable;
                }
            }
            Ok(())
        });
        methods.add_method("IsResizable", |_, this, ()| {
            if let Ok(s) = this.state.try_borrow() {
                if let Some(frame) = s.widgets.get(this.id) {
                    return Ok(frame.resizable);
                }
            }
            Ok(false)
        });
        methods.add_method("SetClampedToScreen", |_, this, clamped: bool| {
            if let Ok(mut s) = this.state.try_borrow_mut() {
                if let Some(frame) = s.widgets.get_mut(this.id) {
                    frame.clamped_to_screen = clamped;
                }
            }
            Ok(())
        });
        methods.add_method("IsClampedToScreen", |_, this, ()| {
            if let Ok(s) = this.state.try_borrow() {
                if let Some(frame) = s.widgets.get(this.id) {
                    return Ok(frame.clamped_to_screen);
                }
            }
            Ok(false)
        });
        methods.add_method("SetClampRectInsets", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("SetResizeBounds", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("GetResizeBounds", |_, _this, ()| Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32)));
        methods.add_method("StartSizing", |_, _this, _point: Option<String>| Ok(()));
        methods.add_method("RegisterForDrag", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("SetUserPlaced", |_, _this, _user_placed: bool| Ok(()));
        methods.add_method("IsUserPlaced", |_, _this, ()| Ok(false));
        methods.add_method("SetDontSavePosition", |_, _this, _dont_save: bool| Ok(()));

        // ScrollBox methods (Mixin callback system)
        methods.add_method("RegisterCallback", |_, _this, _args: mlua::MultiValue| Ok(()));
        methods.add_method("ForEachFrame", |_, _this, _callback: mlua::Function| Ok(()));
        methods.add_method("UnregisterCallback", |_, _this, _args: mlua::MultiValue| Ok(()));

        // ScrollBox/ScrollBar interpolation methods
        methods.add_method("CanInterpolateScroll", |_, _this, ()| Ok(false));
        methods.add_method("SetInterpolateScroll", |_, _this, _enabled: bool| Ok(()));

        // EditBox text measurement methods
        methods.add_method("SetCountInvisibleLetters", |_, _this, _count: bool| Ok(()));
        methods.add_method("GetCursorPosition", |_, _this, ()| Ok(0i32));
        methods.add_method("SetCursorPosition", |_, _this, _pos: i32| Ok(()));
        methods.add_method("HighlightText", |_, _this, _args: mlua::MultiValue| Ok(()));
    }
}
