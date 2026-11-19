#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn frame_xml_base_toc() -> PathBuf {
    find_toc_file(&blizzard_ui_dir().join("Blizzard_FrameXMLBase"))
        .expect("Blizzard_FrameXMLBase TOC for active profile")
}

fn assert_standard_highlight_color(env: &WowLuaEnv) {
    let green: f64 = env
        .eval("return QuestDifficultyHighlightColors['standard'].g")
        .expect("standard highlight color probe should succeed");
    assert_eq!(green, 0.93);
}

fn assert_impossible_quest_color(env: &WowLuaEnv) {
    let impossible_color: (f64, f64, f64) = env
        .eval("local c = QuestDifficultyColors['impossible']; return c.r, c.g, c.b")
        .expect("Impossible color probe should succeed");
    let red = (impossible_color.0 - 1.00).abs() < 1e-6;
    let green_low = (impossible_color.1 - 0.10).abs() < 1e-6;
    let blue_low = (impossible_color.2 - 0.10).abs() < 1e-6;
    assert!(
        red && green_low && blue_low,
        "QuestDifficultyColors.impossible should be {{r=1.00, g=0.10, b=0.10}}; got {:?}",
        impossible_color
    );
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
fn blizzard_frame_xml_base_mainline_toc_is_eager_with_two_deps_and_allow_load_game() {
    let toc =
        TocFile::from_file(&frame_xml_base_toc()).expect("Blizzard_FrameXMLBase_Mainline TOC");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_FrameXMLBase has no `## LoadOnDemand` line — this is the base library \
         tier that publishes Constants.lua (INVSLOT_*, NUM_BAG_SLOTS, CLASS_SORT_ORDER, \
         QuestDifficultyColors, ...), AnimatedStatusBarMixin, GradualAnimatedStatusBarMixin, \
         IconDataProviderMixin, PowerDependencyLineMixin, FlowContainer_*, and \
         PlayerMovementFrameFader. Every downstream Blizzard addon depends on these so they \
         MUST be eagerly loaded at startup"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_FrameXMLBase does not declare `## UseSecureEnvironment` — the base \
         constants/mixins live in the standard taint environment"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_FrameXMLBase declares no `## AllowLoadGameType:` line on the Mainline \
         TOC, so `is_game_type_restricted()` returns false and the addon is eligible for \
         standard mainline retail discovery"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_FrameXMLBase declares no `## SavedVariables` — pure code/constants \
         library, no per-character persistence"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        &[
            "Blizzard_SharedXML".to_string(),
            "Blizzard_SharedXMLGame".to_string()
        ],
        "Blizzard_FrameXMLBase declares exactly two `## Dependencies` in canonical order: \
         Blizzard_SharedXML (provides SharedFontStyles / SharedUIPanelTemplates / FrameUtil) \
         and Blizzard_SharedXMLGame (provides game-side shared XML helpers). Both must \
         load before this base library so Constants.lua's references to `Constants.\
         InventoryConstants.NumBagSlots` and `Enum.BagIndex.Backpack` resolve, and so the \
         AnimatedStatusBarTemplate XML can inherit shared status-bar templates"
    );

    let toc_text =
        std::fs::read_to_string(frame_xml_base_toc()).expect("Blizzard_FrameXMLBase TOC read");
    assert_eq!(
        toc.metadata.get("DefaultState"),
        None,
        "Blizzard_FrameXMLBase omits DefaultState in the current Retail TOC"
    );
    assert!(
        toc.default_enabled(),
        "Absent DefaultState defaults to enabled"
    );
    assert_eq!(
        toc.metadata.get("AllowLoad"),
        None,
        "Blizzard_FrameXMLBase omits AllowLoad; the loader defaults it to Game"
    );
    assert!(
        !toc_text.contains("## LoadFirst"),
        "Blizzard_FrameXMLBase does NOT declare `## LoadFirst:` — the loader runs it in \
         the standard tier; its two dependencies (Blizzard_SharedXML / SharedXMLGame) are \
         enough to order it correctly without forcing a priority bucket"
    );
    assert!(
        !toc_text.contains("## LoadOnDemand"),
        "Blizzard_FrameXMLBase has no `## LoadOnDemand` raw substring — it is eager"
    );
}

