//! Duration availability at the GUI's actual ordinary-cast completion boundary.
use super::{extract_completed_cast, fire_cast_complete_events};
use crate::lua_api::WowLuaEnv;
use std::time::Duration;

#[test]
fn unit_cast_duration_clears_before_completion_callbacks() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        completionEvents = {}
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('UNIT_SPELLCAST_STOP')
        frame:RegisterEvent('UNIT_SPELLCAST_SUCCEEDED')
        frame:SetScript('OnEvent', function(_, event)
            assert(select('#', UnitCastingDuration('player')) == 0,
                'completed cast duration must be absent during ' .. event)
            completionEvents[#completionEvents + 1] = event
        end)
        A_Admin.SetCasting(19750, 'Completion', 'cast-icon', 1)
        retainedDuration = UnitCastingDuration('player')
        assert(retainedDuration:GetTotalDuration() == 1)
        "#,
    )
    .unwrap();
    assert!(extract_completed_cast(env.state()).is_none());

    env.state().borrow_mut().start_time -= Duration::from_secs(2);
    let (cast_id, spell_id) = extract_completed_cast(env.state()).unwrap();
    assert_eq!(spell_id, 19750);
    fire_cast_complete_events(&env, cast_id, spell_id);

    env.exec(
        r#"
        assert(select('#', UnitCastingDuration('player')) == 0)
        assert(retainedDuration:HasExpired())
        assert(#completionEvents == 2)
        assert(completionEvents[1] == 'UNIT_SPELLCAST_STOP')
        assert(completionEvents[2] == 'UNIT_SPELLCAST_SUCCEEDED')
        "#,
    )
    .unwrap();
    assert!(extract_completed_cast(env.state()).is_none());
}
