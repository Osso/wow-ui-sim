//! Knowledge tooltip meta-power behavior for `Blizzard_ArtifactUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use wow_ui_sim::lua_api::state::{ArtifactArtInfo, ArtifactInfo, ColorRgb, MetaPowerEntry};

const ROOT: &str = "Blizzard_ArtifactUI";
const ARTIFACT_ICON: &str = "Interface/Icons/inv_sword_2h_artifactashbringer_d_01";
const ARTIFACT_TITLE: &str = "The Ashbringer";
// ArtifactUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
timerunning_util_panel_fixture! {
#[test]
fn knowledge_tooltip_lists_meta_power_descriptions_with_blank_separator_only_between_matches() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        seed_viewed_artifact_with_meta_powers(env);
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(KNOWLEDGE_TOOLTIP_META_POWERS_PROBE)
            .expect("ArtifactUI knowledge tooltip meta-power probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` knowledge tooltip must list artifact title, purchased ranks, and \
             only non-nil meta-power descriptions; mismatches: {mismatches:?}"
        );
    });
}
}

fn seed_viewed_artifact_with_meta_powers(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.viewed_artifact.info = Some(sample_artifact());
    state.viewed_artifact.art_info = sample_artifact_art_info();
    state.viewed_artifact.total_purchased_ranks = 3;
    state.viewed_artifact.is_at_forge = true;
    state.viewed_artifact.meta_powers = vec![
        MetaPowerEntry {
            spell_id: 12_345,
            cost: 10,
            current_rank: 1,
        },
        MetaPowerEntry {
            spell_id: 67_890,
            cost: 20,
            current_rank: 2,
        },
    ];
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
        "`{ROOT}` must load before knowledge tooltip probe; error={error:?}"
    );
}

const KNOWLEDGE_TOOLTIP_META_POWERS_PROBE: &str = r#"
local mismatches = {}

local function expect(condition, message)
    if not condition then
        table.insert(mismatches, message)
    end
end

local function collectTooltipLines(descriptionBySpellID)
    local originalGetSpellDescription = C_Spell.GetSpellDescription
    C_Spell.GetSpellDescription = function(spellID)
        return descriptionBySpellID[spellID]
    end

    GameTooltip:Hide()
    GameTooltip:ClearLines()
    local knowledgeFrame = CreateFrame("Frame", nil, UIParent)
    local ok, errorMessage = pcall(function()
        ArtifactFrame:OnKnowledgeEnter(knowledgeFrame)
    end)
    C_Spell.GetSpellDescription = originalGetSpellDescription

    if not ok then
        return nil, "OnKnowledgeEnter error:" .. tostring(errorMessage)
    end

    local lines = {}
    for lineIndex = 1, GameTooltip:NumLines() do
        local line = GameTooltip:GetLeftLine(lineIndex)
        table.insert(lines, line and line:GetText() or "")
    end
    return lines
end

local allDescriptions, allError = collectTooltipLines({
    [12345] = "First meta power description.",
    [67890] = "Second meta power description.",
})

local oneDescription, oneError = collectTooltipLines({
    [12345] = "First meta power description.",
})

expect(allDescriptions ~= nil, allError or "all descriptions should collect")
expect(oneDescription ~= nil, oneError or "one description should collect")

if allDescriptions then
    expect(#allDescriptions == 5, "all description line count:" .. tostring(#allDescriptions))
    expect(allDescriptions[1] == "The Ashbringer", "all title:" .. tostring(allDescriptions[1]))
    expect(
        allDescriptions[2] == ARTIFACTS_NUM_PURCHASED_RANKS:format(3),
        "all purchased ranks:" .. tostring(allDescriptions[2])
    )
    expect(allDescriptions[3] == "First meta power description.", "first description:" .. tostring(allDescriptions[3]))
    expect(allDescriptions[4] == " ", "separator:" .. tostring(allDescriptions[4]))
    expect(allDescriptions[5] == "Second meta power description.", "second description:" .. tostring(allDescriptions[5]))
end

if oneDescription then
    expect(#oneDescription == 3, "one description line count:" .. tostring(#oneDescription))
    expect(oneDescription[1] == "The Ashbringer", "one title:" .. tostring(oneDescription[1]))
    expect(
        oneDescription[2] == ARTIFACTS_NUM_PURCHASED_RANKS:format(3),
        "one purchased ranks:" .. tostring(oneDescription[2])
    )
    expect(oneDescription[3] == "First meta power description.", "single description:" .. tostring(oneDescription[3]))
end

return mismatches
"#;
