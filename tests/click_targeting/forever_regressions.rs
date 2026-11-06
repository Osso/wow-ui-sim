use super::click_targeting_full_ui::{
    drain_test_errors, env_with_full_ui, env_with_full_ui_configured, install_test_error_handler,
};
use crate::common;

#[test]
fn blizzard_player_world_entry_updates_combo_frame_from_target_power() {
    common::with_perf_lock(|| {
        test_timeout! {
            let env = env_with_full_ui_configured(|env| {
                env.exec("SetCVar('comboPointLocation', '1')").expect("enable player-frame combo points");
            });
            install_test_error_handler(&env);
            env.exec(r#"
                TargetUnit('enemy1')
                A_Admin.SetPlayerPower(3, 5, Enum.PowerType.ComboPoints)
            "#).expect("configure target combo points");
            env.fire_event("PLAYER_ENTERING_WORLD").expect("update PlayerFrame through event");
            env.exec(r#"
                assert(ComboFrame:IsShown(), 'combo frame hidden after world-entry event')
                assert(COMBO_FRAME_LAST_NUM_POINTS == 3, 'combo count not updated by world-entry event')
                A_Admin.SetPlayerPower(0, 5, Enum.PowerType.ComboPoints)
            "#).expect("read and clear combo power");
            env.fire_event("PLAYER_ENTERING_WORLD").expect("update zero combo through event");
            env.exec(r#"
                assert(not ComboFrame:IsShown(), 'combo frame shown with zero points')
                assert(COMBO_FRAME_LAST_NUM_POINTS == 0, 'zero combo count not updated by event')
            "#).expect("read zero combo state");
            let errors = drain_test_errors(&env);
            assert!(errors.is_empty(), "player world-entry event errors: {errors:?}");
        }
    });
}

// The private-partition `AuraButton.Count` case needs the mainline
// `TargetFrame.Auras` container: `Blizzard_UnitFrame/Blizzard_UnitFrame_Mainline.toc:27-28`
// lists `Shared/TargetFrameAuraContainer.lua|.xml`, and the classic UnitFrame TOC
// (`mists/AddOns/Blizzard_UnitFrame/Blizzard_UnitFrame_Classic.toc:24-27`) lists
// neither file while `Classic/TargetFrame.lua:451-500` creates `TargetFrameBuff<N>`
// buttons instead. See `classic_target_frame_fixture!` for the classic consumer path.
target_frame_aura_container_fixture! {
#[test]
fn blizzard_target_player_aura_updates_keep_count_region() {
    common::with_perf_lock(|| {
        test_timeout! {
            let env = env_with_full_ui();
            install_test_error_handler(&env);
            env.exec("TargetUnit('player')").expect("target player");
            env.fire_on_update(0.016).expect("tick target auras");
            let errors = drain_test_errors(&env);
            assert!(errors.is_empty(), "target aura update errors: {errors:?}");
            let (has_count, public_field_stays_public): (bool, bool) = env.eval(r#"
                local function find_auras(frame, depth)
                    if depth > 15 then return nil end
                    if frame.Auras then return frame.Auras end
                    for i = 1, frame:GetNumChildren() do
                        local child = select(i, frame:GetChildren())
                        local found = child and find_auras(child, depth + 1)
                        if found then return found end
                    end
                end
                local container = assert(find_auras(TargetFrame, 0))
                local button
                for i = 1, container:GetNumChildren() do
                    local child = select(i, container:GetChildren())
                    if child and child:GetObjectType() == 'AuraButton' then
                        button = child
                        break
                    end
                end
                assert(button, 'target aura button missing')
                local count = button.Count
                button.AddonField = 'public-only'
                return count ~= nil and GetForbiddenObjectTable(button).Count == count,
                    GetForbiddenObjectTable(button).AddonField == nil
            "#).expect("inspect target aura partitions");
            assert!(has_count, "target aura private mixin cannot access Count");
            assert!(public_field_stays_public, "ordinary Lua fields leaked into private partition");
        }
    });
}
}

// Classic `TargetFrameMixin:OnEvent` reads
// `C_PlayerInteractionManager.IsReplacingUnit()` on every PLAYER_TARGET_CHANGED
// (`mists|era|anniversary/AddOns/Blizzard_UnitFrame/Classic/TargetFrame.lua:168`)
// and `UpdateAuras` (:451-500) then builds `TargetFrameBuff<N>` buttons from the
// `TargetBuffFrameTemplate` and sizes each one through ShouldAuraBeLarge(caster)
// (:502), where `caster` is the seventh `AuraUtil.ForEachAura` value (:461).
classic_target_frame_fixture! {
#[test]
fn blizzard_classic_target_player_aura_update_runs_without_errors() {
    common::with_perf_lock(|| {
        test_timeout! {
            let env = env_with_full_ui();
            install_test_error_handler(&env);
            env.exec("TargetUnit('player')").expect("target player");
            env.fire_on_update(0.016).expect("tick target auras");
            let errors = drain_test_errors(&env);
            assert!(errors.is_empty(), "classic target aura update errors: {errors:?}");
            // The classic handler reaches `TargetFrameMixin:UpdateAuras`
            // (Classic/TargetFrame.lua:146) after the IsReplacingUnit read. The
            // `TargetFrameBuff<N>` buttons it builds (:469) need the target's
            // helpful auras, which this build does not project for the `target`
            // unit token; see the lane report.
            let (has_target, has_buff_container): (bool, bool) = env
                .eval(
                    r#"
                    return TargetFrame.unit == 'target' and UnitExists('target'),
                        TargetFrame.buffs ~= nil
                "#,
                )
                .expect("classic target frame probe");
            assert!(
                has_target,
                "TargetFrame must remain the selected-target unit frame"
            );
            assert!(
                has_buff_container,
                "classic TargetFrame.xml:339 declares the `$parentBuffs` container"
            );
        }
    });
}
}
