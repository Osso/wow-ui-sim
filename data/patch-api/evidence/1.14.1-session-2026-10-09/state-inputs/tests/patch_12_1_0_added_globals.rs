//! Retail 12.1.0 added Global API functions: state-backed behavior.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn eval_ok(env: &WowLuaEnv, code: &str) {
    let result: String = env.eval(code).unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn housing_layout_floor_range_follows_room_floors() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r#"
        if C_HousingLayout.GetHighestOccupiedFloorIndex() ~= 0 then return "empty-high" end
        if C_HousingLayout.GetLowestOccupiedFloorIndex() ~= 0 then return "empty-low" end
        return "ok"
        "#,
    );
    {
        let mut state = env.state().borrow_mut();
        state.housing.base_room_floors.insert(11, 2);
        state.housing.base_room_floors.insert(12, -1);
        state.housing.base_room_floors.insert(13, 0);
    }
    eval_ok(
        &env,
        r#"
        if C_HousingLayout.GetHighestOccupiedFloorIndex() ~= 2 then return "high" end
        if C_HousingLayout.GetLowestOccupiedFloorIndex() ~= -1 then return "low" end
        return "ok"
        "#,
    );
}

#[test]
fn housing_room_stairs_and_room_export_use_layout_rooms() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.housing.inside_owned_house = true;
        state.housing.base_room_floors.insert(11, 0);
        state.housing.base_room_floors.insert(12, 1);
        state.housing.base_room = Some(11);
        state.housing.stairwell_rooms.push(12);
    }
    eval_ok(
        &env,
        r#"
        if C_HousingLayout.RoomHasStairs(12) ~= true then return "stairs" end
        if C_HousingLayout.RoomHasStairs(11) ~= false then return "no-stairs" end
        if C_HousingLayout.RoomHasStairs("not-a-room") ~= false then return "invalid-stairs" end
        if C_HousingBlueprint.CanExportRoom(12) ~= true then return "export" end
        if C_HousingBlueprint.CanExportRoom(11) ~= false then return "base-room" end
        if C_HousingBlueprint.CanExportRoom(99) ~= false then return "unknown-room" end
        return "ok"
        "#,
    );
    env.state().borrow_mut().housing.inside_owned_house = false;
    eval_ok(
        &env,
        r#"
        if C_HousingBlueprint.CanExportRoom(12) ~= false then return "outside-house" end
        return "ok"
        "#,
    );
}

#[test]
fn housing_blueprint_export_types_follow_owned_location() {
    let env = WowLuaEnv::new().unwrap();
    let probe = r#"
        local T = Enum.HousingBlueprintType
        local function can(t) return C_HousingBlueprint.CanExportTypeFromCurrentLocation(t) end
        return string.format("%s %s %s %s %s",
            tostring(can(T.House)), tostring(can(T.Interior)), tostring(can(T.Exterior)),
            tostring(can(T.Room)), tostring(can(T.None)))
    "#;
    let outside: String = env.eval(probe).unwrap();
    assert_eq!(outside, "false false false false false");
    {
        let mut state = env.state().borrow_mut();
        state.housing.inside_owned_house = true;
        state.housing.room_player_is_in = Some(11);
    }
    let in_house: String = env.eval(probe).unwrap();
    assert_eq!(in_house, "true true false true false");
    {
        let mut state = env.state().borrow_mut();
        state.housing.inside_owned_house = false;
        state.housing.inside_owned_plot = true;
    }
    let on_plot: String = env.eval(probe).unwrap();
    assert_eq!(on_plot, "true false true false false");
}

#[test]
fn housing_blueprint_input_normalizes_hyperlinks() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r##"
        local code = C_HousingBlueprint.ExportBlueprint("abc")
        local link = C_HousingBlueprint.GetBlueprintHyperlink(code)
        if C_HousingBlueprint.UpdateBlueprintStringFromInput("  " .. code .. "\n") ~= code then
            return "trim"
        end
        if C_HousingBlueprint.UpdateBlueprintStringFromInput(link) ~= code then return "link" end
        if select('#', C_HousingBlueprint.UpdateBlueprintStringFromInput("   ")) ~= 0 then
            return "empty"
        end
        return "ok"
        "##,
    );
}

