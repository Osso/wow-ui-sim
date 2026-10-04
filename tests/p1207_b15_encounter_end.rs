#![cfg(feature = "retail-12-0-7")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn tainted_encounter_listener_observes_public_detached_multi_boss_snapshot() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local supplied = {
            {creatureID=711, creatureName='Fixture Alpha', remainingHealthPercent=0},
            {creatureID=722, creatureName='Fixture Beta', remainingHealthPercent=62.5},
        }
        local observed = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ENCOUNTER_END')
        local function receive(_, event, ...)
            assert(not issecure(), 'real addon listener required')
            assert(event == 'ENCOUNTER_END' and select('#', ...) == 6)
            local id, name, difficulty, size, success, statuses = ...
            assert(id == 301 and name == 'Fixture encounter')
            assert(difficulty == 16 and size == 20 and success == 1)
            assert(type(statuses) == 'table' and not issecretvalue(statuses))
            for _, row in ipairs(statuses) do
                assert(not issecretvalue(row.creatureID))
                assert(not issecretvalue(row.creatureName))
                assert(not issecretvalue(row.remainingHealthPercent))
            end
            observed[#observed+1] = statuses
            collectgarbage('collect')
        end
        debug.setobjecttaint(receive, 'EncounterProbe')
        listener:SetScript('OnEvent', receive)
        A_Admin.SimulateBossKill(301, 'Fixture encounter', 16, 20, supplied)
        assert(issecure(), 'secure producer call preserves caller context')
        -- Inspect listener-written slots in a separate call frame: reading them
        -- intentionally taints that frame, not the later producer-call probe.
        local function inspectFirstSnapshot()
            assert(#observed[1] == 2 and observed[1] ~= supplied)
            assert(observed[1][1] ~= supplied[1] and observed[1][2] ~= supplied[2])
            assert(observed[1][1].creatureID == 711)
            assert(observed[1][1].creatureName == 'Fixture Alpha')
            assert(observed[1][1].remainingHealthPercent == 0)
            assert(observed[1][2].creatureID == 722)
            assert(observed[1][2].creatureName == 'Fixture Beta')
            assert(observed[1][2].remainingHealthPercent == 62.5)
            supplied[2].remainingHealthPercent = 11
            observed[1][1].creatureName = 'listener edit'
        end
        inspectFirstSnapshot()
        assert(issecure(), 'inspection taint stays in its own call frame')
        local function addonKill()
            assert(not issecure(), 'second producer call comes from addon code')
            A_Admin.SimulateBossKill(301, 'Fixture encounter', 16, 20, supplied)
        end
        debug.setobjecttaint(addonKill, 'EncounterProducerProbe')
        addonKill()
        assert(issecure(), 'producer call must preserve secure outer context')
        assert(observed[2][1].creatureName == 'Fixture Alpha')
        assert(observed[2][2].remainingHealthPercent == 11)
        assert(observed[1][2].remainingHealthPercent == 62.5)
        A_Admin.SimulateBossKill(301, 'Fixture encounter', 16, 20)
        A_Admin.SimulateBossKill(301, 'Fixture encounter', 16, 20, nil)
        assert(#observed == 4 and next(observed[3]) == nil and next(observed[4]) == nil)
        assert(observed[3] ~= observed[4])
        "#,
    )
    .expect("explicit status snapshots retain public fields under an actual tainted listener");
}
