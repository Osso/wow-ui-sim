//! Original LFG consumers select the normal and disabled rendered check slots.
#![cfg(all(
    feature = "gui",
    any(feature = "client-retail", feature = "client-wrath")
))]

use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
use wow_ui_sim::lua_api::WowLuaEnv;

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

fn exec_original(env: &WowLuaEnv, relative: &str) {
    let path = wow_ui_sim::client_profile::blizzard_ui_addons_dir().join(relative);
    let source = std::fs::read_to_string(&path).expect("SETUP: read original cache");
    env.exec_named(&source, &format!("@{}", path.display()))
        .expect("SETUP: execute whole unchanged original consumer");
}

fn load_consumer() -> WowLuaEnv {
    if cfg!(feature = "client-wrath") {
        let env = WowLuaEnv::new().expect("SETUP: env");
        exec_original(&env, "../FrameXML/LFGFrame.lua");
        exec_original(&env, "../FrameXML/LFDFrame.lua");
        return with_concrete_fixture(env);
    }
    // Retail LFGFrame.lua registers StaticPopupDialogs at load; the original
    // owner replaces the retired placeholder table (c1b07efb79).
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_StaticPopup_Game"], &[]);
    exec_original(&env, "Blizzard_GroupFinder/Shared/LFGFrame.lua");
    with_concrete_fixture(env)
}

fn with_concrete_fixture(env: WowLuaEnv) -> WowLuaEnv {
    env.exec(
        r#"
        C12Parent = CreateFrame("Frame", "C12Parent", UIParent)
        C12Parent:SetAllPoints(UIParent)
        C12Row = CreateFrame("Frame", "C12Row", C12Parent)
        C12Row:SetSize(300, 30)
        C12Row:SetPoint("CENTER")
        C12Row.instanceName = C12Row:CreateFontString(nil, "OVERLAY")
        C12Row.level = C12Row:CreateFontString(nil, "OVERLAY")
        C12Row.heroicIcon = C12Row:CreateTexture(nil, "ARTWORK")
        C12Row.expandOrCollapseButton = CreateFrame("Button", nil, C12Row)
        C12Row.lockedIndicator = C12Row:CreateTexture(nil, "OVERLAY")
        C12Row.enableButton = CreateFrame("CheckButton", "C12Check", C12Row)
        C12Row.enableButton:SetSize(24, 24)
        C12Row.enableButton:SetPoint("LEFT")
        -- Supplied presentation/data fixture, not a replacement API function.
        QuestDifficulty_Header = CreateFont("C12HeaderFont")
        LFGLockList = {}; LFGCollapseList = {}; LFGEnabledList = {}; LFGQueuedForList = {}
    "#,
    )
    .expect("SETUP: concrete widget/data fixture");
    if cfg!(feature = "client-wrath") {
        env.exec(
            r#"
            LFGDungeonInfo = {[-1] = {"C12 header", 1, 1, 80, 40, 1, 80, 0, 0}}
            function C12Refresh(selection)
                LFGQueuedForList[-1] = selection
                LFDQueueFrameSpecificListButton_SetDungeon(C12Row, -1, "queued", nil)
            end
        "#,
        )
        .expect("SETUP: Wrath header data/orchestration");
    } else {
        env.exec(
            r#"
            function C12Refresh(selection)
                LFGDungeonListButton_SetDungeon(C12Row, -1, false, {[-1] = selection})
            end
        "#,
        )
        .expect("SETUP: Retail orchestration");
    }
    env
}

fn assert_emitted(env: &WowLuaEnv, normal: &str, disabled: &str, expected: Option<&str>) {
    env.set_screen_size(1024.0, 768.0);
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();
    let buckets = state.get_strata_buckets().expect("render buckets").clone();
    let batch = build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(&state.widgets, (1024.0, 768.0), &buckets)
            .root_name(Some("C12Row")),
    );
    for path in [normal, disabled] {
        let wanted = format!("interface/buttons/{path}").to_lowercase();
        let present = batch
            .texture_requests
            .iter()
            .any(|request| request.path.replace('\\', "/").to_lowercase() == wanted);
        assert_eq!(
            present,
            expected == Some(path),
            "C12 emitted selection: {path}"
        );
    }
}

#[test]
fn c12_original_lfg_disabled_check_emission_and_reassignment() {
    let env = load_consumer();
    const CHECK: &str = "UI-CheckBox-Check";
    const DISABLED: &str = "UI-CheckBox-Check-Disabled";
    const MULTI: &str = "UI-MultiCheck-Up";
    const MULTI_DISABLED: &str = "UI-MultiCheck-Disabled";
    env.exec("C12Refresh(2)")
        .expect("SETUP: original consumer invocation");
    if std::env::var("WOW_SIM_SHARED_INPUT_MUTATION").as_deref() == Ok("disabled-check") {
        env.exec("C12Check:SetDisabledCheckedTexture('Interface/Buttons/UI-CheckBox-Check')")
            .unwrap();
    }
    assert_emitted(&env, CHECK, DISABLED, Some(DISABLED));
    env.exec(
        r#"
        C12Checked = C12Check:GetCheckedTexture()
        C12Disabled = C12Check:GetDisabledCheckedTexture()
        C12Check:Enable()
    "#,
    )
    .unwrap();
    assert_emitted(&env, CHECK, DISABLED, Some(CHECK));
    env.exec("C12Check:Disable()").unwrap();
    assert_emitted(&env, CHECK, DISABLED, Some(DISABLED));
    env.exec("C12Refresh(1)").unwrap();
    assert_emitted(&env, MULTI, MULTI_DISABLED, Some(MULTI_DISABLED));
    assert_emitted(&env, CHECK, DISABLED, None);
    env.exec(
        r#"
        assert(C12Checked == C12Check:GetCheckedTexture(), "checked child identity")
        assert(C12Disabled == C12Check:GetDisabledCheckedTexture(), "disabled child identity")
        assert(C12Checked:GetParent() == C12Check and C12Disabled:GetParent() == C12Check)
        C12Parent:Hide()
    "#,
    )
    .unwrap();
    assert_emitted(&env, MULTI, MULTI_DISABLED, None);
    env.exec("C12Parent:Show(); C12Refresh(0)").unwrap();
    assert_emitted(&env, CHECK, DISABLED, None);
    env.exec("C12Refresh(2); C12Check:Enable()").unwrap();
    assert_emitted(&env, CHECK, DISABLED, Some(CHECK));
}
