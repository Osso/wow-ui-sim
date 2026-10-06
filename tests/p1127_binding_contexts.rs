#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1127_binding_contexts_activate_independently_and_deactivate_idempotently() {
    let env = WowLuaEnv::new().unwrap();
    let other = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_KeyBindings.IsBindingContextActive(1) == false)
        C_KeyBindings.ActivateBindingContext(1)
        C_KeyBindings.ActivateBindingContext(2)
        C_KeyBindings.ActivateBindingContext(1)
        assert(C_KeyBindings.IsBindingContextActive(1) == true)
        assert(C_KeyBindings.IsBindingContextActive(2) == true)
        C_KeyBindings.DeactivateBindingContext(1)
        C_KeyBindings.DeactivateBindingContext(1)
        assert(C_KeyBindings.IsBindingContextActive(1) == false)
        assert(C_KeyBindings.IsBindingContextActive(2) == true)
        assert(not pcall(C_KeyBindings.ActivateBindingContext, 10))
        assert(not pcall(C_KeyBindings.ActivateBindingContext, 1.5))
    "#).unwrap();
    other.exec("assert(C_KeyBindings.IsBindingContextActive(2) == false)").unwrap();
}
