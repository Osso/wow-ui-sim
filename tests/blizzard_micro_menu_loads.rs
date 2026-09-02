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

fn micro_menu_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_MicroMenu")
}

fn micro_menu_toc() -> PathBuf {
    micro_menu_dir().join("Blizzard_MicroMenu_Mainline.toc")
}

const MICRO_MENU_TOC_FILES: &[&str] = &[
    "Shared/MicroMenuUtil.lua",
    "Mainline/MainMenuBarMicroButtons.lua",
    "Mainline/MainMenuBarMicroButtons.xml",
    "Shared/MicroMenuContainer.lua",
    "Mainline/MicroMenuContainerOverrides.lua",
    "Mainline/MicroMenuContainer.xml",
];

const BUTTON_INSTANCES: &[&str] = &[
    "CharacterMicroButton",
    "ProfessionMicroButton",
    "PlayerSpellsMicroButton",
    "AchievementMicroButton",
    "QuestLogMicroButton",
    "HousingMicroButton",
    "GuildMicroButton",
    "LFDMicroButton",
    "CollectionsMicroButton",
    "EJMicroButton",
    "HelpMicroButton",
    "StoreMicroButton",
    "MainMenuMicroButton",
];

const BUTTON_MIXINS: &[&str] = &[
    "MainMenuBarMicroButtonMixin",
    "CharacterMicroButtonMixin",
    "ProfessionMicroButtonMixin",
    "PlayerSpellsMicroButtonMixin",
    "AchievementMicroButtonMixin",
    "QuestLogMicroButtonMixin",
    "HousingMicroButtonMixin",
    "GuildMicroButtonMixin",
    "LFDMicroButtonMixin",
    "CollectionMicroButtonMixin",
    "EJMicroButtonMixin",
    "StoreMicroButtonMixin",
    "HelpMicroButtonMixin",
    "MainMenuMicroButtonMixin",
];

const GLOBAL_HELPERS: &[&str] = &[
    "LoadMicroButtonTextures",
    "MicroButtonTooltipText",
    "SetKioskTooltip",
    "MicroMenuBar_SetFullScreenFrame",
    "MicroMenuBar_ClearFullScreenFrame",
    "UpdateMicroButtons",
    "MicroButtonPulse",
    "MicroButtonPulseStop",
    "MainMenuMicroButton_Init",
    "MainMenuMicroButton_SetAlertsEnabled",
    "MainMenuMicroButton_UpdateAlertsEnabled",
    "MainMenuMicroButton_AreAlertsEnabled",
    "MainMenuMicroButton_GetAlertPriority",
    "MainMenuMicroButton_ShowAlert",
    "MainMenuMicroButton_HideAlert",
    "CollectionsMicroButton_SetAlert",
    "CollectionsMicroButton_SetAlertShown",
];

const CONTAINER_MIXIN_METHODS: &[&str] = &["OnLoad", "OnEvent", "Layout", "GetPosition"];

const MENU_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "GenerateButtonInfos",
    "ApplyMicroMenuOverrides",
    "InitializeButtons",
    "AddButton",
    "GetEdgeButton",
    "UpdateHelpTicketButtonAnchor",
    "UpdateQueueStatusAnchors",
    "UpdateFramerateFrameAnchor",
    "AnchorToMenuContainer",
    "SetQueueStatusScale",
    "Layout",
    "UpdateScale",
    "SetNormalScale",
    "SetOverrideScale",
    "ClearOverrideScale",
    "ResetMicroMenuPosition",
    "OverrideMicroMenuPosition",
];

