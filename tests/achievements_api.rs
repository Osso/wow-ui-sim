//! Tests for achievements: earn/remove via admin API, query via GetAchievementInfo.

use wow_ui_sim::event::EventArg;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AchievementInfo;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn test_achievement_not_completed_by_default() {
    let env = env();
    let completed: bool = env
        .eval("local _, _, _, c = GetAchievementInfo(6); return c")
        .unwrap();
    assert!(!completed);
}

#[test]
fn test_earn_achievement() {
    let env = env();
    env.exec("A_Admin.SetAchievementEarned(6, true)").unwrap();
    let completed: bool = env
        .eval("local _, _, _, c = GetAchievementInfo(6); return c")
        .unwrap();
    assert!(completed);
}

#[test]
fn test_remove_achievement() {
    let env = env();
    env.exec("A_Admin.SetAchievementEarned(6, true)").unwrap();
    env.exec("A_Admin.SetAchievementEarned(6, false)").unwrap();
    let completed: bool = env
        .eval("local _, _, _, c = GetAchievementInfo(6); return c")
        .unwrap();
    assert!(!completed);
}

#[test]
fn test_has_achievement_false_by_default() {
    let env = env();
    let has: bool = env.eval("return A_Admin.HasAchievement(999)").unwrap();
    assert!(!has);
}

#[test]
fn test_has_achievement_true_after_earning() {
    let env = env();
    env.exec("A_Admin.SetAchievementEarned(999, true)").unwrap();
    let has: bool = env.eval("return A_Admin.HasAchievement(999)").unwrap();
    assert!(has);
}

#[test]
fn test_multiple_achievements_independent() {
    let env = env();
    env.exec("A_Admin.SetAchievementEarned(100, true)").unwrap();
    env.exec("A_Admin.SetAchievementEarned(200, true)").unwrap();
    let (a, b, c): (bool, bool, bool) = env
        .eval(
            "return A_Admin.HasAchievement(100), \
                    A_Admin.HasAchievement(200), \
                    A_Admin.HasAchievement(300)",
        )
        .unwrap();
    assert!(a);
    assert!(b);
    assert!(!c);
}

#[test]
fn test_achievement_info_returns_id() {
    let env = env();
    let id: i64 = env
        .eval("local id = GetAchievementInfo(42); return id")
        .unwrap();
    assert_eq!(id, 42);
}

// ============================================================================
// Achievement Categories
// ============================================================================

#[test]
fn test_get_category_list_returns_expanded_blizzard_category_set() {
    let env = env();
    let (count, fourth, penultimate, last): (i32, i32, i32, i32) = env
        .eval("local ids = GetCategoryList(); return #ids, ids[4], ids[#ids - 1], ids[#ids]")
        .unwrap();
    assert_eq!(count, 13);
    assert_eq!(fourth, 15522);
    assert_eq!(penultimate, 15246);
    assert_eq!(last, 81);
}

#[test]
fn test_get_category_info_general() {
    let env = env();
    let (name, parent, flags): (String, i32, i32) = env
        .eval("local n, p, f = GetCategoryInfo(92); return n, p, f")
        .unwrap();
    assert_eq!(name, "General");
    assert_eq!(parent, -1);
    assert_eq!(flags, 0);
}

#[test]
fn test_get_category_info_feats_of_strength() {
    let env = env();
    let name: String = env.eval("local n = GetCategoryInfo(81); return n").unwrap();
    assert_eq!(name, "Feats of Strength");
}

#[test]
fn test_get_category_info_unknown_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("local n = GetCategoryInfo(99999); return n == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_category_num_achievements_returns_three_values() {
    let env = env();
    let (total, completed, incomplete): (i32, i32, i32) =
        env.eval("return GetCategoryNumAchievements(92)").unwrap();
    assert_eq!(total, 6); // 6 General achievements (Level 10-80)
    assert_eq!(completed, 0);
    assert_eq!(incomplete, 6);
}

#[test]
fn test_get_guild_category_list_matches_blizzard_shape() {
    let env = env();
    let (count, first, last): (i32, i32, i32) = env
        .eval("local ids = GetGuildCategoryList(); return #ids, ids[1], ids[#ids]")
        .unwrap();
    assert_eq!(count, 8);
    assert_eq!(first, 15076);
    assert_eq!(last, 15093);
}

