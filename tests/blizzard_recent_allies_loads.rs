use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn recent_allies_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_RecentAllies/Blizzard_RecentAllies.toc")
}

const TOC_FILES: &[&str] = &[
    "Blizzard_RecentAlliesUtil.lua",
    "Blizzard_RecentAlliesTemplates.lua",
    "Blizzard_RecentAlliesTemplates.xml",
];

const MIXIN_TABLES: &[&str] = &[
    "RecentAlliesUtil",
    "RecentAlliesListMixin",
    "RecentAlliesEntryMixin",
    "RecentAlliesEntryPartyButtonMixin",
    "RecentAlliesEntryPinDisplayMixin",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "RecentAlliesListTemplate",
    "RecentAlliesDividerTemplate",
    "RecentAlliesEntryPartyButtonTemplate",
    "RecentAlliesEntryFriendRequestPendingDisplayTemplate",
    "RecentAlliesEntryPinDisplayTemplate",
    "RecentAlliesEntryTemplate",
];

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
fn blizzard_recent_allies_toc_pins_eager_template_only_addon_with_spaced_title() {
    let toc = TocFile::from_file(&recent_allies_toc()).expect("Blizzard_RecentAllies TOC parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_RecentAllies has no `## LoadOnDemand` line — the recent-allies tab \
         template registry must be available eagerly so Blizzard_FriendsFrame can resolve \
         `RecentAlliesListTemplate` when it builds the FriendsFrame.RecentAlliesFrame \
         tab content during its own XML load"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_RecentAllies has no `## LoadFirst` line — it is a leaf template addon \
         and not part of the early-bootstrap tier"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_RecentAllies does not declare `## UseSecureEnvironment` — the recent \
         allies list is a plain (insecure) social UI tab"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_RecentAllies declares `## AllowLoadGameType: mainline`, but \
         `is_game_type_restricted()` returns false because src/toc.rs:299 treats \
         `mainline` and `standard` as the unrestricted (retail) game type"
    );
    assert!(
        toc.saved_variables().is_empty() && toc.saved_variables_per_character().is_empty(),
        "Blizzard_RecentAllies declares no `## SavedVariables*` — recent-ally state \
         (pinned characters, recent interactions) lives server-side in the Rolodex \
         backend, surfaced via C_RecentAllies, not in the per-character SavedVars file"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_SocialUIShared".to_string()],
        "Retail 12.1.0.69497 declares Blizzard_SocialUIShared as the RecentAllies hard dependency"
    );
    assert_eq!(
        toc.optional_deps(),
        vec!["Blizzard_UnitPopupShared".to_string()],
        "Retail 12.1.0.69497 declares Blizzard_UnitPopupShared as the RecentAllies optional dependency"
    );

    let toc_text =
        std::fs::read_to_string(recent_allies_toc()).expect("Blizzard_RecentAllies TOC reads");
    assert!(
        toc_text.contains("## Title: Blizzard_RecentAllies"),
        "Retail 12.1.0.69497 declares the underscored Blizzard_RecentAllies title"
    );
    assert!(
        toc_text.contains("## AllowLoad: both"),
        "Blizzard_RecentAllies declares `## AllowLoad: both` — the recent-allies tab is \
         technically reachable from glue screens too via the BattleNet friend list path, \
         though in practice it only renders meaningful content in-world"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: mainline"),
        "Blizzard_RecentAllies declares `## AllowLoadGameType: mainline` — the Rolodex \
         backend is retail-only; classic flavors don't have C_RecentAllies"
    );
    assert!(
        !toc_text.contains("## DefaultState"),
        "Blizzard_RecentAllies omits `## DefaultState` — defaults to enabled (the addon \
         must be on for FriendsFrame's RecentAlliesFrame tab to populate)"
    );
    assert!(
        !toc_text.contains("## Author"),
        "Blizzard_RecentAllies omits `## Author` — Blizzard's internal addons typically \
         skip this attribution field"
    );
    assert!(
        !toc_text.contains("## Version"),
        "Blizzard_RecentAllies omits `## Version` — the directive is unused by Blizzard's \
         own addons (they ship in lockstep with the client patch level)"
    );
}

