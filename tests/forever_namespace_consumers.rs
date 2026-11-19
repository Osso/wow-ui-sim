//! Complete hash-pinned original consumers for bounded namespace publication.
#![cfg(any(
    feature = "client-wowforever",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]

#[cfg(feature = "client-wowforever")]
use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use sha2::{Digest, Sha256};
#[cfg(feature = "client-wowforever")]
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(feature = "client-wowforever")]
use wow_ui_sim::screen::ScreenKind;

fn load_original(env: &WowLuaEnv, name: &str) {
    let path = wow_ui_sim::client_profile::blizzard_ui_addons_dir().join(name);
    let source = std::fs::read_to_string(&path).expect("SETUP: cached original consumer");
    let pins: serde_json::Value =
        serde_json::from_str(include_str!("forever_namespace_consumers.json")).unwrap();
    let profile = wow_ui_sim::client_profile::ACTIVE.cache_subdir();
    assert_eq!(
        format!("{:x}", Sha256::digest(source.as_bytes())),
        pins[profile][name].as_str().unwrap(),
        "SETUP: cached consumer changed: {}",
        path.display()
    );
    env.loader_env()
        .exec(&source)
        .expect("original complete chunk");
}

fn consumer_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec("NamespaceErrors = {}; seterrorhandler(function(e) NamespaceErrors[#NamespaceErrors+1] = tostring(e) end)").unwrap();
    env
}

fn assert_no_errors(env: &WowLuaEnv) {
    env.exec(r"assert(#NamespaceErrors == 0, table.concat(NamespaceErrors, '\n'))")
        .unwrap();
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
}

#[cfg(feature = "client-wowforever")]
const LEGACY_ACTION_BAR_GLOBALS_MATCH_STATE: &str = r#"
assert(GetCVarBool("loadDeprecationFallbacks"))
local globals = {
    "GetBonusBarIndex", "GetBonusBarOffset", "GetExtraBarIndex",
    "GetMultiCastBarIndex", "GetOverrideBarIndex", "GetOverrideBarSkin",
    "GetTempShapeshiftBarIndex", "GetVehicleBarIndex", "GetActionBarPage",
}
for _, name in ipairs(globals) do
    assert(_G[name]() == C_ActionBar[name](), name)
end
ChangeActionBarPage(4)
assert(GetActionBarPage() == C_ActionBar.GetActionBarPage())
assert(GetActionBarPage() == 4)
"#;

#[cfg(feature = "client-wowforever")]
fn seed_non_default_action_bar_state(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.action_bar_page = 2;
    state.bonus_bar_index = 9;
    state.has_bonus_action_bar = true;
    state.vehicle_bar_index = 21;
    state.has_vehicle_action_bar = true;
    state.override_bar_index = 24;
    state.override_bar_skin = Some(25);
    state.has_override_action_bar = true;
    state.temp_shapeshift_bar_index = 22;
    state.has_temp_shapeshift_action_bar = true;
    state.action_bar_state.possess_bar_visible = true;
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_deprecated_action_bar_globals_follow_non_default_state() {
    let env = consumer_env();
    let addon_dir = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    let addon = discover_blizzard_addons_for_screen(&addon_dir, ScreenKind::Game)
        .into_iter()
        .find(|(name, _)| name == "Blizzard_DeprecatedActionBar")
        .expect("DeprecatedActionBar must be selected for the Game screen");
    env.state().borrow_mut().addon_base_paths = vec![addon_dir];
    load_addon(&env.loader_env(), &addon.1).expect("load selected DeprecatedActionBar provider");

    seed_non_default_action_bar_state(&env);
    env.exec(LEGACY_ACTION_BAR_GLOBALS_MATCH_STATE)
        .expect("legacy globals must follow non-default C_ActionBar state");
}

#[cfg(feature = "client-wowforever")]
const ACTION_BAR_STATUS_CONSUMER_LUA: &str = r#"
local source = debug.getinfo(SpellSearchUtil.GetActionbarStatusForSpellBookItemInfo, "S").source
assert(source:find("Blizzard_SpellSearch/Blizzard_SpellSearchUtil.lua", 1, true))
C_PetInfo._state.petActionsByID = { [202] = { isPassive = false } }
local status = ActionButtonUtil.ActionBarActionStatus
local function query(itemType, id)
    return SpellSearchUtil.GetActionbarStatusForSpellBookItemInfo({
        itemType = itemType,
        actionID = id,
        isPassive = false,
        isOffSpec = false,
    })
end
assert(SpellSearchUtil.GetActionbarStatusForSpell(31338) == status.MissingFromAllBars)
assert(query(Enum.SpellBookItemType.PetAction, 202) == status.MissingFromAllBars)
assert(query(Enum.SpellBookItemType.Flyout, 303) == status.MissingFromAllBars)
"#;

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_spell_search_uses_action_bar_status_providers() {
    let ui = blizzard_ui_dir();
    let roots = ["Blizzard_SharedTalentUI", "Blizzard_ActionBar"];
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &roots, &[]);
    assert!(loaded.iter().any(|addon| addon == "Blizzard_SpellSearch"));
    env.state().borrow_mut().action_bars.clear();
    env.exec(ACTION_BAR_STATUS_CONSUMER_LUA)
        .expect("original SpellSearch consumer must receive real action-bar statuses");
}

