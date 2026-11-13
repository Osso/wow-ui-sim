//! O UI ownership retirement: bootstrap-absence tests are retirement REDs;
//! original selected-addon tests are preservation, not claimed baseline REDs.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn original_ui(roots: &[&str]) -> WowLuaEnv {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(&env, &cache, roots, &[]);
    for root in roots {
        assert!(loaded.iter().any(|name| name == root), "{root}: {loaded:?}");
    }
    println!("O selected Retail closure: {loaded:?}");
    env
}

#[test]
fn widget_singleton_is_not_fabricated_before_original_xml() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(rawget(_G, 'UIWidgetManager') == nil, 'Lua/XML-owned singleton fabricated at bootstrap')")
        .unwrap();
}

#[test]
fn panel_registration_is_not_fabricated_before_original_lua() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "assert(rawget(_G, 'RegisterUIPanel') == nil, 'Lua-owned registration fabricated at bootstrap');
         assert(rawget(_G, 'UIPanelWindows') == nil, 'Lua-owned registry fabricated at bootstrap')",
    )
    .unwrap();
}

#[test]
fn selected_widget_template_registers_in_original_manager() {
    // Selected Retail TOC loads manager Lua/XML before IconAndText Lua/XML.
    let env = original_ui(&[
        "Blizzard_SharedXMLGame",
        "Blizzard_Fonts_Shared",
        "Blizzard_FrameEffects",
        "Blizzard_UIWidgets",
    ]);
    env.exec(
        r#"
        local function source(fn, suffix)
            local s = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(s:find(suffix, 1, true), s)
        end
        assert(UIWidgetManager:IsObjectType('Frame'))
        source(UIWidgetManager.RegisterWidgetVisTypeTemplate,
            'Blizzard_UIWidgets/Blizzard_UIWidgetManager.lua')
        local kind = Enum.UIWidgetVisualizationType.IconAndText
        local record = assert(UIWidgetManager:GetWidgetTypeInfo(kind), 'selected template registration lost')
        assert(record.templateInfo.frameType == 'FRAME')
        assert(record.templateInfo.frameTemplate == 'UIWidgetTemplateIconAndText')
        source(record.visInfoDataFunction,
            'Blizzard_UIWidgets/Blizzard_UIWidgetTemplateIconAndText.lua')
        -- Invoke the original duplicate branch, not a synthetic registration producer.
        UIWidgetManager:RegisterWidgetVisTypeTemplate(kind, record.templateInfo, record.visInfoDataFunction)
        assert(UIWidgetManager:GetWidgetTypeInfo(kind) == record)
        UIWidgetManager:RegisterWidgetVisTypeTemplate(-912, {}, record.visInfoDataFunction)
        assert(UIWidgetManager:GetWidgetTypeInfo(-912) == nil)
        -- Bind the actual registered XML template, proving more than a table entry.
        local widget = CreateFrame(record.templateInfo.frameType, nil, UIParent, record.templateInfo.frameTemplate)
        assert(widget.Icon and widget.Text, 'original IconAndText children missing')
        assert(widget.Setup == UIWidgetTemplateIconAndTextMixin.Setup)
        "#,
    )
    .expect("original manager/template registration and XML binding");
}

#[test]
fn selected_panel_owner_preserves_registration_identity_and_lazy_attributes() {
    let env = original_ui(&["Blizzard_SharedXML", "Blizzard_UIParentPanelManager"]);
    env.exec(
        r#"
        local function source(fn)
            local s = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(s:find('Blizzard_UIParentPanelManager/Shared/UIParentPanelManager.lua', 1, true), s)
        end
        source(RegisterUIPanel)
        source(SetUIPanelAttribute)
        local panel = CreateFrame('Frame', 'StubOOriginalRegistrationPanel', UIParent)
        local attrs = {area = 'left', pushable = 1, whileDead = true}
        assert(select('#', RegisterUIPanel(panel, attrs)) == 0)
        assert(rawequal(UIPanelWindows[panel:GetName()], attrs), 'registration must retain table identity')
        assert(panel:GetAttribute('UIPanelLayout-defined') == nil, 'registration must not eagerly apply attributes')
        assert(panel:GetAttribute('UIPanelLayout-area') == nil)
        attrs.area = 'center'
        RegisterUIPanel(panel, {area = 'right'})
        assert(rawequal(UIPanelWindows[panel:GetName()], attrs), 'duplicate registration replaced original')
        -- Original consumer reads stored attributes and installs them lazily.
        SetUIPanelAttribute(panel, 'pushable', 2)
        assert(panel:GetAttribute('UIPanelLayout-defined') == true)
        assert(panel:GetAttribute('UIPanelLayout-area') == 'center')
        assert(panel:GetAttribute('UIPanelLayout-whileDead') == true)
        assert(panel:GetAttribute('UIPanelLayout-pushable') == 2)
        assert(attrs.pushable == 1, 'frame attribute update unexpectedly mutated registry input')
        assert(not pcall(RegisterUIPanel, nil, {}), 'invalid frame must not silently succeed')
        "#,
    )
    .expect("original RegisterUIPanel and lazy SetUIPanelAttribute consumer");
}

// Deferred, not a source-patch retirement RED: the generic frame method table
// supplies IsInDefaultPosition to this non-edit-mode XML template. misc/edit_mode.rs
// reads absent systemInfo and returns false, so original AddManagedFrame exits at
// its default-position guard. Do not fake systemInfo or stub the method to pass.
// Old UIParentManagedFrameMixin is absent from both pinned caches; its removal
// and this unrelated method-surface repair are deferred pending scope/evidence.
