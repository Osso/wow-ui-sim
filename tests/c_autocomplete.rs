use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn get_auto_complete_results_returns_named_priority_entries() {
    let env = WowLuaEnv::new().expect("lua env should initialize");

    let (count, name, priority): (i32, String, i32) = env
        .eval(
            r#"
            local results = C_AutoComplete.GetAutoCompleteResults("Ar", 5, 2, true, Enum.AutoCompleteEntryFlag.Friend, 0)
            return #results, results[1].name, results[1].priority
            "#,
        )
        .expect("autocomplete results should be returned");

    assert_eq!(count, 1);
    assert_eq!(name, "Arthax");
    assert_eq!(priority, 5);
}

#[test]
fn get_auto_complete_results_handles_blizzard_all_flags() {
    let env = WowLuaEnv::new().expect("lua env should initialize");

    let (match_count, miss_count): (i32, i32) = env
        .eval(
            r#"
            local includeAll = 2^32 - 1
            local matched = C_AutoComplete.GetAutoCompleteResults("Ar", 5, 0, true, includeAll, 0)
            local missed = C_AutoComplete.GetAutoCompleteResults("DefinitelyNotAName", 5, 0, true, includeAll, 0)
            return #matched, #missed
            "#,
        )
        .expect("autocomplete should accept Blizzard's unsigned all-flags value");

    assert_eq!(match_count, 1);
    assert_eq!(miss_count, 0);
}

#[test]
fn get_auto_complete_results_handles_utf8_cursor_positions() {
    let env = WowLuaEnv::new().expect("lua env should initialize");

    let count: i32 = env
        .eval(
            r#"
            local includeAll = 2^32 - 1
            local results = C_AutoComplete.GetAutoCompleteResults("Á", 5, 1, true, includeAll, 0)
            return #results
            "#,
        )
        .expect("UTF-8 cursor positions should not panic or slice invalid byte ranges");

    assert_eq!(count, 0);
}

#[test]
fn get_auto_complete_results_treats_zero_cursor_as_full_query() {
    let env = WowLuaEnv::new().expect("lua env should initialize");

    let count: i32 = env
        .eval(
            r#"
            local results = C_AutoComplete.GetAutoCompleteResults("zzzz", 5, 0, true, Enum.AutoCompleteEntryFlag.Friend, 0)
            return #results
            "#,
        )
        .expect("zero cursor should not match every candidate");

    assert_eq!(count, 0);
}

#[test]
fn get_auto_complete_results_respects_exclude_and_limit() {
    let env = WowLuaEnv::new().expect("lua env should initialize");

    let (excluded_count, limited_count): (i32, i32) = env
        .eval(
            r#"
            local excluded = C_AutoComplete.GetAutoCompleteResults("", 5, 0, true, Enum.AutoCompleteEntryFlag.Friend, Enum.AutoCompleteEntryFlag.Friend)
            local limited = C_AutoComplete.GetAutoCompleteResults("", 1, 0, true, Enum.AutoCompleteEntryFlag.Friend, 0)
            return #excluded, #limited
            "#,
        )
        .expect("autocomplete filtering should be applied");

    assert_eq!(excluded_count, 0);
    assert_eq!(limited_count, 1);
}

#[test]
fn is_recognized_name_matches_exact_flag_filtered_candidates() {
    let env = WowLuaEnv::new().expect("lua env should initialize");

    let (exact, folded, prefix, excluded, bnet, unknown): (bool, bool, bool, bool, bool, bool) =
        env.eval(
            r#"
            local friend = Enum.AutoCompleteEntryFlag.Friend
            local includeAll = 2^32 - 1
            return C_AutoComplete.IsRecognizedName("Arthax", friend, 0),
                C_AutoComplete.IsRecognizedName("arTHAX", friend, 0),
                C_AutoComplete.IsRecognizedName("Arth", friend, 0),
                C_AutoComplete.IsRecognizedName("Arthax", friend, friend),
                C_AutoComplete.IsRecognizedName("Thrall", includeAll, 0),
                C_AutoComplete.IsRecognizedName("DefinitelyNotAName", includeAll, 0)
            "#,
        )
        .expect("IsRecognizedName should be callable");

    assert!(exact && folded);
    assert!(!prefix, "a prefix is not a recognized name");
    assert!(!excluded, "excluded flags hide the friend");
    assert!(bnet, "Battle.net account names are candidates");
    assert!(!unknown);
}

#[test]
fn get_auto_complete_presence_id_resolves_bnet_account_names() {
    let env = WowLuaEnv::new().expect("lua env should initialize");

    let (thrall, folded, uther, full_tag, character): (f64, f64, f64, bool, bool) = env
        .eval(
            r#"
            return C_AutoComplete.GetAutoCompletePresenceID("Thrall"),
                C_AutoComplete.GetAutoCompletePresenceID("thrall"),
                C_AutoComplete.GetAutoCompletePresenceID("Uther"),
                C_AutoComplete.GetAutoCompletePresenceID("Uther#1000") == nil,
                C_AutoComplete.GetAutoCompletePresenceID("Arthax") == nil
            "#,
        )
        .expect("GetAutoCompletePresenceID should be callable");

    assert_eq!(thrall, 100002.0);
    assert_eq!(folded, 100002.0);
    assert_eq!(uther, 100001.0);
    assert!(
        full_tag,
        "INFERRED: the numbered BattleTag is not a presence name"
    );
    assert!(
        character,
        "character-only friends have no Battle.net presence"
    );
}
