//! Behavior pin: hovering a populated main action button routes through the
//! Blizzard ActionBar tooltip path without passing nil to `GameTooltip:SetAction`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use rilua::Val;

const ROOT: &str = "Blizzard_ActionBar";
const SLOT: u32 = 3;
const SPELL_ID: u32 = 853;

timerunning_util_panel_or_wrath_fixture! {
#[test]
fn action_button_on_enter_sets_tooltip_for_populated_slot() {
    wrath_action_case!(tooltip);

    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        env.state().borrow_mut().action_bars.insert(SLOT, SPELL_ID);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[Val::Num(SLOT as f64)])
            .expect("targeted ACTIONBAR_SLOT_CHANGED must dispatch cleanly");

        let button_id = env
            .state()
            .borrow()
            .widgets
            .get_id_by_name("ActionButton3")
            .expect("ActionButton3 must exist after Blizzard_ActionBar startup");
        let handler_result = env.fire_script_handler(button_id, "OnEnter", vec![]);
        let action: i32 = env
            .eval("return ActionButton3.action")
            .expect("ActionButton3.action probe must run");

        let error_message = handler_result.err();
        assert!(
            error_message.is_none(),
            "ActionButton3 hover must not error after slot 3 is populated. \
             Blizzard `ActionBarActionButtonMixin:SetTooltip` calls \
             `GameTooltip:SetAction(self.action)` at Shared/ActionButton.lua:1112, \
             so `self.action` must be the numeric slot assigned by \
             `UpdateAction()`. Error: {error_message:?}"
        );
        assert_eq!(
            action, SLOT as i32,
            "ActionButton3.action must remain the numeric slot id used by \
             GameTooltip:SetAction during hover"
        );
    });
    }
}
}
