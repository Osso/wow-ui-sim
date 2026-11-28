#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn contribution_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Contribution/Blizzard_Contribution.toc")
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_contribution_toc_is_load_on_demand() {
    let toc =
        TocFile::from_file(&contribution_toc()).expect("Blizzard_Contribution TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_Contribution declares `## LoadOnDemand: 1` (the contribution-table panel is \
         only created when the player clicks an Order Hall contribution NPC, so it must NOT \
         auto-load on Game-screen bring-up)"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_Contribution does not declare UseSecureEnvironment"
    );
}

#[test]
fn blizzard_contribution_is_absent_from_game_auto_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_Contribution");
    assert!(
        !in_game,
        "Blizzard_Contribution is `## LoadOnDemand: 1`, so it must NOT appear in Game-screen \
         auto-discovery — it is loaded explicitly via UIParentLoadAddOn when the contribution \
         table is opened"
    );
}

prefork_full_ui_case! {
fn blizzard_contribution_loads_via_load_addon_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &contribution_toc())
        .expect("Blizzard_Contribution should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_Contribution emitted Lua errors during explicit load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_contribution_collection_frame_is_defined_and_parented_to_uiparent(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &contribution_toc())
        .expect("Blizzard_Contribution should load via Rust loader");

    let frames_present: bool = env
        .eval(
            "return type(_G.ContributionCollectionFrame) == 'table' \
                and ContributionCollectionFrame:GetParent() == UIParent \
                and type(_G.ContributionBuffTooltip) == 'table' \
                and ContributionBuffTooltip:GetParent() == UIParent",
        )
        .expect("toplevel frame query should succeed");
    assert!(
        frames_present,
        "Blizzard_Contribution should define both top-level frames after load: \
         ContributionCollectionFrame (parent UIParent, frameStrata=HIGH, hidden=true, \
         enableMouse=true, inherits=HorizontalLayoutFrame, mixin=ContributionCollectionMixin) \
         and ContributionBuffTooltip (parent UIParent, frameStrata=TOOLTIP, \
         inherits=TooltipBackdropTemplate)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_contribution_registers_uipanelwindows_entry(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &contribution_toc())
        .expect("Blizzard_Contribution should load via Rust loader");

    let entry_present: bool = env
        .eval(
            "local entry = UIPanelWindows and UIPanelWindows['ContributionCollectionFrame']; \
             return type(entry) == 'table' \
                and entry.area == 'center' \
                and entry.allowOtherPanels == 1 \
                and type(entry.showFailedFunc) == 'function'",
        )
        .expect("UIPanelWindows entry query should succeed");
    assert!(
        entry_present,
        "Blizzard_Contribution.lua line 1 sets `UIPanelWindows['ContributionCollectionFrame'] = \
         {{ area = 'center', allowOtherPanels = 1, showFailedFunc = C_ContributionCollector.Close }}` \
         so the panel can participate in the central UIParent panel-stacking system"
    );
}
}

/// The original ContributionCollectionMixin:OnHide (Blizzard_Contribution.lua:412-419)
/// ends the supplied ContributionCollector interaction through C_ContributionCollector.Close.
const CONTRIBUTION_HIDE_CLOSES_INTERACTION: &str = r##"
    local collector = Enum.PlayerInteractionType.ContributionCollector
    A_Admin.OpenContributionCollector()
    local opened = ContributionCollectionFrame ~= nil and ContributionCollectionFrame:IsShown()
    local interacting = C_PlayerInteractionManager.IsInteractingWithNpcOfType(collector)
    HideUIPanel(ContributionCollectionFrame)
    return opened, interacting,
        C_PlayerInteractionManager.IsInteractingWithNpcOfType(collector),
        select("#", C_ContributionCollector.Close())
"##;

prefork_full_ui_case! {
fn blizzard_contribution_hide_closes_collector_interaction(env: &WowLuaEnv) {
    let observed: (bool, bool, bool, i32) = env
        .eval(CONTRIBUTION_HIDE_CLOSES_INTERACTION)
        .expect("contribution interaction probe should run");
    assert_eq!(observed, (true, true, false, 0));
}
}