#[test]
fn test_get_statistics_category_list_includes_root_and_children() {
    let env = env();
    let (count, first, second): (i32, i32, i32) = env
        .eval("local ids = GetStatisticsCategoryList(); return #ids, ids[1], ids[2]")
        .unwrap();
    assert_eq!(count, 5);
    assert_eq!(first, 130);
    assert_eq!(second, 1);
}

#[test]
fn test_category_info_reports_parent_for_nested_category() {
    let env = env();
    let (name, parent, flags): (String, i32, i32) = env
        .eval("local n, p, f = GetCategoryInfo(202); return n, p, f")
        .unwrap();
    assert_eq!(name, "Exalted Reputations");
    assert_eq!(parent, 201);
    assert_eq!(flags, 0);
}

#[test]
fn test_category_num_achievements_counts_nested_children() {
    let env = env();
    let (total, completed, incomplete): (i32, i32, i32) =
        env.eval("return GetCategoryNumAchievements(201)").unwrap();
    assert_eq!(total, 2);
    assert_eq!(completed, 0);
    assert_eq!(incomplete, 2);
}

#[test]
fn test_category_achievement_points_sum_completed_nested_children() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.achievements.insert(
            948,
            AchievementInfo {
                achievement_id: 948,
                name: "Ambassador of the Alliance".into(),
                points: 10,
                description: "Earn exalted reputation with alliance cities.".into(),
                flags: 0,
                icon: 0,
                reward_text: String::new(),
                is_guild: false,
                is_statistic: false,
                reward_item_id: None,
            },
        );
        state.achievements.insert(
            1017,
            AchievementInfo {
                achievement_id: 1017,
                name: "Can I Keep Him?".into(),
                points: 10,
                description: "Collect a companion pet.".into(),
                flags: 0,
                icon: 0,
                reward_text: String::new(),
                is_guild: false,
                is_statistic: false,
                reward_item_id: None,
            },
        );
    }
    env.exec("A_Admin.SetAchievementEarned(948, true)").unwrap();
    env.exec("A_Admin.SetAchievementEarned(1017, true)")
        .unwrap();
    let (parent_total, child_excluded, missing): (i32, i32, i32) = env
        .eval(
            r#"
            return GetCategoryAchievementPoints(201, true),
                   GetCategoryAchievementPoints(201, false),
                   GetCategoryAchievementPoints(999999, true)
            "#,
        )
        .unwrap();

    assert_eq!(parent_total, 20);
    assert_eq!(child_excluded, 10);
    assert_eq!(missing, 0);
}

#[test]
fn test_achievement_info_returns_real_name() {
    let env = env();
    let name: String = env
        .eval("local _, name = GetAchievementInfo(6); return name")
        .unwrap();
    assert_eq!(name, "Level 10");
}

#[test]
fn test_incomplete_achievement_returns_nil_completion_date_fields() {
    let env = env();
    let (month_is_empty, day_is_empty, year_is_empty): (bool, bool, bool) = env
        .eval(
            r#"
            local _, _, _, _, month, day, year = GetAchievementInfo(6)
            return month == nil or month == 0, day == nil or day == 0, year == nil or year == 0
            "#,
        )
        .unwrap();
    assert!(month_is_empty);
    assert!(day_is_empty);
    assert!(year_is_empty);
}

#[test]
fn test_general_summary_seed_has_display_data_for_default_ids() {
    let env = env();
    let seeded_names_present: bool = env
        .eval(
            r#"
            local ids = {6, 7, 8, 9, 10, 11}
            local seededCount = 0
            for _, id in ipairs(ids) do
                local _, name = GetAchievementInfo(id)
                if type(name) == "string" and name ~= "" then
                    seededCount = seededCount + 1
                end
            end
            return seededCount > 0
            "#,
        )
        .unwrap();
    assert!(seeded_names_present);
}

#[test]
fn test_achievement_info_explore_elwynn() {
    let env = env();
    let (name, points): (String, i32) = env
        .eval("local _, name, pts = GetAchievementInfo(776); return name, pts")
        .unwrap();
    assert_eq!(name, "Explore Elwynn Forest");
    assert_eq!(points, 10);
}

