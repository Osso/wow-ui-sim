//! G3: supplied spell usability through unchanged cached public Lua.
//! Wrath FrameXML/ActionButton.lua:313-328 (colors), :394-395 (event).
//! Retail Blizzard_DeprecatedActionBar/Deprecated_ActionBar.lua:4-6,56-58.
#![cfg(any(feature = "client-wrath", feature = "client-retail"))]

use crate::common;
use wow_ui_sim::c_api::action_usability::ActionUsability;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_cached_lua(env: &WowLuaEnv, relative: &str) {
    // Cross-profile cache root: this fixture loads Wrath and Retail sources.
    let cache = common::blizzard_ui_cache_root();
    let path = cache.join(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required original consumer {}: {error}", path.display()));
    env.exec(&source)
        .unwrap_or_else(|error| panic!("load unchanged {}: {error}", path.display()));
}

fn consumer_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    // Full original file, not an extracted/reconstructed function.
    load_cached_lua(&env, "wrath/FrameXML/ActionButton.lua");
    #[cfg(feature = "client-retail")]
    {
        env.exec("assert(GetCVarBool('loadDeprecationFallbacks'), 'wrapper must be enabled')")
            .unwrap();
        load_cached_lua(
            &env,
            "retail/AddOns/Blizzard_DeprecatedActionBar/Deprecated_ActionBar.lua",
        );
    }
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.action_bars.clear();
        sim.action_macros.clear();
        sim.action_outfits.clear();
        sim.action_bars.insert(1, 19750);
        sim.action_bars.insert(2, 19750);
        sim.action_bars.insert(3, 635);
        sim.action_macros.insert(4, 1);
        sim.action_outfits.insert(5, 91);
    }
    env.exec(
        r#"
        G3Button = CreateFrame('CheckButton', 'G3UsabilityButton', UIParent)
        G3UsabilityButtonIcon = G3Button:CreateTexture('G3UsabilityButtonIcon')
        G3UsabilityButtonNormalTexture = G3Button:CreateTexture('G3UsabilityButtonNormalTexture')
        function G3AssertColor(texture, r, g, b)
            local ar, ag, ab = texture:GetVertexColor()
            assert(math.abs(ar-r) < 0.00001, 'red channel')
            assert(math.abs(ag-g) < 0.00001, 'green channel')
            assert(math.abs(ab-b) < 0.00001, 'blue channel')
        end
        function G3AssertOutcome(slot, usable, power, r, g, b, nr, ng, nb)
            G3Button.action = slot
            ActionButton_OnEvent(G3Button, 'ACTIONBAR_UPDATE_USABLE')
            G3AssertColor(G3UsabilityButtonIcon, r, g, b)
            G3AssertColor(G3UsabilityButtonNormalTexture, nr, ng, nb)
            local u, p = IsUsableAction(slot)
            assert(u == usable and p == power, 'legacy usability pair')
            local cu, cp = C_ActionBar.IsUsableAction(slot)
            assert(cu == usable and cp == power, 'namespace usability pair')
        end
        "#,
    )
    .unwrap();
    env
}

#[test]
fn supplied_usability_drives_original_button_white_blue_gray_white() {
    let env = consumer_env();
    env.exec("G3AssertOutcome(1, true, false, 1,1,1, 1,1,1)")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_action_usability
        .insert(19750, ActionUsability::InsufficientPower);
    // Predicted behavioral RED: baseline reports true,false and leaves icon white.
    env.exec("G3AssertOutcome(1, false, true, 0.5,0.5,1, 0.5,0.5,1)")
        .unwrap();
    env.exec("G3AssertOutcome(2, false, true, 0.5,0.5,1, 0.5,0.5,1)")
        .unwrap();
    env.exec("G3AssertOutcome(3, true, false, 1,1,1, 1,1,1)")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_action_usability
        .insert(19750, ActionUsability::Unusable);
    env.exec("G3AssertOutcome(1, false, false, 0.4,0.4,0.4, 1,1,1)")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_action_usability
        .insert(19750, ActionUsability::Usable);
    env.exec("G3AssertOutcome(1, true, false, 1,1,1, 1,1,1)")
        .unwrap();
}

#[test]
fn supplied_spell_usability_does_not_leak_to_empty_or_rebound_slots() {
    let env = consumer_env();
    env.state()
        .borrow_mut()
        .spell_action_usability
        .insert(19750, ActionUsability::InsufficientPower);
    env.exec("G3AssertOutcome(1, false, true, 0.5,0.5,1, 0.5,0.5,1)")
        .unwrap();
    env.exec("G3AssertOutcome(4, true, false, 1,1,1, 1,1,1)")
        .unwrap();
    env.exec("G3AssertOutcome(5, true, false, 1,1,1, 1,1,1)")
        .unwrap();
    env.state().borrow_mut().action_bars.remove(&1);
    env.exec("G3AssertOutcome(1, false, false, 0.4,0.4,0.4, 1,1,1)")
        .unwrap();
    env.state().borrow_mut().action_bars.insert(1, 635);
    env.exec("G3AssertOutcome(1, true, false, 1,1,1, 1,1,1)")
        .unwrap();
}
