//! Event-registration probes for `Blizzard_ArtifactUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ArtifactUI";

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_frame_event_lifecycle_matches_mixin_handlers() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(ARTIFACT_FRAME_EVENT_LIFECYCLE_PROBE)
            .expect("ArtifactFrame event lifecycle probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` must keep load-time events registered and show-only events scoped to \
             OnShow/OnHide; mismatches: {mismatches:?}"
        );
    });
}
}

// ArtifactUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_perks_and_title_event_lifecycles_match_mixin_handlers() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(PERKS_AND_TITLE_EVENT_LIFECYCLE_PROBE)
            .expect("Artifact perks/title event lifecycle probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` must register and unregister ArtifactPerks and title events through \
             their mixin handlers; mismatches: {mismatches:?}"
        );
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_appearances_onload_registers_artifact_update() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(APPEARANCES_EVENT_SURFACE_PROBE)
            .expect("Artifact appearances event surface probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` must register ARTIFACT_UPDATE on ArtifactFrame.AppearancesTab during \
             ArtifactAppearancesMixin:OnLoad; mismatches: {mismatches:?}"
        );
    });
}
}

fn load_artifact_ui(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (loaded, error): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ArtifactUI")"#)
        .expect("C_AddOns.LoadAddOn probe should run cleanly");
    assert!(
        loaded,
        "`{ROOT}` must load before event surface probes; error={error:?}"
    );
}

const ARTIFACT_FRAME_EVENT_LIFECYCLE_PROBE: &str = r#"
local mismatches = {}

local function expect(condition, message)
    if not condition then
        table.insert(mismatches, message)
    end
end

local frame = ArtifactFrame
local loadEvents = {
    "ARTIFACT_UPDATE",
    "ARTIFACT_CLOSE",
}
local showEvents = {
    "ARTIFACT_XP_UPDATE",
    "ARTIFACT_RELIC_INFO_RECEIVED",
    "UI_SCALE_CHANGED",
    "DISPLAY_SIZE_CHANGED",
}

expect(type(frame) == "table", "ArtifactFrame:" .. type(frame))

local function expectRegistered(events, expected, phase)
    for _, event in ipairs(events) do
        local registered = frame and frame:IsEventRegistered(event) or false
        expect(
            registered == expected,
            phase .. " " .. event .. ":" .. tostring(registered)
        )
    end
end

expectRegistered(loadEvents, true, "after OnLoad")
expectRegistered(showEvents, false, "before OnShow")

local originalEvaluateForgeState = frame.EvaulateForgeState
local originalSetupPerArtifactData = frame.SetupPerArtifactData
local originalRefreshKnowledgeRanks = frame.RefreshKnowledgeRanks
local originalPerksOnUiOpened = frame.PerksTab.OnUIOpened
local originalUnderlayHide = ArtifactFrameUnderlay.Hide
local originalClear = C_ArtifactUI.Clear
local originalStaticPopupHide = StaticPopup_Hide

frame.EvaulateForgeState = function() end
frame.SetupPerArtifactData = function() end
frame.RefreshKnowledgeRanks = function() end
frame.PerksTab.OnUIOpened = function() end
ArtifactFrameUnderlay.Hide = function() end
C_ArtifactUI.Clear = function() end
StaticPopup_Hide = function() end

local ok, errorMessage = pcall(function()
    ArtifactUIMixin.OnShow(frame)
    expectRegistered(loadEvents, true, "after OnShow")
    expectRegistered(showEvents, true, "after OnShow")

    ArtifactUIMixin.OnHide(frame)
    expectRegistered(loadEvents, true, "after OnHide")
    expectRegistered(showEvents, false, "after OnHide")
end)

frame.EvaulateForgeState = originalEvaluateForgeState
frame.SetupPerArtifactData = originalSetupPerArtifactData
frame.RefreshKnowledgeRanks = originalRefreshKnowledgeRanks
frame.PerksTab.OnUIOpened = originalPerksOnUiOpened
ArtifactFrameUnderlay.Hide = originalUnderlayHide
C_ArtifactUI.Clear = originalClear
StaticPopup_Hide = originalStaticPopupHide

expect(ok, "OnShow/OnHide error:" .. tostring(errorMessage))

return mismatches
"#;

const APPEARANCES_EVENT_SURFACE_PROBE: &str = r#"
local mismatches = {}

local function expect(condition, message)
    if not condition then
        table.insert(mismatches, message)
    end
end

local appearances = ArtifactFrame and ArtifactFrame.AppearancesTab
expect(type(appearances) == "table", "ArtifactFrame.AppearancesTab:" .. type(appearances))
expect(
    appearances and appearances:IsEventRegistered("ARTIFACT_UPDATE"),
    "AppearancesTab ARTIFACT_UPDATE registered:" ..
        tostring(appearances and appearances:IsEventRegistered("ARTIFACT_UPDATE"))
)

return mismatches
"#;

const PERKS_AND_TITLE_EVENT_LIFECYCLE_PROBE: &str = r#"
local mismatches = {}

local function expect(condition, message)
    if not condition then
        table.insert(mismatches, message)
    end
end

local perks = ArtifactFrame and ArtifactFrame.PerksTab
local title = perks and perks.TitleContainer
local perksEvents = {
    "CURSOR_CHANGED",
    "UI_MODEL_SCENE_INFO_UPDATED",
}
local titleEvents = {
    "ARTIFACT_UPDATE",
    "CURSOR_CHANGED",
}

expect(type(perks) == "table", "ArtifactFrame.PerksTab:" .. type(perks))
expect(type(title) == "table", "ArtifactFrame.PerksTab.TitleContainer:" .. type(title))

local function expectRegistered(frame, events, expected, phase)
    for _, event in ipairs(events) do
        local registered = frame and frame:IsEventRegistered(event) or false
        expect(
            registered == expected,
            phase .. " " .. event .. ":" .. tostring(registered)
        )
    end
end

expectRegistered(perks, perksEvents, false, "perks before OnShow")
expectRegistered(title, titleEvents, false, "title before OnShow")

local originalCancelAllTimedAnimations = perks.CancelAllTimedAnimations
local originalSkipTier2Animation = perks.SkipTier2Animation
local originalRefreshTitle = title.RefreshTitle
local originalEvaluateRelics = title.EvaluateRelics
local originalHideReplacePopup = StaticPopup_Hide

perks.CancelAllTimedAnimations = function() end
perks.SkipTier2Animation = function() end
title.RefreshTitle = function() end
title.EvaluateRelics = function() end
StaticPopup_Hide = function() end

local ok, errorMessage = pcall(function()
    ArtifactPerksMixin.OnShow(perks)
    expectRegistered(perks, perksEvents, true, "perks after OnShow")

    ArtifactPerksMixin.OnHide(perks)
    expectRegistered(perks, perksEvents, false, "perks after OnHide")

    ArtifactTitleTemplateMixin.OnShow(title)
    expectRegistered(title, titleEvents, true, "title after OnShow")

    ArtifactTitleTemplateMixin.OnHide(title)
    expectRegistered(title, titleEvents, false, "title after OnHide")
end)

perks.CancelAllTimedAnimations = originalCancelAllTimedAnimations
perks.SkipTier2Animation = originalSkipTier2Animation
title.RefreshTitle = originalRefreshTitle
title.EvaluateRelics = originalEvaluateRelics
StaticPopup_Hide = originalHideReplacePopup

expect(ok, "perks/title OnShow/OnHide error:" .. tostring(errorMessage))

return mismatches
"#;