#[test]
fn housing_all_placement_budgets_map_budget_types() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.housing.max_indoor_placement_budget = Some(100);
        state.housing.max_outdoor_placement_budget = Some(50);
        state.housing.max_pet_placement_budget = Some(3);
        state.housing.spent_indoor_placement_budget = Some(20);
        state.housing.spent_outdoor_placement_budget = Some(5);
        state.housing.spent_pet_placement_budget = Some(1);
    }
    eval_ok(
        &env,
        r#"
        local maxIn, maxOut = C_HousingDecor.GetAllMaxPlacementBudgets()
        if maxIn ~= nil or maxOut ~= nil then return "outside-owned" end
        return "ok"
        "#,
    );
    env.state().borrow_mut().housing.inside_owned_house = true;
    eval_ok(
        &env,
        r#"
        local B = Enum.HousingBudgetType
        local maxIn, maxOut = C_HousingDecor.GetAllMaxPlacementBudgets()
        local spentIn, spentOut = C_HousingDecor.GetAllSpentPlacementBudgets()
        if maxIn[B.DecorPlacement] ~= 100 or maxIn[B.PetDecor] ~= 3 then return "max-in" end
        if maxOut[B.DecorPlacement] ~= 50 or maxOut[B.PetDecor] ~= nil then return "max-out" end
        if spentIn[B.DecorPlacement] ~= 20 or spentIn[B.PetDecor] ~= 1 then return "spent-in" end
        if spentOut[B.DecorPlacement] ~= 5 then return "spent-out" end
        return "ok"
        "#,
    );
}

#[test]
fn discord_user_name_reads_known_users() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .discord
        .user_names
        .insert("123456789".to_string(), "Moira".to_string());
    eval_ok(
        &env,
        r#"
        if C_Discord.GetDiscordUserName(123456789) ~= "Moira" then return "numeric" end
        if C_Discord.GetDiscordUserName("123456789") ~= "Moira" then return "string" end
        if C_Discord.GetDiscordUserName(42) ~= "" then return "unknown" end
        return "ok"
        "#,
    );
}

