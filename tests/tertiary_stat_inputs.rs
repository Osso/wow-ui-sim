//! Next55 host inputs; 180 rating/percentage point is INFERRED, not native evidence.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

const ASSERTIONS: &str = r#"
function TertiaryPack(...) return {n = select('#', ...), ...} end
function AssertTertiaryResult(query, expected, restricted, ...)
    local results = TertiaryPack(pcall(query, ...))
    assert(results.n == 2 and results[1] == true, 'successful callback, exactly one result')
    local value = results[2]
    assert(issecretvalue(value) == restricted, 'result secrecy')
    assert(canaccessvalue(value) == not restricted, 'result access')
    if restricted then value = secretunwrap(value) end
    assert(value == expected, 'exact current snapshot value')
end
function AssertTertiarySnapshot(avoidance, lifesteal, speed, restricted)
    local fixtures = {
        {GetAvoidance, 21, avoidance},
        {GetLifesteal, 17, lifesteal},
        {GetSpeed, 13, speed},
    }
    for _, fixture in ipairs(fixtures) do
        local query, index, rating = unpack(fixture)
        local percent = math.max(rating / 180, 0)
        AssertTertiaryResult(query, percent, restricted)
        AssertTertiaryResult(GetCombatRating, rating, restricted, index)
        AssertTertiaryResult(GetCombatRatingBonus, percent, restricted, index)
    end
end
"#;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    replace_ratings(&env, 2250, 585, 1395);
    env.state().borrow_mut().player.stats.crit_rating = 360;
    env.exec(ASSERTIONS).unwrap();
    env
}

fn replace_ratings(env: &WowLuaEnv, avoidance: i32, leech: i32, speed: i32) {
    let mut state = env.state().borrow_mut();
    state.player.stats.avoidance_rating = avoidance;
    state.player.stats.leech_rating = leech;
    state.player.stats.speed_rating = speed;
}

fn rating_snapshot(env: &WowLuaEnv) -> (i32, i32, i32, i32, i32, i32, i32) {
    let state = env.state().borrow();
    let stats = &state.player.stats;
    (
        stats.avoidance_rating,
        stats.leech_rating,
        stats.speed_rating,
        stats.crit_rating,
        stats.haste_rating,
        stats.mastery_rating,
        stats.versatility_rating,
    )
}

#[test]
fn avoidance_reads_existing_rating_21_and_shared_bonus() {
    let env = fixture_env();
    env.exec(
        "AssertTertiaryResult(GetAvoidance, 12.5, false); \
         AssertTertiaryResult(GetCombatRating, 2250, false, 21); \
         AssertTertiaryResult(GetCombatRatingBonus, 12.5, false, 21)",
    )
    .unwrap();
}

#[test]
fn lifesteal_reads_existing_leech_rating_17_and_shared_bonus() {
    let env = fixture_env();
    env.exec(
        "AssertTertiaryResult(GetLifesteal, 3.25, false); \
         AssertTertiaryResult(GetCombatRating, 585, false, 17); \
         AssertTertiaryResult(GetCombatRatingBonus, 3.25, false, 17)",
    )
    .unwrap();
}

#[test]
fn speed_reads_existing_rating_13_and_shared_bonus() {
    let env = fixture_env();
    env.exec(
        "AssertTertiaryResult(GetSpeed, 7.75, false); \
         AssertTertiaryResult(GetCombatRating, 1395, false, 13); \
         AssertTertiaryResult(GetCombatRatingBonus, 7.75, false, 13)",
    )
    .unwrap();
}

#[test]
fn live_host_replacement_changes_only_the_selected_field_outputs() {
    let env = fixture_env();
    env.exec("AssertTertiarySnapshot(2250, 585, 1395, false)")
        .unwrap();
    env.state().borrow_mut().player.stats.avoidance_rating = 900;
    env.exec("AssertTertiarySnapshot(900, 585, 1395, false)")
        .unwrap();
    env.state().borrow_mut().player.stats.leech_rating = 1800;
    env.exec("AssertTertiarySnapshot(900, 1800, 1395, false)")
        .unwrap();
    env.state().borrow_mut().player.stats.speed_rating = 450;
    env.exec("AssertTertiarySnapshot(900, 1800, 450, false)")
        .unwrap();
    assert_eq!(rating_snapshot(&env).0, 900);
}

#[test]
fn default_zero_is_distinguished_from_nonzero_input_and_explicit_reset() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(ASSERTIONS).unwrap();
    assert_eq!(
        (
            rating_snapshot(&env).0,
            rating_snapshot(&env).1,
            rating_snapshot(&env).2
        ),
        (0, 0, 0)
    );
    env.exec("AssertTertiarySnapshot(0, 0, 0, false)").unwrap();
    replace_ratings(&env, 2250, 585, 1395);
    env.exec("AssertTertiarySnapshot(2250, 585, 1395, false)")
        .unwrap();
    replace_ratings(&env, 0, 0, 0);
    env.exec("AssertTertiarySnapshot(0, 0, 0, false)").unwrap();
}

