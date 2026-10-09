#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn frame_xml_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_FrameXML/Blizzard_FrameXML.toc")
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
fn blizzard_frame_xml_toc_is_load_first_with_current_dependencies() {
    let toc = TocFile::from_file(&frame_xml_toc()).expect("Blizzard_FrameXML TOC parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_FrameXML has no `## LoadOnDemand` line — this is the foundational \
         FrameXML core that publishes UIErrorsFrame / AlertFrame / ColorPickerFrame / \
         MovieFrame / TalkingHeadFrame / GhostFrame / TutorialFrame / SplashFrame and \
         the AlertFrame / EquipmentFlyout / SecureTemplates / SpellActivationOverlay / \
         TalentFrameBase template libraries; every downstream Blizzard panel addon \
         depends on these so they MUST be available at startup"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_FrameXML does not declare `## UseSecureEnvironment` — the FrameXML \
         core runs in the standard taint environment; only the `secureMixin` and \
         `secureexecuterange` paths within it (e.g. GhostFrame's secureMixin) opt into \
         secure dispatch on a per-frame basis"
    );
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_ObjectAPI".to_string(),
            "Blizzard_FrameXMLBase".to_string(),
            "Blizzard_UIErrorsFrame".to_string(),
            "Blizzard_UIParentPanelManager".to_string(),
            "Blizzard_SettingsDefinitions_Frame".to_string(),
            "Blizzard_ItemButton".to_string(),
            "Blizzard_UnitPopup".to_string(),
            "Blizzard_FrameXMLUtil".to_string(),
            "Blizzard_RaidWarning".to_string(),
            "Blizzard_UIPanelTemplates".to_string(),
            "Blizzard_GameTooltip".to_string(),
            "Blizzard_MoneyFrame".to_string(),
            "Blizzard_Colors".to_string(),
            "Blizzard_TransmogShared".to_string(),
            "Blizzard_LFGUtil".to_string(),
            "Blizzard_ManagedFrameSystem".to_string(),
            "Blizzard_MirrorTimer".to_string(),
        ],
        "Blizzard_FrameXML declares its current 17 dependencies in TOC order"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_FrameXML declares no top-level `## AllowLoadGameType:` restriction"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_FrameXML declares no `## SavedVariables` — FrameXML core widgets are \
         transient and rebuild on every login"
    );

    let toc_text = std::fs::read_to_string(frame_xml_toc())
        .expect("Blizzard_FrameXML TOC should read");
    assert!(
        toc_text.contains("## LoadFirst: 1"),
        "Blizzard_FrameXML declares `## LoadFirst: 1` — the loader gives this addon \
         priority within its dependency tier so the FrameXML core templates (alert \
         frames, secure templates, color picker, talent frame base, equipment \
         flyout, etc.) are available before downstream panel addons try to inherit \
         from them"
    );
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_FrameXML's current bare TOC omits top-level AllowLoad metadata"
    );
}

#[test]
fn blizzard_frame_xml_allows_only_game_screen() {
    let toc = TocFile::from_file(&frame_xml_toc()).expect("Blizzard_FrameXML TOC parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Missing `## AllowLoad:` defaults to allowing the Game screen"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "Missing `## AllowLoad:` rejects Login by the loader's Game-only default"
    );
    assert!(
        !toc.allows_screen(ScreenKind::CharacterSelect),
        "Missing `## AllowLoad:` rejects CharacterSelect by the loader's Game-only default"
    );
}

#[test]
fn blizzard_frame_xml_auto_loads_on_game_and_skips_login() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FrameXML");
    assert!(
        in_game,
        "Blizzard_FrameXML has no `## LoadOnDemand` line and defaults to Game-only, so it \
         MUST appear in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FrameXML");
    assert!(
        !in_login,
        "The default Game-only AllowLoad behavior excludes Blizzard_FrameXML from Login \
         auto-discovery"
    );
}

