//! Modeled external-event promotion state through original Blizzard consumers.
//!
//! `C_ExternalEventURL` is declared by the pinned
//! `Blizzard_APIDocumentationGenerated/ExternalEventURLDocumentation.lua` on
//! Retail, PTR, Mists, Era and Anniversary. Wrath has no such declaration.

#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::c_api::external_event_url::ExternalEventUrlState;
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn supplied(url: &str, is_new: bool, can_show: bool) -> ExternalEventUrlState {
    ExternalEventUrlState {
        url: Some(url.to_owned()),
        is_new,
        can_show,
    }
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn load_game_menu() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let toc = TocFile::from_file(&find_toc_file(&ui.join("Blizzard_GameMenu")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(
        toc.files
            .iter()
            .any(|path| path.to_string_lossy().contains("GameMenuFrame.lua"))
    );
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_GameMenu"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_GameMenu"));
    env
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn modern_game_menu_click_launches_only_a_supplied_event_url() {
    let env = load_game_menu();
    env.exec(
        r#"
        local click = GameMenuFrameMixin.ExternalEventClickCallback
        local source = debug.getinfo(click, 'S').source:gsub('\\', '/')
        assert(source:find('Shared/GameMenuFrame.lua', 1, true))
        assert(not C_ExternalEventURL.HasURL())
        click(GameMenuFrameMixin)
        assert(select('#', C_ExternalEventURL.LaunchURL()) == 0)
    "#,
    )
    .expect("original game-menu click consumes the empty promotion state");
    assert_eq!(env.state().borrow().last_launched_url, None);

    env.state().borrow_mut().external_event_url =
        supplied("https://example.invalid/promotion", true, true);
    env.exec(
        r#"
        assert(C_ExternalEventURL.HasURL())
        local click = GameMenuFrameMixin.ExternalEventClickCallback
        click(GameMenuFrameMixin)
    "#,
    )
    .expect("original game-menu click launches the supplied promotion URL");
    assert_eq!(
        env.state().borrow().last_launched_url.as_deref(),
        Some("https://example.invalid/promotion")
    );

    env.state().borrow_mut().external_event_url.url = None;
    env.exec(
        r#"
        assert(not C_ExternalEventURL.HasURL())
        C_ExternalEventURL.LaunchURL()
    "#,
    )
    .expect("clearing the host URL removes the promotion");
    assert_eq!(
        env.state().borrow().last_launched_url.as_deref(),
        Some("https://example.invalid/promotion")
    );
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn declared_is_new_projects_host_newness() {
    let env = load_game_menu();
    for is_new in [true, false] {
        env.state().borrow_mut().external_event_url =
            supplied("https://example.invalid/promotion", is_new, false);
        let observed: bool = env
            .eval("return C_ExternalEventURL.IsNew()")
            .expect("declared IsNew returns one non-nil boolean");
        assert_eq!(observed, is_new);
    }
}

#[cfg(any(feature = "client-mists", feature = "client-era", feature = "client-anniversary"))]
fn load_full_game_ui() -> WowLuaEnv {
    use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
    use wow_ui_sim::startup::settle_headless_startup;

    let ui = blizzard_ui_dir();
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    crate::common::load_profile_framexml(&env);
    for (name, toc_path) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("Failed to load Blizzard addon {name}: {error}"));
    }
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

/// Mists, Era and Anniversary call `C_ExternalEventURL` from the local callback
/// built inside `GameMenuFrameMixin:InitButtons`, so the original game menu is
/// the consumer.
#[cfg(any(feature = "client-mists", feature = "client-era", feature = "client-anniversary"))]
#[test]
fn classic_game_menu_builds_the_external_event_button_from_host_state() {
    let env = load_full_game_ui();
    env.exec(
        r#"
        local source = debug.getinfo(GameMenuFrameMixin.InitButtons, 'S').source:gsub('\\', '/')
        assert(source:find('Shared/GameMenuFrame.lua', 1, true))
        local function activeButtons()
            local buttons = {}
            for button in GameMenuFrame.buttonPool:EnumerateActive() do
                buttons[#buttons + 1] = button
            end
            return buttons
        end
        local function firstButton()
            for _, button in ipairs(activeButtons()) do
                if button.layoutIndex == 1 then
                    return button
                end
            end
        end
        GameMenuFrame:InitButtons()
        baselineButtons = #activeButtons()
        -- The external event AddButton is the first call in the original method.
        assert(firstButton() ~= nil)
        assert(not C_ExternalEventURL.HasURL())
        assert(not GameMenuFrame.NewExternalEventFrame:IsShown())
    "#,
    )
    .expect("original game menu omits the external event button without a URL");

    env.state().borrow_mut().external_event_url =
        supplied("https://example.invalid/promotion", true, false);
    env.exec(
        r#"
        local function activeButtons()
            local buttons = {}
            for button in GameMenuFrame.buttonPool:EnumerateActive() do
                buttons[#buttons + 1] = button
            end
            return buttons
        end
        local function firstButton()
            for _, button in ipairs(activeButtons()) do
                if button.layoutIndex == 1 then
                    return button
                end
            end
        end
        GameMenuFrame:InitButtons()
        assert(#activeButtons() == baselineButtons + 1)
        local button = firstButton()
        assert(button, "original game menu adds the supplied external event button")
        if type(GAMEMENU_EXTERNALEVENT) == "string" then
            assert(button:GetText() == GAMEMENU_EXTERNALEVENT)
        end
        assert(GameMenuFrame.NewExternalEventFrame:IsShown())
        local onclick = button:GetScript("OnClick")
        assert(type(onclick) == "function")
        onclick(button)
    "#,
    )
    .expect("original game menu launches the supplied promotion URL");
    assert_eq!(
        env.state().borrow().last_launched_url.as_deref(),
        Some("https://example.invalid/promotion")
    );

    env.state().borrow_mut().external_event_url.is_new = false;
    env.exec(
        r#"
        GameMenuFrame:InitButtons()
        assert(not GameMenuFrame.NewExternalEventFrame:IsShown())
    "#,
    )
    .expect("original game menu hides the new badge for a seen promotion");
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_minimap_button_projects_visibility_and_newness() {
    let ui = blizzard_ui_dir();
    let toc = TocFile::from_file(&find_toc_file(&ui.join("Blizzard_Minimap")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(toc.files.iter().any(|path| {
        path.to_string_lossy()
            .contains("ExternalEventURLMinimapButton.lua")
    }));
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_Minimap"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_Minimap"));

    env.state().borrow_mut().external_event_url =
        supplied("https://example.invalid/promotion", true, false);
    env.exec(
        r#"
        local refresh = ExternalEventURLMinimapButtonMixin.Refresh
        local source = debug.getinfo(refresh, 'S').source:gsub('\\', '/')
        assert(source:find('Shared/ExternalEventURLMinimapButton.lua', 1, true))
        local button = {shown=nil, effect=nil,
            SetShown=function(self, value) self.shown = value end,
            SetEffectShown=function(self, value) self.effect = value end,
            IsShown=function(self) return self.shown end,
            UpdateTransform=function() end}
        refresh(button)
        assert(not button.shown and not button.effect)
    "#,
    )
    .expect("original minimap refresh hides a promotion the host does not show");

    env.state()
        .borrow_mut()
        .external_event_url
        .can_show = true;
    env.exec(
        r#"
        local button = {shown=nil, effect=nil,
            SetShown=function(self, value) self.shown = value end,
            SetEffectShown=function(self, value) self.effect = value end,
            IsShown=function(self) return self.shown end,
            UpdateTransform=function() end}
        ExternalEventURLMinimapButtonMixin.Refresh(button)
        assert(button.shown and button.effect)
    "#,
    )
    .expect("original minimap refresh shows a new host promotion");
}
