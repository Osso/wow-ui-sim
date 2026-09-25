use super::click_targeting_full_ui::{drain_test_errors, env_with_full_ui, install_test_error_handler};
use crate::common;

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
