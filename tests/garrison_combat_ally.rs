#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn combat_ally_mission_tracks_two_supplied_missions_and_clears() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    let selected: (i64, i64, bool) = env
        .eval(
            r#"
            A_Admin.SetGarrisonMissions({
                { missionID = 901, name = "First", followerTypeID = 7, level = 1,
                  durationSeconds = 60, duration = "1 minute", timeLeft = "", xp = 0,
                  canBeCompleted = false, rewards = {}, inProgress = false, completed = false },
                { missionID = 902, name = "Second", followerTypeID = 7, level = 1,
                  durationSeconds = 60, duration = "1 minute", timeLeft = "", xp = 0,
                  canBeCompleted = false, rewards = {}, inProgress = false, completed = false },
            })
            A_Admin.SetGarrisonCombatAllyMission(7, 901)
            local first = C_Garrison.GetCombatAllyMission(7).missionID
            A_Admin.SetGarrisonCombatAllyMission(7, 902)
            local second = C_Garrison.GetCombatAllyMission(7).missionID
            A_Admin.SetGarrisonCombatAllyMission(7, nil)
            return first, second, C_Garrison.GetCombatAllyMission(7) == nil
            "#,
        )
        .expect("combat-ally query should use supplied mission state");

    assert_eq!(selected, (901, 902, true));
}
