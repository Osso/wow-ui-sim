//! Tooltip hide behavior for `ArdenwealdGardeningButtonMixin:OnLeave`.

use wow_ui_sim::lua_api::WowLuaEnv;

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ArdenwealdGardening";

timerunning_util_panel_fixture! {
#[test]
fn onleave_hides_tooltip_after_each_onenter_branch() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        assert_onleave_hides_tooltip_after(env, GardenTooltipSeed::active());
        assert_onleave_hides_tooltip_after(env, GardenTooltipSeed::ready());
        assert_onleave_hides_tooltip_after(env, GardenTooltipSeed::dormant());
    });
}
}

type OnLeaveProbe = (bool, bool);

struct GardenTooltipSeed {
    active: i32,
    ready: i32,
    remaining_seconds: i64,
}

impl GardenTooltipSeed {
    fn active() -> Self {
        Self {
            active: 3,
            ready: 0,
            remaining_seconds: 600,
        }
    }

    fn ready() -> Self {
        Self {
            active: 0,
            ready: 4,
            remaining_seconds: 0,
        }
    }

    fn dormant() -> Self {
        Self {
            active: 0,
            ready: 0,
            remaining_seconds: 0,
        }
    }
}

fn assert_onleave_hides_tooltip_after(env: &WowLuaEnv, seed: GardenTooltipSeed) {
    seed_garden_state(env, seed);

    let (shown_after_enter, shown_after_leave): OnLeaveProbe = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", nil, UIParent)
            ArdenwealdGardening.Create(parent)
            local button = ArdenwealdGardeningButtonTemplate

            GameTooltip:ClearLines()
            button:GetScript("OnEnter")(button)
            local shownAfterEnter = GameTooltip:IsShown()

            button:GetScript("OnLeave")(button)
            local shownAfterLeave = GameTooltip:IsShown()

            return shownAfterEnter, shownAfterLeave
            "#,
        )
        .expect("Ardenweald Gardening OnLeave probe must run cleanly");

    assert!(
        shown_after_enter,
        "OnEnter must show GameTooltip before OnLeave"
    );
    assert!(
        !shown_after_leave,
        "OnLeave must hide GameTooltip after the current OnEnter branch"
    );
}

fn seed_garden_state(env: &WowLuaEnv, seed: GardenTooltipSeed) {
    let mut state = env.state().borrow_mut();
    state.gardenweald.active = seed.active;
    state.gardenweald.ready = seed.ready;
    state.gardenweald.remaining_seconds = seed.remaining_seconds;
}
