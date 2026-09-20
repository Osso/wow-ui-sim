#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load(env: &WowLuaEnv, path: &str) {
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.exec(&std::fs::read_to_string(root.join(path)).unwrap())
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