#[test]
fn blizzard_frame_xml_base_allows_only_game_screen() {
    let toc =
        TocFile::from_file(&frame_xml_base_toc()).expect("Blizzard_FrameXMLBase_Mainline TOC");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` must permit the Game screen (src/toc.rs:307)"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: Game` must reject the Login screen — base FrameXML constants are \
         in-world only"
    );
    assert!(
        !toc.allows_screen(ScreenKind::CharacterSelect),
        "`## AllowLoad: Game` must reject CharacterSelect — base FrameXML constants are \
         in-world only"
    );
}

#[test]
fn blizzard_frame_xml_base_auto_loads_on_game_and_skips_login() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FrameXMLBase");
    assert!(
        in_game,
        "Blizzard_FrameXMLBase has no `## LoadOnDemand` line and `## AllowLoad: Game`, \
         so it MUST appear in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FrameXMLBase");
    assert!(
        !in_login,
        "`## AllowLoad: Game` excludes Blizzard_FrameXMLBase from Login auto-discovery"
    );
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_loads_via_full_game_ui_without_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("FrameXMLBase")
                || message.contains("AnimatedStatusBar")
                || message.contains("GradualAnimatedStatusBar")
                || message.contains("IconDataProvider")
                || message.contains("PowerDependencyLine")
                || message.contains("FlowContainer")
                || message.contains("PlayerMovementFrameFader")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_FrameXMLBase emitted Lua errors during the full Game-screen load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_is_addon_loaded_returns_true_after_full_game_ui_load(env: &WowLuaEnv) {

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_FrameXMLBase') and true or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        post_load,
        "After full Game-screen load, IsAddOnLoaded('Blizzard_FrameXMLBase') must return \
         true — auto-discovery picks up the addon (no LoadOnDemand) and \
         `mark_addon_loaded` registers it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_status_bar_and_data_provider_mixins(env: &WowLuaEnv) {

    let mixins: (bool, bool, bool, bool) = env
        .eval(
            "return type(AnimatedStatusBarMixin) == 'table', \
                    type(GradualAnimatedStatusBarMixin) == 'table', \
                    type(IconDataProviderMixin) == 'table', \
                    type(PowerDependencyLineMixin) == 'table'",
        )
        .expect("FrameXMLBase mixin probe should succeed");
    assert_eq!(
        mixins,
        (true, true, true, true),
        "Blizzard_FrameXMLBase publishes four core mixin globals: \
         AnimatedStatusBarMixin (AnimatedStatusBar.lua:1 — XP/reputation/honor bar with \
         glow-line tile animations and accumulation timeout), \
         GradualAnimatedStatusBarMixin (GradualAnimatedStatusBar.lua:2 — smooth-fill \
         status bar with gain-flare animation), \
         IconDataProviderMixin (Mainline/IconDataProvider.lua:43 — shared icon picker \
         data source backing macro/spell/item icon selection UIs), \
         PowerDependencyLineMixin (PowerDependencyLine.lua:1 — talent/power-tree \
         connector line widget with connected/disconnected/locked states)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_inventory_slot_constants(env: &WowLuaEnv) {

    let slots: (i64, i64, i64, i64, i64, i64) = env
        .eval(
            "return INVSLOT_HEAD, INVSLOT_TABARD, INVSLOT_FIRST_EQUIPPED, \
                    INVSLOT_LAST_EQUIPPED, NUM_INVSLOTS, INVSLOT_AMMO",
        )
        .expect("INVSLOT_* probe should succeed");
    assert_eq!(
        slots,
        (1, 19, 1, 19, 19, 0),
        "Blizzard_FrameXMLBase/Constants.lua:151-173 publishes the canonical inventory \
         slot constants: INVSLOT_AMMO=0, INVSLOT_HEAD=1 (= INVSLOT_FIRST_EQUIPPED), \
         INVSLOT_TABARD=19 (= INVSLOT_LAST_EQUIPPED), NUM_INVSLOTS=19 (TABARD - HEAD + 1). \
         These drive every paper-doll / equipment-manager UI in the game"
    );

    let combat_slots: (bool, bool, bool, bool) = env
        .eval(
            "return INVSLOTS_EQUIPABLE_IN_COMBAT[INVSLOT_MAINHAND] == true, \
                    INVSLOTS_EQUIPABLE_IN_COMBAT[INVSLOT_OFFHAND] == true, \
                    INVSLOTS_EQUIPABLE_IN_COMBAT[INVSLOT_RANGED] == true, \
                    INVSLOTS_EQUIPABLE_IN_COMBAT[INVSLOT_HEAD] == nil",
        )
        .expect("INVSLOTS_EQUIPABLE_IN_COMBAT probe should succeed");
    assert_eq!(
        combat_slots,
        (true, true, true, true),
        "INVSLOTS_EQUIPABLE_IN_COMBAT (Constants.lua:175-179) is the set of three weapon \
         slots that can be equipped in combat — head/chest/etc. are NOT in the set"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_class_sort_order_constants(env: &WowLuaEnv) {

    let class_count: (bool, i64, bool, bool) = env
        .eval(
            "return type(CLASS_SORT_ORDER) == 'table', MAX_CLASSES, \
                    CLASS_SORT_ORDER[1] == 'WARRIOR', CLASS_SORT_ORDER[13] == 'EVOKER'",
        )
        .expect("CLASS_SORT_ORDER probe should succeed");
    assert_eq!(
        class_count,
        (true, 13, true, true),
        "Blizzard_FrameXMLBase/Constants.lua:37-52 publishes CLASS_SORT_ORDER (13 entries: \
         WARRIOR..EVOKER) and MAX_CLASSES = #CLASS_SORT_ORDER = 13. Every class-color and \
         class-icon path in the UI iterates this list — its presence and length are \
         load-bearing"
    );

    let localized: (bool, bool) = env
        .eval(
            "return type(LOCALIZED_CLASS_NAMES_MALE) == 'table', \
                    type(LOCALIZED_CLASS_NAMES_FEMALE) == 'table'",
        )
        .expect("LOCALIZED_CLASS_NAMES probe should succeed");
    assert_eq!(
        localized,
        (true, true),
        "Constants.lua:54-55 calls LocalizedClassList(false/true) at module-load to \
         populate LOCALIZED_CLASS_NAMES_{{MALE,FEMALE}} — these tables MUST be tables \
         after addon load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_bag_slot_constants_from_constants_namespace(env: &WowLuaEnv) {

    let bag: (bool, bool, bool) = env
        .eval(
            "return type(NUM_BAG_SLOTS) == 'number' and NUM_BAG_SLOTS > 0, \
                    type(NUM_TOTAL_EQUIPPED_BAG_SLOTS) == 'number', \
                    type(BACKPACK_CONTAINER) == 'number'",
        )
        .expect("Bag slot constants probe should succeed");
    assert_eq!(
        bag,
        (true, true, true),
        "Constants.lua:184-188 publishes NUM_BAG_SLOTS / NUM_REAGENTBAG_SLOTS / \
         NUM_TOTAL_EQUIPPED_BAG_SLOTS / BACKPACK_CONTAINER. These resolve `Constants.\
         InventoryConstants.NumBagSlots` (from Blizzard_SharedXML's Constants surface) \
         and `Enum.BagIndex.Backpack` at addon-load time — verifies the dependency \
         chain is wired correctly"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_loot_and_item_location_bitflags(env: &WowLuaEnv) {
    const ITEM_INVENTORY_LOCATION_PLAYER: i64 = 0x00100000;
    const ITEM_INVENTORY_LOCATION_BAGS: i64 = 0x00200000;
    const ITEM_INVENTORY_LOCATION_BANK: i64 = 0x00400000;


    let flags: (i64, i64, i64, i64, i64, i64, i64) = env
        .eval(
            "return LOOT_ROLL_TYPE_PASS, LOOT_ROLL_TYPE_NEED, LOOT_ROLL_TYPE_GREED, \
                    LOOT_ROLL_TYPE_DISENCHANT, ITEM_INVENTORY_LOCATION_PLAYER, \
                    ITEM_INVENTORY_LOCATION_BAGS, ITEM_INVENTORY_LOCATION_BANK",
        )
        .expect("Loot/inventory bitflag probe should succeed");
    assert_eq!(
        flags,
        (
            0,
            1,
            2,
            3,
            ITEM_INVENTORY_LOCATION_PLAYER,
            ITEM_INVENTORY_LOCATION_BAGS,
            ITEM_INVENTORY_LOCATION_BANK
        ),
        "Constants.lua:140-148 publishes loot-roll types (PASS=0..DISENCHANT=3) and \
         the three item inventory location bitflags PLAYER / BAGS / BANK — these are \
         the canonical bitflag layout consumed by every loot-roll dialog and bag-slot \
         serializer"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_totem_priority_tables(env: &WowLuaEnv) {

    let totems: (i64, i64, i64, i64, i64) = env
        .eval(
            "return MAX_TOTEMS, FIRE_TOTEM_SLOT, EARTH_TOTEM_SLOT, \
                    WATER_TOTEM_SLOT, AIR_TOTEM_SLOT",
        )
        .expect("Totem slot probe should succeed");
    assert_eq!(
        totems,
        (4, 1, 2, 3, 4),
        "Constants.lua:239-244 publishes the four shaman totem slot indices and \
         MAX_TOTEMS=4. SHAMAN_TOTEM_PRIORITIES (lines 248-253) reorders these for the \
         shaman totem-bar UI"
    );

    let priority_table: (bool, bool, i64, i64) = env
        .eval(
            "return type(SHAMAN_TOTEM_PRIORITIES) == 'table', \
                    type(STANDARD_TOTEM_PRIORITIES) == 'table', \
                    SHAMAN_TOTEM_PRIORITIES[1], STANDARD_TOTEM_PRIORITIES[1]",
        )
        .expect("Totem priority table probe should succeed");
    assert_eq!(
        priority_table,
        (true, true, 2, 1),
        "STANDARD_TOTEM_PRIORITIES = {{1,2,3,4}} (raw slot order); \
         SHAMAN_TOTEM_PRIORITIES[1] == EARTH_TOTEM_SLOT == 2 (shamans see earth first)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_quest_difficulty_color_table(env: &WowLuaEnv) {

    let colors: (bool, bool, bool, bool) = env
        .eval(
            "return type(QuestDifficultyColors) == 'table', \
                    type(QuestDifficultyColors['impossible']) == 'table', \
                    type(QuestDifficultyColors['standard']) == 'table', \
                    type(QuestDifficultyHighlightColors) == 'table'",
        )
        .expect("QuestDifficultyColors probe should succeed");
    assert_eq!(
        colors,
        (true, true, true, true),
        "Constants.lua:210-228 publishes QuestDifficultyColors and \
         QuestDifficultyHighlightColors — seven difficulty buckets each \
         (impossible/verydifficult/difficult/standard/trivial/header/disabled) with \
         {{r,g,b,font}} entries. Every quest-log title color binding reads from these"
    );

    assert_impossible_quest_color(env);
    assert_standard_highlight_color(env);
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_does_not_publish_removed_frame_lock_globals(env: &WowLuaEnv) {
    let removed_globals: (bool, bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return SmartShow == nil, SmartHide == nil, IsFrameSmartShown == nil, \
                    IsFrameLockActive == nil, AddFrameLock == nil, RemoveFrameLock == nil, \
                    SetFrameLock == nil, FRAMELOCK_STATES == nil, \
                    FRAMELOCK_STATE_PRIORITIES == nil",
        )
        .expect("removed FrameLocks global probe should succeed");
    assert_eq!(
        removed_globals,
        (true, true, true, true, true, true, true, true, true),
        "Retail 12.1 no longer loads the legacy FrameLocks module through \
         Blizzard_FrameXMLBase, so none of its former public globals may be required by \
         the current-source contract"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_flow_container_helpers(env: &WowLuaEnv) {

    let flow_container: (bool, bool, bool, bool, bool) = env
        .eval(
            "return type(FlowContainer_Initialize) == 'function', \
                    type(FlowContainer_PauseUpdates) == 'function', \
                    type(FlowContainer_ResumeUpdates) == 'function', \
                    type(FlowContainer_SetOrientation) == 'function', \
                    type(FlowContainer_SetMaxPerLine) == 'function'",
        )
        .expect("FlowContainer probe should succeed");
    assert_eq!(
        flow_container,
        (true, true, true, true, true),
        "FlowContainer.lua publishes the FlowContainer_* helper family used by the chat \
         frame docking, alert toast queueing, and DropDownMenu reflow paths: Initialize \
         (bootstrap), PauseUpdates / ResumeUpdates (batch insert), SetOrientation \
         (horizontal/vertical), SetMaxPerLine (cap)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_html_and_spec_constants(env: &WowLuaEnv) {

    let html: (String, String, String) = env
        .eval("return HTML_START, HTML_START_CENTERED, HTML_END")
        .expect("HTML constants probe should succeed");
    assert_eq!(
        html,
        (
            "<html><body><p>".to_string(),
            "<html><body><p align=\"center\">".to_string(),
            "</p></body></html>".to_string()
        ),
        "Constants.lua:24-26 publishes the HTML wrapping constants used by every \
         SimpleHTML frame (quest detail body, ChatConfigFrame help, gossip dialogs). \
         These are the canonical wrapping fragments the C HTML parser expects"
    );

    let specs: (i64, i64, i64) = env
        .eval("return SPECIALIZATION_TAB, TALENTS_TAB, NUM_TALENT_FRAME_TABS")
        .expect("Talent tab constant probe should succeed");
    assert_eq!(
        specs,
        (1, 2, 2),
        "Constants.lua:78-80 publishes SPECIALIZATION_TAB=1 / TALENTS_TAB=2 / \
         NUM_TALENT_FRAME_TABS=2 — the canonical PlayerSpellsFrame two-tab layout. \
         Drives every talent-tab switching path in the UI"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_animated_status_bar_template(env: &WowLuaEnv) {

    let templates: (bool, bool) = env
        .eval(
            "return type(AnimatedStatusBarMixin.OnLoad) == 'function', \
                    type(AnimatedStatusBarMixin.Reset) == 'function'",
        )
        .expect("AnimatedStatusBarMixin method probe should succeed");
    assert_eq!(
        templates,
        (true, true),
        "AnimatedStatusBar.lua:5,18 defines AnimatedStatusBarMixin:OnLoad and :Reset. \
         OnLoad sets DEFAULT_ACCUMULATION_TIMEOUT_SEC=0.1, matchLevelOnFirstWrap=true, \
         matchBarValueToAnimation=false; Reset sets pendingReset=true so the next \
         OnUpdate snaps the bar back to its base values"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_base_publishes_max_raid_member_constants_from_shared(env: &WowLuaEnv) {

    let raid: (i64, i64, i64) = env
        .eval("return MAX_RAID_MEMBERS, NUM_RAID_GROUPS, MEMBERS_PER_RAID_GROUP")
        .expect("MAX_RAID_MEMBERS probe should succeed");
    assert_eq!(
        raid,
        (40, 8, 5),
        "Shared/Constants.lua:1-3 publishes MAX_RAID_MEMBERS=40 / NUM_RAID_GROUPS=8 / \
         MEMBERS_PER_RAID_GROUP=5. The TOC's `Shared\\Constants.lua` line ensures these \
         are loaded — confirms the Mainline-vs-Shared subdir resolution works"
    );
}
}
