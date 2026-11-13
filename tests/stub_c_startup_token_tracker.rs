//! Retail original-consumer coverage through the application startup loader.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::screen::ScreenKind;

#[test]
fn original_startup_token_tracker_observes_template_and_width_changes() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_name) in [
        (
            "Blizzard_UIPanels_Game",
            "Blizzard_UIPanels_Game_Mainline.toc",
        ),
        ("Blizzard_TokenUI", "Blizzard_TokenUI.toc"),
    ] {
        let addon = startup
            .iter()
            .find(|a| a.name == name)
            .expect("selected game addon");
        assert_eq!(addon.toc_path.file_name().unwrap().to_str(), Some(toc_name));
    }
    // Match wow_sim/addon_loading.rs: register LoadWith, normal startup
    // discovery/kinds/order, skip already-loaded followers, restore cleanup.
    // No manual events, post-load workaround, source override or fake frame.
    for addon in startup {
        let loaded = env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|a| a.folder_name == addon.name && a.loaded);
        if !loaded {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|e| panic!("startup {}: {e}", addon.name));
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    eprintln!(
        "unfiltered startup diagnostics: {:?}",
        env.state().borrow().lua_errors
    );
    env.exec(
        r#"
        local function original(fn, path)
            assert(type(fn) == 'function', path .. ' missing')
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(path, 1, true), source)
        end
        original(ContainerFrameSettingsManager.OnAddonLoaded,
            'Blizzard_UIPanels_Game/Mainline/ContainerFrame.lua')
        original(BackpackTokenFrameMixin.GetMaxTokensWatched,
            'Blizzard_TokenUI/Blizzard_TokenUI.lua')
        local tracker = ContainerFrameSettingsManager.TokenTracker
        assert(tracker and tracker == BackpackTokenFrame, 'original startup tracker publication')
        assert(tracker.Border and tracker.tokenPool, 'XML child and original OnLoad pool')
        assert(tracker:GetScript('OnLoad'), 'XML OnLoad binding')
        assert(tracker.GetMaxTokensWatched == BackpackTokenFrameMixin.GetMaxTokensWatched,
            'original installed capacity consumer')
        local template = C_XMLUtil.GetTemplateInfo('BackpackTokenTemplate')
        assert(template and template.width == 50, 'pinned original XML width')
        for _, slots in ipairs({2, 5, 2}) do
            tracker:SetWidth(template.width * slots)
            assert(tracker:GetMaxTokensWatched() == slots, 'original width-dependent capacity')
        end
        assert(tracker.tokenWidth == template.width, 'original cached template width')
        assert(ContainerFrameSettingsManager.TokenTracker == tracker, 'stable tracker identity')
    "#,
    )
    .expect("original selected TokenUI consumer without post-load repair");
}