#[test]
fn battle_net_search_friends_filters_seeded_friends() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().bnet_friends[1].friend_tags = vec!["raid".to_string()];
    let search = |fields: &str| -> String {
        env.eval(&format!(
            r#"
            local info = {{ searchText = "", isOnline = false, isOffline = false, isDND = false,
                isAFK = false, isInQueue = false, isAvailableForQueue = false, tags = {{}} }}
            for key, value in pairs({{ {fields} }}) do info[key] = value end
            return table.concat(C_BattleNet.SearchFriends(info), ",")
            "#
        ))
        .unwrap()
    };
    assert_eq!(search(""), "1,2");
    assert_eq!(search(r#"searchText = "THR""#), "2");
    assert_eq!(search(r#"searchText = "lightbringer""#), "1");
    assert_eq!(search("isOnline = true"), "1");
    assert_eq!(search("isOffline = true"), "2");
    assert_eq!(search("isOnline = true, isOffline = true"), "1,2");
    assert_eq!(search(r#"tags = { "raid" }"#), "2");
    assert_eq!(search("isInQueue = true"), "");
}

#[test]
fn title_friend_requests_record_by_name_and_club_member() {
    use wow_ui_sim::lua_api::state::GuildMember;
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().world.guild_members = vec![
        GuildMember {
            name: "Uther".to_string(),
            rank_index: 1,
            online: true,
        },
        GuildMember {
            name: "Jaina".to_string(),
            rank_index: 2,
            online: false,
        },
    ];
    env.exec(
        r#"
        C_BattleNet.SendTitleFriendInviteByName("  Varian ")
        C_BattleNet.SendTitleFriendInviteByName("varian")
        C_BattleNet.SendTitleFriendInviteByName("")
        local ids = C_Club.GetClubMembers('guild-0')
        C_Club.SendTitleFriendRequest("guild-0", ids[2])
        C_Club.SendTitleFriendRequest("guild-0", ids[1])
        C_Club.SendTitleFriendRequest("guild-0", 'missing-member')
        C_Club.SendTitleFriendRequest("other-club", ids[2])
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().title_friend_requests,
        vec!["Varian".to_string(), "Jaina".to_string()]
    );
}

#[test]
fn battle_net_high_res_toggle_follows_installed_textures() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"SetCVar("useHighResTextures", "0")"#).unwrap();
    let before: bool = env
        .eval("return C_BattleNet.CanToggleHighResTexturesWithoutClientReload()")
        .unwrap();
    env.exec(r#"SetCVar("useHighResTextures", "1")"#).unwrap();
    let after: bool = env
        .eval("return C_BattleNet.CanToggleHighResTexturesWithoutClientReload()")
        .unwrap();
    assert!(!before);
    assert!(after);
}

#[test]
fn delves_lair_state_drives_lair_and_matchmade_raid_queries() {
    let env = WowLuaEnv::new().unwrap();
    let probe = r#"
        return string.format("%s %s %s %s", tostring(C_DelvesUI.HasActiveLair()),
            tostring(C_DelvesUI.HasActiveLFGLair()), tostring(C_DelvesUI.IsInLair()),
            tostring(C_LFGInfo.IsInMatchmadeRaidWithoutRoleRequirements()))
    "#;
    let idle: String = env.eval(probe).unwrap();
    assert_eq!(idle, "false false false false");
    env.state().borrow_mut().has_active_lair = true;
    let premade: String = env.eval(probe).unwrap();
    assert_eq!(premade, "true false false false");
    {
        let mut state = env.state().borrow_mut();
        state.active_lair_is_lfg = true;
        state.has_active_delve = true;
    }
    let inside: String = env.eval(probe).unwrap();
    assert_eq!(inside, "true true true false");
    {
        let mut state = env.state().borrow_mut();
        let template = state.party_members[0].clone();
        state.party_members = vec![template; 6];
        state.party_group_active = true;
    }
    let raid: String = env.eval(probe).unwrap();
    assert_eq!(raid, "true true true true");
}

#[test]
fn spell_last_category_cooldown_source_tracks_latest_start() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.spell_cooldown_categories.insert(19750, 77);
        state.spell_cooldown_categories.insert(642, 77);
        state.spell_cooldown_item_sources.insert(642, 5512);
    }
    eval_ok(
        &env,
        r#"
        if select('#', C_Spell.GetLastCategoryCooldownSource(77)) ~= 0 then return "idle" end
        A_Admin.SetSpellCooldown(19750, 30)
        local spellID, itemID = C_Spell.GetLastCategoryCooldownSource(77)
        if spellID ~= 19750 or itemID ~= nil then return "first" end
        return "ok"
        "#,
    );
    std::thread::sleep(std::time::Duration::from_millis(5));
    eval_ok(
        &env,
        r#"
        A_Admin.SetSpellCooldown(642, 30)
        local spellID, itemID = C_Spell.GetLastCategoryCooldownSource(77)
        if spellID ~= 642 or itemID ~= 5512 then return "latest" end
        if select('#', C_Spell.GetLastCategoryCooldownSource(78)) ~= 0 then return "other" end
        return "ok"
        "#,
    );
}

#[test]
fn spell_description_for_item_location_uses_spell_text() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r#"
        local location = { equipmentSlotIndex = 16 }
        local plain = C_Spell.GetSpellDescription(19750)
        if plain == "" then return "fixture" end
        if C_Spell.GetSpellDescriptionForItemLocation(19750, location) ~= plain then
            return "description"
        end
        if select('#', C_Spell.GetSpellDescriptionForItemLocation(999999999, location)) ~= 0 then
            return "unknown"
        end
        return "ok"
        "#,
    );
}

#[test]
fn cancel_aura_by_instance_id_removes_player_buff_and_fires_unit_aura() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r#"
        A_Admin.ClearBuffs()
        A_Admin.AddBuff(21562, "Power Word: Fortitude", 135987, 0, 0)
        A_Admin.AddDebuff(589, "Shadow Word: Pain", 136207, 18, 0, "Magic")
        local buff = C_UnitAuras.GetAuraDataBySpellName("player", "Power Word: Fortitude")
        local debuff = C_UnitAuras.GetAuraDataBySpellName("player", "Shadow Word: Pain")
        local removed
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("UNIT_AURA")
        frame:SetScript("OnEvent", function(_, _, unit, info)
            removed = unit == "player" and info.removedAuraInstanceIDs and info.removedAuraInstanceIDs[1]
        end)
        C_UnitAuras.CancelAuraByInstanceID("player", debuff.auraInstanceID)
        if removed ~= nil then return "debuff-event" end
        if not C_UnitAuras.GetAuraDataByAuraInstanceID("player", debuff.auraInstanceID) then
            return "debuff-removed"
        end
        C_UnitAuras.CancelAuraByInstanceID("player", buff.auraInstanceID)
        if removed ~= buff.auraInstanceID then return "event" end
        if C_UnitAuras.GetAuraDataByAuraInstanceID("player", buff.auraInstanceID) then
            return "buff-kept"
        end
        return "ok"
        "#,
    );
}