prefork_full_ui_case! {
fn blizzard_frame_xml_loads_via_full_game_ui_without_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("Blizzard_FrameXML"))
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_FrameXML emitted Lua errors during the full Game-screen load \
         (the FrameXML core has 10 deps and ~80 files — any single error here would \
         cascade-break downstream panel addons):\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_is_addon_loaded_returns_true_after_full_game_ui_load(env: &WowLuaEnv) {

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_FrameXML') and true or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        post_load,
        "After full Game-screen load, IsAddOnLoaded('Blizzard_FrameXML') must return \
         true — auto-discovery picks up the addon (no LoadOnDemand, AllowLoad: Game) \
         and `mark_addon_loaded` registers it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_publishes_canonical_top_level_singleton_frames(env: &WowLuaEnv) {

    let singletons_present: (bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return UIErrorsFrame ~= nil, \
                    GhostFrame ~= nil, \
                    AlertFrame ~= nil, \
                    MovieFrame ~= nil, \
                    ColorPickerFrame ~= nil, \
                    SplashFrame ~= nil, \
                    TalkingHeadFrame ~= nil",
        )
        .expect("FrameXML singleton probe should succeed");
    assert_eq!(
        singletons_present,
        (true, true, true, true, true, true, true),
        "Blizzard_FrameXML publishes seven canonical UIParent-anchored singletons: \
         UIErrorsFrame (UIErrorsFrame.xml:5 — MessageFrame parent=UIParent \
         frameStrata=DIALOG mixin=UIErrorsMixin, the in-world error scroll), \
         GhostFrame (GhostFrame.xml:4 — Button parent=UIParent secureMixin=GhostFrameMixin, \
         the ghost-mode revive button), AlertFrame (AlertFrames.xml:18 — \
         mixin=AlertFrameMixin inherits=AlertContainerTemplate, the achievement / \
         loot toast container), MovieFrame (MovieFrame.xml:3 — MovieFrame type, \
         enableKeyboard, mixin=MovieFrameMixin, the cinematic playback host), \
         ColorPickerFrame (ColorPickerFrame.xml:75 — toplevel parent=UIParent \
         mixin=ColorPickerFrameMixin, the modal color picker), SplashFrame \
         (SplashFrame.xml:25 — toplevel parent=UIParent mixin=SplashFrameMixin, the \
         expansion-feature splash screen), TalkingHeadFrame (TalkingHeadUI.xml:5 — \
         ContainedAlertFrame parent=UIParent inherits=UIParentBottomManagedFrameTemplate \
         mixin=TalkingHeadFrameMixin, the NPC dialog overlay)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_publishes_secondary_top_level_singleton_frames(env: &WowLuaEnv) {

    let singletons_present: (bool, bool, bool, bool, bool) = env
        .eval(
            "return TutorialFrame ~= nil, \
                    LossOfControlFrame ~= nil, \
                    TimerTracker ~= nil, \
                    EquipmentFlyoutFrame ~= nil, \
                    UIErrorsFrame:GetParent():GetName() == 'UIParent'",
        )
        .expect("FrameXML secondary singleton probe should succeed");
    assert_eq!(
        singletons_present,
        (true, true, true, true, true),
        "Blizzard_FrameXML publishes four additional top-level singletons plus the \
         UIParent-parent contract: TutorialFrame (TutorialFrame.xml:3 — \
         frameStrata=HIGH parent=UIParent hidden, the new-player tutorial overlay), \
         LossOfControlFrame (LossOfControlFrame.xml:4 — frameStrata=MEDIUM \
         parent=UIParent mixin=LossOfControlMixin hidden, the CC indicator), \
         TimerTracker (Timer.xml:156 — frameStrata=DIALOG parent=UIParent \
         setAllPoints, the C_Timer-driven event timer host), EquipmentFlyoutFrame \
         (EquipmentFlyout.xml:40 — frameStrata=HIGH hidden, the inventory flyout \
         used by the character panel for equipment swap suggestions). UIErrorsFrame's \
         parent must round-trip back to UIParent via the Rust loader's parent-key \
         resolution"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_publishes_canonical_alert_and_panel_mixins(env: &WowLuaEnv) {

    let mixins_present: (bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(AlertFrameMixin) == 'table', \
                    type(AlertContainerMixin) == 'table', \
                    type(ContainedAlertSubSystemMixin) == 'table', \
                    type(ColorPickerFrameMixin) == 'table', \
                    type(ColorPickerHexBoxMixin) == 'table', \
                    type(MovieFrameMixin) == 'table', \
                    type(SplashFrameMixin) == 'table'",
        )
        .expect("FrameXML alert/panel mixin probe should succeed");
    assert_eq!(
        mixins_present,
        (true, true, true, true, true, true, true),
        "Blizzard_FrameXML publishes the canonical alert / panel mixin globals: \
         AlertFrameMixin (AlertFrames.lua:464 — AlertFrame singleton driver, the \
         master toast queue), AlertContainerMixin (lua:257 — bounding-box layout for \
         alert subsystems), ContainedAlertSubSystemMixin (lua:6 — abstract base for \
         all alert subsystems with three concrete derivatives \
         AlertFrameExternallyAnchoredMixin / AlertFrameAutoAnchoredMixin / \
         AlertFrameQueueMixin), ColorPickerFrameMixin (ColorPickerFrame.lua:1 — the \
         modal color-picker driver), ColorPickerHexBoxMixin (lua:117 — the hex-input \
         editbox), MovieFrameMixin (MovieFrame.lua — cinematic playback driver), \
         SplashFrameMixin (SplashFrame — expansion-feature splash driver)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_publishes_canonical_overlay_and_toast_mixins(env: &WowLuaEnv) {

    let mixins_present: (bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(LossOfControlMixin) == 'table', \
                    type(MotionSicknessMixin) == 'table', \
                    type(SpellActivationOverlayMixin) == 'table', \
                    type(GhostFrameMixin) == 'table', \
                    type(TalkingHeadFrameMixin) == 'table', \
                    type(AchievementDisplayMixin) == 'table', \
                    type(AzeriteIslandsToastMixin) == 'table'",
        )
        .expect("FrameXML overlay/toast mixin probe should succeed");
    assert_eq!(
        mixins_present,
        (true, true, true, true, true, true, true),
        "Blizzard_FrameXML publishes the canonical overlay / toast mixin globals: \
         LossOfControlMixin (the CC indicator driver), MotionSicknessMixin (the \
         camera-spin nausea-reduction overlay), SpellActivationOverlayMixin (the \
         proc/highlight glow on action buttons, with three sub-mixins for the texture \
         and fade-in/fade-out animations), GhostFrameMixin (the corpse-recovery button \
         driver, attached via secureMixin so corpse interaction stays insecure-safe), \
         TalkingHeadFrameMixin (the NPC dialog overlay driver, with companion \
         TalkingHeadFrameModelMixin for the 3D head model), AchievementDisplayMixin \
         (the achievement toast renderer), AzeriteIslandsToastMixin (the BfA Islands \
         toast — kept around for completeness even though the Islands feature is \
         retired)"
    );
}
}

prefork_full_ui_case! {
fn earning_achievement_displays_achievement_alert_toast(env: &WowLuaEnv) {
    wow_ui_sim::startup::run_extra_update_ticks(&env, 1);

    let result: String = env
        .eval(
            r#"
            if not AlertFrame or not AchievementAlertSystem then
                return "missing_alert_system"
            end

            A_Admin.EarnAchievement(6)

            local visibleCount = AchievementAlertSystem:GetNumVisibleAlerts()
            local toastName = nil
            local toastShown = false
            for frame in AchievementAlertSystem.alertFramePool:EnumerateActive() do
                toastShown = frame:IsShown()
                toastName = frame.Name and frame.Name:GetText() or nil
                break
            end

            return table.concat({
                tostring(AlertFrame:AreAlertsEnabled()),
                tostring(visibleCount),
                tostring(toastShown),
                tostring(toastName),
            }, "|")
            "#,
        )
        .expect("earning an achievement should let FrameXML show the achievement alert");

    assert_eq!(
        result, "true|1|true|Level 10",
        "earning achievement 6 should display one visible achievement alert toast with the achievement name"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_publishes_equipment_manager_inventory_and_bag_slot_tables(env: &WowLuaEnv) {

    let tables_present: (bool, bool) = env
        .eval(
            "return type(EQUIPMENTMANAGER_INVENTORYSLOTS) == 'table', \
                    type(EQUIPMENTMANAGER_BAGSLOTS) == 'table'",
        )
        .expect("EquipmentManager slot table probe should succeed");
    assert_eq!(
        tables_present,
        (true, true),
        "Blizzard_FrameXML publishes the two EquipmentManager slot-tracking tables \
         (EquipmentManager.lua) — `EQUIPMENTMANAGER_INVENTORYSLOTS` (a sparse table \
         keyed by the slot ID for each equipped item) and \
         `EQUIPMENTMANAGER_BAGSLOTS` (a sparse table keyed by `bagID, slot` pair for \
         every bag inventory slot). Both are built up by the equipment manager as it \
         scans inventory and used by SaveEquipmentSet / EquipmentSetContainsItem, so \
         they must publish at addon load time even before any equipment-set save is \
         attempted"
    );
}
}

prefork_full_ui_case! {
fn equipment_manager_free_space_update_handles_bank_tab_count(env: &WowLuaEnv) {

    let last_bag_slot: i32 = env
        .eval(
            "return NUM_TOTAL_EQUIPPED_BAG_SLOTS \
                 + C_Bank.FetchNumPurchasedBankTabs(Enum.BankType.Character)",
        )
        .expect("EquipmentManager bag-slot upper bound should not error");
    assert!(
        last_bag_slot >= 5,
        "EquipmentManager.lua adds FetchNumPurchasedBankTabs() to NUM_TOTAL_EQUIPPED_BAG_SLOTS; \
         the bank API must return a non-nil numeric tab count"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_publishes_alert_frame_subsystem_derivative_mixins(env: &WowLuaEnv) {

    let derived_mixins: (bool, bool, bool) = env
        .eval(
            "return type(AlertFrameExternallyAnchoredMixin) == 'table', \
                    type(AlertFrameAutoAnchoredMixin) == 'table', \
                    type(AlertFrameQueueMixin) == 'table'",
        )
        .expect("AlertFrame subsystem derivative probe should succeed");
    assert_eq!(
        derived_mixins,
        (true, true, true),
        "AlertFrames.lua publishes three concrete derivatives of \
         ContainedAlertSubSystemMixin via CreateFromMixins (lua:28/50/82): \
         AlertFrameExternallyAnchoredMixin (toasts whose final position is dictated \
         by an external anchor — used by edit-mode-positioned achievement / loot \
         toasts), AlertFrameAutoAnchoredMixin (toasts that anchor to each other in a \
         vertical stack as they queue), AlertFrameQueueMixin (toasts that share a \
         single onscreen slot and FIFO-cycle through it). All three round-trip back \
         to ContainedAlertSubSystemMixin via CreateFromMixins, so the parent base \
         table must also publish for the inheritance chain to resolve"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_top_level_singleton_default_visibility_matches_xml(env: &WowLuaEnv) {

    let visibility: (bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return MovieFrame:IsShown(), \
                    ColorPickerFrame:IsShown(), \
                    SplashFrame:IsShown(), \
                    LossOfControlFrame:IsShown(), \
                    TutorialFrame:IsShown(), \
                    GhostFrame:IsShown()",
        )
        .expect("Top-level visibility probe should succeed");
    assert_eq!(
        visibility,
        (false, false, false, false, false, false),
        "All six modal / overlay singletons declare `hidden=\"true\"` in XML \
         (MovieFrame.xml:3, ColorPickerFrame.xml:75, SplashFrame.xml:25, \
         LossOfControlFrame.xml:4, TutorialFrame.xml:3, GhostFrame.xml:4) so they \
         start hidden after addon load — visible only when summoned by their \
         respective triggers (cinematic-playback / Settings color picker / \
         expansion-feature splash flow / CC application / new-player tutorial step / \
         player-death). Any of these summoning automatically would block player input \
         on every login"
    );
}
}
