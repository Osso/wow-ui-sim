#![cfg(feature = "client-wowforever")]

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn selected_deprecated_action_bar() -> PathBuf {
    let addons = wow_ui_sim::blizzard_ui_sync::cached_blizzard_ui_addons_path()
        .expect("a complete Forever source cache is required");
    let toc = addons.join("Blizzard_DeprecatedActionBar/Blizzard_DeprecatedActionBar.toc");
    assert!(
        discover_blizzard_addons_for_screen(&addons, ScreenKind::Game)
            .iter()
            .any(|(name, path)| name == "Blizzard_DeprecatedActionBar" && path == &toc),
        "the unchanged legacy wrapper must be selected for Game startup"
    );
    toc
}

#[test]
fn selected_legacy_range_wrapper_uses_supplied_true_false_and_unknown_observations() {
    let env = WowLuaEnv::new().unwrap();
    let toc = selected_deprecated_action_bar();
    // Test binaries already isolate CVar output; this explicitly selects the
    // source file's real deprecation-fallback branch, without changing that file.
    env.state()
        .borrow()
        .cvars
        .set("loadDeprecationFallbacks", "1");
    env.exec(
        "assert(type(rawget(C_ActionBar, 'IsActionInRange')) == 'function'); \
         assert(type(rawget(C_ActionBar, 'HasRangeRequirements')) == 'function'); \
         assert(type(rawget(C_ActionBar, 'GetActionAutocast')) == 'function')",
    )
    .unwrap();
    load_addon(&env.loader_env(), &toc).unwrap();
    env.state().borrow_mut().action_bars.insert(3, 19750);
    env.exec(
        "assert(type(IsActionInRange) == 'function'); \
         assert(select('#', IsActionInRange(3)) == 1); \
         assert(IsActionInRange(3) == nil); \
         assert(C_ActionBar.IsActionInRange(3, 'missing') == nil); \
         TargetUnit('player')",
    )
    .unwrap();
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    for supplied in [false, true] {
        env.state()
            .borrow_mut()
            .action_spell_ranges
            .entry(19750)
            .or_default()
            .in_range_by_guid
            .insert(guid.clone(), supplied);
        let actual: bool = env.eval("return IsActionInRange(3)").unwrap();
        assert_eq!(actual, supplied);
        let direct: bool = env
            .eval("return C_ActionBar.IsActionInRange(3, 'player')")
            .unwrap();
        assert_eq!(direct, supplied);
    }
    env.state().borrow_mut().action_bars.insert(3, 999999);
    env.exec("assert(IsActionInRange(3) == nil)").unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn range_requirements_and_pet_autocast_reuse_existing_backing_state() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.action_bars.insert(3, 19750);
        state
            .action_spell_ranges
            .entry(19750)
            .or_default()
            .has_range_requirements = true;
        let pet = &mut state.pet_actions[0];
        pet.has_action = true;
        pet.spell_id = Some(19750);
        pet.auto_cast_allowed = true;
        pet.auto_cast_enabled = true;
    }
    env.exec(
        "assert(C_ActionBar.HasRangeRequirements(3)); \
         local allowed, enabled = C_ActionBar.GetActionAutocast(3); \
         assert(allowed and enabled); \
         assert(not pcall(C_ActionBar.IsActionInRange, 0))",
    )
    .unwrap();
    env.state().borrow_mut().action_bars.insert(3, 999999);
    env.exec(
        "assert(not C_ActionBar.HasRangeRequirements(3)); \
         local allowed, enabled = C_ActionBar.GetActionAutocast(3); \
         assert(not allowed and not enabled)",
    )
    .unwrap();
}
