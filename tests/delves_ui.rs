use wow_ui_sim::lua_api::WowLuaEnv;

const DELVES_UI_SCRIPT: &str = r#"
    if C_DelvesUI.GetDelveEntranceBackgroundWidgetSetID() ~= 5501 then
        return "wrong_background_widget_set"
    end

    if C_DelvesUI.GetDelveEntranceHeaderString() ~= "Fungal Folly" then
        return "wrong_header"
    end

    if C_DelvesUI.GetDelveEntranceDescriptionString() ~= "The Fungal Folly winds deeper with every tier." then
        return "wrong_description"
    end

    if C_DelvesUI.GetDelveEntranceMapID() ~= 2339 then
        return "wrong_map_id"
    end

    if C_DelvesUI.HasActiveDelve() ~= false or C_DelvesUI.HasActiveDelve(0) ~= false then
        return "wrong_default_active_delve"
    end

    if C_DelvesUI.HasActiveDelve(2339) ~= false then
        return "former_map_argument_changed_active_delve"
    end

    if C_DelvesUI.GetTieredEntranceOptionalAffixTraitTreeID() ~= 77001 then
        return "wrong_optional_affix_tree"
    end

    local companionTreeID = C_DelvesUI.GetTraitTreeForCompanion(C_DelvesUI.GetCompanionInfoForActivePlayer())
    if companionTreeID ~= 9201 then
        return "wrong_companion_tree"
    end

    local companionConfigID = C_Traits.GetConfigIDByTreeID(companionTreeID)
    if companionConfigID ~= 9201 then
        return "wrong_companion_config"
    end

    local companionNodes = C_Traits.GetTreeNodes(companionTreeID)
    if #companionNodes ~= 3 then
        return "wrong_companion_node_count:" .. tostring(#companionNodes)
    end

    local activeTier = C_DelvesUI.GetActiveDelveTier()
    if not activeTier or activeTier.tier ~= 4 or activeTier.modifierUIWidgetSetID ~= 4404 then
        return "wrong_active_tier"
    end

    local tiers = C_DelvesUI.GetDelveEntranceTiers()
    if #tiers ~= 5 then
        return "wrong_tier_count:" .. tostring(#tiers)
    end

    if tiers[1].tier ~= 1 or tiers[1].unlocked ~= true or tiers[1].suggestedILvl ~= 571 then
        return "wrong_first_tier"
    end

    if tiers[5].tier ~= 5 or tiers[5].unlocked ~= false or tiers[5].lockedReason ~= "Complete Tier 4 to unlock this delve tier." then
        return "wrong_locked_tier"
    end

    local tierFourEnabled, tierFourFailure = C_DelvesUI.IsDelveEntranceTierEnabled(4)
    if tierFourEnabled ~= true or tierFourFailure ~= nil then
        return "wrong_enabled_tier_result"
    end

    local tierFiveEnabled, tierFiveFailure = C_DelvesUI.IsDelveEntranceTierEnabled(5)
    if tierFiveEnabled ~= false or tierFiveFailure ~= "Complete Tier 4 to unlock this delve tier." then
        return "wrong_locked_tier_result"
    end

    local unknownEnabled, unknownFailure = C_DelvesUI.IsDelveEntranceTierEnabled(99)
    if unknownEnabled ~= false or unknownFailure ~= "Unknown tier" then
        return "wrong_unknown_tier_result"
    end

    local worldTierDifficulty = C_DelvesUI.GetWorldTierDifficultyForActivePlayer()
    if worldTierDifficulty ~= Enum.WorldTierDifficulty.Normal then
        return "wrong_world_tier_difficulty:" .. tostring(worldTierDifficulty)
    end

    C_DelvesUI.SelectDelveEntranceTier(2)
    local selectedActiveTier = C_DelvesUI.GetActiveDelveTier()
    if not selectedActiveTier or selectedActiveTier.tier ~= 2 or selectedActiveTier.modifierUIWidgetSetID ~= 4402 then
        return "select_tier_not_applied"
    end

    return "ok"
"#;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn delves_ui_entrance_methods_use_seeded_tier_state() {
    let env = env();
    let result: String = env
        .eval(DELVES_UI_SCRIPT)
        .expect("seeded C_DelvesUI methods should be queryable");
    assert_eq!(result, "ok");
}