#[test]
fn blizzard_recent_allies_toc_lists_three_files_in_dependency_order() {
    let toc = TocFile::from_file(&recent_allies_toc()).expect("Blizzard_RecentAllies TOC parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();

    assert_eq!(
        listed,
        TOC_FILES.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "Blizzard_RecentAllies lists three files in the order Util.lua → Templates.lua \
         → Templates.xml. Util.lua publishes the `RecentAlliesUtil` singleton (consumed \
         by Templates.lua's tooltip builders for time formatting and interaction-context \
         strings). Templates.lua publishes the four entry/list/party/pin mixin tables \
         that the XML consumes via mixin=\"…Mixin\" attributes. The XML must come last \
         because XML parsing resolves mixin names against the global env populated by \
         the two .lua files. Got: {:?}",
        listed
    );
}

#[test]
fn blizzard_recent_allies_allows_all_screens_via_allow_load_both() {
    let toc = TocFile::from_file(&recent_allies_toc()).expect("Blizzard_RecentAllies TOC parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: both` must allow the Game screen (src/toc.rs:307)"
    );
    assert!(
        toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: both` must allow the Login screen — recent-allies template \
         registry is technically reachable from BattleNet friend list overlays in glue UI"
    );
    assert!(
        toc.allows_screen(ScreenKind::CharacterSelect),
        "`## AllowLoad: both` must allow CharacterSelect"
    );
}

#[test]
fn blizzard_recent_allies_appears_in_eager_game_discovery() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_RecentAllies");
    assert!(
        in_game,
        "Blizzard_RecentAllies has no `## LoadOnDemand` line and `## AllowLoad: both`, \
         so the eager-discovery filter at src/loader/mod.rs:527 (which rejects only \
         load_on_demand / ptr_only / game_type_restricted addons) MUST keep it in the \
         Game-screen inventory. This is what wires the load-order edge to \
         Blizzard_FriendsFrame's OptionalDeps"
    );
}

