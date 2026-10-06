#![cfg(feature = "gui")]

//! Tests for hero talent spec resolution.

#[path = "hero_talents/rendering.rs"]
mod hero_talents_rendering;

use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::render::shader::load_texture_or_crop;
use wow_ui_sim::texture::TextureManager;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn env_with_full_ui() -> WowLuaEnv {
    load_full_ui(env())
}

fn env_with_protection_ui() -> WowLuaEnv {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.player.class_index = 2;
        state.player.active_spec_index = 2;
        state.talents.switch_to_spec(66);
    }
    load_full_ui(env)
}

fn load_full_ui(env: WowLuaEnv) -> WowLuaEnv {
    env.set_screen_size(1024.0, 768.0);

    let ui = default_blizzard_ui_addons_path().expect("Blizzard UI cache should be synced");

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    let addons = discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if let Err(e) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }

    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events(&env);
    env.apply_post_event_workarounds();
    wow_ui_sim::startup::process_pending_timers(&env);
    wow_ui_sim::startup::fire_one_on_update_tick(&env);

    env
}

#[test]
fn test_protection_hero_subtrees() {
    let env = env();
    // Protection (specID=66) should get Templar (48) and Lightsmith (49)
    let result: String = env
        .eval(
            r#"
            local ids, level = C_ClassTalents.GetHeroTalentSpecsForClassSpec(1, 66)
            assert(ids, "subtree IDs should not be nil")
            assert(level == 71, "unlock level should be 71")
            table.sort(ids)
            return table.concat(ids, ",")
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "48,49",
        "Protection should have Templar(48) + Lightsmith(49)"
    );
}

#[test]
fn test_retribution_hero_subtrees() {
    let env = env();
    // Retribution (specID=70) should get Templar (48) and Herald of the Sun (50)
    let result: String = env
        .eval(
            r#"
            local ids, level = C_ClassTalents.GetHeroTalentSpecsForClassSpec(1, 70)
            assert(ids, "subtree IDs should not be nil")
            table.sort(ids)
            return table.concat(ids, ",")
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "48,50",
        "Retribution should have Templar(48) + Herald of the Sun(50)"
    );
}

#[test]
fn test_holy_hero_subtrees() {
    let env = env();
    // Holy (specID=65) should get Lightsmith (49) and Herald of the Sun (50)
    let result: String = env
        .eval(
            r#"
            local ids, level = C_ClassTalents.GetHeroTalentSpecsForClassSpec(1, 65)
            assert(ids, "subtree IDs should not be nil")
            table.sort(ids)
            return table.concat(ids, ",")
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "49,50",
        "Holy should have Lightsmith(49) + Herald of the Sun(50)"
    );
}