const VIRTUAL_TEMPLATES_THAT_MUST_NOT_LEAK: &[&str] = &[
    "MainMenuBarMicroButtonNotificationTemplate",
    "MainMenuBarMicroButton",
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
fn blizzard_micro_menu_find_toc_resolves_mainline_variant() {
    let resolved = find_toc_file(&micro_menu_dir()).expect("Blizzard_MicroMenu TOC should resolve");
    assert_eq!(
        resolved,
        micro_menu_toc(),
        "Blizzard_MicroMenu ships a `_Mainline.toc` suffix variant alongside Classic variants.          `find_toc_file` (src/loader/mod.rs:65) prefers the `_Mainline.toc` suffix on retail          game-type discovery, so `Blizzard_MicroMenu_Mainline.toc` takes precedence over any          bare or other-suffixed TOC"
    );
}

#[test]
fn blizzard_micro_menu_toc_declares_eager_game_screen_load_with_communities_dependency() {
    let toc = TocFile::from_file(&micro_menu_toc()).expect("Blizzard_MicroMenu TOC parses");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_MicroMenu does NOT declare `## LoadOnDemand: 1` — the micro-menu bar is part          of the always-present Game screen HUD and must auto-load on Game screen discovery.          `Blizzard_ActionBar` declares `## Dependencies: Blizzard_StoreUI, Blizzard_QuickKeybind,          Blizzard_UIParent, Blizzard_EditMode, Blizzard_UIPanels_Game, Blizzard_TextStatusBar,          Blizzard_Flyout, Blizzard_Colors, Blizzard_HelpPlate, Blizzard_MicroMenu` —          Blizzard_MicroMenu must be loaded eagerly so the ActionBar dependency chain resolves"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_MicroMenu does NOT declare `## UseSecureEnvironment` — runs in the standard          Lua environment; the micro-menu buttons are public-facing UI and do not require the          restricted secure environment used by protected action bars"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_MicroMenu declares `## AllowLoadGameType: mainline` but          src/toc.rs:299 treats `mainline` / `standard` as the unrestricted retail game type,          so `is_game_type_restricted()` returns false"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_Communities".to_string()],
        "Retail TOC declares `## Dependencies: Blizzard_Communities`"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_MicroMenu declares NO `## SavedVariables*` — micro-menu layout and alert          state are driven by EditMode (CVar-persisted) and server-side game rules, not          per-session saved-variable files"
    );
}

#[test]
fn blizzard_micro_menu_toc_allow_load_game_restricts_to_game_screen() {
    let toc = TocFile::from_file(&micro_menu_toc()).expect("Blizzard_MicroMenu TOC parses");
    let raw = std::fs::read_to_string(micro_menu_toc()).expect("Blizzard_MicroMenu TOC reads");
    assert!(
        raw.contains("## AllowLoad: Game"),
        "Blizzard_MicroMenu declares `## AllowLoad: Game` — the HUD micro-menu bar is          in-world UI only; it is absent from Login and CharacterSelect glue screens"
    );
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_MicroMenu must allow the Game screen — the micro-menu bar is a          persistent Game-HUD element"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_MicroMenu must NOT allow {screen:?} — `## AllowLoad: Game` explicitly              excludes all glue screens from the auto-discovery pass"
        );
    }
}

#[test]
fn blizzard_micro_menu_toc_lists_six_files_in_shared_and_mainline_subdirs() {
    let toc = TocFile::from_file(&micro_menu_toc()).expect("Blizzard_MicroMenu TOC parses");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files, MICRO_MENU_TOC_FILES,
        "Blizzard_MicroMenu TOC body must list exactly 6 files across two subdirectories:          Shared/ (MicroMenuUtil.lua, MicroMenuContainer.lua) contains the game-type-agnostic          mixin foundations shared with Classic variants; Mainline/ (MainMenuBarMicroButtons.lua,          MainMenuBarMicroButtons.xml, MicroMenuContainerOverrides.lua, MicroMenuContainer.xml)          contains retail-specific button definitions, overrides, and the frame declarations          for MicroMenu / MicroMenuContainer"
    );
}

#[test]
fn blizzard_micro_menu_auto_loads_on_game_screen_absent_from_login() {
    let ui = blizzard_ui_dir();
    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let login_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Login);

    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_MicroMenu");
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_MicroMenu");

    assert!(
        in_game,
        "Blizzard_MicroMenu must appear in Game-screen auto-discovery — `## AllowLoad: Game`          with eager loading means it is included in the Game-screen addon set"
    );
    assert!(
        !in_login,
        "Blizzard_MicroMenu must NOT appear in Login-screen auto-discovery — the micro-menu          bar is in-world UI only"
    );
}

