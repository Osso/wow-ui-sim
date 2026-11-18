//! Behavior probe for ArrowCalloutFrame text width clamping.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::loader::load_addon;

const ROOT: &str = "Blizzard_ArrowCalloutFrame";
const TEXT_WIDTH_CLAMP_PROBE: &str = r#"
local failures = {}

local function expect(condition, message)
    if not condition then
        table.insert(failures, message)
    end
end

local function makeAnchor(name, x)
    local anchor = CreateFrame("Frame", name, UIParent)
    anchor:SetSize(100, 40)
    anchor:SetPoint("TOPLEFT", UIParent, "TOPLEFT", x, -200)
    return anchor
end

makeAnchor("LongTextCalloutAnchor", 120)
makeAnchor("ShortTextCalloutAnchor", 320)

local function showCallout(calloutID, anchorName, text)
    return C_ArrowCalloutManager.ShowCallout({
        calloutID = calloutID,
        calloutFrame = anchorName,
        calloutDirection = Enum.ArrowCalloutDirection.Up,
        calloutType = Enum.ArrowCalloutType.Generic,
        calloutText = text,
        offsetX = 0,
        offsetY = 0,
    })
end

local longText = "This callout message is intentionally long enough to exceed the maximum text width clamp."
expect(showCallout(80, "LongTextCalloutAnchor", longText), "long text callout should show")

local longTextFrame = ArrowCalloutFrameManager.currentCallouts[80].Content.Text
local longNaturalWidth = longTextFrame:GetStringWidth()
local longClampedWidth = longTextFrame:GetWidth()
local longRawWidth = longTextFrame:GetWidth(true)

expect(longNaturalWidth > 226, "long text should naturally measure wider than the clamp: " .. tostring(longNaturalWidth))
expect(longClampedWidth == 226, "long text width should clamp exactly to 226: width=" .. tostring(longClampedWidth) .. " raw=" .. tostring(longRawWidth))

local shortText = "Hi"
expect(showCallout(81, "ShortTextCalloutAnchor", shortText), "short text callout should show")

local shortTextFrame = ArrowCalloutFrameManager.currentCallouts[81].Content.Text
local shortNaturalWidth = shortTextFrame:GetStringWidth()
local shortWidth = shortTextFrame:GetWidth()
local shortRawWidth = shortTextFrame:GetWidth(true)

expect(shortNaturalWidth < 226, "short text should naturally measure below the clamp: " .. tostring(shortNaturalWidth))
expect(shortWidth == shortNaturalWidth, "short text width should remain its natural width: width=" .. tostring(shortWidth) .. " raw=" .. tostring(shortRawWidth) .. " natural=" .. tostring(shortNaturalWidth))
expect(shortWidth ~= 226, "short text width should not be forced to the maximum clamp: " .. tostring(shortWidth))

return table.concat(failures, "\n")
"#;

// Wrath has no ArrowCallout vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[test]
fn callout_text_width_clamps_to_maximum_only_for_long_text() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_arrow_callout_frame(env);

        let failures: String = env
            .eval(TEXT_WIDTH_CLAMP_PROBE)
            .expect("text-width clamp behavior probe must run cleanly");
        assert!(
            failures.is_empty(),
            "ArrowCalloutFrame text-width clamp behavior mismatches:\n{failures}"
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