#[test]
fn test_achievement_info_full_signature() {
    let env = env();
    env.exec("A_Admin.SetAchievementEarned(776, true)").unwrap();
    let (
        id,
        name,
        points,
        completed,
        month,
        day,
        year,
        desc,
        flags,
        icon,
        reward,
        is_guild,
        was_earned,
    ): (
        i32,
        String,
        i32,
        bool,
        i32,
        i32,
        i32,
        String,
        i32,
        i32,
        String,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local id, name, pts, comp, m, d, y, desc, fl, ic, rw, ig, we = GetAchievementInfo(776)
            return id, name, pts, comp, m, d, y, desc, fl, ic, rw, ig, we
            "#,
        )
        .unwrap();
    assert_eq!(id, 776);
    assert_eq!(name, "Explore Elwynn Forest");
    assert_eq!(points, 10);
    assert!(completed);
    assert_eq!(month, 1);
    assert_eq!(day, 15);
    assert_eq!(year, 2025);
    assert_eq!(
        desc,
        "Explore Elwynn Forest, revealing the covered areas of the world map."
    );
    assert_eq!(flags, 0);
    assert_eq!(icon, 236809);
    assert_eq!(reward, "");
    assert!(!is_guild);
    assert!(was_earned);
}

#[test]
fn test_category_num_updates_with_earned() {
    let env = env();
    env.exec("A_Admin.SetAchievementEarned(6, true)").unwrap();
    let (total, completed, incomplete): (i32, i32, i32) =
        env.eval("return GetCategoryNumAchievements(92)").unwrap();
    assert_eq!(total, 6);
    assert_eq!(completed, 1);
    assert_eq!(incomplete, 5);
}

#[test]
fn test_has_completed_any_achievement_is_available_for_ui_gating() {
    let env = env();
    let completed_any: bool = env.eval("return HasCompletedAnyAchievement()").unwrap();
    assert!(completed_any);
}

#[test]
fn test_can_show_achievement_ui_is_enabled() {
    let env = env();
    let can_show: bool = env.eval("return CanShowAchievementUI()").unwrap();
    assert!(can_show);
}

#[test]
fn test_get_achievement_category_returns_seeded_category() {
    let env = env();
    let (general, exploration, pvp): (i32, i32, i32) = env
        .eval(
            r#"
            return GetAchievementCategory(6),
                   GetAchievementCategory(776),
                   GetAchievementCategory(558)
            "#,
        )
        .unwrap();
    assert_eq!(general, 92);
    assert_eq!(exploration, 97);
    assert_eq!(pvp, 95);
}

#[test]
fn test_previous_and_next_achievement_follow_category_order() {
    let env = env();
    let (prev, next_id, next_complete): (i32, i32, bool) = env
        .eval(
            r#"
            local prev = GetPreviousAchievement(7)
            local next_id, completed = GetNextAchievement(7)
            return prev, next_id, completed
            "#,
        )
        .unwrap();
    assert_eq!(prev, 6);
    assert_eq!(next_id, 8);
    assert!(!next_complete);
}

#[test]
fn test_previous_and_next_achievement_handle_category_edges_and_completion() {
    let env = env();
    env.exec("A_Admin.SetAchievementEarned(8, true)").unwrap();
    let (prev_is_nil, next_id, next_complete, last_next_is_nil): (bool, i32, bool, bool) = env
        .eval(
            r#"
            local prev = GetPreviousAchievement(6)
            local next_id, completed = GetNextAchievement(7)
            local last_next = GetNextAchievement(11)
            return prev == nil, next_id, completed, last_next == nil
            "#,
        )
        .unwrap();
    assert!(prev_is_nil);
    assert_eq!(next_id, 8);
    assert!(next_complete);
    assert!(last_next_is_nil);
}

#[test]
fn test_latest_completed_achievements_returns_varargs_ids() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetAchievementEarned(776, true)
        A_Admin.SetAchievementEarned(6, true)
        "#,
    )
    .unwrap();
    let (count, first, second): (i32, i32, i32) = env
        .eval(
            r##"
            local first, second = GetLatestCompletedAchievements(false)
            return select("#", GetLatestCompletedAchievements(false)), first, second
            "##,
        )
        .unwrap();
    assert_eq!(count, 2);
    assert_eq!(first, 6);
    assert_eq!(second, 776);
}

