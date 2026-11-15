//! J StoreUI guard retirement. Hidden/default escape is preservation;
//! original-function ownership is the only intended retirement RED.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common;
use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use wow_ui_sim::loader::{discover_blizzard_addon_closure_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn original_store_env() -> common::LockedEnv {
    common::lock_env(|| {
        let ui = blizzard_ui_dir();
        let toc = find_toc_file(&ui.join("Blizzard_StoreUI")).unwrap();
        assert!(toc.ends_with("Blizzard_StoreUI.toc"));
        let contents = std::fs::read_to_string(&toc).unwrap();
        assert!(contents.contains("## AllowLoad: Both"));
        assert!(contents.contains("## UseSecureEnvironment: 1"));
        assert!(
            contents.contains("Blizzard_Shared_StoreUIToggle.lua [AllowLoadGameType mainline]")
        );
        assert!(contents.contains("Blizzard_Shared_StoreUI.xml [AllowLoadGameType mainline]"));
        let xml = std::fs::read_to_string(ui.join("Blizzard_StoreUI/Blizzard_Shared_StoreUI.xml"))
            .unwrap();
        assert!(xml.contains("<Script file=\"Blizzard_Shared_StoreUIInbound.lua\""));
        assert!(xml.contains("<Script file=\"Blizzard_Shared_StoreUISecure.lua\""));
        assert!(xml.contains("<OnAttributeChanged function=\"StoreFrame_OnAttributeChanged\""));

        let env = WowLuaEnv::new().unwrap();
        env.set_screen_size(1024.0, 768.0);
        env.set_screen_mode(ScreenKind::Game);
        env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
        wow_ui_sim::xml::register_intrinsic_templates();
        let addons = discover_blizzard_addon_closure_for_screen(
            &ui,
            ScreenKind::Game,
            &["Blizzard_FrameXMLBase", "Blizzard_StoreUI"],
        );
        assert!(
            addons
                .iter()
                .any(|(name, path)| name == "Blizzard_StoreUI" && path == &toc)
        );
        for (name, path) in addons {
            load_addon(&env.loader_env(), &path)
                .unwrap_or_else(|error| panic!("[load {name}] {error}"));
        }
        env.apply_post_load_workarounds();
        env.exec(r#"
            assert(C_AddOns.IsAddOnLoaded('Blizzard_StoreUI'), 'actual StoreUI owner not loaded')
            assert(StoreFrame and ServicesLogoutPopup, 'original StoreUI XML frames missing')
            assert(StoreFrame:GetScript('OnAttributeChanged'), 'original XML handler not bound')
            local handler = StoreFrame_OnAttributeChanged or
                (__secureenv and rawget(__secureenv, 'StoreFrame_OnAttributeChanged'))
            assert(type(handler) == 'function', 'original secure handler missing')
            assert(debug.getinfo(handler, 'S').source:find('Blizzard_Shared_StoreUISecure.lua', 1, true),
                'secure handler must be original Lua')
            assert(debug.getinfo(StoreEscapePressed, 'S').source:find('Blizzard_Shared_StoreUIToggle.lua', 1, true),
                'original toggle/escape owner must be runtime-selected')
            assert(not StoreFrame:IsShown(), 'original store must start hidden')
            assert(not ServicesLogoutPopup:IsShown(), 'original logout popup must start hidden')
        "#).unwrap();
        env
    })
}

#[test]
fn store_escape_hidden_original_attribute_lifecycle_preservation() {
    test_timeout! {
        let env = original_store_env();
        env.exec(r#"
            -- Use the original inbound->attribute->secure XML callback route.
            -- No showing store, checkout, service request, or fabricated result.
            local handled = StoreFrame_EscapePressed()
            assert(handled == false, 'hidden default store must not handle escape')
            assert(StoreFrame:GetAttribute('action') == 'EscapePressed', 'inbound action not delivered')
            assert(StoreFrame:GetAttribute('escaperesult') == false, 'secure handler did not publish false')
            assert(not StoreFrame:IsShown(), 'escape unexpectedly showed store')
            assert(not ServicesLogoutPopup:IsShown(), 'escape unexpectedly showed service popup')
        "#).unwrap();
        for error in &env.state().borrow().lua_error_records {
            assert!(
                !error.message.contains("StoreFrame_OnAttributeChanged")
                    && !error.message.contains("Blizzard_Shared_StoreUISecure.lua"),
                "original secure StoreUI callback diagnostic: {}", error.message
            );
        }
    }
}

#[test]
fn store_escape_original_ownership_after_postload_retirement_red() {
    test_timeout! {
        let env = original_store_env();
        env.exec(r#"
            local source = debug.getinfo(StoreFrame_EscapePressed, 'S').source
            assert(source:find('Blizzard_Shared_StoreUIInbound.lua', 1, true),
                'original inbound escape ownership replaced by ' .. source)
            assert(rawget(_G, '__wow_store_frame_escape_wrapped') == nil,
                'retired escape guard marker must not be installed')
        "#).unwrap();
    }
}