#[test]
fn negative_ratings_remain_raw_but_percentage_outputs_clamp_to_zero() {
    let env = fixture_env();
    env.exec("AssertTertiarySnapshot(2250, 585, 1395, false)")
        .unwrap();
    replace_ratings(&env, -2250, -585, -1395);
    for restricted in [false, true] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!(
            "AssertTertiarySnapshot(-2250, -585, -1395, {restricted})"
        ))
        .unwrap();
    }
    assert_eq!(
        (
            rating_snapshot(&env).0,
            rating_snapshot(&env).1,
            rating_snapshot(&env).2
        ),
        (-2250, -585, -1395)
    );
}

#[test]
fn environments_keep_rating_snapshots_and_restriction_inputs_isolated() {
    let first = fixture_env();
    let second = fixture_env();
    replace_ratings(&second, 900, 1800, 450);
    second.state().borrow_mut().unit_stats_restricted = true;
    first
        .exec("AssertTertiarySnapshot(2250, 585, 1395, false)")
        .unwrap();
    second
        .exec("AssertTertiarySnapshot(900, 1800, 450, true)")
        .unwrap();
    replace_ratings(&first, 180, 360, 540);
    first
        .exec("AssertTertiarySnapshot(180, 360, 540, false)")
        .unwrap();
    second
        .exec("AssertTertiarySnapshot(900, 1800, 450, true)")
        .unwrap();
    assert!(!first.state().borrow().unit_stats_restricted);
    assert!(second.state().borrow().unit_stats_restricted);
}

#[test]
fn repeated_queries_do_not_mutate_the_current_rating_snapshot() {
    let env = fixture_env();
    let before = rating_snapshot(&env);
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!(
            "for i = 1, 3 do AssertTertiarySnapshot(2250, 585, 1395, {restricted}) end"
        ))
        .unwrap();
        assert_eq!(rating_snapshot(&env), before);
        assert_eq!(env.state().borrow().unit_stats_restricted, restricted);
    }
}

#[test]
fn public_callbacks_preserve_single_result_values_across_restriction_toggle() {
    let env = fixture_env();
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!(
            "assert(C_Secrets.ShouldUnitStatsBeSecret() == {restricted}); \
             AssertTertiarySnapshot(2250, 585, 1395, {restricted})"
        ))
        .unwrap();
    }
}

