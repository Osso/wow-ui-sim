//! Pre-created global frames and frame/texture attribute surface checks.

use super::super::*;

#[test]
fn test_uiparent_exists() {
    let env = WowLuaEnv::new().unwrap();
    let ty: String = env.eval("return UIParent:GetObjectType()").unwrap();
    assert_eq!(ty, "Frame");
}

#[test]
fn test_create_frame_exposes_core_event_methods() {
    let env = WowLuaEnv::new().unwrap();
    let registry_mt_type: String = {
        let mut lua = env.lua.borrow_mut();
        let state = lua.state_mut();
        match crate::lua_api::methods::registry_get(state, "__rilua_frame_mt") {
            rilua::Val::Table(_) => "table".to_string(),
            other => other.type_name().to_string(),
        }
    };
    let (set_forbidden, set_script, register_event, get_object_type, mt_type, mt_index_type, mt_set_forbidden, mt_get_object_type): (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ) = env
        .eval(
            r#"
            local f = CreateFrame("Frame")
            local mt = getmetatable(f)
            return type(f.SetForbidden), type(f.SetScript), type(f.RegisterEvent), type(f.GetObjectType),
                type(mt), type(mt and mt.__index), type(mt and mt.SetForbidden), type(mt and mt.GetObjectType)
            "#,
        )
        .unwrap();
    assert_eq!(
        (set_forbidden, set_script, register_event, get_object_type),
        (
            "function".to_string(),
            "function".to_string(),
            "function".to_string(),
            "function".to_string(),
        ),
        "frame surface mismatch: registry={registry_mt_type}, mt={mt_type}, mt.__index={mt_index_type}, mt.SetForbidden={mt_set_forbidden}, mt.GetObjectType={mt_get_object_type}",
    );
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_frame_texture_statusbar_method_surface() {
    let env = WowLuaEnv::new().unwrap();
    let (
        forbidden_ty,
        object_table_ty,
        roleset_ty,
        roleset_names_ty,
        on_update_mode,
        radial_percent,
        radial_reverse,
        status_render_mode,
        minimap_icon_scale_ty,
        clear_scripts_ty,
        vector_ty,
        vector_object_type,
        vector_has_svg,
        vector_file_id,
    ): (
        String,
        String,
        String,
        String,
        i32,
        f64,
        bool,
        String,
        String,
        String,
        String,
        String,
        bool,
        i64,
    ) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            frame:AddForbiddenAspects(Enum.ForbiddenAspect.UntrustedScriptExecution)
            frame:SetRolesets("combat", "healer")
            frame:SetOnUpdateMode(Enum.OnUpdateMode.RunAlways)

            local tex = frame:CreateTexture(nil, "ARTWORK")
            tex:SetRadialProgressBarPercent(0.75)
            tex:SetRadialProgressBarReverse(true)

            local status = CreateFrame("StatusBar")
            status:SetRenderMode("Standard")

            local vector = frame:CreateVectorGraphics(nil, "ARTWORK")
            vector:SetSVG(12345)

            return type(frame.GetForbiddenAspects),
                type(frame:GetObjectTable()),
                type(frame.AddRoleset),
                type(frame:GetRolesetNames()),
                frame:GetOnUpdateMode(),
                tex:GetRadialProgressBarPercent(),
                tex:GetRadialProgressBarReverse(),
                status:GetRenderMode(),
                type(Minimap.SetIconScale),
                type(frame.ClearScripts),
                type(frame.CreateVectorGraphics),
                vector:GetObjectType(),
                vector:HasSVG(),
                vector:GetSVGFileID()
            "#,
        )
        .unwrap();

    assert_eq!(forbidden_ty, "function");
    assert_eq!(object_table_ty, "table");
    assert_eq!(roleset_ty, "function");
    assert_eq!(roleset_names_ty, "table");
    assert_eq!(on_update_mode, 4);
    assert_eq!(radial_percent, 0.75);
    assert!(radial_reverse);
    assert_eq!(status_render_mode, "Standard");
    assert_eq!(minimap_icon_scale_ty, "function");
    assert_eq!(clear_scripts_ty, "function");
    assert_eq!(vector_ty, "function");
    assert_eq!(vector_object_type, "VectorGraphics");
    assert!(vector_has_svg);
    assert_eq!(vector_file_id, 12345);

    let (forbidden_mask_ty, inheritable_mask_ty, is_set): (String, String, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            frame:AddForbiddenAspects(Enum.ForbiddenAspect.UntrustedScriptExecution)
            return type(frame:GetForbiddenAspects()),
                type(frame:GetInheritableForbiddenAspects(Enum.ScriptObjectPropagationPath.Hierarchy)),
                FlagsUtil.IsSet(frame:GetForbiddenAspects(), Enum.ForbiddenAspect.UntrustedScriptExecution)
            "#,
        )
        .unwrap();
    assert_eq!(forbidden_mask_ty, "number");
    assert_eq!(inheritable_mask_ty, "number");
    assert!(is_set);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_texture_radial_progress_surface_and_state() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local texture = CreateFrame("Frame"):CreateTexture(nil, "ARTWORK")
            if texture:GetObjectType() ~= "Texture" then return "receiver" end

            local methods = {
                "ClearRadialProgressBar",
                "SetRadialProgressBarPercent",
                "GetRadialProgressBarPercent",
                "SetRadialProgressBarStartOffset",
                "GetRadialProgressBarStartOffset",
                "SetRadialProgressBarEndOffset",
                "GetRadialProgressBarEndOffset",
                "SetRadialProgressBarFeather",
                "GetRadialProgressBarFeather",
                "SetRadialProgressBarReverse",
                "GetRadialProgressBarReverse",
                "SetVisualRadialProgressBarMode",
            }
            for _, method in ipairs(methods) do
                if type(texture[method]) ~= "function" then return "missing:" .. method end
            end

            if texture:GetRadialProgressBarPercent() ~= 0 then return "default-percent" end
            if texture:GetRadialProgressBarStartOffset() ~= 0 then return "default-start" end
            if texture:GetRadialProgressBarEndOffset() ~= 1 then return "default-end" end
            if texture:GetRadialProgressBarFeather() ~= 0 then return "default-feather" end
            if texture:GetRadialProgressBarReverse() ~= false then return "default-reverse" end

            texture:SetRadialProgressBarPercent(0.75)
            texture:SetRadialProgressBarStartOffset(0.25)
            texture:SetRadialProgressBarEndOffset(0.875)
            texture:SetRadialProgressBarFeather(0.125)
            texture:SetRadialProgressBarReverse(true)
            texture:SetVisualRadialProgressBarMode()

            if texture:GetRadialProgressBarPercent() ~= 0.75 then return "set-percent" end
            if texture:GetRadialProgressBarStartOffset() ~= 0.25 then return "set-start" end
            if texture:GetRadialProgressBarEndOffset() ~= 0.875 then return "set-end" end
            if texture:GetRadialProgressBarFeather() ~= 0.125 then return "set-feather" end
            if texture:GetRadialProgressBarReverse() ~= true then return "set-reverse" end

            texture:ClearRadialProgressBar()
            if texture:GetRadialProgressBarPercent() ~= 0 then return "clear-percent" end
            if texture:GetRadialProgressBarStartOffset() ~= 0 then return "clear-start" end
            if texture:GetRadialProgressBarEndOffset() ~= 1 then return "clear-end" end
            if texture:GetRadialProgressBarFeather() ~= 0 then return "clear-feather" end
            if texture:GetRadialProgressBarReverse() ~= false then return "clear-reverse" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_forbidden_aspects_block_parent_and_layout_gain() {
    let env = WowLuaEnv::new().unwrap();
    let (parent_blocked, parent_allowed, layout_blocked, layout_allowed): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local aspect = Enum.ForbiddenAspect.UntrustedLayoutScriptExecution
            local parent = CreateFrame("Frame")
            parent:AddForbiddenAspects(aspect)

            local child = CreateFrame("Frame")
            local parentBlocked = not pcall(function() child:SetParent(parent) end)
            child:AddForbiddenAspects(aspect)
            local parentAllowed = pcall(function() child:SetParent(parent) end)

            local anchor = CreateFrame("Frame")
            anchor:AddForbiddenAspects(aspect)
            local layoutChild = CreateFrame("Frame")
            local layoutBlocked = not pcall(function() layoutChild:SetPoint("CENTER", anchor, "CENTER") end)
            layoutChild:AddForbiddenAspects(aspect)
            local layoutAllowed = pcall(function() layoutChild:SetPoint("CENTER", anchor, "CENTER") end)

            return parentBlocked, parentAllowed, layoutBlocked, layoutAllowed
            "#,
        )
        .unwrap();

    assert!(parent_blocked);
    assert!(parent_allowed);
    assert!(layout_blocked);
    assert!(layout_allowed);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_clear_scripts_removes_handlers() {
    let env = WowLuaEnv::new().unwrap();
    let (before_ty, after_ty): (String, String) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            frame:SetScript("OnShow", function() end)
            local before = type(frame:GetScript("OnShow"))
            frame:ClearScripts()
            return before, type(frame:GetScript("OnShow"))
            "#,
        )
        .unwrap();

    assert_eq!(before_ty, "function");
    assert_eq!(after_ty, "nil");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_disabled_on_update_mode_skips_dispatch() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(
        r#"
        Patch121OnUpdateCalls = 0
        Patch121OnUpdateFrame = CreateFrame("Frame", nil, UIParent)
        Patch121OnUpdateFrame:SetScript("OnUpdate", function()
            Patch121OnUpdateCalls = Patch121OnUpdateCalls + 1
        end)
        Patch121OnUpdateFrame:SetOnUpdateMode(Enum.OnUpdateMode.Disabled)
        "#,
    )
    .unwrap();

    env.fire_on_update(0.016).unwrap();
    let calls: i64 = env.eval("return Patch121OnUpdateCalls").unwrap();
    assert_eq!(calls, 0);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_run_always_on_update_fires_while_hidden() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(
        r#"
        Patch121HiddenRunAlwaysCalls = 0
        Patch121HiddenRunAlwaysFrame = CreateFrame("Frame", nil, UIParent)
        Patch121HiddenRunAlwaysFrame:Hide()
        Patch121HiddenRunAlwaysFrame:SetOnUpdateMode(Enum.OnUpdateMode.RunAlways)
        Patch121HiddenRunAlwaysFrame:SetScript("OnUpdate", function()
            Patch121HiddenRunAlwaysCalls = Patch121HiddenRunAlwaysCalls + 1
        end)
        "#,
    )
    .unwrap();

    env.fire_on_update(0.016).unwrap();
    let calls: i64 = env.eval("return Patch121HiddenRunAlwaysCalls").unwrap();
    assert_eq!(calls, 1);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_run_when_visible_on_update_skips_hidden() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(
        r#"
        Patch121HiddenVisibleCalls = 0
        Patch121HiddenVisibleFrame = CreateFrame("Frame", nil, UIParent)
        Patch121HiddenVisibleFrame:Hide()
        Patch121HiddenVisibleFrame:SetOnUpdateMode(Enum.OnUpdateMode.RunWhenVisible)
        Patch121HiddenVisibleFrame:SetScript("OnUpdate", function()
            Patch121HiddenVisibleCalls = Patch121HiddenVisibleCalls + 1
        end)
        "#,
    )
    .unwrap();

    env.fire_on_update(0.016).unwrap();
    let calls: i64 = env.eval("return Patch121HiddenVisibleCalls").unwrap();
    assert_eq!(calls, 0);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_aura_frame_types_create() {
    let env = WowLuaEnv::new().unwrap();
    let (container_type, button_type, managed_type): (String, String, String) = env
        .eval(
            r#"
            local container = CreateFrame("AuraContainer", nil, UIParent)
            local button = CreateFrame("AuraButton", nil, container)
            local managed = CreateFrame("ManagedAuraContainer", nil, UIParent)
            return container:GetObjectType(), button:GetObjectType(), managed:GetObjectType()
            "#,
        )
        .unwrap();

    assert_eq!(container_type, "AuraContainer");
    assert_eq!(button_type, "AuraButton");
    assert_eq!(managed_type, "ManagedAuraContainer");
}

#[test]
fn test_get_frame_metatable_without_instance_returns_shared_metatable() {
    let env = WowLuaEnv::new().unwrap();
    let (mt_ty, mt_index_ty, get_object_type_ty, set_forbidden_ty): (String, String, String, String) = env
        .eval(
            r#"
            local mt = GetFrameMetatable()
            return type(mt), type(mt and mt.__index), type(mt and mt.GetObjectType), type(mt and mt.SetForbidden)
            "#,
        )
        .unwrap();
    assert_eq!(mt_ty, "table");
    assert_eq!(mt_index_ty, "table");
    assert_eq!(get_object_type_ty, "function");
    assert_eq!(set_forbidden_ty, "function");
}

#[test]
fn test_get_font_string_metatable_exposes_text_methods() {
    let env = WowLuaEnv::new().unwrap();
    let (mt_ty, mt_index_ty, set_text_ty, set_formatted_text_ty): (String, String, String, String) =
        env.eval(
            r#"
            local mt = GetFontStringMetatable()
            return type(mt),
                type(mt and mt.__index),
                type(mt and mt.__index and mt.__index.SetText),
                type(mt and mt.__index and mt.__index.SetFormattedText)
            "#,
        )
        .unwrap();
    assert_eq!(mt_ty, "table");
    assert_eq!(mt_index_ty, "table");
    assert_eq!(set_text_ty, "function");
    assert_eq!(set_formatted_text_ty, "function");
}

#[test]
fn test_create_texture_exposes_core_visual_methods() {
    let env = WowLuaEnv::new().unwrap();
    let (
        set_texture,
        set_color_texture,
        set_vertex_color,
        set_blend_mode,
        set_tex_coord,
        set_horiz_tile,
        set_vert_tile,
        set_texel_snapping_bias,
        get_texel_snapping_bias,
        set_snap_to_pixel_grid,
        set_desaturated_ty,
        is_desaturated_ty,
        get_desaturation_ty,
        bias,
        is_desaturated,
        desaturation,
    ): (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        f64,
        bool,
        f64,
    ) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local tex = frame:CreateTexture()
            tex:SetTexture("Interface\\Buttons\\WHITE8X8")
            tex:SetVertexColor(0.1, 0.2, 0.3, 0.4)
            tex:SetBlendMode("ADD")
            tex:SetColorTexture(0.5, 0.6, 0.7, 0.8)
            tex:SetTexCoord(0, 1, 0, 1)
            tex:SetHorizTile(true)
            tex:SetVertTile(true)
            tex:SetTexelSnappingBias(0.25)
            tex:SetSnapToPixelGrid(true)
            tex:SetDesaturated(true)
            return type(tex.SetTexture), type(tex.SetColorTexture), type(tex.SetVertexColor),
                type(tex.SetBlendMode), type(tex.SetTexCoord), type(tex.SetHorizTile),
                type(tex.SetVertTile), type(tex.SetTexelSnappingBias),
                type(tex.GetTexelSnappingBias), type(tex.SetSnapToPixelGrid),
                type(tex.SetDesaturated), type(tex.IsDesaturated), type(tex.GetDesaturation),
                tex:GetTexelSnappingBias(), tex:IsDesaturated(), tex:GetDesaturation()
            "#,
        )
        .unwrap();
    for ty in [
        set_texture,
        set_color_texture,
        set_vertex_color,
        set_blend_mode,
        set_tex_coord,
        set_horiz_tile,
        set_vert_tile,
        set_texel_snapping_bias,
        get_texel_snapping_bias,
        set_snap_to_pixel_grid,
        set_desaturated_ty,
        is_desaturated_ty,
        get_desaturation_ty,
    ] {
        assert_eq!(ty, "function");
    }
    assert_eq!(bias, 0.25);
    assert!(is_desaturated);
    assert_eq!(desaturation, 1.0);
}

