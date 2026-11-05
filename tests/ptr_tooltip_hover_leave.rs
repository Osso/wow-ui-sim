//! Independent original-consumer RED for ActionButton.lua:1512.
#[cfg(feature = "client-ptr")]
mod tooltip_full_env_helpers;

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_original_action_button_leave_clears_supplied_visualization() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec("ActionButton5.action = 5").unwrap();
    let id = {
        let mut sim = env.state().borrow_mut();
        sim.spell_range_visualization.stored_slot = Some(4);
        sim.spell_range_visualization.value = 71;
        sim.spell_range_visualization.enabled = true;
        sim.lua_errors.clear();
        sim.widgets.get_id_by_name("ActionButton5").unwrap()
    };
    env.fire_script_handler(id, "OnLeave", vec![]).unwrap();
    let sim = env.state().borrow();
    assert!(
        sim.lua_errors.is_empty(),
        "original OnLeave errors: {:?}",
        sim.lua_errors
    );
    assert_eq!(sim.spell_range_visualization.stored_slot, Some(4));
    assert_eq!(sim.spell_range_visualization.value, 0);
    assert!(!sim.spell_range_visualization.enabled);
}