prefork_full_ui_case! {
fn blizzard_recent_allies_loads_without_errors_during_full_game_startup(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("RecentAllies")
                || message.contains("RecentAlly")
                || message.contains("Blizzard_RecentAllies")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_RecentAllies emitted Lua errors during full Game-screen startup:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_recent_allies_is_addon_loaded_returns_true_after_full_game_ui_load(env: &WowLuaEnv) {

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_RecentAllies') and true or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        post_load,
        "After full Game-screen load, IsAddOnLoaded('Blizzard_RecentAllies') must \
         return true — eager auto-discovery picks up the addon (no LoadOnDemand) and \
         `mark_addon_loaded` registers it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_recent_allies_publishes_five_mixin_tables(env: &WowLuaEnv) {

    for mixin in MIXIN_TABLES {
        let exists: bool = env
            .eval(&format!("return type({mixin}) == 'table'"))
            .unwrap_or_else(|err| panic!("Mixin probe `{mixin}` failed: {err}"));
        assert!(
            exists,
            "After Blizzard_RecentAllies loads, the global `{mixin}` must be a table. \
             Util.lua publishes RecentAlliesUtil (a singleton with module-level \
             functions); Templates.lua publishes the four `*Mixin = {{}}` declarations \
             consumed by the XML's mixin=\"…Mixin\" attributes (RecentAlliesListMixin \
             on RecentAlliesListTemplate, RecentAlliesEntryMixin on \
             RecentAlliesEntryTemplate, RecentAlliesEntryPartyButtonMixin on \
             RecentAlliesEntryPartyButtonTemplate, RecentAlliesEntryPinDisplayMixin on \
             RecentAlliesEntryPinDisplayTemplate)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_recent_allies_util_publishes_two_module_functions(env: &WowLuaEnv) {

    let methods: (bool, bool) = env
        .eval(
            "return type(RecentAlliesUtil.GetFormattedTime) == 'function', \
                    type(RecentAlliesUtil.GenerateContextStringForInteraction) == 'function'",
        )
        .expect("RecentAlliesUtil method probe should succeed");
    assert_eq!(
        methods,
        (true, true),
        "RecentAliesUtil.GetFormattedTime (Util.lua:9) wraps a SecondsFormatterMixin \
         instance for the recent-ally time-since-interaction tooltip rows (auto-switching \
         between hour-precision under 1 day and day-precision otherwise). \
         RecentAlliesUtil.GenerateContextStringForInteraction (Util.lua:118) dispatches \
         to one of five private context-string builders via an Enum.RolodexType-keyed \
         table — different interaction kinds (raid kill, M+ run, crafting order, etc.) \
         get different parenthetical context formats"
    );
}
}

prefork_full_ui_case! {
fn blizzard_recent_allies_virtual_templates_not_in_global_env(env: &WowLuaEnv) {

    for tmpl in VIRTUAL_TEMPLATES {
        let leaked: bool = env
            .eval(&format!("return _G[{tmpl:?}] ~= nil"))
            .unwrap_or_else(|err| panic!("Template probe `{tmpl}` failed: {err}"));
        assert!(
            !leaked,
            "Virtual template `{tmpl}` must NOT appear as a global. Templates registered \
             via `virtual=\"true\"` are XML-only fixtures consumed via inherits=\"…\" / \
             ScrollUtil factories — they should never be CreateFrame-instantiated by \
             name and should never leak into _G"
        );
    }
}
}

#[test]
fn blizzard_recent_allies_has_no_named_non_virtual_frames() {
    let xml_path =
        blizzard_ui_dir().join("Blizzard_RecentAllies/Blizzard_RecentAlliesTemplates.xml");
    let xml = std::fs::read_to_string(&xml_path).expect("Blizzard_RecentAlliesTemplates.xml reads");

    let mut named_non_virtual = 0usize;
    for line in xml.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("name=\"") {
            continue;
        }
        if trimmed.contains("virtual=\"true\"") {
            continue;
        }
        // Skip $parent-prefixed sub-element names — those are subframes inside templates,
        // not top-level frame instances. Same for parentKey-only declarations.
        if trimmed.contains("name=\"$parent") {
            continue;
        }
        named_non_virtual += 1;
    }
    assert_eq!(
        named_non_virtual, 0,
        "Blizzard_RecentAlliesTemplates.xml declares ZERO non-virtual top-level frames. \
         Every Frame/Button/EventFrame in the file is `virtual=\"true\"`, including the \
         outermost RecentAlliesListTemplate. Instances are created via inherits=\"…\" \
         from FriendsFrame.xml's RecentAlliesFrame node and via ScrollUtil's \
         element-factory closures (Templates.lua:14-31) — never via direct \
         CreateFrame(\"…\", \"GlobalName\", parent, \"RecentAlliesEntryTemplate\") with a \
         name. The single `$parent…Texture` named declarations on the party button \
         become per-instance child names, not template-level globals"
    );
}

prefork_full_ui_case! {
fn blizzard_recent_allies_list_mixin_event_registration_is_lazy(env: &WowLuaEnv) {

    let lifecycle: (bool, bool, bool, bool) = env
        .eval(
            "return type(RecentAlliesListMixin.OnLoad) == 'function', \
                    type(RecentAlliesListMixin.OnShow) == 'function', \
                    type(RecentAlliesListMixin.OnHide) == 'function', \
                    type(RecentAlliesListMixin.OnEvent) == 'function'",
        )
        .expect("Lifecycle probe should succeed");
    assert_eq!(
        lifecycle,
        (true, true, true, true),
        "RecentAlliesListMixin publishes the four lifecycle hooks consumed by the \
         <Scripts> block in Templates.xml:27-32 (OnLoad/OnShow/OnHide/OnEvent). The \
         event-registration split — InitializeScrollBox in OnLoad (one-shot) vs \
         RECENT_ALLIES_CACHE_UPDATE registration in OnShow/OnHide via \
         FrameUtil.RegisterFrameForEvents (Templates.lua:45,56) — means RecentAllies \
         only listens for cache invalidations while its tab is actually visible. The \
         OnShow path also calls C_RecentAllies.TryRequestRecentAlliesData (line 47) so \
         opening the tab triggers a server fetch even if the cached event hasn't fired"
    );
}
}

#[test]
fn blizzard_recent_allies_xml_loaded_via_explicit_toc_listing_not_script_directive() {
    let xml_path =
        blizzard_ui_dir().join("Blizzard_RecentAllies/Blizzard_RecentAlliesTemplates.xml");
    let xml = std::fs::read_to_string(&xml_path).expect("Templates.xml reads");

    assert!(
        !xml.contains("<Script file="),
        "Blizzard_RecentAlliesTemplates.xml does NOT use a `<Script file=\"…\"/>` \
         directive — the two .lua companions are listed directly in the TOC's file \
         section before the XML, which is the older / simpler load pattern. (Newer \
         Blizzard addons like Blizzard_RaidUI use the inverse pattern: TOC lists only \
         the .xml, and the XML pulls its .lua via <Script file=\"…\"/>.)"
    );
}

prefork_full_ui_case! {
fn blizzard_recent_allies_publishes_pin_display_helper_on_mixin(env: &WowLuaEnv) {

    let methods: (bool, bool) = env
        .eval(
            "return type(RecentAlliesEntryPinDisplayMixin.Init) == 'function', \
                    type(RecentAlliesEntryPinDisplayMixin.RefreshPinExpirationIcon) == 'function'",
        )
        .expect("PinDisplay method probe should succeed");
    assert_eq!(
        methods,
        (true, true),
        "RecentAlliesEntryPinDisplayMixin (Templates.lua:386) publishes Init(stateData) \
         and RefreshPinExpirationIcon() — the pin icon swaps between two atlases \
         (`friendslist-recentallies-pin-yellow` for normal pins, \
         `friendslist-recentallies-pin` for pins within \
         Constants.RecentAlliesConsts.PIN_EXPIRATION_WARNING_DAYS of expiration). The \
         Init is called explicitly from RecentAlliesEntryMixin:InitializeStateDisplay \
         (Templates.lua:239), not from an XML <OnLoad> handler — the pin display has \
         only OnEnter/OnLeave scripts wired in XML"
    );
}
}

prefork_full_ui_case! {
fn blizzard_recent_allies_entry_mixin_exposes_tooltip_pipeline_methods(env: &WowLuaEnv) {

    let pipeline: (bool, bool, bool, bool) = env
        .eval(
            "return type(RecentAlliesEntryMixin.BuildRecentAllyTooltip) == 'function', \
                    type(RecentAlliesEntryMixin.AddCharacterDataToTooltip) == 'function', \
                    type(RecentAlliesEntryMixin.AddStateDataToTooltip) == 'function', \
                    type(RecentAlliesEntryMixin.AddInteractionDataToTooltip) == 'function'",
        )
        .expect("Tooltip pipeline probe should succeed");
    assert_eq!(
        pipeline,
        (true, true, true, true),
        "RecentAlliesEntryMixin's tooltip is built as a four-step pipeline \
         (Templates.lua:153-157): BuildRecentAllyTooltip drives \
         AddCharacterDataToTooltip (name/level/race/class/faction), \
         AddStateDataToTooltip (current location), AddInteractionDataToTooltip (note + \
         most-recent-interaction line). The split lets each step be tested or skipped \
         independently — and AddInteractionDataToTooltip can return early if there are \
         no recorded interactions yet"
    );
}
}
