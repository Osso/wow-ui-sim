//! Cached consumer regressions. Staged only: no runtime or native proof.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-retail")]
fn load_original(env: &WowLuaEnv, name: &str) {
    use sha2::{Digest, Sha256};
    let path = wow_ui_sim::client_profile::blizzard_ui_addons_dir().join(name);
    let source = std::fs::read_to_string(&path).expect("SETUP: original cached consumer");
    let pins: serde_json::Value =
        serde_json::from_str(include_str!("retail_namespace_consumers.json")).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(source.as_bytes())),
        pins[name].as_str().unwrap(),
        "SETUP: cached consumer changed: {}",
        path.display()
    );
    env.loader_env()
        .exec(&source)
        .expect("original complete chunk");
}

/// e35289a6a1 and c1b07efb79 retired the modern-profile EventRegistry,
/// CallbackRegistry and StaticPopupDialogs copies, so the cached consumers run
/// after their original owner addons.
#[cfg(feature = "client-retail")]
fn consumer_env() -> WowLuaEnv {
    consumer_env_after(&["Blizzard_SharedXMLBase"])
}

#[cfg(feature = "client-retail")]
fn consumer_env_after(owner_roots: &[&str]) -> WowLuaEnv {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, owner_roots, &[]);
    env.exec("NamespaceErrors = {}; seterrorhandler(function(e) NamespaceErrors[#NamespaceErrors+1] = tostring(e) end)").unwrap();
    env
}

#[cfg(feature = "client-retail")]
fn assert_no_errors(env: &WowLuaEnv) {
    env.exec(r"assert(#NamespaceErrors == 0, table.concat(NamespaceErrors, '\n'))")
        .unwrap();
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
}

#[test]
fn namespace_documented_profile_surface() {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
    let env = WowLuaEnv::new().unwrap();
    let session = ACTIVE != ClientProfile::Wrath;
    let modern = matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    );
    let source = format!(
        r#"
        assert((rawget(_G, 'C_QuestSession') ~= nil) == {session})
        assert((rawget(_G, 'C_LobbyMatchmakerInfo') ~= nil) == {session})
        assert((rawget(_G, 'C_FrameManager') ~= nil) == {modern})
        assert((rawget(_G, 'C_CovenantCallings') ~= nil) == {modern})
        assert((rawget(_G, 'C_ClientScene') ~= nil) == {session})
        if C_ClientScene then
            assert((rawget(C_ClientScene, 'IsSceneTypeActive') ~= nil) == {modern})
        end
        if C_LobbyMatchmakerInfo then
            assert(C_LobbyMatchmakerInfo.IsInQueue() == false)
            assert((rawget(C_LobbyMatchmakerInfo, 'GetQueueFromMainlineEnabled') ~= nil) == {modern})
            if {modern} then assert(C_LobbyMatchmakerInfo.GetQueueFromMainlineEnabled() == false) end
            assert(rawget(C_LobbyMatchmakerInfo, 'EnterQueue') == nil)
            assert(rawget(C_LobbyMatchmakerInfo, 'AbandonQueue') == nil)
            assert(rawget(C_LobbyMatchmakerInfo, 'RespondToQueuePop') == nil)
            assert(C_LobbyMatchmakerInfo.NotARealMethod == nil)
        end
        if C_QuestSession then
            assert(C_QuestSession.CanStart() == false and C_QuestSession.CanStop() == false)
            assert(C_QuestSession.Exists() == false and C_QuestSession.HasJoined() == false)
            assert(C_QuestSession.HasPendingCommand() == false)
            assert(select('#', C_QuestSession.GetSessionBeginDetails()) == 1)
            assert(C_QuestSession.GetSessionBeginDetails() == nil)
            assert(C_QuestSession.GetSuperTrackedQuest() == nil)
            assert(C_QuestSession.GetPendingCommand() == 0)
            assert(C_QuestSession.GetAvailableSessionCommand() == 0)
            assert(C_QuestSession.GetProposedMaxLevelForSession() == 0)
            assert(C_QuestSession.RequestSessionStart == nil)
            assert(C_QuestSession.SendSessionBeginResponse == nil)
            assert(C_QuestSession.SetQuestIsSuperTracked == nil)
        end
        assert(rawget(_G, 'C_KEY_TEXT') == nil)
        "#
    );
    env.exec(&source).unwrap();
}