#[test]
fn test_subtree_info_has_selection_node_ids() {
    let env = env();
    // Templar (subtree 48) should have subTreeSelectionNodeIDs
    let result: String = env
        .eval(
            r#"
            local info = C_Traits.GetSubTreeInfo(1, 48)
            assert(info, "subtree info should not be nil")
            assert(info.name == "Templar", "name should be Templar, got: " .. tostring(info.name))
            assert(info.subTreeSelectionNodeIDs, "should have subTreeSelectionNodeIDs")
            assert(#info.subTreeSelectionNodeIDs > 0, "should have at least one selection node")
            return tostring(#info.subTreeSelectionNodeIDs)
            "#,
        )
        .unwrap();
    assert!(
        result.parse::<i32>().unwrap() > 0,
        "Templar should have selection nodes"
    );
}

#[test]
fn test_activate_hero_spec_lightsmith() {
    let env = env();
    // Activate Lightsmith by selecting entry 123361 on node 99838 (Protection's selection node)
    // Then verify: nodeInfo has the selection, and GetActiveHeroTalentSpec returns subtree 49
    let result: String = env
        .eval(
            r#"
            -- Clear auto-selected hero spec so we start from a clean state
            C_Traits.SetSelection(1, 99838, nil)
            assert(C_ClassTalents.GetActiveHeroTalentSpec() == nil, "should be nil after clearing")

            -- Selection node 99838 should be visible (Protection spec)
            local nodeInfo = C_Traits.GetNodeInfo(1, 99838)
            assert(nodeInfo.isVisible, "selection node should be visible for Protection")
            assert(nodeInfo.activeEntry == nil, "no entry selected yet")

            -- Activate Lightsmith: select entry 123361 (subtree 49) on node 99838
            local ok = C_Traits.SetSelection(1, 99838, 123361)
            assert(ok, "SetSelection should succeed")

            -- Verify nodeInfo now reflects the selection
            local updated = C_Traits.GetNodeInfo(1, 99838)
            assert(updated.activeEntry.entryID == 123361, "entry should be 123361, got: " .. tostring(updated.activeEntry.entryID))
            assert(updated.ranksPurchased == 1, "should have rank 1 after selection")

            -- Verify GetActiveHeroTalentSpec returns Lightsmith's subtree
            local active = C_ClassTalents.GetActiveHeroTalentSpec()
            assert(active == 49, "active hero spec should be 49 (Lightsmith), got: " .. tostring(active))

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_tcontains_nil_safe() {
    let env = env();
    // tContains should return false when passed nil table
    let result: bool = env.eval("return tContains(nil, 42)").unwrap();
    assert!(!result, "tContains(nil, x) should return false");
}

#[test]
fn test_set_atlas_numeric_element_id() {
    let env = env();
    // SetAtlas should accept numeric element IDs (e.g. iconElementID from subtree info)
    // Element 26680 = "talents-heroclass-paladin-lightsmith"
    let result: String = env
        .eval(
            r#"
            local t = UIParent:CreateTexture(nil, "ARTWORK")
            t:SetAtlas(26680)
            return t:GetAtlas() or "nil"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "talents-heroclass-paladin-lightsmith",
        "SetAtlas should resolve numeric element ID to atlas name"
    );
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn test_class_talents_switch_methods_update_seeded_spec_and_loadout_state() {
    crate::secure_aura_header_helpers::assert_seeded_helper_switch_lifecycle();
}

#[cfg(not(feature = "retail-12-0-5"))]
#[test]
fn test_class_talents_switch_methods_update_seeded_spec_and_loadout_state() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local protectionConfigs = C_ClassTalents.GetConfigIDsBySpecID(66)
            assert(#protectionConfigs == 2, "Protection should expose two seeded loadouts")
            assert(C_ClassTalents.GetActiveConfigID() == protectionConfigs[1], "Protection default loadout should start active")

            C_ClassTalents.SwitchToLoadoutByName("Protection Mythic+")
            local protectionMythic = C_ClassTalents.GetActiveConfigID()
            assert(protectionMythic == protectionConfigs[2], "SwitchToLoadoutByName should pick the named Protection loadout")
            assert(C_ClassTalents.GetLastSelectedSavedConfigID(66) == protectionMythic, "last selected Protection config should update")
            assert(C_Traits.GetConfigInfo(protectionMythic).name == "Protection Mythic+", "config info should expose the active loadout name")

            C_ClassTalents.SwitchToSpecializationByName("Holy")
            assert(GetSpecialization() == 1, "Holy should become the active specialization")
            local holyConfigs = C_ClassTalents.GetConfigIDsBySpecID(65)
            assert(#holyConfigs == 2, "Holy should expose two seeded loadouts")
            assert(C_ClassTalents.GetActiveConfigID() == holyConfigs[1], "SwitchToSpecializationByName should activate Holy's default loadout")
            assert(C_ClassTalents.GetLastSelectedSavedConfigID(65) == holyConfigs[1], "Holy's last selected config should track the active default")
            local holyHeroSpecs = C_ClassTalents.GetHeroTalentSpecsForClassSpec(1, 65)
            assert(tContains(holyHeroSpecs, C_ClassTalents.GetActiveHeroTalentSpec()), "Holy reset should auto-select one of Holy's hero subtrees")

            C_ClassTalents.SwitchToLoadoutByIndex(2)
            assert(C_ClassTalents.GetActiveConfigID() == holyConfigs[2], "SwitchToLoadoutByIndex should use the current specialization's loadout order")
            assert(C_Traits.GetConfigInfo(holyConfigs[2]).name == "Holy Raid", "Holy loadout names should be queryable")

            C_ClassTalents.SwitchToSpecializationByIndex(3)
            assert(GetSpecialization() == 3, "Retribution should become the active specialization")
            local retributionConfigs = C_ClassTalents.GetConfigIDsBySpecID(70)
            assert(C_ClassTalents.GetActiveConfigID() == retributionConfigs[1], "Retribution default loadout should become active")
            assert(C_ClassTalents.GetLastSelectedSavedConfigID(70) == retributionConfigs[1], "Retribution's last selected config should update")
            local retributionHeroSpecs = C_ClassTalents.GetHeroTalentSpecsForClassSpec(1, 70)
            assert(tContains(retributionHeroSpecs, C_ClassTalents.GetActiveHeroTalentSpec()), "Retribution reset should auto-select one of Retribution's hero subtrees")

            return table.concat({
                tostring(protectionMythic),
                tostring(holyConfigs[2]),
                tostring(retributionConfigs[1]),
            }, ",")
            "#,
        )
        .unwrap();

    assert_eq!(result, "202,102,301");
}

#[test]
fn test_load_config_switches_loadout_and_returns_ready() {
    let env = env();
    let (load_result, before, after, after_loadout): (i32, i32, i32, i32) = env
        .eval(
            r#"
            local configs = C_ClassTalents.GetConfigIDsBySpecID(66)
            assert(#configs == 2, "Protection should expose two seeded loadouts")

            local before = C_ClassTalents.GetActiveConfigID()
            assert(before == configs[1], "default loadout should start active")

            local result, err, newlyLearned = C_ClassTalents.LoadConfig(configs[2], true)
            assert(err == nil, "no error string expected on success")
            assert(newlyLearned == nil, "no newly-learned-nodes table expected on instant load")

            local after = C_ClassTalents.GetActiveConfigID()
            local lastSelected = C_ClassTalents.GetLastSelectedSavedConfigID(66)
            return result, before, after, lastSelected
            "#,
        )
        .unwrap();

    // Enum.LoadConfigResult.Ready == 3
    assert_eq!(load_result, 3);
    assert_ne!(after, before);
    assert_eq!(after, after_loadout);
}

#[test]
fn test_load_config_returns_no_changes_necessary_for_active_loadout() {
    let env = env();
    let (load_result, before, after): (i32, i32, i32) = env
        .eval(
            r#"
            local before = C_ClassTalents.GetActiveConfigID()
            local result, err, newlyLearned = C_ClassTalents.LoadConfig(before, true)
            assert(err == nil, "no error string expected when reloading active config")
            assert(newlyLearned == nil, "no newly-learned-nodes table expected on no-op load")

            local after = C_ClassTalents.GetActiveConfigID()
            return result, before, after
            "#,
        )
        .unwrap();

    // Enum.LoadConfigResult.NoChangesNecessary == 1
    assert_eq!(load_result, 1);
    assert_eq!(after, before);
}

#[test]
fn test_active_hero_node_icon_file_id_resolves_to_real_asset() {
    let env = env();
    let icon_id: i64 = env
        .eval(
            r#"
            local config = C_ClassTalents.GetActiveConfigID()
            local configInfo = C_Traits.GetConfigInfo(config)
            local treeID = configInfo and configInfo.treeIDs and configInfo.treeIDs[1]
            assert(treeID, "expected active trait tree")

            local nodes = C_Traits.GetTreeNodes(treeID) or {}
            for _, nodeID in ipairs(nodes) do
                local node = C_Traits.GetNodeInfo(config, nodeID)
                local entryID = node and node.activeEntry and node.activeEntry.entryID
                if entryID and node.subTreeID then
                    local entry = C_Traits.GetEntryInfo(config, entryID)
                    local definitionID = entry and entry.definitionID
                    if definitionID and definitionID > 0 then
                        local definition = C_Traits.GetDefinitionInfo(definitionID)
                        local spellID = definition and definition.spellID
                        if spellID and spellID > 0 then
                            local iconID = C_Spell.GetSpellTexture(spellID)
                            if type(iconID) == "number" and iconID > 0 then
                                return iconID
                            end
                        end
                    end
                end
            end

            error("expected at least one visible hero node with spell-backed icon file ID")
            "#,
        )
        .unwrap();

    let relative_path = wow_ui_sim::manifest_interface_data::get_texture_path(icon_id as u32)
        .unwrap_or_else(|| panic!("hero node icon file ID has no texture mapping: {icon_id}"));
    let texture_path = format!("Interface\\\\{}", relative_path.replace('/', "\\\\"));
    let mut mgr = TextureManager::new();
    let texture = load_texture_or_crop(&mut mgr, &texture_path).unwrap_or_else(|| {
        panic!("hero node spell texture did not resolve: {texture_path}");
    });

    assert!(
        texture.rgba.chunks_exact(4).any(|px| px[3] > 0),
        "hero node texture is fully transparent: {}",
        texture_path
    );
}

#[test]
fn test_active_hero_subtree_exposes_multiple_visible_nodes_and_edges() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            -- Force a deterministic active hero subtree for Protection.
            local ok = C_Traits.SetSelection(1, 99838, 123361) -- Lightsmith
            assert(ok, "expected SetSelection for hero subtree to succeed")
            assert(C_ClassTalents.GetActiveHeroTalentSpec() == 49, "expected active subtree 49")

            local treeID = C_ClassTalents.GetTraitTreeForSpec(66)
            local nodes = C_Traits.GetTreeNodes(treeID) or {}
            local total, visible, edgeReady = 0, 0, 0
            for _, nodeID in ipairs(nodes) do
                local node = C_Traits.GetNodeInfo(1, nodeID)
                if node and node.subTreeID == 49 then
                    total = total + 1
                    if node.isVisible then visible = visible + 1 end
                    if node.meetsEdgeRequirements then edgeReady = edgeReady + 1 end
                end
            end

            local root = C_Traits.GetNodeInfo(1, 95228)
            assert(root and root.isVisible, "expected representative hero node to be visible")
            assert(total >= 10, "expected many hero nodes in subtree; got " .. tostring(total))
            assert(visible >= 10, "expected most hero nodes visible; got " .. tostring(visible))
            assert(edgeReady >= 2, "expected at least two edge-ready hero nodes; got " .. tostring(edgeReady))

            return table.concat({tostring(total), tostring(visible), tostring(edgeReady)}, ",")
            "#,
        )
        .unwrap();

    let parts: Vec<u32> = result
        .split(',')
        .map(|s| s.parse::<u32>().expect("count should parse"))
        .collect();
    assert_eq!(parts.len(), 3, "expected total,visible,edgeReady");
    assert!(parts[0] >= 10, "unexpected total hero node count: {result}");
    assert!(
        parts[1] >= 10,
        "unexpected visible hero node count: {result}"
    );
    assert!(
        parts[2] >= 2,
        "unexpected edge-ready hero node count: {result}"
    );
}

#[test]
fn test_non_selectable_hero_nodes_do_not_show_selectable_glow() {
    let env = env_with_protection_ui();
    let result: String = env
        .eval(
            r#"
            local ok = C_Traits.SetSelection(1, 99838, 123361) -- Lightsmith
            assert(ok, "expected deterministic hero subtree selection")
            assert(C_ClassTalents.GetActiveHeroTalentSpec() == 49, "expected active subtree 49")

            assert(PlayerSpellsUtil and PlayerSpellsUtil.OpenToClassTalentsTab, "expected class talents UI helper")
            PlayerSpellsUtil.OpenToClassTalentsTab()

            local frame = PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame
            assert(frame and frame.EnumerateAllTalentButtons, "expected class talents frame")

            local selectable = 0
            local nonSelectable = 0
            local nonSelectableGlowShown = 0

            for button in frame:EnumerateAllTalentButtons() do
                local node = button:GetNodeInfo()
                if node and node.subTreeID == 49 and button:IsShown() then
                    if button:IsSelectable() then
                        selectable = selectable + 1
                    else
                        nonSelectable = nonSelectable + 1
                        local glow = button.SelectableGlow
                        if glow and glow:IsShown() then
                            nonSelectableGlowShown = nonSelectableGlowShown + 1
                        end
                    end
                end
            end

            assert(selectable >= 1, "expected at least one selectable hero node")
            assert(nonSelectable >= 1, "expected at least one non-selectable hero node")
            assert(nonSelectableGlowShown == 0, "non-selectable hero nodes should not show selectable glow")

            return table.concat({
                tostring(selectable),
                tostring(nonSelectable),
                tostring(nonSelectableGlowShown),
            }, ",")
            "#,
        )
        .unwrap();

    let parts: Vec<u32> = result
        .split(',')
        .map(|s| s.parse::<u32>().expect("count should parse"))
        .collect();
    assert_eq!(
        parts.len(),
        3,
        "expected selectable,nonSelectable,glowShown"
    );
    assert!(
        parts[0] >= 1,
        "unexpected selectable hero node count: {result}"
    );
    assert!(
        parts[1] >= 1,
        "unexpected non-selectable hero node count: {result}"
    );
    assert_eq!(
        parts[2], 0,
        "non-selectable hero nodes unexpectedly showed selectable glow: {result}"
    );
}
