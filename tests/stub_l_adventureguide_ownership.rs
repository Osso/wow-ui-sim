//! Retail default-state PRESERVATION through the original XML microbutton.
//! No alternate native availability state or RED claim: current producers are constants.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::screen::ScreenKind;

#[test]
fn original_micromenu_preserves_adventureguide_is_available() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_name) in [
        ("Blizzard_FrameXMLUtil", "Blizzard_FrameXMLUtil.toc"),
        ("Blizzard_MicroMenu", "Blizzard_MicroMenu_Mainline.toc"),
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
    for name in ["Blizzard_FrameXMLUtil", "Blizzard_MicroMenu"] {
        assert!(
            env.state()
                .borrow()
                .addons
                .iter()
                .any(|a| a.folder_name == name && a.loaded),
            "{name} loaded normally"
        );
    }
    env.sync_string_metatable_to_global_string();
    eprintln!(
        "unfiltered startup diagnostics: {:?}",
        env.state().borrow().lua_errors
    );
    env.exec(r#"
        local function owner(fn)
            return debug.getinfo(fn, 'S').source:gsub('\\', '/')
        end
        assert(owner(AdventureGuideUtil.IsAvailable):find('Blizzard_FrameXMLUtil/AdventureGuideUtil.lua',1,true))
        assert(type(EJMicroButton)=='table')
        assert(owner(EJMicroButton.UpdateDisplay):find('Blizzard_MicroMenu/Mainline/MainMenuBarMicroButtons.lua',1,true))
        assert(EJMicroButton.UpdateDisplay == EJMicroButtonMixin.UpdateDisplay)
        assert(not EncounterJournal or not EncounterJournal:IsShown())
        -- Existing simulator defaults only: no fabricated provider or native-state claim.
        assert(not Kiosk.IsEnabled() and C_AdventureJournal.CanBeShown())
        local helper = AdventureGuideUtil.IsAvailable
        local consumer = EJMicroButton.UpdateDisplay
        for i=1,3 do
            EJMicroButton:Disable()
            assert(not EJMicroButton:IsEnabled())
            assert(AdventureGuideUtil.IsAvailable()==true)
            EJMicroButton:UpdateDisplay()
            assert(EJMicroButton:IsEnabled())
            assert(AdventureGuideUtil.IsAvailable==helper and EJMicroButton.UpdateDisplay==consumer)
        end
    "#).expect("original default-state MicroMenu consumer PRESERVATION");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "unfiltered errors: {:?}",
        env.state().borrow().lua_errors
    );
}
