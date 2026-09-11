//! Real cached Blizzard handlers consume simulator channel inputs without patches.
#![cfg(feature = "retail-12-1-0")]
use super::spell_casting::{drain_test_errors, env_with_full_blizzard_ui, install_test_error_handler};

#[test]
fn channel_blizzard_view_observes_start_update_and_natural_completion() {
    test_timeout! {
        let env = env_with_full_blizzard_ui();
        install_test_error_handler(&env);
        env.exec(r#"
            CastingBarMixin.OnLoad(PlayerCastingBarFrame, 'player', true, false)
            A_Admin.StartChannel(15407, 'Synthetic channel', 'Interface/Icons/Spell_Shadow_SiphonMana', 4)
            local bar = PlayerCastingBarFrame
            assert(bar:IsShown() and bar.channeling and not bar.casting)
            assert(math.abs(bar.maxValue - 4) < 0.00001)
            assert(bar.Text:GetText() == 'Synthetic channel')
            assert(A_Admin.UpdateChannel(6))
            assert(math.abs(bar.maxValue - 6) < 0.00001)
        "#).unwrap();
        env.state().borrow_mut().start_time -= std::time::Duration::from_secs(7);
        env.fire_on_update(0.016).unwrap();
        env.exec(r#"
            assert(UnitChannelInfo('player') == nil)
            assert(not PlayerCastingBarFrame.channeling)
            PlayerCastingBarFrame:StopAnims()
            A_Admin.StartChannel(15407, 'Cancel', '', 4)
            assert(A_Admin.StopChannel())
            assert(not PlayerCastingBarFrame.channeling)
        "#).unwrap();
        assert!(drain_test_errors(&env).is_empty(), "channel consumer errors");
    }
}

#[test]
fn channel_blizzard_empower_pips_use_milliseconds_and_release_flags() {
    test_timeout! {
        let env = env_with_full_blizzard_ui();
        install_test_error_handler(&env);
        env.exec(r#"
            CastingBarMixin.OnLoad(PlayerCastingBarFrame, 'player', true, false)
            A_Admin.StartEmpower(357208, 'Synthetic empower', 'Interface/Icons/Spell_Fire_Fire', {0.8, 1.2, 1.5}, 2)
            local bar = PlayerCastingBarFrame
            assert(bar:IsShown() and bar.casting and bar.reverseChanneling and not bar.channeling)
            assert(math.abs(bar.maxValue - 5.5) < 0.00001, 'hold must add milliseconds before conversion')
            assert(bar.NumStages == 4 and #bar.StagePips == 3)
            assert(bar.StagePoints[1] == 800 and bar.StagePoints[2] == 2000 and bar.StagePoints[3] == 3500)
            local width = bar:GetRight() - bar:GetLeft()
            for i, pip in ipairs(bar.StagePips) do
                assert(pip:IsShown())
                local point, relative, relativePoint, x = pip:GetPoint(1)
                assert(point == 'TOP' and relative == bar and relativePoint == 'TOPLEFT')
                assert(math.abs(x - width * bar.StagePoints[i] / 5500) < 0.0001)
            end
            assert(A_Admin.StopChannel(true))
            assert(not bar.casting and not bar.reverseChanneling)
            bar:StopAnims()
            A_Admin.StartEmpower(357208, 'Cancel empower', '', {1, 1}, 1)
            assert(A_Admin.StopChannel(false))
            assert(not bar.casting and not bar.reverseChanneling)
            bar:StopAnims()
            A_Admin.StartEmpower(357208, 'Natural empower', '', {1, 1}, 1)
        "#).unwrap();
        env.state().borrow_mut().start_time -= std::time::Duration::from_secs(4);
        env.fire_on_update(0.016).unwrap();
        env.exec("assert(UnitChannelInfo('player') == nil); assert(not PlayerCastingBarFrame.reverseChanneling)").unwrap();
        let errors = drain_test_errors(&env);
        assert!(errors.is_empty(), "empower consumer errors: {errors:?}");
    }
}

#[test]
fn channel_blizzard_empower_update_exposes_vendor_hold_boundary() {
    test_timeout! {
        let env = env_with_full_blizzard_ui();
        install_test_error_handler(&env);
        env.exec(r#"
            CastingBarMixin.OnLoad(PlayerCastingBarFrame, 'player', true, false)
            A_Admin.StartEmpower(357208, 'Update empower', '', {1, 1}, 2)
            local bar = PlayerCastingBarFrame
            assert(math.abs(bar.maxValue - 4) < 0.00001)
            local id = select(11, UnitChannelInfo('player'))
            assert(A_Admin.UpdateEmpower({2, 2}, 3))
            assert(select(11, UnitChannelInfo('player')) == id)
            assert(GetUnitEmpowerStageDuration('player', 0) == 2000)
            assert(GetUnitEmpowerHoldAtMaxTime('player') == 3000)
            local _, _, _, startMs, endMs = UnitChannelInfo('player')
            assert(math.abs(endMs - startMs - 4000) < 0.00001)
            -- Unmodified UPDATE handler omits hold and does not rebuild pips.
            assert(math.abs(bar.maxValue - 4) < 0.00001)
            assert(bar.StagePoints[1] == 1000 and bar.StagePoints[2] == 2000)
            assert(A_Admin.StopChannel(true))
        "#).unwrap();
        let errors = drain_test_errors(&env);
        assert!(errors.is_empty(), "empower update errors: {errors:?}");
    }
}
