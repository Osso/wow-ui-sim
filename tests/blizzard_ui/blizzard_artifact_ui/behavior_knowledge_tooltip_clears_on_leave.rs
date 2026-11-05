//! Knowledge tooltip cleanup behavior for `Blizzard_ArtifactUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use wow_ui_sim::lua_api::state::{ArtifactArtInfo, ArtifactInfo, ColorRgb};

const ROOT: &str = "Blizzard_ArtifactUI";
const ARTIFACT_ICON: &str = "Interface/Icons/inv_sword_2h_artifactashbringer_d_01";
const ARTIFACT_TITLE: &str = "The Ashbringer";

// ArtifactUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
timerunning_util_panel_fixture! {
#[test]
fn knowledge_tooltip_update_callback_is_cleared_and_tooltip_hidden_on_leave() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        seed_viewed_artifact(env);
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(KNOWLEDGE_TOOLTIP_LEAVE_PROBE)
            .expect("ArtifactUI knowledge tooltip leave probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` knowledge tooltip leave must clear UpdateTooltip and hide GameTooltip; \
             mismatches: {mismatches:?}"
        );
    });
}
}

fn seed_viewed_artifact(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.viewed_artifact.info = Some(sample_artifact());
    state.viewed_artifact.art_info = sample_artifact_art_info();
    state.viewed_artifact.total_purchased_ranks = 3;
    state.viewed_artifact.is_at_forge = true;
}

fn sample_artifact() -> ArtifactInfo {
    ArtifactInfo {
        item_id: 128_910,
        alt_item_id: 128_911,
        name: "Ashbringer".to_string(),
        icon: ARTIFACT_ICON.to_string(),
        total_xp: 12_500,
        points_spent: 3,
        quality: 6,
        artifact_appearance_id: 41,
        appearance_mod_id: 0,
        item_appearance_id: 0,
        alt_item_appearance_id: 0,
        alt_on_top: false,
        tier: 1,
        maxed: false,
        disabled: false,
        category: 1,
    }
}

fn sample_artifact_art_info() -> ArtifactArtInfo {
    ArtifactArtInfo {
        texture_kit: "Paladin".to_string(),
        title_name: ARTIFACT_TITLE.to_string(),
        title_color: ColorRgb {
            r: 1.0,
            g: 0.8,
            b: 0.2,
        },
        bar_connected_color: ColorRgb {
            r: 0.9,
            g: 0.7,
            b: 0.1,
        },
        bar_disconnected_color: ColorRgb {
            r: 0.2,
            g: 0.2,
            b: 0.2,
        },
        ui_model_scene_id: 700,
        spell_visual_kit_id: 800,
    }
}

fn load_artifact_ui(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (loaded, error): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ArtifactUI")"#)
        .expect("C_AddOns.LoadAddOn probe should run cleanly");
    assert!(
        loaded,
        "`{ROOT}` must load before knowledge tooltip leave probe; error={error:?}"
    );
}

const KNOWLEDGE_TOOLTIP_LEAVE_PROBE: &str = r#"
local mismatches = {}

local function expect(condition, message)
    if not condition then
        table.insert(mismatches, message)
    end
end

local originalHide = GameTooltip.Hide
local hideCalls = 0
GameTooltip.Hide = function(self)
    hideCalls = hideCalls + 1
    return originalHide(self)
end

local knowledgeFrame = CreateFrame("Frame", nil, UIParent)
local enterOk, enterError = pcall(function()
    ArtifactFrame:OnKnowledgeEnter(knowledgeFrame)
end)
local updateTooltipAfterEnter = knowledgeFrame.UpdateTooltip

local leaveOk, leaveError = pcall(function()
    ArtifactFrame:OnKnowledgeLeave(knowledgeFrame)
end)
local updateTooltipAfterLeave = knowledgeFrame.UpdateTooltip

GameTooltip.Hide = originalHide

expect(enterOk, "OnKnowledgeEnter error:" .. tostring(enterError))
expect(leaveOk, "OnKnowledgeLeave error:" .. tostring(leaveError))
expect(type(updateTooltipAfterEnter) == "function", "UpdateTooltip after enter:" .. type(updateTooltipAfterEnter))
expect(updateTooltipAfterLeave == nil, "UpdateTooltip after leave:" .. tostring(updateTooltipAfterLeave))
expect(hideCalls == 1, "GameTooltip Hide calls:" .. tostring(hideCalls))

return mismatches
"#;