#[test]
fn cancel_temporary_enchantment_clears_only_temporary_weapon_enchants() {
    use wow_ui_sim::c_api::weapon_enchants::{TEMPORARY_ENCHANT_TYPE, WeaponEnchant};
    let env = WowLuaEnv::new().unwrap();
    let enchant = |enchant_type, enchant_id| WeaponEnchant {
        enchant_type,
        time_left: 60_000.0,
        charges: 0,
        enchant_id,
        icon_id: 0,
    };
    {
        let mut state = env.state().borrow_mut();
        state.weapon_enchants[0] = vec![enchant(TEMPORARY_ENCHANT_TYPE, 7001), enchant(1, 3368)];
        state.weapon_enchants[1] = vec![enchant(TEMPORARY_ENCHANT_TYPE, 7002)];
    }
    env.exec("C_PaperDollInfo.CancelTemporaryEnchantment(16); C_PaperDollInfo.CancelTemporaryEnchantment(5)")
        .unwrap();
    let state = env.state().borrow();
    let main_hand: Vec<u32> = state.weapon_enchants[0].iter().map(|e| e.enchant_id).collect();
    let off_hand: Vec<u32> = state.weapon_enchants[1].iter().map(|e| e.enchant_id).collect();
    assert_eq!(main_hand, vec![3368]);
    assert_eq!(off_hand, vec![7002]);
}

#[test]
fn transmog_slot_availability_follows_player_class() {
    let env = WowLuaEnv::new().unwrap();
    let probe = r#"
        local S = Enum.TransmogOutfitSlot
        return string.format("%s %s %s %s", tostring(C_TransmogOutfitInfo.CanPlayerTransmogSlot(S.Head)),
            tostring(C_TransmogOutfitInfo.CanPlayerTransmogSlot(S.WeaponMainHand)),
            tostring(C_TransmogOutfitInfo.CanPlayerTransmogSlot(S.WeaponRanged)),
            tostring(C_TransmogOutfitInfo.CanPlayerTransmogSlot(99)))
    "#;
    env.state().borrow_mut().player.class_index = 2;
    let paladin: String = env.eval(probe).unwrap();
    assert_eq!(paladin, "true true false false");
    env.state().borrow_mut().player.class_index = 3;
    let hunter: String = env.eval(probe).unwrap();
    assert_eq!(hunter, "true true true false");
}

#[test]
fn transmog_enabled_reads_account_availability() {
    let env = WowLuaEnv::new().unwrap();
    let enabled: bool = env.eval("return C_TransmogOutfitInfo.IsTransmogEnabled()").unwrap();
    env.state().borrow_mut().transmog_enabled = false;
    let disabled: bool = env.eval("return C_TransmogOutfitInfo.IsTransmogEnabled()").unwrap();
    assert!(enabled);
    assert!(!disabled);
}

#[test]
fn roleset_filters_round_trip_through_active_lists() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r#"
        if #C_Roleset.GetActiveBlockedRolesets() ~= 0 then return "initial-blocked" end
        if #C_Roleset.GetActiveAllowedRolesets() ~= 0 then return "initial-allowed" end
        C_Roleset.ApplyRolesetFilters({ "minimap", "microMenu" }, { "encounterUI" })
        local blocked = C_Roleset.GetActiveBlockedRolesets()
        local allowed = C_Roleset.GetActiveAllowedRolesets()
        if table.concat(blocked, ",") ~= "minimap,microMenu" then return "blocked" end
        if table.concat(allowed, ",") ~= "encounterUI" then return "allowed" end
        blocked[1] = "mutated"
        if C_Roleset.GetActiveBlockedRolesets()[1] ~= "minimap" then return "copy" end
        C_Roleset.ApplyRolesetFilters({}, { "unitFrames" })
        if #C_Roleset.GetActiveBlockedRolesets() ~= 0 then return "cleared" end
        if C_Roleset.GetActiveAllowedRolesets()[1] ~= "unitFrames" then return "replaced" end
        return "ok"
        "#,
    );
}

