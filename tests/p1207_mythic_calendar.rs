#![cfg(feature = "retail-12-0-7")]
use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_bool, wrap_host_secret_number};
use wow_ui_sim::c_api::c_mythic_plus_calendar::{CalendarTime, MythicPlusMember};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{MythicPlusRun, MythicPlusWeeklyBest};

const A: CalendarTime = CalendarTime {
    month_day: 3,
    month: 10,
    weekday: 7,
    year: 2026,
    hour: 12,
    minute: 34,
};
const B: CalendarTime = CalendarTime {
    month_day: 29,
    month: 2,
    weekday: 5,
    year: 2024,
    hour: 23,
    minute: 59,
};
const C: CalendarTime = CalendarTime {
    month_day: 1,
    month: 1,
    weekday: 1,
    year: 2030,
    hour: 0,
    minute: 7,
};

fn best(map: i32, date: Option<CalendarTime>) -> MythicPlusWeeklyBest {
    MythicPlusWeeklyBest {
        map_challenge_mode_id: map,
        level: 12,
        duration_sec: 1500,
        score: 180.5,
        completion_date: date,
        affix_ids: vec![9, 10],
        members: vec![
            MythicPlusMember {
                name: Some("Fixture One".into()),
                spec_id: 70,
                class_id: 2,
            },
            MythicPlusMember {
                name: None,
                spec_id: 259,
                class_id: 4,
            },
        ],
    }
}

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.mythic_plus.run_history = vec![
            MythicPlusRun {
                map_challenge_mode_id: 399,
                level: 12,
                completed: true,
                season: 14,
                run_score: 180.5,
                this_week: true,
                duration_sec: 1500,
                completion_date: Some(A),
            },
            MythicPlusRun {
                map_challenge_mode_id: 400,
                level: 15,
                completed: false,
                season: 13,
                run_score: 222.25,
                this_week: false,
                duration_sec: 1900,
                completion_date: Some(B),
            },
        ];
        sim.mythic_plus.weekly_best_per_map =
            [(399, best(399, Some(A))), (400, best(400, Some(B)))].into();
        sim.mythic_plus.season_best_per_map = [
            (399, (Some(best(399, Some(A))), Some(best(399, Some(B))))),
            (400, (None, Some(best(400, Some(B))))),
        ]
        .into();
    }
    env.exec(r#"
        Dates = {{3,10,7,2026,12,34}, {29,2,5,2024,23,59}, {1,1,1,2030,0,7}}
        function Date(value, index)
            local keys = {'monthDay','month','weekday','year','hour','minute'}
            local expected = Dates[index]
            assert(type(value) == 'table' and value.day == nil)
            local count = 0
            for _, _ in pairs(value) do count = count + 1 end
            assert(count == 6, 'exact CalendarTime output fields')
            for position, key in ipairs(keys) do
                assert(type(value[key]) == 'number' and value[key] == expected[position])
                assert(not issecretvalue(value[key]))
            end
        end
        function Best(value, date)
            assert(type(value) == 'table' and value.durationSec == 1500 and value.level == 12)
            assert(value.dungeonScore == 180.5)
            Date(value.completionDate, date)
            assert(#value.affixIDs == 2 and value.affixIDs[1] == 9 and value.affixIDs[2] == 10)
            assert(#value.members == 2 and value.members[1].name == 'Fixture One')
            assert(value.members[1].specID == 70 and value.members[1].classID == 2)
            assert(value.members[2].name == nil and value.members[2].specID == 259 and value.members[2].classID == 4)
        end
    "#).unwrap();
    env
}

#[test]
fn history_calendar_fields_are_distinct_live_and_detached() {
    let env = fixture();
    env.exec(r#"
        local function check(...)
            assert(select('#', ...) == 1)
            local runs = ...; assert(#runs == 2)
            assert(runs[1].mapChallengeModeID == 399 and runs[1].runScore == 180.5)
            assert(runs[1].level == 12 and runs[1].durationSec == 1500 and runs[1].completed and runs[1].thisWeek)
            assert(runs[2].mapChallengeModeID == 400 and runs[2].runScore == 222.25)
            assert(runs[2].season == 13 and not runs[2].completed and not runs[2].thisWeek)
            Date(runs[1].completionDate, 1); Date(runs[2].completionDate, 2)
            SavedHistory = runs
        end
        check(C_MythicPlus.GetRunHistory(true, true, false))
        SavedHistory[1].completionDate.monthDay = 99
        local next = C_MythicPlus.GetRunHistory(true, true, false)
        assert(next ~= SavedHistory); Date(next[1].completionDate, 1)
    "#).unwrap();
    assert_eq!(
        env.state().borrow().mythic_plus.run_history[0].completion_date,
        Some(A)
    );
    env.state().borrow_mut().mythic_plus.run_history[0].completion_date = Some(C);
    env.exec("Date(C_MythicPlus.GetRunHistory(true, true, false)[1].completionDate, 3); assert(SavedHistory[1].completionDate.monthDay == 99)").unwrap();
    env.state().borrow_mut().mythic_plus.run_history.remove(0);
    env.exec("local a=C_MythicPlus.GetRunHistory(true,true,false); assert(#a==1); Date(a[1].completionDate,2)").unwrap();
}

#[test]
fn weekly_calendar_uses_six_declared_tuple_slots_live() {
    let env = fixture();
    env.exec(
        r#"
        local function check(...)
            assert(select('#', ...) == 6)
            local duration, level, date, affixes, members, score = ...
            assert(duration == 1500 and level == 12 and score == 180.5)
            Date(date, 1)
            assert(#affixes == 2 and affixes[1] == 9 and affixes[2] == 10)
            assert(#members == 2 and members[1].name == 'Fixture One' and members[2].name == nil)
            assert(members[1].specID == 70 and members[2].classID == 4)
            SavedWeeklyDate = date; SavedAffixes = affixes; SavedMembers = members
        end
        check(C_MythicPlus.GetWeeklyBestForMap(399))
        local _,_,b = C_MythicPlus.GetWeeklyBestForMap(400); Date(b,2)
        SavedWeeklyDate.monthDay = 99; SavedAffixes[1] = 999; SavedMembers[1].name = 'changed'
        local _,_,date,ids,members = C_MythicPlus.GetWeeklyBestForMap(399)
        Date(date,1); assert(ids[1] == 9 and members[1].name == 'Fixture One')
    "#,
    )
    .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let row = sim.mythic_plus.weekly_best_per_map.get_mut(&399).unwrap();
        row.completion_date = Some(C);
        row.score = 244.25;
        row.duration_sec = 900;
        row.level = 17;
        row.affix_ids = vec![11];
        row.members.clear();
    }
    env.exec(
        r#"
        local duration, level, date, ids, members, score = C_MythicPlus.GetWeeklyBestForMap(399)
        assert(duration == 900 and level == 17 and score == 244.25)
        Date(date,3); assert(#ids == 1 and ids[1] == 11 and #members == 0)
        assert(SavedWeeklyDate.monthDay == 99 and SavedAffixes[1] == 999)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .mythic_plus
        .weekly_best_per_map
        .remove(&399);
    let count: f64 = env
        .eval("return select('#', C_MythicPlus.GetWeeklyBestForMap(399))")
        .unwrap();
    assert_eq!(count, 0.0);
}

#[test]
fn season_in_time_overtime_dates_and_nil_sides_are_live() {
    let env = fixture();
    env.exec(
        r#"
        local function check(...)
            assert(select('#', ...) == 2)
            local a,b = ...; Best(a,1); Best(b,2)
            SavedSeason = a; SavedSeason.completionDate.monthDay = 99
            SavedSeason.affixIDs[1] = 999; SavedSeason.members[1].name = 'changed'
        end
        check(C_MythicPlus.GetSeasonBestForMap(399))
        local a,b = C_MythicPlus.GetSeasonBestForMap(399); Best(a,1); Best(b,2)
        local a,b = C_MythicPlus.GetSeasonBestForMap(400); assert(a == nil); Best(b,2)
        local a,b = C_MythicPlus.GetSeasonBestForMap(999); assert(a == nil and b == nil)
        assert(select('#', C_MythicPlus.GetSeasonBestForMap(999)) == 2)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .mythic_plus
        .season_best_per_map
        .insert(399, (Some(best(399, Some(C))), None));
    env.exec(
        r#"
        local a,b = C_MythicPlus.GetSeasonBestForMap(399); Best(a,3); assert(b == nil)
        assert(SavedSeason.completionDate.monthDay == 99)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .mythic_plus
        .season_best_per_map
        .remove(&399);
    env.exec("local a,b=C_MythicPlus.GetSeasonBestForMap(399); assert(a==nil and b==nil)")
        .unwrap();
}

#[test]
fn missing_dates_are_explicit_data_gaps_not_fabricated_calendar_values() {
    let env = fixture();
    {
        let mut sim = env.state().borrow_mut();
        sim.mythic_plus.run_history[0].completion_date = None;
        sim.mythic_plus
            .weekly_best_per_map
            .get_mut(&399)
            .unwrap()
            .completion_date = None;
        sim.mythic_plus
            .season_best_per_map
            .insert(399, (Some(best(399, None)), Some(best(399, Some(B)))));
    }
    env.exec(
        r#"
        local runs = C_MythicPlus.GetRunHistory(true,true,false)
        assert(#runs == 1 and runs[1].mapChallengeModeID == 400); Date(runs[1].completionDate,2)
        assert(select('#', C_MythicPlus.GetWeeklyBestForMap(399)) == 0)
        local a,b=C_MythicPlus.GetSeasonBestForMap(399); assert(a==nil); Best(b,2)
        for _,invalid in ipairs({false, '399', {}, 399.5, 1/0}) do
            assert(select('#', C_MythicPlus.GetWeeklyBestForMap(invalid)) == 0)
            local a,b=C_MythicPlus.GetSeasonBestForMap(invalid); assert(a==nil and b==nil)
        end
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().mythic_plus.run_history.len(), 2);
    let other = WowLuaEnv::new().unwrap();
    other
        .exec(
            r#"
        local first = C_MythicPlus.GetRunHistory(); first[1] = {level=99}
        assert(#C_MythicPlus.GetRunHistory() == 0)
        assert(select('#', C_MythicPlus.GetWeeklyBestForMap(399)) == 0)
        local a,b=C_MythicPlus.GetSeasonBestForMap(399); assert(a==nil and b==nil)
    "#,
        )
        .unwrap();
}

#[test]
fn calendar_authenticates_every_flag_map_and_extra_before_validation() {
    let env = fixture();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        let map = wrap_host_secret_number(lua.state_mut(), 399.0);
        lua.state_mut().push(map);
        lua.set_global_val("CalendarSecretMap", map).unwrap();
        lua.state_mut().pop();
        let flag = wrap_host_secret_bool(lua.state_mut(), true);
        lua.state_mut().push(flag);
        lua.set_global_val("CalendarSecretFlag", flag).unwrap();
        lua.state_mut().pop();
    }
    env.exec(r#"
        local function checkSecure()
            local _,_,date=C_MythicPlus.GetWeeklyBestForMap(CalendarSecretMap); Date(date,1)
            local a,b=C_MythicPlus.GetSeasonBestForMap(CalendarSecretMap); Best(a,1); Best(b,2)
            for _,flags in ipairs({{CalendarSecretFlag,true,false}, {true,CalendarSecretFlag,false},
                {true,true,CalendarSecretFlag}, {CalendarSecretFlag,CalendarSecretFlag,CalendarSecretFlag}}) do
                local runs=C_MythicPlus.GetRunHistory(unpack(flags)); Date(runs[1].completionDate,1)
            end
        end
        checkSecure(); collectgarbage('collect'); checkSecure()
        local function probe()
            Date(C_MythicPlus.GetRunHistory(true,true,false)[1].completionDate,1)
            for _,call in ipairs({
                function() return C_MythicPlus.GetRunHistory(CalendarSecretFlag,true,false) end,
                function() return C_MythicPlus.GetRunHistory('invalid',CalendarSecretFlag,false) end,
                function() return C_MythicPlus.GetRunHistory('invalid',false,CalendarSecretFlag) end,
                function() return C_MythicPlus.GetRunHistory('invalid',false,false,CalendarSecretFlag) end,
                function() return C_MythicPlus.GetWeeklyBestForMap(CalendarSecretMap) end,
                function() return C_MythicPlus.GetWeeklyBestForMap(false,CalendarSecretFlag) end,
                function() return C_MythicPlus.GetWeeklyBestForMap(999,CalendarSecretFlag) end,
                function() return C_MythicPlus.GetSeasonBestForMap(CalendarSecretMap) end,
                function() return C_MythicPlus.GetSeasonBestForMap(false,CalendarSecretFlag) end,
                function() return C_MythicPlus.GetSeasonBestForMap(999,CalendarSecretFlag) end,
            }) do
                local ok,err=pcall(call)
                assert(not ok and string.find(err,'untainted caller',1,true))
            end
            assert(debug.getstacktaint() == 'CalendarProbe')
        end
        debug.setobjecttaint(probe,'CalendarProbe'); probe()
        assert(issecure() and issecretvalue(CalendarSecretMap) and issecretvalue(CalendarSecretFlag))
        checkSecure()
    "#).unwrap();
}
