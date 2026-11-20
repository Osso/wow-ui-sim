//! `GuildControl*` rank probes — SimState-backed round-trip.

use wow_ui_sim::lua_api::WowLuaEnv;

fn install_three_ranks(env: &WowLuaEnv) {
    env.exec(
        r#"
        A_Admin.SetGuildInfo("Heroes of Azeroth", "Member", 2)
        A_Admin.SetGuildRanks({
            { name = "Guild Leader", flags = { true, true, true } },
            { name = "Officer",      flags = { true, false, true } },
            { name = "Member",       flags = { false, false, false } },
        })
        "#,
    )
    .unwrap();
}

#[test]
fn supplied_guild_has_rank_names() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);
    let (count, first, second, third): (i32, String, String, String) = env
        .eval(
            r#"
            return GuildControlGetNumRanks(),
                   GuildControlGetRankName(1),
                   GuildControlGetRankName(2),
                   GuildControlGetRankName(3)
            "#,
        )
        .unwrap();
    assert_eq!(count, 3);
    assert_eq!(first, "Guild Leader");
    assert_eq!(second, "Officer");
    assert_eq!(third, "Member");
}

#[test]
fn no_guild_returns_no_ranks() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.ClearGuild()").unwrap();
    let (count, name): (i32, String) = env
        .eval(r#"return GuildControlGetNumRanks(), GuildControlGetRankName()"#)
        .unwrap();
    assert_eq!(count, 0);
    assert_eq!(name, "");

    // Wrath's unselected-rank flags read is undefined natively. The separate
    // native contract case supplies a valid selected rank before removing guild state.
    if wow_ui_sim::client_profile::ACTIVE != wow_ui_sim::client_profile::ClientProfile::Wrath {
        let flags_count: i32 = env.eval(r#"return #GuildControlGetRankFlags()"#).unwrap();
        assert_eq!(flags_count, 0);
    }
}

#[test]
fn set_ranks_updates_count() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);
    let count: i32 = env.eval("return GuildControlGetNumRanks()").unwrap();
    assert_eq!(count, 3);
}

#[test]
fn get_num_members_in_rank_counts_roster_members_by_rank() {
    use wow_ui_sim::lua_api::state::GuildMember;

    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().world.guild_name = Some("Heroes of Azeroth".into());
    env.state().borrow_mut().world.guild_members = vec![
        GuildMember {
            name: "Alpha".into(),
            guid: None,
            rank_index: 1,
            online: true,
        },
        GuildMember {
            name: "Beta".into(),
            guid: None,
            rank_index: 2,
            online: false,
        },
        GuildMember {
            name: "Gamma".into(),
            guid: None,
            rank_index: 2,
            online: true,
        },
    ];

    let counts: (i32, i32, i32) = env
        .eval("return GetNumMembersInRank(1), GetNumMembersInRank(2), GetNumMembersInRank(3)")
        .unwrap();

    assert_eq!(counts, (1, 2, 0));
}

#[test]
fn get_num_members_in_rank_returns_zero_without_guild() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.ClearGuild()").unwrap();
    let count: i32 = env.eval("return GetNumMembersInRank(1)").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn get_allowed_shifts_reports_rank_order_movement() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);

    let shifts: (bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local firstUp, firstDown = GuildControlGetAllowedShifts(1)
            local secondUp, secondDown = GuildControlGetAllowedShifts(2)
            local thirdUp, thirdDown = GuildControlGetAllowedShifts(3)
            return firstUp, firstDown, secondUp, secondDown, thirdUp, thirdDown
            "#,
        )
        .unwrap();

    assert_eq!(
        shifts,
        (false, false, false, true, true, false),
        "guild leader cannot move; middle ranks move down; last editable rank moves up",
    );
}

#[test]
fn get_allowed_shifts_returns_false_without_guild() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.ClearGuild()").unwrap();
    let shifts: (bool, bool) = env.eval("return GuildControlGetAllowedShifts(2)").unwrap();
    assert_eq!(shifts, (false, false));
}

#[test]
fn get_rank_name_by_explicit_index() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);
    let names: (String, String, String) = env
        .eval(
            r#"
            return GuildControlGetRankName(1),
                   GuildControlGetRankName(2),
                   GuildControlGetRankName(3)
            "#,
        )
        .unwrap();
    assert_eq!(names.0, "Guild Leader");
    assert_eq!(names.1, "Officer");
    assert_eq!(names.2, "Member");
}

#[test]
fn set_rank_selects_which_getter_returns_by_default() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);
    env.exec("GuildControlSetRank(2)").unwrap();
    let name: String = env.eval("return GuildControlGetRankName()").unwrap();
    assert_eq!(
        name, "Officer",
        "after SetRank(2), GetRankName() should return the 2nd rank",
    );

    env.exec("GuildControlSetRank(3)").unwrap();
    let name: String = env.eval("return GuildControlGetRankName()").unwrap();
    assert_eq!(name, "Member");
}

#[test]
fn out_of_range_explicit_index_returns_empty() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);
    let (zero, too_high): (String, String) = env
        .eval(
            r#"
            return GuildControlGetRankName(0), GuildControlGetRankName(99)
            "#,
        )
        .unwrap();
    assert_eq!(zero, "");
    assert_eq!(too_high, "");
}

#[test]
fn rank_flags_round_trip_as_boolean_array() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        env.exec(
            r#"
            GuildControlSetRank(2)
            local first, second, third = GuildControlGetRankFlags()
            assert(first == 1 and second == nil and third == 1)
        "#,
        )
        .expect("Wrath returns positional stored rights, not a boolean table");
        return;
    }
    let flags: (bool, bool, bool) = env
        .eval(
            r#"
            local f = GuildControlGetRankFlags(2)
            return f[1], f[2], f[3]
            "#,
        )
        .unwrap();
    assert_eq!(flags, (true, false, true));
}

#[test]
fn c_guild_info_rank_flags_use_same_backing_state() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);
    let flags: (bool, bool, bool) = env
        .eval(
            r#"
            local f = C_GuildInfo.GuildControlGetRankFlags(2)
            return f[1], f[2], f[3]
            "#,
        )
        .unwrap();
    assert_eq!(flags, (true, false, true));
}

#[test]
fn clearing_ranks_resets_selection_and_returns() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);
    env.exec("GuildControlSetRank(2)").unwrap();
    env.exec("A_Admin.SetGuildRanks({})").unwrap();
    let (count, name): (i32, String) = env
        .eval(r#"return GuildControlGetNumRanks(), GuildControlGetRankName()"#)
        .unwrap();
    assert_eq!(count, 0);
    assert_eq!(name, "");
}

#[test]
fn set_rank_to_out_of_range_clears_selection() {
    let env = WowLuaEnv::new().unwrap();
    install_three_ranks(&env);
    env.exec("GuildControlSetRank(2)").unwrap();
    env.exec("GuildControlSetRank(99)").unwrap();
    let name: String = env.eval("return GuildControlGetRankName()").unwrap();
    assert_eq!(
        name, "",
        "out-of-range SetRank should clear the selection; GetRankName without index returns empty",
    );
}