#[test]
fn recent_allies_search_filters_text_status_and_interests() {
    use wow_ui_sim::c_api::c_recent_allies::{
        RecentAllyCharacterData, RecentAllyData, RecentAllyInteractionData, RecentAllyStateData,
    };
    let ally = |name: &str, online: bool, dnd: bool, note: Option<&str>| RecentAllyData {
        state_data: RecentAllyStateData {
            is_online: online,
            is_dnd: dnd,
            is_afk: false,
            is_converted_legacy_friend: false,
            pin_expiration_date: None,
            friend_request_sent_this_session: false,
            current_location: None,
        },
        character_data: RecentAllyCharacterData {
            guid: format!("Player-1-{name}"),
            name: name.to_string(),
            full_name: format!("{name}-Realm"),
            realm_name: "Realm".to_string(),
            level: 80,
            class_id: 2,
            race_id: 1,
            sex: 2,
        },
        interaction_data: RecentAllyInteractionData {
            interactions: Vec::new(),
            note: note.map(str::to_string),
        },
    };
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.recent_allies.enabled = true;
        state.recent_allies.entries = vec![
            ally("Aster", true, false, Some("tank for keys")),
            ally("Birch", false, false, None),
            ally("Cedar", true, true, None),
        ];
    }
    let search = |fields: &str| -> String {
        env.eval(&format!(
            r#"
            local info = {{ searchText = "", isOnline = false, isDND = false, isAFK = false,
                isOffline = false, interests = {{}} }}
            for key, value in pairs({{ {fields} }}) do info[key] = value end
            local names = {{}}
            for _, ally in ipairs(C_RecentAllies.SearchRecentAllies(info)) do
                names[#names + 1] = ally.characterData.name
            end
            return table.concat(names, ",")
            "#
        ))
        .unwrap()
    };
    assert_eq!(search(""), "Aster,Birch,Cedar");
    assert_eq!(search(r#"searchText = "TANK""#), "Aster");
    assert_eq!(search(r#"searchText = "birch-realm""#), "Birch");
    assert_eq!(search("isOffline = true"), "Birch");
    assert_eq!(search("isDND = true, isOffline = true"), "Birch,Cedar");
    assert_eq!(search("interests = { 1 }"), "");
    env.state().borrow_mut().recent_allies.enabled = false;
    assert_eq!(search(""), "");
}

#[test]
fn quest_hub_relation_reads_hub_quests() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .quest_hub_related_quests
        .insert(7001, [84001, 84002].into_iter().collect());
    eval_ok(
        &env,
        r#"
        if C_QuestHub.IsQuestCurrentlyRelatedToHub(84001, 7001) ~= true then return "related" end
        if C_QuestHub.IsQuestCurrentlyRelatedToHub(84003, 7001) ~= false then return "other-quest" end
        if C_QuestHub.IsQuestCurrentlyRelatedToHub(84001, 7002) ~= false then return "other-hub" end
        return "ok"
        "#,
    );
}

#[test]
fn initiative_task_reward_scaling_applies_task_scale() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .neighborhood_task_reward_scales
        .insert(55, 1.5);
    eval_ok(
        &env,
        r#"
        if C_NeighborhoodInitiative.GetInitiativeTaskRewardScaling(55, 10) ~= 15 then return "scaled" end
        if C_NeighborhoodInitiative.GetInitiativeTaskRewardScaling(55, 3) ~= 4 then return "floor" end
        if C_NeighborhoodInitiative.GetInitiativeTaskRewardScaling(56, 10) ~= 10 then return "unscaled" end
        return "ok"
        "#,
    );
}

#[test]
fn random_training_ground_joins_queue_battlefield() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r#"
        C_PvP.JoinRandomTrainingGroundBattleground()
        local status, name = GetBattlefieldStatus(1)
        if status ~= "queued" or name ~= "Random Training Ground" then return "bg:" .. tostring(name) end
        C_PvP.JoinRandomTrainingGroundArena()
        status, name = GetBattlefieldStatus(1)
        if status ~= "queued" or name ~= "Random Training Ground Arena" then return "arena" end
        return "ok"
        "#,
    );
    let queued_updates = env
        .state()
        .borrow()
        .events
        .pending()
        .iter()
        .filter(|event| event.name == "UPDATE_BATTLEFIELD_STATUS")
        .count();
    assert_eq!(queued_updates, 2);
}

#[test]
fn close_fullscreen_browser_is_callable_without_browser() {
    let env = WowLuaEnv::new().unwrap();
    let returned: i32 = env
        .eval("return select('#', C_Browser.CloseFullscreenBrowser())")
        .unwrap();
    assert_eq!(returned, 0);
}