#[test]
fn classic_original_deprecated_combat_text_aliases_publish_and_select_unit() {
    let env = consumer_env();
    env.exec("assert(GetCVarBool('loadDeprecationFallbacks'), 'SETUP: deprecation enabled')")
        .unwrap();
    load_original(&env, "Blizzard_CombatLogBase/CombatLogUtil.lua");
    load_original(
        &env,
        "Blizzard_DeprecatedCombatLog/Deprecated_CombatLog.lua",
    );
    env.exec(
        r#"
        CombatTextSetActiveUnit('vehicle')
        assert(C_CombatText.GetActiveUnit() == 'vehicle')
        assert(select('#', GetCurrentCombatTextEventInfo()) == 0)
        CombatTextSetActiveUnit('player')
        assert(C_CombatText.GetActiveUnit() == 'player')
    "#,
    )
    .unwrap();
    assert_no_errors(&env);
}

fn load_action_button(env: &WowLuaEnv) {
    load_original(env, "Blizzard_SharedXMLBase/EnumUtil.lua");
    load_original(env, "Blizzard_SharedXMLBase/TableUtil.lua");
    load_original(env, "Blizzard_SharedXMLBase/MathUtil.lua");
    load_original(env, "Blizzard_SharedXMLBase/CallbackRegistry.lua");
    load_original(env, "Blizzard_ActionBar/Shared/ActionButton.lua");
}

