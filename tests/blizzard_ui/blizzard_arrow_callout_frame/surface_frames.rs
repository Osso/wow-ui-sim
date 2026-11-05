//! Frame-surface probes for `Blizzard_ArrowCalloutFrame`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::loader::load_addon;

const ROOT: &str = "Blizzard_ArrowCalloutFrame";
const FRAME_MANAGER_SURFACE_PROBE: &str = r#"
local failures = {}

local function expect(condition, message)
    if not condition then
        table.insert(failures, message)
    end
end

local frame = ArrowCalloutFrameManager
expect(type(frame) == "table", "ArrowCalloutFrameManager missing")
expect(frame and frame:GetObjectType() == "Frame", "manager object type")
expect(frame and frame:GetParent() == UIParent, "manager parent")
expect(frame and frame:GetFrameStrata() == "HIGH", "manager frame strata")

local sawTopLeft = false
local sawBottomRight = false
if frame then
    for pointIndex = 1, frame:GetNumPoints() do
        local point, relativeTo, relativePoint, xOfs, yOfs = frame:GetPoint(pointIndex)
        local matchesUiParent = relativeTo == UIParent and xOfs == 0 and yOfs == 0
        if point == "TOPLEFT" and relativePoint == "TOPLEFT" and matchesUiParent then
            sawTopLeft = true
        elseif point == "BOTTOMRIGHT" and relativePoint == "BOTTOMRIGHT" and matchesUiParent then
            sawBottomRight = true
        end
    end
end
expect(frame and frame:GetNumPoints() == 2, "manager setAllPoints anchor count")
expect(sawTopLeft, "manager setAllPoints TOPLEFT anchor")
expect(sawBottomRight, "manager setAllPoints BOTTOMRIGHT anchor")

expect(frame and frame.OnLoad == ArrowCalloutMixin.OnLoad, "manager OnLoad mixin method")
expect(frame and frame.OnEvent == ArrowCalloutMixin.OnEvent, "manager OnEvent mixin method")
expect(frame and type(frame:GetScript("OnLoad")) == "function", "manager OnLoad script")
expect(frame and type(frame:GetScript("OnEvent")) == "function", "manager OnEvent script")

expect(frame and type(frame.currentCallouts) == "table", "currentCallouts table")
expect(frame and next(frame.currentCallouts) == nil, "currentCallouts initially empty")
expect(frame and type(frame.calloutPool) == "table", "calloutPool table")

local expectedTemplates = {
    "ArrowCalloutContainerTemplate",
    "ArrowCalloutContainerTemplateWithCloseButtonTemplate",
    "WidgetContainerCalloutTemplate",
}
if frame and frame.calloutPool then
    for _, template in ipairs(expectedTemplates) do
        local pool = frame.calloutPool:GetPool(template)
        expect(type(pool) == "table", "calloutPool missing " .. template)

        local acquired = frame.calloutPool:Acquire(template)
        expect(type(acquired) == "table", "calloutPool acquire " .. template)
    end
end
expect(frame and frame.calloutPool and frame.calloutPool:GetNumActive() == 3, "calloutPool active count after acquiring expected templates")
if frame and frame.calloutPool then
    frame.calloutPool:ReleaseAll()
end

return table.concat(failures, "\n")
"#;

// Wrath has no ArrowCallout vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn arrow_callout_frame_manager_matches_xml_surface() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_arrow_callout_frame(env);

        let failures: String = env
            .eval(FRAME_MANAGER_SURFACE_PROBE)
            .expect("ArrowCalloutFrameManager surface probe must run cleanly");
        assert!(
            failures.is_empty(),
            "ArrowCalloutFrameManager surface mismatches:\n{failures}"
        );
    });
}
}

fn load_arrow_callout_frame(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    load_addon(&env.loader_env(), &arrow_callout_toc())
        .expect("Blizzard_ArrowCalloutFrame should load directly from its TOC");
}

fn arrow_callout_toc() -> std::path::PathBuf {
    blizzard_ui_dir()
        .join(ROOT)
        .join("Blizzard_ArrowCalloutFrame.toc")
}
