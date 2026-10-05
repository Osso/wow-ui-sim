//! Duration availability at the GUI's actual ordinary-cast completion boundary.
use super::{extract_completed_cast, fire_cast_complete_events};
use crate::lua_api::WowLuaEnv;
use std::time::Duration;

#[test]
fn completion_arguments_survive_collection_in_stop_error_handler() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        completionErrors = 0
        completionDeliveries = 0
        seterrorhandler(function()
            completionErrors = completionErrors + 1
            collectgarbage('collect')
        end)
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('UNIT_SPELLCAST_STOP')
        frame:RegisterEvent('UNIT_SPELLCAST_SUCCEEDED')
        frame:SetScript('OnEvent', function(_, event, unit, guid, spell)
            if event == 'UNIT_SPELLCAST_STOP' then error('audit cast stop') end
            assert(unit == 'player' and spell == 19750)
            -- Do not intern the full expected GUID in this closure's constants.
            assert(guid:sub(1, 9) == 'Cast-Sim-' and tonumber(guid:sub(10)) == 98765432)
            assert(select('#', UnitCastingDuration('player')) == 0)
            completionDeliveries = completionDeliveries + 1
        end)
    "#,
    )
    .unwrap();
    fire_cast_complete_events(&env, 98765432, 19750);
    env.exec("assert(completionErrors == 1 and completionDeliveries == 1)")
        .unwrap();
}

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
