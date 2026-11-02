#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
//! Original consumer: Blizzard_RuneforgeFrame.lua:99 (OnHide close) and
//! :114–115 (RUNEFORGE_LEGENDARY_CRAFTING_CLOSED hides the panel).

use wow_ui_sim::c_api::c_legendary_crafting::RuneforgeInteraction;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

/// `Enum.PlayerInteractionType.LegendaryCrafting`,
/// PlayerInteractionManagerConstantsDocumentation.lua:61.
const LEGENDARY_CRAFTING: i32 = 48;
const MAIL_INFO: i32 = 17;

fn open_runeforge_frame(env: &WowLuaEnv, interaction_type: i32) {
    let toc = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
        .join("Blizzard_RuneforgeUI/Blizzard_RuneforgeUI.toc");
    load_addon(&env.loader_env(), &toc).expect("RuneforgeUI loads");
    env.state().borrow_mut().runeforge_interaction = RuneforgeInteraction {
        interaction_type,
        target_guid: "Creature-0-3767-2222-9-172054-0000ABCDEF".into(),
        close_callback_installed: true,
        transport_available: true,
        submitted_close_requests: 0,
    };
    env.exec(
        r#"
        RuneforgeCloseLog = { closed = 0, hides = {} }
        local probe = CreateFrame('Frame')
        probe:RegisterEvent('RUNEFORGE_LEGENDARY_CRAFTING_CLOSED')
        probe:RegisterEvent('PLAYER_INTERACTION_MANAGER_FRAME_HIDE')
        probe:SetScript('OnEvent', function(_, event, interactionType)
            if event == 'RUNEFORGE_LEGENDARY_CRAFTING_CLOSED' then
                RuneforgeCloseLog.closed = RuneforgeCloseLog.closed + 1
            else
                table.insert(RuneforgeCloseLog.hides, interactionType)
            end
        end)
        ShowUIPanel(RuneforgeFrame)
        assert(RuneforgeFrame:IsShown())
        "#,
    )
    .expect("original RuneforgeFrame opens through the panel manager");
    // Opening reports unrelated missing runeforge queries; only the close path is asserted.
    let mut state = env.state().borrow_mut();
    state.lua_errors.clear();
    state.lua_error_records.clear();
    state.lua_error_counts.clear();
}

fn read_close_log(env: &WowLuaEnv) -> (i32, String, bool) {
    env.eval(
        r#"
        local hides = {}
        for _, interactionType in ipairs(RuneforgeCloseLog.hides) do
            table.insert(hides, tostring(interactionType))
        end
        return RuneforgeCloseLog.closed, table.concat(hides, ','), RuneforgeFrame:IsShown()
        "#,
    )
    .expect("close log reads")
}

prefork_full_ui_case! {
fn runeforge_close_reenters_through_original_on_hide(env: &WowLuaEnv) {
    open_runeforge_frame(env, LEGENDARY_CRAFTING);

    env.exec("assert(select('#', C_LegendaryCrafting.CloseRuneforgeInteraction()) == 0)")
        .expect("close runs through the original event and OnHide consumers");

    // EMULATED (WM33-A follow-up): the nested OnHide close finishes first,
    // so its hide carries 48 and the outer hide sees the reset type.
    assert_eq!(read_close_log(env), (2, "48,0".into(), false));
    let state = env.state().borrow();
    assert!(state.lua_errors.is_empty(), "close errors: {:?}", state.lua_errors);
    let interaction = &state.runeforge_interaction;
    assert_eq!(interaction.submitted_close_requests, 2);
    assert_eq!(interaction.interaction_type, 0);
    assert!(interaction.target_guid.is_empty());
    assert!(!interaction.close_callback_installed);
}
}

prefork_full_ui_case! {
fn runeforge_close_with_other_interaction_only_submits_request(env: &WowLuaEnv) {
    open_runeforge_frame(env, MAIL_INFO);

    env.exec("C_LegendaryCrafting.CloseRuneforgeInteraction()")
        .expect("close with another interaction type is not an error");

    assert_eq!(read_close_log(env), (0, String::new(), true));
    let state = env.state().borrow();
    assert!(state.lua_errors.is_empty(), "close errors: {:?}", state.lua_errors);
    let interaction = &state.runeforge_interaction;
    assert_eq!(interaction.submitted_close_requests, 1);
    assert_eq!(interaction.interaction_type, MAIL_INFO);
    assert!(interaction.close_callback_installed);
}
}

prefork_full_ui_case! {
fn runeforge_close_without_transport_still_closes_locally(env: &WowLuaEnv) {
    open_runeforge_frame(env, LEGENDARY_CRAFTING);
    env.state().borrow_mut().runeforge_interaction.transport_available = false;

    env.exec("C_LegendaryCrafting.CloseRuneforgeInteraction()")
        .expect("close without transport is not an error");

    assert_eq!(read_close_log(env), (2, "48,0".into(), false));
    let state = env.state().borrow();
    assert_eq!(state.runeforge_interaction.submitted_close_requests, 0);
    assert_eq!(state.runeforge_interaction.interaction_type, 0);
}
}