#[test]
fn stamped_public_callers_keep_taint_and_receive_plain_nonzero_outputs() {
    let env = fixture_env();
    env.exec(
        r#"
        local function probe()
            assert(debug.getstacktaint() == 'TertiaryPublicProbe')
            AssertTertiarySnapshot(2250, 585, 1395, false)
            assert(debug.getstacktaint() == 'TertiaryPublicProbe')
        end
        debug.setobjecttaint(probe, 'TertiaryPublicProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertTertiarySnapshot(2250, 585, 1395, false)
        "#,
    )
    .unwrap();
}

#[test]
fn stamped_restricted_callers_receive_opaque_real_getter_outputs() {
    let env = fixture_env();
    env.state().borrow_mut().unit_stats_restricted = true;
    let before = rating_snapshot(&env);
    env.exec(
        r#"
        local function probe()
            for _, query in ipairs({
                GetAvoidance, GetLifesteal, GetSpeed,
                function() return GetCombatRating(21) end,
                function() return GetCombatRating(17) end,
                function() return GetCombatRating(13) end,
                function() return GetCombatRatingBonus(21) end,
                function() return GetCombatRatingBonus(17) end,
                function() return GetCombatRatingBonus(13) end,
            }) do
                local results = TertiaryPack(pcall(query))
                assert(results.n == 2 and results[1] == true)
                for position = 2, results.n do
                    local value = results[position]
                    assert(issecretvalue(value) and not canaccessvalue(value))
                    assert(not pcall(secretunwrap, value))
                    assert(not pcall(function() return value + 1 end))
                end
                assert(debug.getstacktaint() == 'TertiaryRestrictedProbe')
            end
        end
        debug.setobjecttaint(probe, 'TertiaryRestrictedProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertTertiarySnapshot(2250, 585, 1395, true)
        "#,
    )
    .unwrap();
    assert_eq!(rating_snapshot(&env), before);
}

#[test]
fn authentic_getter_secrets_keep_rooted_identity_after_gc_and_trusted_recovery() {
    let env = fixture_env();
    let before = rating_snapshot(&env);
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec(
        r#"
        TertiaryRoots = {GetAvoidance(), GetLifesteal(), GetSpeed()}
        TertiaryExpected = {12.5, 3.25, 7.75}
        TertiaryAliases = {unpack(TertiaryRoots)}
        collectgarbage('collect')
        collectgarbage('collect')
        for i = 1, 3 do
            local value = TertiaryRoots[i]
            assert(issecretvalue(value) and not canaccessvalue(value))
            -- NUMBER wrapper raw identity is permitted by the current VM.
            assert(rawequal(value, TertiaryAliases[i]))
            assert(secretunwrap(value) == TertiaryExpected[i])
        end
        local function probe()
            for i = 1, 3 do
                local value = TertiaryRoots[i]
                assert(issecretvalue(value) and not canaccessvalue(value))
                assert(not pcall(secretunwrap, value))
                assert(not pcall(function() return value * 2 end))
                assert(debug.getstacktaint() == 'TertiaryRootProbe')
            end
        end
        debug.setobjecttaint(probe, 'TertiaryRootProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        collectgarbage('collect')
        for i = 1, 3 do
            assert(rawequal(TertiaryRoots[i], TertiaryAliases[i]))
            assert(issecretvalue(TertiaryRoots[i]))
            assert(secretunwrap(TertiaryRoots[i]) == TertiaryExpected[i])
        end
        "#,
    )
    .unwrap();
    env.state().borrow_mut().unit_stats_restricted = false;
    env.exec(
        r#"
        AssertTertiarySnapshot(2250, 585, 1395, false)
        for i = 1, 3 do
            assert(issecretvalue(TertiaryRoots[i]))
            assert(rawequal(TertiaryRoots[i], TertiaryAliases[i]))
            assert(secretunwrap(TertiaryRoots[i]) == TertiaryExpected[i])
        end
        "#,
    )
    .unwrap();
    assert_eq!(rating_snapshot(&env), before);
}

#[test]
fn actual_equipment_recompute_replaces_snapshot_and_resets_unpopulated_inputs() {
    let env = fixture_env();
    env.exec("AssertTertiarySnapshot(2250, 585, 1395, false)")
        .unwrap();
    // Existing public admin callback invokes CharacterStats::compute and replaces stats.
    // PlayerState has no public recompute_stats method; no sticky override is implied.
    env.exec("A_Admin.UnequipItem(1)").unwrap();
    assert!(!env.state().borrow().player.equipped_items.contains_key(&1));
    assert_eq!(
        (
            rating_snapshot(&env).0,
            rating_snapshot(&env).1,
            rating_snapshot(&env).2
        ),
        (0, 0, 0)
    );
    env.exec("AssertTertiarySnapshot(0, 0, 0, false)").unwrap();
    replace_ratings(&env, 900, 1800, 450);
    env.exec("AssertTertiarySnapshot(900, 1800, 450, false)")
        .unwrap();
}

#[test]
fn explicit_ratio_queries_preserve_existing_crit_and_unknown_index_controls() {
    let env = fixture_env();
    env.exec(
        r#"
        AssertTertiarySnapshot(2250, 585, 1395, false)
        for _, fixture in ipairs({{21, 2250, 12.5}, {17, 585, 3.25}, {13, 1395, 7.75}}) do
            AssertTertiaryResult(GetCombatRatingBonusForCombatRatingValue, fixture[3], false,
                                 fixture[1], fixture[2])
            AssertTertiaryResult(GetCombatRatingBonusForCombatRatingValue, 0, false,
                                 fixture[1], -fixture[2])
        end
        AssertTertiaryResult(GetCombatRating, 360, false, 9)
        AssertTertiaryResult(GetCombatRatingBonus, 2, false, 9)
        AssertTertiaryResult(GetCombatRatingBonusForCombatRatingValue, 2, false, 9, 360)
        AssertTertiaryResult(GetCombatRating, 0, false, 999)
        AssertTertiaryResult(GetCombatRatingBonus, 0, false, 999)
        AssertTertiaryResult(GetCombatRatingBonusForCombatRatingValue, 2, false, 999, 360)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec(
        r#"
        AssertTertiarySnapshot(2250, 585, 1395, true)
        AssertTertiaryResult(GetCombatRating, 360, true, 9)
        AssertTertiaryResult(GetCombatRatingBonus, 2, true, 9)
        AssertTertiaryResult(GetCombatRating, 0, true, 999)
        AssertTertiaryResult(GetCombatRatingBonus, 0, true, 999)
        -- Explicit-value conversion is outside the output-restriction annotation.
        AssertTertiaryResult(GetCombatRatingBonusForCombatRatingValue, 12.5, false, 21, 2250)
        AssertTertiaryResult(GetCombatRatingBonusForCombatRatingValue, 2, false, 999, 360)
        "#,
    )
    .unwrap();
}
