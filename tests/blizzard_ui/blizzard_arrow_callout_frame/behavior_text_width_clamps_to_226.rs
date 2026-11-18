//! Behavior probe for ArrowCalloutFrame text width clamping.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::loader::load_addon;

const ROOT: &str = "Blizzard_ArrowCalloutFrame";
// INFERRED for Retail 12.1.5.70783 from verified R88az behavior: the original
// 0x0 TOP-only Content parent leaves its all-points FontString rect invalid.
// See Blizzard_ArrowCalloutFrame/ArrowCalloutFrame.xml and pr46-49-decomp-answers.md §R88az.
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

local longCallout = ArrowCalloutFrameManager.currentCallouts[80]
local longTextFrame = longCallout.Content.Text
local longNaturalWidth = longTextFrame:GetStringWidth()
local longContentWidth = longCallout.Content:GetWidth()
local longAppliedWidth = longContentWidth - longCallout.widthPadding

expect(longNaturalWidth > 226, "long text should naturally measure wider than the clamp: " .. tostring(longNaturalWidth))
expect(math.abs(longAppliedWidth - math.min(226, longNaturalWidth)) < 0.01, "long callout should apply min(226, measured text width): " .. tostring(longAppliedWidth))

local shortText = "Hi"
expect(showCallout(81, "ShortTextCalloutAnchor", shortText), "short text callout should show")

local shortCallout = ArrowCalloutFrameManager.currentCallouts[81]
local shortNaturalWidth = shortCallout.Content.Text:GetStringWidth()
local shortAppliedWidth = shortCallout.Content:GetWidth() - shortCallout.widthPadding

expect(shortNaturalWidth < 226, "short text should naturally measure below the clamp: " .. tostring(shortNaturalWidth))
expect(math.abs(shortAppliedWidth - math.min(226, shortNaturalWidth)) < 0.01, "short callout should apply min(226, measured text width): " .. tostring(shortAppliedWidth))
expect(shortAppliedWidth ~= 226, "short text width should not be forced to the maximum clamp: " .. tostring(shortAppliedWidth))

return table.concat(failures, "\n")
"#;

// Wrath has no ArrowCallout vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "client-retail")]
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