prefork_full_ui_case! {
fn blizzard_contribution_collection_mixin_methods_are_defined(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &contribution_toc())
        .expect("Blizzard_Contribution should load via Rust loader");

    let methods_present: bool = env
        .eval(
            "return type(ContributionCollectionMixin) == 'table' \
                and type(ContributionCollectionMixin.OnLoad) == 'function' \
                and type(ContributionCollectionMixin.OnShowCollection) == 'function' \
                and type(ContributionCollectionMixin.OnHide) == 'function' \
                and type(ContributionCollectionMixin.OnEvent) == 'function' \
                and type(ContributionCollectionMixin.Update) == 'function' \
                and type(ContributionCollectionMixin.UpdateSingle) == 'function' \
                and type(ContributionCollectionMixin.EnumerateContributions) == 'function' \
                and type(ContributionCollectionMixin.HandleContributionResult) == 'function' \
                and type(ContributionCollectionMixin.UpdatePendingContribution) == 'function' \
                and type(ContributionCollectionMixin.AddContribution) == 'function' \
                and type(ContributionCollectionMixin.FindContribution) == 'function' \
                and type(ContributionCollectionMixin.AcquireReward) == 'function' \
                and type(ContributionCollectionMixin.ReleaseReward) == 'function'",
        )
        .expect("ContributionCollectionMixin query should succeed");
    assert!(
        methods_present,
        "ContributionCollectionMixin should expose its 13 methods (OnLoad / OnShowCollection / \
         OnHide / OnEvent / Update / UpdateSingle / EnumerateContributions / \
         HandleContributionResult / UpdatePendingContribution / AddContribution / \
         FindContribution / AcquireReward / ReleaseReward) wired to \
         CONTRIBUTION_COLLECTOR_UPDATE / _PENDING / _UPDATE_SINGLE events"
    );
}
}

prefork_full_ui_case! {
fn blizzard_contribution_per_entry_mixins_are_defined(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &contribution_toc())
        .expect("Blizzard_Contribution should load via Rust loader");

    let methods_present: bool = env
        .eval(
            "return type(ContributionMixin) == 'table' \
                and type(ContributionMixin.OnHide) == 'function' \
                and type(ContributionMixin.OnReset) == 'function' \
                and type(ContributionMixin.Setup) == 'function' \
                and type(ContributionMixin.Update) == 'function' \
                and type(ContributionMixin.Contribute) == 'function' \
                and type(ContributionMixin.ReleaseRewards) == 'function' \
                and type(ContributionMixin.FindOrAcquireReward) == 'function' \
                and type(ContributionMixin.UpdateRewards) == 'function' \
                and type(ContributionMixin.AddReward) == 'function' \
                and type(ContributionMixin.UpdateStatus) == 'function' \
                and type(ContributionMixin.UpdateContributeButton) == 'function' \
                and type(ContributionMixin.QueueAnimation) == 'function' \
                and type(ContributionMixin.StopAnimations) == 'function' \
                and type(ContributionStatusMixin) == 'table' \
                and type(ContributionStatusMixin.OnLoad) == 'function' \
                and type(ContributionStatusMixin.Update) == 'function' \
                and type(ContributionStatusMixin.PlayFlashAnimation) == 'function' \
                and type(ContributionStatusMixin.UpdateTextVisibility) == 'function' \
                and type(ContributionRewardMixin) == 'table' \
                and type(ContributionRewardMixin.Setup) == 'function' \
                and type(ContributionRewardMixin.OnEnter) == 'function' \
                and type(ContributionRewardMixin.OnLeave) == 'function' \
                and type(ContributionRewardMouseOverMixin) == 'table' \
                and type(ContributionRewardMouseOverMixin.OnEnter) == 'function' \
                and type(ContributionRewardMouseOverMixin.OnLeave) == 'function' \
                and type(ContributeButtonMixin) == 'table' \
                and type(ContributeButtonMixin.OnShow) == 'function' \
                and type(ContributeButtonMixin.OnEvent) == 'function' \
                and type(ContributeButtonMixin.OnClick) == 'function' \
                and type(ContributeButtonMixin.UpdateTooltip) == 'function' \
                and type(ContributeButtonMixin.SetContributionID) == 'function' \
                and type(ContributeButtonMixin.Update) == 'function'",
        )
        .expect("per-entry mixin query should succeed");
    assert!(
        methods_present,
        "Blizzard_Contribution.lua should define its 5 secondary mixins: ContributionMixin (13 \
         methods covering pool reset, contribution setup, reward/status/button refresh, and \
         pending-animation queueing), ContributionStatusMixin (status-bar update + flash \
         animation), ContributionRewardMixin (Setup + tooltip OnEnter/OnLeave), \
         ContributionRewardMouseOverMixin (delegates to parent), and ContributeButtonMixin \
         (CURRENCY_DISPLAY_UPDATE / BAG_UPDATE_DELAYED registration + tooltip + Update)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_contribution_xml_templates_are_registered(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &contribution_toc())
        .expect("Blizzard_Contribution should load via Rust loader");

    for template_name in [
        "ContributionHeaderTemplate",
        "ContributionStateTemplate",
        "ContributionRewardTemplate",
        "ContributionTemplate",
    ] {
        assert!(
            wow_ui_sim::xml::get_template(template_name).is_some(),
            "{template_name} (`<Frame virtual=\"true\">` from Blizzard_Contribution.xml) should \
             be registered in the template registry after Blizzard_Contribution loads"
        );
    }
}
}

