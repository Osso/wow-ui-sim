#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load(env: &WowLuaEnv, path: &str) {
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.exec(&std::fs::read_to_string(root.join(path)).unwrap())
        .unwrap();
}

#[test]
fn forever_stable_slot_constants_preserve_common_pet_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local pets = Constants.PetConsts
        assert(pets.MAX_STABLE_SLOTS == 2, "missing Forever stable capacity")
        assert(pets.NUM_PET_SLOTS_HUNTER == 3, "missing Forever hunter slot count")
        assert(pets.MAX_SUMMONABLE_PETS == 25)
        assert(pets.PETNUMBER_INVALIDPET == 0)
        assert(pets.PETNUMBER_INVALIDSLOT == -1)
        assert(pets.PETNUMBER_PENDINGPET == -1)
        assert(Constants.PetConsts_PostCata.MAX_STABLE_SLOTS == 200)
        assert(Constants.PetConsts_PostCata.NUM_PET_SLOTS_HUNTER == 205)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_stable_slot_constants_survive_bootstrap_restore() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        stablePetConstants = Constants.PetConsts
        stablePetConstants.addonMarker = "retained"
        assert(stablePetConstants.MAX_STABLE_SLOTS == 2)
        assert(stablePetConstants.NUM_PET_SLOTS_HUNTER == 3)
        "#,
    )
    .unwrap();
    for _ in 0..2 {
        env.loader_env().restore_post_cleanup_globals().unwrap();
        env.exec(
            r#"
            assert(Constants.PetConsts == stablePetConstants)
            assert(Constants.PetConsts.addonMarker == "retained")
            assert(Constants.PetConsts.MAX_STABLE_SLOTS == 2)
            assert(Constants.PetConsts.NUM_PET_SLOTS_HUNTER == 3)
            assert(Constants.PetConsts.MAX_SUMMONABLE_PETS == 25)
            assert(Constants.PetConsts.PETNUMBER_INVALIDSLOT == -1)
            "#,
        )
        .unwrap();
    }
}