#[test]
fn test_get_children_excludes_layer_regions() {
    let env = WowLuaEnv::new().unwrap();
    let (num_children, num_regions, first_child_key, first_region_key): (i64, i64, String, String) =
        env.eval(
            r#"
            local frame = CreateFrame("Frame")
            local child = CreateFrame("Frame", nil, frame)
            child:SetParentKey("childFrame")
            local texture = frame:CreateTexture()
            texture:SetParentKey("regionTexture")
            local fontString = frame:CreateFontString()
            fontString:SetParentKey("regionText")
            return frame:GetNumChildren(), frame:GetNumRegions(),
                ({ frame:GetChildren() })[1]:GetParentKey(),
                ({ frame:GetRegions() })[1]:GetParentKey()
            "#,
        )
        .unwrap();

    assert_eq!(num_children, 1);
    assert_eq!(num_regions, 2);
    assert_eq!(first_child_key, "childFrame");
    assert_eq!(first_region_key, "regionTexture");
}

#[test]
fn test_set_attribute_fires_on_attribute_changed() {
    let env = WowLuaEnv::new().unwrap();
    let (name_ty, seen_name, value_ty, seen_value, stored_ty, stored_value): (
        String,
        String,
        String,
        String,
        String,
        String,
    ) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local seenName, seenValue

            frame:SetScript("OnAttributeChanged", function(_, name, value)
                seenName = name
                seenValue = value
            end)

            frame:SetAttribute("count", 7)

            return type(seenName), tostring(seenName), type(seenValue), tostring(seenValue),
                type(frame:GetAttribute("count")), tostring(frame:GetAttribute("count"))
            "#,
        )
        .unwrap();

    assert_eq!(name_ty, "string");
    assert_eq!(seen_name, "count");
    assert_eq!(
        value_ty, "number",
        "seen_name={seen_name} seen_value={seen_value} stored_ty={stored_ty} stored_value={stored_value}"
    );
    assert_eq!(seen_value, "7");
    assert_eq!(stored_ty, "number");
    assert_eq!(stored_value, "7");
}

#[test]
fn test_global_unpack_exists() {
    let env = WowLuaEnv::new().unwrap();
    let (global_ty, table_ty, first, second, third): (String, String, i64, i64, i64) = env
        .eval(
            r#"
            local values = {11, 22, 33}
            return type(unpack), type(table.unpack), table.unpack(values)
            "#,
        )
        .unwrap();

    assert_eq!(global_ty, "function");
    assert_eq!(table_ty, "function");
    assert_eq!((first, second, third), (11, 22, 33));
}
