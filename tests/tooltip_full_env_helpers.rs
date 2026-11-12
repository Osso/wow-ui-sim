// Shared test helpers. Individual test binaries that `mod` this file use
// a subset of the helpers, so per-binary dead_code warnings are expected.
#![allow(dead_code)]

use std::path::Path;

use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "common/event_helpers.rs"]
mod event_helpers;

const TOOLTIP_TEST_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLBase",
    "Blizzard_LoadLocale",
    "Blizzard_Fonts_Shared",
    "Blizzard_HelpPlate",
    "Blizzard_Menu",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_StaticPopup",
    "Blizzard_GameTooltip",
    "Blizzard_StaticPopup_Game",
    "Blizzard_TransmogShared",
    "Blizzard_GameMenuEsc",
    "Blizzard_Communities",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_EditMode",
    "Blizzard_Minimap",
    "Blizzard_BuffFrame",
    "Blizzard_GarrisonBase",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_CharacterFrame",
    "Blizzard_UIPanels_Game",
    "Blizzard_TokenUI",
    "Blizzard_ActionBar",
];

pub fn setup_full_env() -> WowLuaEnv {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_size(1024.0, 768.0);
        env.set_screen_mode(wow_ui_sim::screen::ScreenKind::Game);
        let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
        env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
        crate::common::load_profile_framexml(&env);
        for addon in wow_ui_sim::loader::discover_blizzard_startup_addons_for_screen(
            &ui,
            wow_ui_sim::screen::ScreenKind::Game,
        ) {
            wow_ui_sim::loader::load_startup_addon(
                &env.loader_env(),
                &addon.toc_path,
                addon.kind,
                None,
            )
            .expect("original Wrath startup addon should load");
        }
        env.apply_post_load_workarounds();
        wow_ui_sim::startup::fire_startup_events(&env);
        return env;
    }
    let env = load_full_env_without_post_load();
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    wow_ui_sim::startup::process_pending_timers(&env);
    wow_ui_sim::startup::run_extra_update_ticks(&env, 3);
    env
}

pub fn load_full_env_without_post_load() -> WowLuaEnv {
    load_full_env_without_post_load_with_addon_hook(|_, _| {})
}

pub fn load_full_env_without_post_load_with_addon_hook(
    mut before_addon_load: impl FnMut(&WowLuaEnv, &str),
) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1024.0, 768.0);

    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];

    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::WowForever {
        crate::common::load_game_startup_inventory(&env);
    } else {
        load_blizzard_addons_with_hook(&env, &ui, &mut before_addon_load);
    }
    env.exec(
        r#"
        if type(UIPanelWindows) == "table" and type(UIPanelWindows_Initialize) == "function" then
            UIPanelWindows_Initialize()
        end
        "#,
    )
    .expect("refresh UIPanelWindows after tooltip fixture load order");
    env
}

fn load_blizzard_addons_with_hook(
    env: &WowLuaEnv,
    ui: &Path,
    before_addon_load: &mut impl FnMut(&WowLuaEnv, &str),
) {
    for name in TOOLTIP_TEST_ADDONS {
        let addon_dir = ui.join(name);
        // `find_toc_file` applies `active_profile_toc_suffixes()`. Classic
        // caches also ship some Retail `_Mainline.toc` names without the
        // Mainline body they list, so forcing that name leaves the template
        // registry partially empty.
        let Some(toc_path) = find_toc_file(&addon_dir) else {
            continue;
        };
        before_addon_load(env, name);
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("load {name} from {}: {error}", toc_path.display()));
    }
}

fn fire_startup_events(env: &WowLuaEnv) {
    ensure_player_frame_for_aura_tests(env);

    fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }

    refresh_aura_frames(env);
}

fn fire_addon_loaded(env: &WowLuaEnv, addon_name: &str) {
    let _ = env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(addon_name)]);
}

use event_helpers::fire_player_entering_world;

fn ensure_player_frame_for_aura_tests(env: &WowLuaEnv) {
    env.exec(PLAYER_FRAME_STATUS_BAR_STUBS_LUA)
        .expect("Failed to create PlayerFrame stub for aura tests");
}

const PLAYER_FRAME_STATUS_BAR_STUBS_LUA: &str = r#"
    if not PlayerFrame then
        PlayerFrame = CreateFrame("Frame", "PlayerFrame", UIParent)
    end
    PlayerFrame.unit = "player"
    local function ensureStatusBar(name)
        local bar = _G[name] or CreateFrame("StatusBar", name, UIParent)
        function bar:ShowStatusBarText()
            self.statusBarTextShown = true
        end
        function bar:HideStatusBarText()
            self.statusBarTextShown = nil
        end
        return bar
    end
    PlayerFrameHealthBar = ensureStatusBar("PlayerFrameHealthBar")
    PlayerFrameManaBar = ensureStatusBar("PlayerFrameManaBar")
    PetFrameHealthBar = ensureStatusBar("PetFrameHealthBar")
    PetFrameManaBar = ensureStatusBar("PetFrameManaBar")
    function PlayerFrame_GetHealthBar()
        return PlayerFrameHealthBar
    end
    function PlayerFrame_GetManaBar()
        return PlayerFrameManaBar
    end
    function PlayerFrame_GetAlternatePowerBar()
        return nil
    end
    StatusTrackingBarManager = StatusTrackingBarManager or {}
    function StatusTrackingBarManager:SetTextLocked(locked)
        self.textLocked = locked
    end
"#;

pub fn refresh_aura_frames(env: &WowLuaEnv) {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        // Wrath FrameXML/BuffFrame.lua:80 exposes a global update function.
        env.exec("BuffFrame_Update()")
            .expect("Failed to refresh the original Wrath BuffFrame");
        return;
    }
    env.exec(
        r#"
        assert(BuffFrame, "BuffFrame should exist")
        assert(BuffFrame.UpdateAuras, "BuffFrame should expose UpdateAuras")
        local updateInfo = { isFullUpdate = true }
        if BuffFrame:IsEventRegistered("UNIT_AURA") and BuffFrame:GetScript("OnEvent") then
            BuffFrame:GetScript("OnEvent")(BuffFrame, "UNIT_AURA", "player", updateInfo)
        end
        BuffFrame:UpdateAuras()
        BuffFrame:Update()
        "#,
    )
    .expect("Failed to refresh BuffFrame after seeding buffs");
}