#[test]
fn forever_finite_events_register_deliver_and_reject_unknown() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local events = {"CHAT_MSG_COLLECTED_APPEARANCE", "CHAT_MSG_GUILD_DISCORD", "UNIT_AURA_BLOCKED"}
        for _, event in ipairs(events) do
            local frame = CreateFrame("Frame")
            local received = {}
            frame:SetScript("OnEvent", function(self, name, first, second)
                assert(self == frame)
                received[#received + 1] = {name, first, second}
            end)
            frame:RegisterEvent(event)
            assert(frame:IsEventRegistered(event), event)
            A_Admin.FireEvent(event, "player", 42)
            assert(#received == 1, event .. " callback missing")
            assert(received[1][1] == event)
            assert(received[1][2] == "player" and received[1][3] == 42)
            frame:UnregisterEvent(event)
            A_Admin.FireEvent(event, "player", 43)
            assert(#received == 1, event .. " callback after unregister")
        end
        local frame = CreateFrame("Frame")
        local unknown = "WOW_SIM_INVENTED_FOREVER_EVENT"
        assert(not pcall(frame.RegisterEvent, frame, unknown))
        assert(not frame:IsEventRegistered(unknown))
        "#,
    )
    .unwrap();
}

#[test]
fn forever_aura_block_list_cleared_register_event_delivers_unit_payload() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local event = "UNIT_AURA_BLOCK_LIST_CLEARED"
        local frame = CreateFrame("Frame")
        local received = {}
        frame:SetScript("OnEvent", function(self, name, ...)
            assert(self == frame and name == event)
            assert(select('#', ...) == 1, "cleared event must deliver one unit")
            received[#received + 1] = ...
        end)
        frame:RegisterEvent(event)
        assert(frame:IsEventRegistered(event), "cleared event registration missing")
        A_Admin.FireEvent(event, "player")
        A_Admin.FireEvent(event, "target")
        assert(#received == 2, "unfiltered cleared event must deliver both units")
        assert(received[1] == "player" and received[2] == "target")
        frame:UnregisterEvent(event)
        assert(not frame:IsEventRegistered(event))
        A_Admin.FireEvent(event, "player")
        assert(#received == 2, "cleared event delivered after unregister")
        "#,
    )
    .expect("Forever RegisterEvent must accept and deliver the declared cleared event");
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn forever_aura_block_list_cleared_register_unit_event_filters_target() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local event = "UNIT_AURA_BLOCK_LIST_CLEARED"
        local frame = CreateFrame("Frame")
        local received = {}
        frame:SetScript("OnEvent", function(self, name, ...)
            assert(self == frame and name == event)
            assert(select('#', ...) == 1, "cleared event must deliver one unit")
            received[#received + 1] = ...
        end)
        frame:RegisterUnitEvent(event, "player")
        local registered, unit = frame:IsEventRegistered(event)
        assert(registered and unit == "player", "player unit registration missing")
        A_Admin.FireEvent(event, "target")
        assert(#received == 0, "player registration received mismatched target")
        A_Admin.FireEvent(event, "player")
        assert(#received == 1 and received[1] == "player", "player delivery missing")
        A_Admin.FireEvent(event, "target")
        assert(#received == 1, "player registration received subsequent target")
        frame:UnregisterEvent(event)
        assert(not frame:IsEventRegistered(event))
        A_Admin.FireEvent(event, "player")
        assert(#received == 1, "unit event delivered after unregister")
        "#,
    )
    .expect("Forever RegisterUnitEvent must accept the cleared event and filter units");
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn forever_aura_block_list_cleared_unknown_event_control_rejects_both_methods() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local unknown = "WOW_SIM_INVENTED_FOREVER_REGISTRATION_CONTROL"
        for _, method in ipairs({"RegisterEvent", "RegisterUnitEvent"}) do
            local frame = CreateFrame("Frame")
            local ok, message = pcall(frame[method], frame, unknown, "player")
            assert(not ok, method .. " must reject unrelated unknown events")
            assert(type(message) == "string" and string.find(message, unknown, 1, true),
                method .. " rejection must identify the unknown name")
            assert(not frame:IsEventRegistered(unknown), "unknown registration retained")
        end
        "#,
    )
    .expect("Forever registration must remain strict for unrelated unknown events");
}

#[test]
fn forever_player_swing_preserves_payload_order_and_rejects_unknown_events() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        local received = {}
        frame:SetScript("OnEvent", function(self, event, ...)
            assert(self == frame and event == "PLAYER_SWING")
            assert(select('#', ...) == 2, "swing payload arity changed")
            local duration, swingType = ...
            assert(type(duration) == "number" and type(swingType) == "number")
            received[#received + 1] = {duration, swingType}
        end)
        frame:RegisterEvent("PLAYER_SWING")
        local payloads = {{1.60, 0}, {0.80, 1}, {2.091, 2}}
        for index, payload in ipairs(payloads) do
            A_Admin.FireEvent("PLAYER_SWING", payload[1], payload[2])
            assert(#received == index, "swing callback was not synchronous")
            assert(received[index][1] == payload[1], "duration moved or changed")
            assert(received[index][2] == payload[2], "swing type moved or changed")
        end
        frame:UnregisterEvent("PLAYER_SWING")
        A_Admin.FireEvent("PLAYER_SWING", 3.0, 0)
        assert(#received == 3, "swing callback ran after unregister")
        local unknown = "WOW_SIM_INVENTED_PLAYER_SWING"
        assert(not pcall(frame.RegisterEvent, frame, unknown))
        assert(not frame:IsEventRegistered(unknown))
        "#,
    )
    .unwrap();
}

#[test]
fn forever_player_swing_range_preserves_boolean_payloads() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        local received = {}
        frame:SetScript("OnEvent", function(self, event, ...)
            assert(self == frame and event == "PLAYER_SWING_RANGE_UPDATE")
            assert(select('#', ...) == 3)
            local swingType, isInRange, checksRange = ...
            assert(type(swingType) == "number")
            assert(type(isInRange) == "boolean" and type(checksRange) == "boolean")
            received[#received + 1] = {swingType, isInRange, checksRange}
        end)
        frame:RegisterEvent("PLAYER_SWING_RANGE_UPDATE")
        local types = Enum.PlayerSwingType
        local payloads = {
            {types.MainHand, false, true},
            {types.OffHand, true, true},
            {types.Ranged, false, false},
        }
        for index, payload in ipairs(payloads) do
            A_Admin.FireEvent("PLAYER_SWING_RANGE_UPDATE", unpack(payload))
            assert(#received == index, "range callback was not synchronous")
            for field = 1, 3 do assert(received[index][field] == payload[field]) end
        end
        frame:UnregisterEvent("PLAYER_SWING_RANGE_UPDATE")
        A_Admin.FireEvent("PLAYER_SWING_RANGE_UPDATE", types.MainHand, true, false)
        assert(#received == 3)
        assert(not pcall(frame.RegisterEvent, frame, "WOW_SIM_INVENTED_SWING_RANGE"))
        "#,
    )
    .unwrap();
}

#[test]
fn forever_player_swing_enum_and_registration_survive_bootstrap_cleanup() {
    let env = WowLuaEnv::new().unwrap();
    let check = r#"
        local types = assert(Enum.PlayerSwingType, "missing player swing enum")
        assert(types.MainHand == 0 and types.OffHand == 1 and types.Ranged == 2)
        local count = 0
        for _ in pairs(types) do count = count + 1 end
        assert(count == 3)
        local meta = assert(Enum.PlayerSwingTypeMeta, "missing player swing metadata")
        assert(meta.MinValue == 0 and meta.MaxValue == 2 and meta.NumValues == 3)
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("PLAYER_SWING")
        assert(frame:IsEventRegistered("PLAYER_SWING"))
        frame:UnregisterEvent("PLAYER_SWING")
        frame:RegisterEvent("PLAYER_SWING_RANGE_UPDATE")
        assert(frame:IsEventRegistered("PLAYER_SWING_RANGE_UPDATE"))
        frame:UnregisterEvent("PLAYER_SWING_RANGE_UPDATE")
    "#;
    env.exec(check).unwrap();
    env.exec("Enum.PlayerSwingType = nil; Enum.PlayerSwingTypeMeta = nil")
        .unwrap();
    env.loader_env().restore_post_cleanup_globals().unwrap();
    env.exec(check).unwrap();
}

#[test]
fn forever_aura_sound_trigger_values_survive_cleanup() {
    let env = WowLuaEnv::new().unwrap();
    let check = r#"
        local triggers = assert(Enum.UnitAuraSoundTrigger, "missing aura sound triggers")
        assert(triggers.Added == 0)
        assert(triggers.ApplicationsIncreased == 1)
        assert(triggers.Removed == 2)
        local count = 0
        for _ in pairs(triggers) do count = count + 1 end
        assert(count == 3)
        local meta = assert(Enum.UnitAuraSoundTriggerMeta)
        assert(meta.MinValue == 0 and meta.MaxValue == 2 and meta.NumValues == 3)
    "#;
    env.exec(check).unwrap();
    for _ in 0..2 {
        env.exec("Enum.UnitAuraSoundTrigger = nil; Enum.UnitAuraSoundTriggerMeta = nil")
            .unwrap();
        env.loader_env().restore_post_cleanup_globals().unwrap();
        env.exec(check).unwrap();
    }
}

#[test]
fn forever_bag_index_enumerates_disjoint_bank_tabs() {
    let env = WowLuaEnv::new().unwrap();
    // Exact enumeration loop from BetterBags core/constants.lua at 411a6f6ee1ea.
    // Expected IDs come from Forever 69913 BagIndexConstantsDocumentation.
    env.exec(
        r#"
        local function enumerateBagIndices(prefix)
            local ids = {}
            local n = 1
            while Enum.BagIndex[prefix .. n] ~= nil do
                ids[#ids + 1] = Enum.BagIndex[prefix .. n]
                n = n + 1
            end
            return ids
        end
        local seen = {}
        for prefix, first in pairs({CharacterBankTab_ = 6, AccountBankTab_ = 15}) do
            local ids = enumerateBagIndices(prefix)
            assert(#ids == 9, prefix .. " must enumerate nine tabs")
            assert(Enum.BagIndex[prefix .. 10] == nil, "unexpected tenth tab")
            for index, id in ipairs(ids) do
                assert(id == first + index - 1, prefix .. index .. " has wrong ID")
                assert(not seen[id], "bank tab IDs overlap")
                seen[id] = true
            end
        end
        for id = 6, 23 do assert(seen[id], "missing bank tab ID " .. id) end
        "#,
    )
    .unwrap();
}

#[test]
fn forever_bag_index_preserves_non_bank_values_and_exact_metadata() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local expected = {
            Accountbanktab = -3, Characterbanktab = -2, Keyring = -1,
            Backpack = 0, Bag_1 = 1, Bag_2 = 2, Bag_3 = 3, Bag_4 = 4,
            ReagentBag = 5,
        }
        for name, value in pairs(expected) do
            assert(Enum.BagIndex[name] == value, name)
        end
        local count = 0
        for _ in pairs(Enum.BagIndex) do count = count + 1 end
        assert(count == 27, "BagIndex must publish exactly 27 members")
        local meta = Enum.BagIndexMeta
        assert(meta.MinValue == -3)
        assert(meta.MaxValue == 23)
        assert(meta.NumValues == 27)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_finite_constants_publish_legacy_and_level_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local legacy = Constants.LegacyConsts
        assert(legacy, "missing LegacyConsts")
        assert(legacy.LEGACY_REWARD_TRACK_FACTION_ID == 2802)
        assert(legacy.LEGACY_POINTS_TRAIT_CURRENCY_ID == 4225)
        assert(legacy.LEGACY_TREE_PROFESSIONS_ID == 1187)
        assert(legacy.LEGACY_TREE_ADVENTURE_ID == 1188)
        assert(legacy.LEGACY_TREE_PROGRESSION_ID == 1189)
        assert(legacy.LEGACY_TREE_ADVENTURE_TALENTED_NODE_ID == 110298)
        local level = Constants.LevelConstsExposed
        assert(level.MIN_RES_SICKNESS_LEVEL == 10)
        assert(level.MIN_ACHIEVEMENT_LEVEL == 10)
        assert(level.MIN_TALENT_LEVEL == 10)
    "#,
    )
    .unwrap();
}

#[test]
fn forever_finite_constants_camelot_talent_unlock_without_config() {
    let env = WowLuaEnv::new().unwrap();
    // The override file extends these mixin tables; no API or consumer is mocked.
    env.exec("PlayerSpellsMicroButtonMixin = {}; CharacterMicroButtonMixin = {}")
        .unwrap();
    load(
        &env,
        "Blizzard_MicroMenu/Camelot/MainMenuBarMicroButtonsOverrides.lua",
    );
    env.exec(
        r#"
        assert(C_Traits.GetConfigIDByTreeID(1188) == nil)
        assert(PlayerSpellsMicroButtonMixin:GetTalentUnlockLevel() == 10)
    "#,
    )
    .unwrap();
}

#[test]
fn forever_finite_constants_construct_minimap_filters() {
    let env = WowLuaEnv::new().unwrap();
    load(&env, "Blizzard_Minimap/Camelot/MinimapConstants.lua");
    env.exec("assert(MinimapConstants.OPTIONAL_FILTERS[8388608] == true); assert(Enum.MinimapTrackingFilter.VendorAmmo == 16777216)").unwrap();
}

#[test]
fn forever_finite_constants_load_ping_and_transmog_consumers() {
    let env = WowLuaEnv::new().unwrap();
    load(&env, "Blizzard_PingUI/Blizzard_PingManager.lua");
    load(&env, "Blizzard_TransmogShared/Blizzard_TransmogShared.lua");
    env.exec("assert(type(PingManager.SetupDefaultPingOptions) == 'function'); assert(type(TransmogUtil.GetInfoForEquippedSlot) == 'function')").unwrap();
}

#[test]
fn forever_stance_override_initializes_vendor_mapping() {
    let env = WowLuaEnv::new().unwrap();
    // Supply the page owner's anchor contract without loading its unrelated UI.
    env.exec(
        r#"
        GamepadActionBarMixin = {}
        GamepadActionBarPageUnitMixin = {
            GetTopAnchorFrame = function(self) return self.top end,
            GetLeftAnchorFrame = function(self) return self.left end,
            GetRightAnchorFrame = function(self) return self.right end,
            GetBottomAnchorFrame = function(self) return self.bottom end,
        }
    "#,
    )
    .unwrap();
    load(
        &env,
        "Blizzard_GamepadActionBars/GamepadOverrideBarMixin.lua",
    );
    load(&env, "Blizzard_GamepadActionBars/StanceBar.lua");
    env.exec(r#"
        local page = CreateFrame("Frame")
        for _, name in ipairs({"top", "left", "right", "bottom"}) do
            page[name] = CreateFrame("Frame", nil, page)
            page[name].Bar = CreateFrame("Frame", nil, page[name])
        end
        local bar = CreateFrame("Frame", nil, page)
        Mixin(bar, GamepadStanceBarMixin)
        bar.overrideMap = {}
        bar.overrideCVar = "GamepadStanceBarOverride"
        bar.isOverrideBarActive = false
        bar.pagingUnitOwner = page
        page.actionBars = {stanceBar = bar}
        GamepadActionBarPageUnitMixin.InitializeStanceBar(page)
        assert(bar:GetParent() == page.right)
        assert(bar:GetActionBarLinkedWithOverrideBar() == page.right.Bar)
        local anchors = {false, page.left, page.right, page.bottom,
            page.top, page.left, page.right, page.bottom,
            page.top, page.left, page.right, page.bottom}
        for value = 1, 12 do
            assert(bar:GetLinkedOverrideBarAnchorFrame(value) == (anchors[value] or nil))
            assert(bar:GetLinkedOverrideBarPage(value) == math.floor((value - 1) / 4) + (value == 1 and 0 or 1))
        end
        assert(SetCVar("GamepadStanceBarOverride", "1"))
        bar:ApplyInitialOverridePositioning()
        assert(bar:GetParent() == nil)
        assert(bar:GetActionBarLinkedWithOverrideBar() == bar)
        local names = {"None", "Page1LeftBar", "Page1RightBar", "Page1BottomBar",
            "Page2TopBar", "Page2LeftBar", "Page2RightBar", "Page2BottomBar",
            "Page3TopBar", "Page3LeftBar", "Page3RightBar", "Page3BottomBar"}
        for value, name in ipairs(names) do assert(Enum.GamepadStanceBarOverride[name] == value) end
        local meta = Enum.GamepadStanceBarOverrideMeta
        assert(meta.MinValue == 1 and meta.MaxValue == 12 and meta.NumValues == 12)
    "#).unwrap();
}

#[test]
fn forever_finite_constants_publish_source_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(Enum.PingResult.FailedSilent == 8)
        assert(Constants.Transmog.NoTransmogID == 0)
        local names = {"SpecialPageTopBar", "Page1LeftBar", "Page1RightBar", "Page1BottomBar", "Page2TopBar", "Page2LeftBar", "Page2RightBar", "Page2BottomBar", "Page3TopBar", "Page3LeftBar", "Page3RightBar", "Page3BottomBar"}
        for index, name in ipairs(names) do assert(Enum.GamepadPossessBarOverride[name] == index, name) end
        assert(Enum.GamepadPossessBarOverrideMeta.NumValues == 12)
        assert(Enum.PingResultMeta.MaxValue == 8)
    "#).unwrap();
}
