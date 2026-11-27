//! API-level coverage for binding contexts and creature IDs, which Retail
//! 12.1.5 and Forever 70707 declare with the same signatures:
//! KeyBindingsDocumentation.lua:11/:22/:118 and CreatureInfoDocumentation.lua:50.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const BINDING_CONTEXTS: &str = r#"
    assert(C_KeyBindings.IsBindingContextActive(3) == false)
    C_KeyBindings.ActivateBindingContext(3)
    C_KeyBindings.ActivateBindingContext(5)
    assert(C_KeyBindings.IsBindingContextActive(3) == true)
    assert(C_KeyBindings.IsBindingContextActive(5) == true)
    assert(C_KeyBindings.IsBindingContextActive(4) == false)
    C_KeyBindings.DeactivateBindingContext(3)
    assert(C_KeyBindings.IsBindingContextActive(3) == false)
    assert(C_KeyBindings.IsBindingContextActive(5) == true)
    assert(not pcall(C_KeyBindings.ActivateBindingContext, 10))
"#;

const CREATURE_IDS: &str = r#"
    assert(C_CreatureInfo.GetCreatureID("Creature-0-1-2-3-1234-0000000001") == 1234)
    assert(C_CreatureInfo.GetCreatureID("Creature-0-1-2-3-98765-0000000002") == 98765)
    assert(C_CreatureInfo.GetCreatureID("Player-1-00000001") == nil)
"#;

#[test]
fn binding_contexts_and_creature_ids_follow_their_inputs() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(BINDING_CONTEXTS).expect("binding contexts");
    env.exec(CREATURE_IDS).expect("creature IDs");
}