prefork_full_ui_case! {
fn contribution_panel_uses_supplied_result_and_currency_through_interaction(env: &WowLuaEnv) {
    env.exec(
        r#"
        A_Admin.SetContributionCollectors({
            { mapID=2248, areaPoiID=901, collectorCreatureID=301, name="Collector", atlasName="",
              x=0.2, y=0.3, contributions={
                { contributionID=501, name="Ready", state=1, amount=0.5, startTime=100,
                  description="Ready description", result=Enum.ContributionResult.Success,
                  orderIndex=1, rewardQuestID=0, active=true,
                  requiredCurrency={id=2245, amount=5}, atlases={""}, buffs={},
                  appearance={ stateName="", stateColor={r=1, g=1, b=1}, tooltipLine="",
                    tooltipUseTimeRemaining=false, statusBarAtlas="", borderAtlas="", bannerAtlas="" } },
                { contributionID=502, name="Blocked", state=1, amount=0.5, startTime=100,
                  description="Blocked description", result=Enum.ContributionResult.FailedConditionCheck,
                  orderIndex=2, rewardQuestID=0, active=true,
                  requiredCurrency={id=2245, amount=11}, requiredItem={id=6948, count=2},
                  atlases={""}, buffs={},
                  appearance={ stateName="", stateColor={r=1, g=1, b=1}, tooltipLine="",
                    tooltipUseTimeRemaining=false, statusBarAtlas="", borderAtlas="", bannerAtlas="" } },
              } }
        })
        "#,
    )
    .expect("contribution scenario should load");
    load_addon(&env.loader_env(), &contribution_toc())
        .expect("Contribution bootstrap should register its interaction");
    env.exec("A_Admin.FireEvent(\"PLAYER_INTERACTION_MANAGER_FRAME_SHOW\", Enum.PlayerInteractionType.ContributionCollector)")
        .expect("contribution interaction event should dispatch");
    let result: (bool, i32, bool, bool, String, String) = env
        .eval("local ids={C_ContributionCollector.GetActive()}; local a=ContributionCollectionFrame:FindContribution(501); local b=ContributionCollectionFrame:FindContribution(502); return ContributionCollectionFrame:IsShown(), #ids, a and a.ContributeButton:IsEnabled() or false, b and b.ContributeButton:IsEnabled() or false, a and a.ContributeButton.Text:GetText() or '', b and b.ContributeButton.Text:GetText() or ''")
        .expect("contribution button state should be observable");
    assert!(result.0, "interaction manager should show ContributionCollectionFrame");
    assert_eq!(result.1, 2, "both supplied active contributions should render");
    assert!(result.2, "Success result should enable its contribution button");
    assert!(!result.3, "FailedConditionCheck should disable its contribution button");
    let currency: (Option<i32>, Option<f64>, bool, String) = env
        .eval("local id, amount = C_ContributionCollector.GetRequiredContributionCurrency(501); return id, amount, C_CurrencyInfo.GetCurrencyInfo(2245) ~= nil, GetCurrencyString(2245, 5)")
        .expect("supplied currency cost should be readable");
    assert_eq!(currency.0, Some(2245));
    assert_eq!(currency.1, Some(5.0));
    assert!(currency.2, "fixture currency should have display metadata");
    assert!(!currency.3.is_empty(), "fixture currency should format as text");
    let button_cost: (Option<i32>, Option<String>) = env
        .eval("local b = ContributionCollectionFrame:FindContribution(501).ContributeButton; return b.requiredCurrencyID, b.formatString")
        .expect("original consumer should retain its currency selection");
    assert_eq!(button_cost.0, Some(2245));
    assert_eq!(button_cost.1.as_deref(), Some("Contribute %s"));
    let button_state: String = env
        .eval("local b = ContributionCollectionFrame:FindContribution(501).ContributeButton; return tostring(b.contributionID)..'/'..tostring(b.requiredCurrencyID)..'/'..tostring(b.formatString)..'/'..tostring(b:GetText())")
        .expect("contribution button diagnostic state should be readable");
    assert_ne!(result.4, result.5, "distinct supplied costs should produce distinct button text; {button_state}");
    let details_valid: bool = env
        .eval(
            "return C_ContributionCollector.GetDescription(501) == 'Ready description' and C_ContributionCollector.GetOrderIndex(502) == 2 and C_ContributionCollector.GetRewardQuestID(502) == 0 and #C_ContributionCollector.GetAtlases(501) == 1 and select('#', C_ContributionCollector.GetBuffs(502)) == 0 and select(1, C_ContributionCollector.GetRequiredContributionItem(502)) == 6948",
        )
        .expect("supplied contribution details should be readable");
    assert!(details_valid, "all supplied contribution fields should reach their getters");
}
}
