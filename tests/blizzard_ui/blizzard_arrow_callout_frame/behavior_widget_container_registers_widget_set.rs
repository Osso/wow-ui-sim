//! Behavior probe for WidgetContainerNoBorder callout widget-set registration.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::loader::load_addon;

const ROOT: &str = "Blizzard_ArrowCalloutFrame";
const WIDGET_SET_REGISTRATION_PROBE: &str = r#"
local failures = {}

local function expect(condition, message)
    if not condition then
        table.insert(failures, message)
    end
end

local registerCalls = {}
local originalRegisterForWidgetSet = UIWidgetContainerMixin and UIWidgetContainerMixin.RegisterForWidgetSet
expect(type(originalRegisterForWidgetSet) == "function", "UIWidgetContainerMixin.RegisterForWidgetSet should exist")

if originalRegisterForWidgetSet then
    UIWidgetContainerMixin.RegisterForWidgetSet = function(self, widgetSetID, layoutFunc, ...)
        table.insert(registerCalls, {
            frame = self,
            widgetSetID = widgetSetID,
            layoutFunc = layoutFunc,
        })

        return originalRegisterForWidgetSet(self, widgetSetID, layoutFunc, ...)
    end
end

local anchor = CreateFrame("Frame", "WidgetSetCalloutAnchor", UIParent)
anchor:SetSize(100, 40)
anchor:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 200, -200)

local function showWidgetCallout(calloutID, widgetSetID)
    local calloutInfo = {
        calloutID = calloutID,
        calloutFrame = "WidgetSetCalloutAnchor",
        calloutDirection = Enum.ArrowCalloutDirection.Up,
        calloutType = Enum.ArrowCalloutType.WidgetContainerNoBorder,
        calloutText = "Widget",
        offsetX = 0,
        offsetY = 0,
    }

    if widgetSetID ~= nil then
        calloutInfo.uiWidgetSetID = widgetSetID
    end

    return C_ArrowCalloutManager.ShowCallout(calloutInfo)
end

expect(showWidgetCallout(70, 42), "widget callout with uiWidgetSetID should show")

local manager = ArrowCalloutFrameManager
local widgetPool = manager.calloutPool:GetPool("WidgetContainerCalloutTemplate")
local withWidgetSet = manager.currentCallouts[70]

expect(type(withWidgetSet) == "table", "uiWidgetSetID callout should allocate a frame")
expect(widgetPool:IsActive(withWidgetSet), "uiWidgetSetID callout should use WidgetContainerCalloutTemplate")
expect(#registerCalls == 1, "uiWidgetSetID callout should register exactly once")

if registerCalls[1] then
    expect(registerCalls[1].frame == withWidgetSet, "RegisterForWidgetSet should be invoked on the acquired widget container")
    expect(registerCalls[1].widgetSetID == 42, "RegisterForWidgetSet should receive uiWidgetSetID")
    expect(registerCalls[1].layoutFunc == DefaultWidgetLayout, "RegisterForWidgetSet should receive DefaultWidgetLayout")
end

expect(showWidgetCallout(71, nil), "widget callout without uiWidgetSetID should show")

local withoutWidgetSet = manager.currentCallouts[71]
expect(type(withoutWidgetSet) == "table", "nil uiWidgetSetID callout should allocate a frame")
expect(widgetPool:IsActive(withoutWidgetSet), "nil uiWidgetSetID callout should use WidgetContainerCalloutTemplate")
expect(#registerCalls == 1, "nil uiWidgetSetID callout should not call RegisterForWidgetSet")

return table.concat(failures, "\n")
"#;

#[test]
fn widget_container_callout_registers_widget_set_only_when_id_is_present() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_arrow_callout_frame(env);

        let failures: String = env
            .eval(WIDGET_SET_REGISTRATION_PROBE)
            .expect("widget-set registration behavior probe must run cleanly");
        assert!(
            failures.is_empty(),
            "ArrowCalloutFrame widget-set registration behavior mismatches:\n{failures}"
        );
    });
}

fn load_arrow_callout_frame(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    load_addon(&env.loader_env(), &ui_widgets_toc())
        .expect("Blizzard_UIWidgets should load before widget container callouts");
    load_addon(&env.loader_env(), &arrow_callout_toc())
        .expect("Blizzard_ArrowCalloutFrame should load directly from its TOC");
}

fn ui_widgets_toc() -> std::path::PathBuf {
    blizzard_ui_dir()
        .join("Blizzard_UIWidgets")
        .join("Blizzard_UIWidgets.toc")
}

fn arrow_callout_toc() -> std::path::PathBuf {
    blizzard_ui_dir()
        .join(ROOT)
        .join("Blizzard_ArrowCalloutFrame.toc")
}