#[test]
fn test_get_achievement_guild_rep_defaults_to_false_for_non_guild_achievements() {
    let env = env();
    let (requires_rep, has_rep, rep_level_is_nil): (bool, bool, bool) = env
        .eval(
            r#"
            local r, h, lvl = GetAchievementGuildRep(6)
            return r, h, lvl == nil
            "#,
        )
        .unwrap();
    assert!(!requires_rep);
    assert!(!has_rep);
    assert!(rep_level_is_nil);
}

// ============================================================================
// GetAchievementCriteriaInfo
// ============================================================================

#[test]
fn test_achievement_num_criteria() {
    let env = env();
    let count: i32 = env.eval("return GetAchievementNumCriteria(948)").unwrap();
    assert_eq!(count, 5); // Ambassador: 5 factions
}

#[test]
fn test_achievement_num_criteria_returns_zero_for_seeded_rows_without_criteria() {
    let env = env();
    let count: i32 = env.eval("return GetAchievementNumCriteria(6)").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_achievement_criteria_info_returns_name() {
    let env = env();
    let name: String = env
        .eval("return GetAchievementCriteriaInfo(948, 1)")
        .unwrap();
    assert_eq!(name, "Exalted with Stormwind");
}

#[test]
fn test_achievement_criteria_info_matches_blizzard_multiret_shape() {
    let env = env();
    let (
        name,
        criteria_type,
        completed,
        quantity,
        required_quantity,
        char_name,
        criteria_flags,
        asset_id,
        quantity_string,
        criteria_id,
        eligible,
        duration,
        elapsed,
    ): (
        String,
        i32,
        bool,
        i32,
        i32,
        String,
        i32,
        i32,
        String,
        i32,
        bool,
        i32,
        i32,
    ) = env
        .eval(
            r#"
            local name, criteriaType, completed, quantity, reqQuantity, charName, criteriaFlags, assetID, quantityString, criteriaID, eligible, duration, elapsed =
                GetAchievementCriteriaInfo(948, 1)
            return name, criteriaType, completed, quantity, reqQuantity, charName, criteriaFlags, assetID, quantityString, criteriaID, eligible, duration, elapsed
            "#,
        )
        .unwrap();
    assert_eq!(name, "Exalted with Stormwind");
    assert_eq!(criteria_type, 0);
    assert!(!completed);
    assert_eq!(quantity, 0);
    assert_eq!(required_quantity, 1);
    assert_eq!(char_name, "");
    assert_eq!(criteria_flags, 0);
    assert_eq!(asset_id, 0);
    assert_eq!(quantity_string, "0/1");
    assert_eq!(criteria_id, 5331);
    assert!(eligible);
    assert_eq!(duration, 0);
    assert_eq!(elapsed, 0);
}

#[test]
fn test_achievement_criteria_info_defaults_progress_to_zero_before_earning() {
    let env = env();
    let (name, completed, qty, req): (String, bool, i32, i32) = env
        .eval(
            r#"
            local n, _, c, q, r = GetAchievementCriteriaInfo(948, 2)
            return n, c, q, r
            "#,
        )
        .unwrap();
    assert_eq!(name, "Exalted with Ironforge");
    assert!(!completed);
    assert_eq!(qty, 0);
    assert_eq!(req, 1);
}

#[test]
fn test_achievement_criteria_completed_tracks_earned() {
    let env = env();
    env.exec("A_Admin.SetAchievementEarned(513, true)").unwrap();
    let (name, completed, qty, req): (String, bool, i32, i32) = env
        .eval(
            r#"
            local n, _, c, q, r = GetAchievementCriteriaInfo(513, 1)
            return n, c, q, r
            "#,
        )
        .unwrap();
    assert_eq!(name, "Honorable kills");
    assert!(completed);
    assert_eq!(qty, 100);
    assert_eq!(req, 100);
}

#[test]
fn test_achievement_criteria_nil_for_invalid_index() {
    let env = env();
    let is_nil: bool = env
        .eval("return GetAchievementCriteriaInfo(6, 99) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_achievement_criteria_nil_for_unknown_achievement() {
    let env = env();
    let is_nil: bool = env
        .eval("return GetAchievementCriteriaInfo(999999, 1) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_achievement_criteria_by_id_matches_index_tuple_not_index_position() {
    let env = env();
    let (stormwind_id, ironforge_id, tuples_equal): (i32, i32, bool) = env
        .eval(
            r#"
            local function same_tuple(achievementID, index, criteriaID)
                local indexed = { GetAchievementCriteriaInfo(achievementID, index) }
                local byID = { GetAchievementCriteriaInfoByID(achievementID, criteriaID) }
                for i = 1, 13 do
                    if indexed[i] ~= byID[i] then return false end
                end
                return true
            end
            local _, _, _, _, _, _, _, _, _, stormwindID = GetAchievementCriteriaInfo(948, 1)
            local _, _, _, _, _, _, _, _, _, ironforgeID = GetAchievementCriteriaInfo(948, 2)
            return stormwindID, ironforgeID,
                same_tuple(948, 1, 5331) and same_tuple(948, 2, 5330)
            "#,
        )
        .unwrap();
    assert_eq!(stormwind_id, 5331);
    assert_eq!(ironforge_id, 5330);
    assert!(tuples_equal);
}

#[test]
fn test_achievement_criteria_by_id_tracks_completion_of_same_criterion() {
    let env = env();
    env.exec("A_Admin.SetAchievementEarned(513, true)").unwrap();
    let (name, completed, quantity, required, criteria_id): (String, bool, i32, i32, i32) = env
        .eval(
            r#"
            local name, _, completed, quantity, required, _, _, _, _, criteriaID =
                GetAchievementCriteriaInfoByID(513, 13253)
            return name, completed, quantity, required, criteriaID
            "#,
        )
        .unwrap();
    assert_eq!(name, "Honorable kills");
    assert!(completed);
    assert_eq!((quantity, required, criteria_id), (100, 100, 13253));
}

#[test]
fn test_achievement_criteria_by_id_returns_nil_for_missing_id_or_achievement() {
    let env = env();
    let (index_not_id, foreign_id, unknown_id, unknown_achievement, no_criteria): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            return GetAchievementCriteriaInfoByID(948, 1) == nil,
                GetAchievementCriteriaInfoByID(513, 5331) == nil,
                GetAchievementCriteriaInfoByID(948, 999999) == nil,
                GetAchievementCriteriaInfoByID(999999, 5331) == nil,
                GetAchievementCriteriaInfoByID(6, 5331) == nil
            "#,
        )
        .unwrap();
    assert!(index_not_id && foreign_id && unknown_id && unknown_achievement && no_criteria);
}

// ============================================================================
// A_Admin.EarnAchievement
// ============================================================================

#[test]
fn test_earn_achievement_sets_earned_and_fires_event() {
    let env = env();
    let completed: bool = env
        .eval(
            r#"
            A_Admin.EarnAchievement(6)
            local _, _, _, c = GetAchievementInfo(6)
            return c
            "#,
        )
        .unwrap();
    let event_fired = {
        let state = env.state();
        let state = state.borrow();
        state.events.pending().iter().any(|event| {
            event.name == "ACHIEVEMENT_EARNED"
                && matches!(event.args.as_slice(), [EventArg::Number(id)] if *id == 6.0)
        })
    };
    assert!(completed);
    assert!(event_fired);
}

#[test]
fn test_earn_achievement_does_not_fire_duplicate_event() {
    let env = env();
    env.exec(
        r#"
        A_Admin.EarnAchievement(6)
        A_Admin.EarnAchievement(6)
        "#,
    )
    .unwrap();

    let matching_events = {
        let state = env.state();
        let state = state.borrow();
        state
            .events
            .pending()
            .iter()
            .filter(|event| {
                event.name == "ACHIEVEMENT_EARNED"
                    && matches!(event.args.as_slice(), [EventArg::Number(id)] if *id == 6.0)
            })
            .count()
    };

    assert_eq!(matching_events, 1);
}
