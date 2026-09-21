//! Real Forever consumers of the shared forbidden-aspect capability.
#![cfg(feature = "client-wowforever")]

const FIXTURE: &str = include_str!("fixtures/forever_forbidden_consumers.lua");

fn setup() -> wow_ui_sim::lua_api::WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &[
            "Blizzard_RestrictedAddOnEnvironment",
            "Blizzard_AuraContainer",
        ],
        &[],
    );
    env.exec(FIXTURE).unwrap();
    env
}

#[test]
fn forever_forbidden_consumers_secure_handler_accepts_plain_and_rejects_marked() {
    let env = setup();
    env.exec("ForeverForbiddenConsumers.secure_handlers()")
        .unwrap();
}

#[test]
fn forever_forbidden_consumers_tainted_aura_creation_and_dirty_lifecycle() {
    let env = setup();
    env.exec("ForeverForbiddenConsumers.aura_shell()").unwrap();
    env.fire_on_update(0.016).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec(
        "assert(ForbiddenConsumerDirtyCalls == 1); \
         assert(not GetForbiddenObjectTable(ForbiddenConsumerAura):IsDirty()); \
         assert(ForbiddenConsumerAura:GetOnUpdateMode() == Enum.OnUpdateMode.Disabled)",
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn forever_forbidden_consumers_native_masks_and_inheritance_rejections() {
    let env = setup();
    env.exec("ForeverForbiddenConsumers.enums_and_inheritance()")
        .unwrap();
}