#[test]
fn classic_original_action_button_alert_tracks_supplied_proc_membership() {
    let env = consumer_env();
    load_action_button(&env);
    env.exec(r#"
        local slot, spell
        for i = 1, 180 do
            local kind, id = GetActionInfo(i)
            if kind == 'spell' and id then slot, spell = i, id; break end
        end
        assert(slot, 'SETUP: seeded spell action required')
        FixtureSpell = spell
        Alert = CreateFrame('Frame')
        local button = { action = slot, EvaluateTutorials = ActionBarActionButtonMixin.EvaluateTutorials }
        FixtureButton = button
        -- Presentation-only observer: actual query, action lookup and consumer stay original.
        ActionButtonSpellAlertManager = {
            ShowAlert = function(_, b) assert(b == button); Alert:Show() end,
            HideAlert = function(_, b) assert(b == button); Alert:Hide() end,
        }
        Alert:Show()
        for i = 1, 208 do ActionBarActionButtonMixin.UpdateSpellAlert(button) end
        assert(not Alert:IsShown())
    "#).unwrap();
    let spell: i32 = env.eval("return FixtureSpell").unwrap();
    let spell = u32::try_from(spell).expect("SETUP: seeded spell ID fits u32");
    env.state()
        .borrow_mut()
        .spell_activation_overlays
        .insert(spell);
    env.exec("ActionBarActionButtonMixin.UpdateSpellAlert(FixtureButton); assert(Alert:IsShown())")
        .unwrap();
    env.state().borrow_mut().spell_activation_overlays.clear();
    env.exec(
        "ActionBarActionButtonMixin.UpdateSpellAlert(FixtureButton); assert(not Alert:IsShown())",
    )
    .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_original_chat_mentor_predicates_are_false_without_membership() {
    let env = consumer_env();
    // ChatFrameUtil is the receiver of function declarations in the original chunk.
    env.exec("ChatFrameUtil = ChatFrameUtil or {}").unwrap();
    load_original(
        &env,
        "Blizzard_ChatFrameBase/Mainline/ChatFrameUtilOverrides.lua",
    );
    env.exec(
        r#"
        assert(not C_Glue.IsOnGlueScreen(), 'SETUP: game environment')
        assert(DoesActivePlayerHaveMentorStatus() == false)
        assert(IsActivePlayerGuide() == false)
        assert(IsActivePlayerNewcomer() == false)
    "#,
    )
    .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_original_social_control_disables_unmodeled_social_ui() {
    let env = consumer_env();
    load_original(&env, "Blizzard_SocialUIShared/SocialUIControl.lua");
    env.exec("SocialUIFrame = CreateFrame('Frame'); assert(SocialUIControl.IsEnabled() == false)")
        .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-wowforever")]
fn load_recent_allies_util(env: &WowLuaEnv) {
    load_original(env, "Blizzard_SharedXMLBase/TableUtil.lua");
    load_original(env, "Blizzard_SharedXML/TimeUtil.lua");
    // Concrete UI labels keep the original chunk's label-count assertion meaningful.
    env.exec(
        r#"
        SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_PROFESSIONS = 'Professions'
        SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_PVP = 'PvP'
        SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_RAIDING = 'Raiding'
        SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_DUNGEONS = 'Dungeons'
        SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_DELVE = 'Delves'
        SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_QUESTING = 'Questing'
    "#,
    )
    .unwrap();
    load_original(env, "Blizzard_RecentAllies/Blizzard_RecentAlliesUtil.lua");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_original_recent_allies_filter_list_is_empty() {
    let env = consumer_env();
    load_recent_allies_util(&env);
    env.exec(
        r#"
        local filters = RecentAlliesUtil.GetSupportedInteractionCategoryFiltersForCurrentGameType()
        assert(type(filters) == 'table' and #filters == 0)
        assert(C_RecentAllies.IsSystemEnabled() == false)
        assert(C_RecentAllies.IsSystemSupported() == false)
        assert(C_RecentAllies.NotARealMethod == nil)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().recent_allies.enabled = true;
    env.exec("assert(C_RecentAllies.IsSystemEnabled() == true)")
        .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_original_player_choice_update_hides_toggle_without_choice() {
    let env = consumer_env();
    load_original(
        &env,
        "Blizzard_PlayerChoice/Blizzard_PlayerChoiceToggleButton.lua",
    );
    env.exec(
        r#"
        local button = CreateFrame('Button')
        Mixin(button, PlayerChoiceToggleButtonMixin)
        button:Show()
        button:UpdateButtonState()
        assert(not button:IsShown())
        assert(select('#', C_PlayerChoice.GetCurrentPlayerChoiceInfo()) == 0)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().player_choice.waiting_for_response = true;
    env.exec("assert(PlayerChoiceToggle_ShouldShow() == true)")
        .unwrap();
    assert_no_errors(&env);
}

#[test]
fn classic_original_combat_text_unit_switch_uses_shared_state() {
    let env = consumer_env();
    load_original(&env, "Blizzard_CombatText/Shared/CombatText.lua");
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        frame.unit = 'player'
        CombatTextMixin.OnEvent(frame, 'UNIT_ENTERED_VEHICLE', 'player', true)
        assert(frame.unit == 'vehicle')
        assert(C_CombatText.GetActiveUnit() == 'vehicle')
        CombatTextMixin.OnEvent(frame, 'UNIT_EXITING_VEHICLE', 'player')
        assert(frame.unit == 'player')
        assert(C_CombatText.GetActiveUnit() == 'player')
        assert(select('#', C_CombatText.GetCurrentEventInfo()) == 0)
    "#,
    )
    .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-wowforever")]
fn load_recent_allies_templates(env: &WowLuaEnv) {
    load_recent_allies_util(env);
    for name in [
        "Blizzard_SharedXMLBase/EnumUtil.lua",
        "Blizzard_SharedXMLBase/CallbackRegistry.lua",
        "Blizzard_SharedXML/DataProvider.lua",
        "Blizzard_SharedXML/TreeListDataProvider.lua",
        "Blizzard_SocialUIShared/SocialUISystemMixin.lua",
    ] {
        load_original(env, name);
    }
    env.exec(
        r#"
        -- Receiver-only rendering mixin; these tests never measure scroll extents.
        -- Loading SocialUIUtil would execute unrelated BattleNet capability queries.
        SocialUIScrollableElementExtentPreviewerMixin = {}
        SOCIAL_UI_RECENT_ALLIES_VIEW_HEADER_LEGACY_FRIENDS = 'Legacy %d/%d'
        SOCIAL_UI_RECENT_ALLIES_VIEW_HEADER_PINNED = 'Pinned %d/%d'
        SOCIAL_UI_RECENT_ALLIES_VIEW_HEADER_UNPINNED = 'Unpinned %d/%d'
        "#,
    )
    .unwrap();
    load_original(
        env,
        "Blizzard_RecentAllies/Blizzard_RecentAlliesTemplates.lua",
    );
}

#[cfg(feature = "client-wowforever")]
fn supply_recent_allies(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_recent_allies::{
        RecentAllyCharacterData, RecentAllyData, RecentAllyInteractionData, RecentAllyStateData,
    };
    let aster = RecentAllyData {
        state_data: RecentAllyStateData {
            is_online: true,
            is_dnd: false,
            is_afk: false,
            is_converted_legacy_friend: false,
            pin_expiration_date: Some(1_760_100_000),
            friend_request_sent_this_session: false,
            current_location: Some("Dornogal".into()),
        },
        character_data: RecentAllyCharacterData {
            guid: "Player-100-00000001".into(),
            name: "Aster".into(),
            full_name: "Aster-FixtureRealm".into(),
            realm_name: "FixtureRealm".into(),
            level: 80,
            class_id: 2,
            race_id: 1,
            sex: 3,
        },
        interaction_data: RecentAllyInteractionData {
            interactions: vec![],
            note: Some("Dungeon guide".into()),
        },
    };
    let mut birch = aster.clone();
    birch.character_data.guid = "Player-100-00000002".into();
    birch.character_data.name = "Birch".into();
    birch.character_data.full_name = "Birch-FixtureRealm".into();
    birch.state_data.is_online = false;
    birch.state_data.pin_expiration_date = None;
    birch.interaction_data.note = None;
    env.state().borrow_mut().recent_allies.entries = vec![aster, birch];
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_original_recent_allies_list_consumes_disabled_and_supplied_rows() {
    let env = consumer_env();
    load_recent_allies_templates(&env);
    supply_recent_allies(&env);
    env.exec(
        r#"
        assert(C_RecentAllies.IsSystemEnabled() == false)
        assert(select('#', C_RecentAllies.GetRecentAllies()) == 1)
        assert(#C_RecentAllies.GetRecentAllies() == 0)
        assert(not C_RecentAllies.IsRecentAllyByGUID('Player-100-00000001'))
        assert(not C_RecentAllies.IsRecentAllyByFullName('Aster-FixtureRealm'))
        local provider = RecentAlliesListMixin.BuildRecentAlliesDataProvider({})
        assert(provider:IsEmpty())
        "#,
    )
    .unwrap();
    env.state().borrow_mut().recent_allies.enabled = true;
    env.exec(
        r#"
        local rows = C_RecentAllies.GetRecentAllies()
        assert(#rows == 2 and rows[1].characterData.name == 'Aster')
        assert(rows[2].characterData.name == 'Birch' and not rows[2].stateData.isOnline)
        assert(rows[1].interactionData.note == 'Dungeon guide')
        assert(C_RecentAllies.IsRecentAllyByGUID('Player-100-00000001'))
        assert(C_RecentAllies.IsRecentAllyByFullName('Aster-FixtureRealm'))
        assert(not C_RecentAllies.IsRecentAllyByGUID('Player-100-99999999'))
        assert(not C_RecentAllies.IsRecentAllyByFullName('Absent-FixtureRealm'))
        rows[1].characterData.name = 'Changed snapshot'
        assert(C_RecentAllies.GetRecentAllies()[1].characterData.name == 'Aster')
        local provider = RecentAlliesListMixin.BuildRecentAlliesDataProvider({})
        local collection = provider:GetCollection()
        assert(provider:GetSize() == 3)
        assert(collection[1].characterData.name == 'Aster')
        assert(collection[2].isDivider == true)
        assert(collection[3].characterData.name == 'Birch')
        "#,
    )
    .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_original_recent_allies_social_view_searches_supplied_rows() {
    let env = consumer_env();
    load_recent_allies_templates(&env);
    supply_recent_allies(&env);
    env.exec(
        r#"
        FixtureSearchText = ''
        FixtureView = CreateFromMixins(RecentAlliesSocialViewMixin)
        FixtureView.FilterBar = { SearchBar = { GetText = function() return FixtureSearchText end } }
        assert(#FixtureView:GenerateDataProvider():GetRootNode():GetNodes() == 0)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().recent_allies.enabled = true;
    env.exec(
        r#"
        FixtureSearchText = 'AsTeR'
        local nodes = FixtureView:GenerateDataProvider():GetRootNode():GetNodes()
        assert(#nodes == 1 and nodes[1]:GetData().headerText == 'Pinned 1/1')
        assert(nodes[1]:GetNodes()[2]:GetData().characterData.name == 'Aster')
        FixtureSearchText = 'absent-name'
        assert(#FixtureView:GenerateDataProvider():GetRootNode():GetNodes() == 0)
        FixtureSearchText = ''
        FixtureView.selectedSearchFilterOptions = { [{searchInfo = {isOffline = true}}] = true }
        nodes = FixtureView:GenerateDataProvider():GetRootNode():GetNodes()
        assert(#nodes == 1 and nodes[1]:GetData().headerText == 'Unpinned 0/1')
        assert(nodes[1]:GetNodes()[2]:GetData().characterData.name == 'Birch')
        "#,
    )
    .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_original_gamepad_flyout_clears_missing_texture() {
    let env = consumer_env();
    load_action_button(&env);
    load_original(&env, "Blizzard_GamepadActionBars/ActionBarButton.lua");
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        frame.icon = frame:CreateTexture()
        frame.icon:SetTexture(134400)
        frame.isFlyoutAction = true
        frame.flyoutID = 987654
        -- Fixture represents no selected spell, forcing the original flyout branch.
        frame.GetSingleSpellID = function() return nil end
        frame.GetActiveSpellID = function() return nil end
        GamepadActionBarButtonFlyoutMixin.UpdateFlyoutActionIcon(frame)
        assert(frame.icon:GetTexture() == nil)
        assert(select('#', C_Flyout.GetFlyoutTexture(987654)) == 0)
        assert(C_Flyout.NotARealMethod == nil)
    "#,
    )
    .unwrap();
    assert_no_errors(&env);
}
