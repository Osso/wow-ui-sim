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

fn supplied(url: &str, is_new: bool, can_show: bool) -> ExternalEventUrlState {
    ExternalEventUrlState {
        url: Some(url.to_owned()),
        is_new,
        can_show,
    }
}

#[test]
fn game_menu_click_launches_only_a_supplied_event_url() {
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
