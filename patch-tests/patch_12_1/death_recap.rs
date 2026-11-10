use super::new_game_env;
use wow_ui_sim::lua_api::state::{DeathRecapEntry, DeathRecapEventInfo};

fn recap(
    recap_id: u32,
    default_eligible: bool,
    events: Vec<DeathRecapEventInfo>,
) -> DeathRecapEntry {
    DeathRecapEntry {
        recap_id,
        default_eligible,
        events,
        max_health: 100,
        zone_name: "fixture".into(),
        killing_blows: Vec::new(),
    }
}

fn event(current_hp: u64) -> DeathRecapEventInfo {
    DeathRecapEventInfo {
        current_hp,
        amount: Some(100 - current_hp as i64),
        timestamp: None,
        source_guid: None,
    }
}

#[test]
fn ptr_death_recap_probes_match_native_argument_and_miss_rules() {
    let env = new_game_env();
    env.exec("DEATH_RECAP_TEXT = 'no recap'; DEATH_RECAP_LINK = 'recap:%d'")
        .expect("install localized recap fixture strings");

    let empty_results: (i32, bool, String) = env
        .eval(
            r#"
            return select('#', C_DeathRecap.GetRecapEvents()),
                C_DeathRecap.HasRecapEvents(),
                C_DeathRecap.GetRecapLink()
            "#,
        )
        .expect("PTR C_DeathRecap methods should be callable");
    assert_eq!(empty_results, (0, false, "no recap".into()));

    {
        let mut state = env.state().borrow_mut();
        state.death_recaps = vec![
            recap(0, false, vec![event(10)]),
            recap(11, true, vec![event(80)]),
            recap(22, false, vec![event(45)]),
            recap(33, false, Vec::new()),
        ];
    }

    env.exec(
        r#"
        assert(select('#', C_DeathRecap.GetRecapEvents()) == 1)
        assert(C_DeathRecap.HasRecapEvents() == true)
        assert(C_DeathRecap.GetRecapLink() == 'recap:11')

        assert(select('#', C_DeathRecap.GetRecapEvents('22.9')) == 1)
        assert(C_DeathRecap.HasRecapEvents('22.9') == true)
        assert(C_DeathRecap.GetRecapLink('22.9') == 'recap:22')
        assert(select('#', C_DeathRecap.GetRecapEvents('not numeric')) == 1)
        assert(select('#', C_DeathRecap.GetRecapEvents(false)) == 1)

        assert(select('#', C_DeathRecap.GetRecapEvents(33)) == 1)
        assert(#C_DeathRecap.GetRecapEvents(33) == 0)
        assert(select('#', C_DeathRecap.GetRecapEvents(999)) == 0)
        assert(C_DeathRecap.HasRecapEvents(999) == false)
        assert(C_DeathRecap.GetRecapLink(999) == 'no recap')
        assert(select('#', C_DeathRecap.GetRecapEvents(-1)) == 0)
        assert(C_DeathRecap.HasRecapEvents(-1) == false)
        assert(C_DeathRecap.GetRecapLink(-1) == 'no recap')

        for _, value in ipairs({false, {}, 'not numeric'}) do
            assert(not pcall(C_DeathRecap.HasRecapEvents, value))
            assert(not pcall(C_DeathRecap.GetRecapLink, value))
        end
        for _, value in ipairs({2147483648, -2147483649}) do
            assert(not pcall(C_DeathRecap.GetRecapEvents, value))
            assert(not pcall(C_DeathRecap.HasRecapEvents, value))
            assert(not pcall(C_DeathRecap.GetRecapLink, value))
        end
        "#,
    )
    .expect("PTR recap argument and miss results should match the native surface");

    {
        let mut state = env.state().borrow_mut();
        state.death_recaps = (1..=61)
            .map(|recap_id| recap(recap_id, recap_id == 61, Vec::new()))
            .collect();
    }

    env.exec(
        r#"
        assert(select('#', C_DeathRecap.GetRecapEvents(1)) == 0)
        assert(C_DeathRecap.HasRecapEvents(1) == false)
        assert(C_DeathRecap.GetRecapLink(1) == 'no recap')
        assert(C_DeathRecap.HasRecapEvents() == true)
        assert(C_DeathRecap.GetRecapLink() == 'recap:61')
        "#,
    )
    .expect("overwritten recap IDs should leave the 60-slot lookup window");
}