prefork_full_ui_case! {
fn blizzard_micro_menu_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    let state = env.state().borrow();
    let errors: Vec<&String> = state
        .lua_errors
        .iter()
        .filter(|e| {
            let lc = e.to_lowercase();
            lc.contains("micromenu")
                || lc.contains("microbutton")
                || lc.contains("microbuttonandbagsbar")
                || lc.contains("mainmenumicrobutton")
                || lc.contains("charactermicro")
                || lc.contains("professionmicro")
                || lc.contains("guildmicro")
                || lc.contains("collectionmicro")
                || lc.contains("ejmicro")
                || lc.contains("storemicro")
                || lc.contains("helpmicro")
        })
        .collect();
    assert!(
        errors.is_empty(),
        "Blizzard_MicroMenu load produced {} addon-specific Lua error(s): {:?}",
        errors.len(),
        errors
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_is_addon_loaded_after_game_startup(env: &WowLuaEnv) {
    let loaded: bool = env
        .eval("return IsAddOnLoaded('Blizzard_MicroMenu')")
        .expect("IsAddOnLoaded eval");
    assert!(
        loaded,
        "IsAddOnLoaded('Blizzard_MicroMenu') must return true after full Game-screen startup"
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_util_publishes_table_and_three_factory_functions(env: &WowLuaEnv) {
    let result: String = env
        .eval(
            r#"
            if type(MicroMenuUtil) ~= "table" then
                return "MicroMenuUtil not a table: " .. type(MicroMenuUtil)
            end
            for _, name in ipairs({"GenerateButtonInfo", "GenerateButtonGameRuleInfo", "GenerateButtonCallbackInfo"}) do
                if type(MicroMenuUtil[name]) ~= "function" then
                    return name .. " not a function: " .. type(MicroMenuUtil[name])
                end
            end
            -- Validate GenerateButtonInfo returns expected shape
            local btn = {}; local info = MicroMenuUtil.GenerateButtonInfo(btn, 42, nil)
            if info.button ~= btn then return "button field mismatch" end
            if info.gameRule ~= 42 then return "gameRule field mismatch" end
            return "ok"
            "#,
        )
        .expect("MicroMenuUtil eval");
    assert_eq!(
        result, "ok",
        "MicroMenuUtil should publish as a table with three factory functions: {result}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_displayed_communities_invitations_is_table(env: &WowLuaEnv) {
    let result: String = env
        .eval(
            r#"
            if type(DISPLAYED_COMMUNITIES_INVITATIONS) ~= "table" then
                return "not table: " .. type(DISPLAYED_COMMUNITIES_INVITATIONS)
            end
            return "ok"
            "#,
        )
        .expect("DISPLAYED_COMMUNITIES_INVITATIONS eval");
    assert_eq!(
        result, "ok",
        "DISPLAYED_COMMUNITIES_INVITATIONS must publish as a table (initialized to empty at          module load time; GuildMicroButtonMixin.MarkCommunitiesInvitiationDisplayed writes to it          to track which community invitation popups have been shown in the current session): {result}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_position_enum_has_four_cardinal_values(env: &WowLuaEnv) {
    let result: String = env
        .eval(
            r#"
            if type(MicroMenuPositionEnum) ~= "table" then
                return "not table: " .. type(MicroMenuPositionEnum)
            end
            local expected = {BottomLeft=1, BottomRight=2, TopLeft=3, TopRight=4}
            for k, v in pairs(expected) do
                if MicroMenuPositionEnum[k] ~= v then
                    return k .. "=" .. tostring(MicroMenuPositionEnum[k]) .. " want " .. v
                end
            end
            return "ok"
            "#,
        )
        .expect("MicroMenuPositionEnum eval");
    assert_eq!(
        result, "ok",
        "MicroMenuPositionEnum must publish with 4 ordinal values (BottomLeft=1, BottomRight=2,          TopLeft=3, TopRight=4); these are consumed by FramerateFrame:UpdatePosition,          MicroMenuMixin:AnchorToMenuContainer, and UpdateHelpTicketButtonAnchor to decide          which corner of the screen the micro-menu is currently docked to: {result}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_all_thirteen_button_instances_are_tables(env: &WowLuaEnv) {
    let errors: Vec<String> = BUTTON_INSTANCES
        .iter()
        .filter_map(|name| {
            let code = format!("return type({name})");
            let ty: String = env.eval(&code).unwrap_or_else(|_| "error".to_string());
            if ty != "table" {
                Some(format!("{name}={ty}"))
            } else {
                None
            }
        })
        .collect();
    assert!(
        errors.is_empty(),
        "All 13 micro-menu button instances must publish as tables (frames/buttons in Lua are          tables); failed: {:?}. CharacterMicroButton…MainMenuMicroButton are created as          non-virtual Button inheriting MainMenuBarMicroButton in MainMenuBarMicroButtons.xml",
        errors
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_help_micro_button_is_hidden_at_load(env: &WowLuaEnv) {
    let hidden: bool = env
        .eval(
            r#"
            return HelpMicroButton ~= nil and HelpMicroButton:IsShown() == false
            "#,
        )
        .expect("HelpMicroButton:IsShown eval");
    assert!(
        hidden,
        "HelpMicroButton is declared with `hidden=\"true\"` in MainMenuBarMicroButtons.xml —          it starts hidden at load time and is conditionally shown based on          HelpMicroButtonMixin:OnLoad (registers OPEN_MASTER_LOOT_LIST, GROUP_ROSTER_UPDATE,          PARTY_LEADER_CHANGED) + game-rule evaluation"
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_virtual_templates_do_not_leak_as_globals(env: &WowLuaEnv) {
    let leaks: Vec<String> = VIRTUAL_TEMPLATES_THAT_MUST_NOT_LEAK
        .iter()
        .filter_map(|name| {
            let ty: String = env
                .eval(&format!("return type(_G[{name:?}])"))
                .unwrap_or_else(|_| "error".to_string());
            if ty != "nil" {
                Some(format!("{name}={ty}"))
            } else {
                None
            }
        })
        .collect();
    assert!(
        leaks.is_empty(),
        "Virtual XML templates must NOT leak as `_G.*` globals — `virtual=\"true\"` in XML          means the template is registered in the frame template registry only, not assigned          to a global name. Leaked: {:?}. Note: `MainMenuBarMicroButton` is the VIRTUAL base          button template; `MainMenuMicroButton` (the concrete button) IS a global and is expected",
        leaks
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_button_mixins_all_publish_as_tables(env: &WowLuaEnv) {
    let errors: Vec<String> = BUTTON_MIXINS
        .iter()
        .filter_map(|name| {
            let ty: String = env
                .eval(&format!("return type({name})"))
                .unwrap_or_else(|_| "error".to_string());
            if ty != "table" {
                Some(format!("{name}={ty}"))
            } else {
                None
            }
        })
        .collect();
    assert!(
        errors.is_empty(),
        "All 14 micro-button mixin tables must publish as tables in _G — each is defined as          `XxxMixin = {{}}` in MainMenuBarMicroButtons.lua and then populated with colon-methods.          Note: CollectionMicroButtonMixin (no s) is the Lua mixin, while CollectionsMicroButton          (with s) is the XML button instance. Failed: {:?}",
        errors
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_container_and_menu_frames_have_correct_parents(env: &WowLuaEnv) {
    let result: String = env
        .eval(
            r#"
            if type(MicroButtonAndBagsBar) ~= "table" then
                return "MicroButtonAndBagsBar not table: " .. type(MicroButtonAndBagsBar)
            end
            if MicroButtonAndBagsBar:GetParent():GetName() ~= "UIParent" then
                return "MicroButtonAndBagsBar parent=" .. tostring(MicroButtonAndBagsBar:GetParent():GetName())
            end
            local bw, bh = MicroButtonAndBagsBar:GetSize()
            if math.abs(bw - 232) > 0.1 or math.abs(bh - 80) > 0.1 then
                return string.format("MicroButtonAndBagsBar size=%gx%g want 232x80", bw, bh)
            end
            if type(MicroMenuContainer) ~= "table" then
                return "MicroMenuContainer not table: " .. type(MicroMenuContainer)
            end
            if MicroMenuContainer:GetParent():GetName() ~= "UIParent" then
                return "MicroMenuContainer parent=" .. tostring(MicroMenuContainer:GetParent():GetName())
            end
            if type(MicroMenu) ~= "table" then
                return "MicroMenu not table: " .. type(MicroMenu)
            end
            if MicroMenu:GetParent():GetName() ~= "MicroMenuContainer" then
                return "MicroMenu parent=" .. tostring(MicroMenu:GetParent():GetName())
            end
            return "ok"
            "#,
        )
        .expect("frame parent/size eval");
    assert_eq!(
        result, "ok",
        "Frame hierarchy: MicroButtonAndBagsBar(UIParent, 232x80) → MicroMenuContainer(UIParent)          → MicroMenu(MicroMenuContainer). MicroButtonAndBagsBar is the positional anchor for          the container. MicroMenuContainer inherits EditModeMicroMenuSystemTemplate so EditMode          can drag the entire micro-menu group. MicroMenu inherits GridLayoutFrame and holds the          individual buttons: {result}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_global_helpers_publish_as_functions(env: &WowLuaEnv) {
    let errors: Vec<String> = GLOBAL_HELPERS
        .iter()
        .filter_map(|name| {
            let ty: String = env
                .eval(&format!("return type({name})"))
                .unwrap_or_else(|_| "error".to_string());
            if ty != "function" {
                Some(format!("{name}={ty}"))
            } else {
                None
            }
        })
        .collect();
    assert!(
        errors.is_empty(),
        "All 17 global helper functions must publish as functions in _G. These are the          addon-level (non-mixin) public APIs: UpdateMicroButtons drives button enable/disable          state; MainMenuMicroButton_ShowAlert / HideAlert drive the alert-pulse system;          MicroButtonPulse / Stop drive the CSS-like keyframe flash animation. Failed: {:?}",
        errors
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_container_mixin_methods_are_functions(env: &WowLuaEnv) {
    let errors: Vec<String> = CONTAINER_MIXIN_METHODS
        .iter()
        .filter_map(|method| {
            let ty: String = env
                .eval(&format!("return type(MicroMenuContainerMixin.{method})"))
                .unwrap_or_else(|_| "error".to_string());
            if ty != "function" {
                Some(format!("{method}={ty}"))
            } else {
                None
            }
        })
        .collect();
    assert!(
        errors.is_empty(),
        "MicroMenuContainerMixin must publish 4 methods as functions: OnLoad (registers          PLAYER_LEVEL_UP + TRIAL_STATUS_UPDATE), OnEvent (calls UpdateMicroButtons),          Layout (custom GridLayout that accounts for hidden buttons and the QueueStatusButton          tacked on the side), GetPosition (maps screen quadrant to MicroMenuPositionEnum).          Failed: {:?}",
        errors
    );
}
}

prefork_full_ui_case! {
fn blizzard_micro_menu_mixin_methods_are_functions(env: &WowLuaEnv) {
    let errors: Vec<String> = MENU_MIXIN_METHODS
        .iter()
        .filter_map(|method| {
            let ty: String = env
                .eval(&format!("return type(MicroMenuMixin.{method})"))
                .unwrap_or_else(|_| "error".to_string());
            if ty != "function" {
                Some(format!("{method}={ty}"))
            } else {
                None
            }
        })
        .collect();
    assert!(
        errors.is_empty(),
        "MicroMenuMixin must publish 18 methods as functions: GenerateButtonInfos (overridden by          MicroMenuContainerOverrides.lua to return the 13 retail button infos),          InitializeButtons (iterates the infos and calls AddButton for each one not gated by a          disabled GameRule), ResetMicroMenuPosition / OverrideMicroMenuPosition (used by          EditMode and the Override Action Bar to reposition the menu). Failed: {:?}",
        errors
    );
}
}