// Consumer-inferred popup signature; user-approved temporary unavailable policy.
// No cached API declaration establishes native popup behavior.
#[cfg(feature = "client-retail")]
#[test]
fn original_retail_boost_tutorial_initialization_keeps_public_popup_hidden() {
    let env = consumer_env();
    load_original(&env, "Blizzard_SharedXMLBase/SecureTypes.lua");
    load_original(
        &env,
        "Blizzard_MicroMenu/Mainline/MainMenuBarMicroButtons.lua",
    );
    env.exec("NPE_TutorialKeyboardMouseFrame_Frame = CreateFrame('Frame'); NPE_TutorialKeyboardMouseFrame_Frame:Hide()")
        .unwrap();
    load_original(&env, "Blizzard_BoostTutorial/Blizzard_BoostTutorial.lua");
    env.exec(
        r#"
        assert(MainMenuMicroButton_AreAlertsEnabled() == false)
        assert(C_CharacterServicesPublic.ShouldSeeControlPopup() == false)
        assert(not NPE_TutorialKeyboardMouseFrame_Frame:IsShown())
        assert(next(BoostTutorial.spellQueue) == nil)
    "#,
    )
    .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-retail")]
#[test]
fn original_retail_quest_session_start_prompt_stays_hidden() {
    let env = consumer_env();
    load_original(&env, "Blizzard_FrameXML/QuestSession.lua");
    env.exec(
        r#"
        local dialog = CreateFrame('Frame')
        Mixin(dialog, QuestSessionStartDialogMixin)
        dialog:Hide()
        local manager = CreateFrame('Frame')
        Mixin(manager, QuestSessionManagerMixin)
        manager.StartDialog = dialog
        manager.SessionManagementDialogs = {dialog}
        QuestSessionManager = manager
        manager:CheckShowSessionStartPrompt()
        assert(not dialog:IsShown())
        assert(manager:GetSystemSessionCommand() == Enum.QuestSessionCommand.None)
        assert(not manager:ShouldSessionManagementUIBeVisible())
    "#,
    )
    .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-retail")]
#[test]
fn original_retail_client_scene_onload_keeps_mode_inactive() {
    let env = consumer_env();
    load_original(&env, "Blizzard_UIModes/Blizzard_UIModeManager.lua");
    load_original(&env, "Blizzard_UIModes/Blizzard_UIModeUtil.lua");
    load_original(
        &env,
        "Blizzard_ClientSceneVisManager/Blizzard_ClientSceneVisManager.lua",
    );
    env.exec(
        r#"
        local manager = CreateFrame('Frame')
        Mixin(manager, ClientSceneVisManagerMixin)
        manager:OnLoad()
        assert(manager:IsEventRegistered('CLIENT_SCENE_OPENED'))
        assert(manager:IsEventRegistered('CLIENT_SCENE_CLOSED'))
        assert(UIModeUtil.IsModeActive('ClientScene') == false)
        manager:OnEvent('CLIENT_SCENE_CLOSED')
        assert(UIModeUtil.IsModeActive('ClientScene') == false)
    "#,
    )
    .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-retail")]
#[test]
fn original_retail_frame_manager_applies_visibility_on_registration_and_refresh() {
    let env = consumer_env();
    load_original(&env, "Blizzard_UIFrameManager/Blizzard_UIFrameManager.lua");
    env.exec(r#"
        local manager = CreateFrame('Frame')
        Mixin(manager, UIFrameManagerMixin)
        manager:OnLoad()
        UIFrameManager = manager
        assert(Enum.UIFrameType.JailersTowerBuffs == 0)
        assert(Enum.UIFrameType.InterruptTutorial == 1)
        local tested = 0
        for _, frameType in ipairs({Enum.UIFrameType.JailersTowerBuffs, Enum.UIFrameType.InterruptTutorial}) do
            local frame = CreateFrame('Frame')
            Mixin(frame, UIFrameManager_ManagedFrameMixin)
            frame.frameType = frameType
            frame:Show()
            frame:OnLoad()
            assert(manager.registeredFrames[frame])
            assert(not frame:IsShown())
            manager:OnEvent('FRAME_MANAGER_UPDATE_FRAME', frameType, true)
            assert(frame:IsShown())
            manager:OnEvent('FRAME_MANAGER_UPDATE_ALL')
            assert(not frame:IsShown())
            tested = tested + 1
        end
        assert(tested == 2)
    "#).unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-retail")]
#[test]
fn original_retail_lfd_update_reaches_role_warning_not_lobby_block() {
    let env = consumer_env_after(&["Blizzard_StaticPopup_Game"]);
    load_original(&env, "Blizzard_GroupFinder/Shared/LFGFrame.lua");
    load_original(&env, "Blizzard_GroupFinder/Mainline/LFDFrame.lua");
    env.exec(
        r#"
        assert(not C_PlayerInfo.IsPlayerNPERestricted(), 'SETUP: non-NPE character')
        FIND_A_GROUP = 'Find a fixture group'
        INSTANCE_ROLE_WARNING_TITLE = 'Fixture role warning'
        LFDQueueFrameFindGroupButton = CreateFrame('Button')
        LFDQueueFrameFindGroupButton:SetFontString(LFDQueueFrameFindGroupButton:CreateFontString())
        LFDQueueFrameRoleButtonTank = {checkButton = CreateFrame('CheckButton')}
        LFDQueueFrameRoleButtonHealer = {checkButton = CreateFrame('CheckButton')}
        LFDQueueFrameRoleButtonDPS = {checkButton = CreateFrame('CheckButton')}
        LFDQueueFrameRoleButtonTank.checkButton:SetChecked(false)
        LFDQueueFrameRoleButtonHealer.checkButton:SetChecked(false)
        LFDQueueFrameRoleButtonDPS.checkButton:SetChecked(false)
        LFDQueueFrameFindGroupButton_Update()
        assert(not LFDQueueFrameFindGroupButton:IsEnabled())
        assert(LFDQueueFrameFindGroupButton.tooltip == INSTANCE_ROLE_WARNING_TITLE)
        assert(LFDQueueFrameFindGroupButton:GetText() == FIND_A_GROUP)
    "#,
    )
    .unwrap();
    assert_no_errors(&env);
}

#[cfg(feature = "client-retail")]
#[test]
fn original_retail_callings_request_completes_with_empty_payload() {
    let env = consumer_env();
    load_original(&env, "Blizzard_ObjectAPI/Mainline/CovenantCalling.lua");
    env.exec(
        r#"
        local receiver = CreateFrame('Frame')
        local events = 0
        receiver:RegisterEvent('COVENANT_CALLINGS_UPDATED')
        receiver:SetScript('OnEvent', function(_, event, callings)
            assert(event == 'COVENANT_CALLINGS_UPDATED')
            assert(type(callings) == 'table' and next(callings) == nil)
            events = events + 1
        end)
        CovenantCalling_CheckCallings()
        assert(events == 1)
        assert(CovenantCalling_AreCallingsUnlocked() == false)
        assert(CovenantCalling_GetCompletedCount() == 0)
        assert(CovenantCalling_GetAvailableCount() == 0)
        CovenantCalling_CheckCallings()
        assert(events == 1, 'original one-second request throttle')
    "#,
    )
    .unwrap();
    assert_no_errors(&env);
}
