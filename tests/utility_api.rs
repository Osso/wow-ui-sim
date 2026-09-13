//! Tests for utility API functions (utility_api.rs).

use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "utility_api/error_handler.rs"]
mod error_handler;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(feature = "retail-12-0-0")]
fn assert_string_trim_default_cases(cases: &str) {
    let result: String = env()
        .eval(&format!(
            r#"
            local failures = {{}}
            local cases = {cases}
            for _, name in ipairs({{"string.trim", "strtrim"}}) do
                local trim = string.trim
                if name == "strtrim" then trim = strtrim end
                assert(type(trim) == "function", name .. " not published")
                for index, case in ipairs(cases) do
                    for mode = 1, 2 do
                        local actual
                        if mode == 1 then actual = trim(case[1])
                        else actual = trim(case[1], nil) end
                        if actual ~= case[2] then
                            failures[#failures + 1] = string.format(
                                "%s case %d %s: expected %q, got %q",
                                name, index, mode == 1 and "omitted" or "nil",
                                case[2], tostring(actual))
                        end
                    end
                end
            end
            return table.concat(failures, "\n")
            "#,
        ))
        .unwrap();
    assert_eq!(result, "");
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn test_string_trim_default_four_bytes_and_embedded_whitespace() {
    assert_string_trim_default_cases(
        r#"{
            {" hello ", "hello"},
            {"\rhello\r", "hello"},
            {"\nhello\n", "hello"},
            {"\thello\t", "hello"},
            {" \r\n\thello\t\n\r ", "hello"},
            {" \r\n\t", ""},
            {"", ""},
            {" \tleft \r\n\t\011\012 right\n ", "left \r\n\t\011\012 right"},
        }"#,
    );
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn test_string_trim_default_preserves_edge_vertical_tab_and_form_feed() {
    assert_string_trim_default_cases(
        r#"{
            {"\011hello\011", "\011hello\011"},
            {"\012hello\012", "\012hello\012"},
            {"\011\012", "\011\012"},
        }"#,
    );
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn test_string_trim_default_stops_at_vertical_tab_and_form_feed() {
    assert_string_trim_default_cases(
        r#"{
            {" \r\n\t\011 hello \011\t\n\r ", "\011 hello \011"},
            {" \r\n\t\012 hello \012\t\n\r ", "\012 hello \012"},
            {"\011 \thello\r\n ", "\011 \thello"},
            {" \r\nhello\t \012", "hello\t \012"},
        }"#,
    );
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn test_string_trim_custom_character_set_control() {
    env()
        .eval::<()>(
            r#"
        assert(type(string.trim) == "function")
        assert(type(strtrim) == "function")
        for _, trim in ipairs({string.trim, strtrim}) do
            assert(trim("xyhelloyx", "xy") == "hello")
            assert(trim("xy hello yx", "xy") == " hello ")
            assert(trim("xyhexyl loyx", "xy") == "hexyl lo")
            assert(trim(" \txy\n ", "") == " \txy\n ")
        end
        "#,
        )
        .unwrap();
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn test_string_trim_one_string_return_control() {
    env()
        .eval::<()>(
            r#"
        assert(type(string.trim) == "function")
        assert(type(strtrim) == "function")
        local function check(...)
            assert(select('#', ...) == 1)
            assert(type((...)) == "string")
        end
        for _, trim in ipairs({string.trim, strtrim}) do
            check(trim(" hello "))
            check(trim(" hello ", nil))
            check(trim("xyhelloyx", "xy"))
            check(trim(" \r\n\t"))
            check(trim("", ""))
        end
        "#,
        )
        .unwrap();
}

// ============================================================================
// strsplit
// ============================================================================

#[test]
fn test_strsplit_basic() {
    let env = env();
    let result: (String, String, String) = env.eval("return strsplit(',', 'a,b,c')").unwrap();
    assert_eq!(result, ("a".into(), "b".into(), "c".into()));
}

#[test]
fn test_strsplit_with_limit() {
    let env = env();
    let result: (String, String) = env.eval("return strsplit(',', 'a,b,c', 2)").unwrap();
    assert_eq!(result, ("a".into(), "b,c".into()));
}

#[test]
fn test_strsplit_no_delimiter_found() {
    let env = env();
    let result: String = env.eval("return strsplit(',', 'abc')").unwrap();
    assert_eq!(result, "abc");
}

#[test]
fn test_strsplit_empty_string() {
    let env = env();
    let result: String = env.eval("return strsplit(',', '')").unwrap();
    assert_eq!(result, "");
}

// ============================================================================
// getglobal / setglobal
// ============================================================================

#[test]
fn test_getglobal_setglobal() {
    let env = env();
    env.eval::<()>("setglobal('MY_TEST_VAR', 42)").unwrap();
    let val: i32 = env.eval("return getglobal('MY_TEST_VAR')").unwrap();
    assert_eq!(val, 42);
}

#[test]
fn test_getglobal_nil_for_missing() {
    let env = env();
    let is_nil: bool = env
        .eval("return getglobal('NONEXISTENT_VAR_XYZ') == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// loadstring
// ============================================================================

#[test]
fn test_loadstring_valid_code() {
    let env = env();
    let result: i32 = env
        .eval("local f = loadstring('return 1 + 2'); return f()")
        .unwrap();
    assert_eq!(result, 3);
}

#[test]
fn test_loadstring_syntax_error() {
    let env = env();
    let is_nil: bool = env
        .eval("local f, err = loadstring('invalid @@@ code'); return f == nil")
        .unwrap();
    assert!(is_nil);
    let has_err: bool = env
        .eval("local f, err = loadstring('invalid @@@ code'); return err ~= nil")
        .unwrap();
    assert!(has_err);
}

// ============================================================================
// wipe / table.wipe
// ============================================================================

#[test]
fn test_wipe_clears_table() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local t = {1, 2, 3, a = "b"}
            wipe(t)
            local n = 0
            for _ in pairs(t) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_wipe_returns_table() {
    let env = env();
    let same: bool = env
        .eval(
            r#"
            local t = {1, 2, 3}
            local r = wipe(t)
            return t == r
            "#,
        )
        .unwrap();
    assert!(same);
}

#[test]
fn test_table_wipe_alias() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local t = {1, 2, 3}
            table.wipe(t)
            local n = 0
            for _ in pairs(t) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_table_create_returns_empty_mutable_tables_for_capacity_variants() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local tables = {
                table.create(),
                table.create(3),
                table.create(0, 4),
                table.create(3, 4),
            }
            for index, value in ipairs(tables) do
                if type(value) ~= "table" or next(value) ~= nil then
                    return "initial:" .. index
                end
                value[3] = "array"
                value.key = "hash"
                if value[3] ~= "array" or value.key ~= "hash" then
                    return "mutation:" .. index
                end
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

// ============================================================================
// tinsert / tremove
// ============================================================================

#[test]
fn test_tinsert_append() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local t = {1, 2}
            tinsert(t, 3)
            return t[3]
            "#,
        )
        .unwrap();
    assert_eq!(val, 3);
}

#[test]
fn test_tinsert_at_position() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local t = {1, 3}
            tinsert(t, 2, 99)
            return t[2]
            "#,
        )
        .unwrap();
    assert_eq!(val, 99);
}

#[test]
fn test_tremove() {
    let env = env();
    let (removed, len): (i32, i32) = env
        .eval(
            r#"
            local t = {10, 20, 30}
            local v = tremove(t, 1)
            return v, #t
            "#,
        )
        .unwrap();
    assert_eq!(removed, 10);
    assert_eq!(len, 2);
}

// ============================================================================
// tInvert
// ============================================================================

#[test]
fn test_tinvert() {
    let env = env();
    let result: (String, String) = env
        .eval(
            r#"
            local t = {a = "x", b = "y"}
            local inv = tInvert(t)
            return inv.x, inv.y
            "#,
        )
        .unwrap();
    assert_eq!(result, ("a".into(), "b".into()));
}

// ============================================================================
// tContains
// ============================================================================

#[test]
fn test_tcontains_found() {
    let env = env();
    let found: bool = env.eval("return tContains({10, 20, 30}, 20)").unwrap();
    assert!(found);
}

#[test]
fn test_tcontains_not_found() {
    let env = env();
    let found: bool = env.eval("return tContains({10, 20, 30}, 99)").unwrap();
    assert!(!found);
}

#[test]
fn test_tcontains_checks_hash_entries_too() {
    let env = env();
    let found: bool = env
        .eval(
            r#"
            local t = {}
            t.onlyHash = 42
            return tContains(t, 42)
            "#,
        )
        .unwrap();
    assert!(found, "tContains should match Blizzard pairs() semantics");
}

// ============================================================================
// tIndexOf
// ============================================================================

#[test]
fn test_tindexof_found() {
    let env = env();
    let idx: i32 = env.eval("return tIndexOf({10, 20, 30}, 20)").unwrap();
    assert_eq!(idx, 2);
}

#[test]
fn test_tindexof_not_found() {
    let env = env();
    let is_nil: bool = env
        .eval("return tIndexOf({10, 20, 30}, 99) == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// tFilter
// ============================================================================

#[test]
fn test_tfilter_removes_matching() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local t = {1, 2, 3, 4, 5}
            tFilter(t, function(v) return v > 3 end)
            local n = 0
            for _ in pairs(t) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 2); // only 4 and 5 remain
}

// ============================================================================
// CopyTable
// ============================================================================

#[test]
fn test_copytable_shallow() {
    let env = env();
    let (a, b, independent): (i32, i32, bool) = env
        .eval(
            r#"
            local orig = {x = 1, y = 2}
            local copy = CopyTable(orig)
            copy.x = 99
            return orig.x, copy.x, orig.x ~= copy.x
            "#,
        )
        .unwrap();
    assert_eq!(a, 1);
    assert_eq!(b, 99);
    assert!(independent);
}

#[test]
fn test_copytable_deep() {
    let env = env();
    let independent: bool = env
        .eval(
            r#"
            local orig = {inner = {val = 10}}
            local copy = CopyTable(orig)
            copy.inner.val = 99
            return orig.inner.val == 10
            "#,
        )
        .unwrap();
    assert!(independent);
}

// ============================================================================
// MergeTable
// ============================================================================

#[test]
fn test_mergetable() {
    let env = env();
    let (a, b): (i32, i32) = env
        .eval(
            r#"
            local dest = {a = 1}
            local src = {b = 2}
            MergeTable(dest, src)
            return dest.a, dest.b
            "#,
        )
        .unwrap();
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}

#[test]
fn test_mergetable_overwrites() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local dest = {a = 1}
            local src = {a = 99}
            MergeTable(dest, src)
            return dest.a
            "#,
        )
        .unwrap();
    assert_eq!(val, 99);
}

// ============================================================================
// nop / sound stubs
// ============================================================================

#[test]
fn test_nop_exists() {
    let env = env();
    let is_func: bool = env.eval("return type(nop) == 'function'").unwrap();
    assert!(is_func);
}

#[test]
fn test_sound_stubs_exist() {
    let env = env();
    for func in &["PlaySound", "StopSound", "PlaySoundFile"] {
        let is_func: bool = env
            .eval(&format!("return type({}) == 'function'", func))
            .unwrap();
        assert!(is_func, "{} should be a function", func);
    }
}

#[test]
fn test_sound_shims_record_latest_requests() {
    let env = env();
    env.eval::<()>(
        r#"
        PlaySound(839)
        PlaySoundFile("sound/interface/test.ogg")
        StopSound(17)
        "#,
    )
    .unwrap();

    let sim = env.state().borrow();
    assert_eq!(
        sim.last_sound_kit_requested,
        Some(839),
        "PlaySound should record the latest requested sound kit"
    );
    assert_eq!(
        sim.last_sound_file_requested.as_deref(),
        Some("sound/interface/test.ogg"),
        "PlaySoundFile should record the latest requested file path"
    );
    assert_eq!(
        sim.last_stopped_sound_handle,
        Some(17),
        "StopSound should record the latest stopped handle"
    );
}

#[test]
fn launch_url_records_request_on_sim_state() {
    // StaticPopupDialogs["ACCOUNT_SAVE_SUCCESS"].OnAccept calls LaunchURL(data)
    // when the save success popup is accepted (Blizzard_AccountSaveUI.lua line 90).
    // The simulator never opens an external browser; tests assert the URL.
    let env = env();
    env.eval::<()>(r#"LaunchURL("https://example.com/account-save")"#)
        .unwrap();
    assert_eq!(
        env.state().borrow().last_launched_url.as_deref(),
        Some("https://example.com/account-save"),
        "LaunchURL should record the requested URL on SimState"
    );
}

#[test]
fn launch_url_overwrites_previous_request() {
    let env = env();
    env.eval::<()>(
        r#"
        LaunchURL("https://first.example/")
        LaunchURL("https://second.example/")
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().last_launched_url.as_deref(),
        Some("https://second.example/"),
        "LaunchURL should record only the latest URL"
    );
}

#[test]
fn test_map_scene_character_highlight_shims_track_guid() {
    let env = env();
    env.eval::<()>(r#"MapSceneCharacterHighlightStart("Player-1-0000BEEF")"#)
        .unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .highlighted_map_scene_character_guid
            .as_deref(),
        Some("Player-1-0000BEEF"),
        "highlight start should store the active character guid"
    );

    env.eval::<()>(r#"MapSceneCharacterHighlightEnd("Player-1-0000BEEF")"#)
        .unwrap();
    assert!(
        env.state()
            .borrow()
            .highlighted_map_scene_character_guid
            .is_none(),
        "highlight end should clear the active character guid"
    );
}

#[test]
fn test_can_hearth_and_resurrect_from_area_tracks_player_state() {
    let env = env();
    let unavailable: bool = env.eval("return CanHearthAndResurrectFromArea()").unwrap();
    assert!(
        !unavailable,
        "area resurrection should stay unavailable without a pending resurrect"
    );

    {
        let mut sim = env.state().borrow_mut();
        sim.pending_resurrect = Some("Tyrande".into());
    }
    let available: bool = env.eval("return CanHearthAndResurrectFromArea()").unwrap();
    assert!(
        available,
        "a pending resurrect plus the default teleport/hearth state should enable the probe"
    );

    {
        let mut sim = env.state().borrow_mut();
        sim.has_hearthstone = false;
    }
    let without_hearthstone: bool = env.eval("return CanHearthAndResurrectFromArea()").unwrap();
    assert!(
        !without_hearthstone,
        "the probe should turn off when the player cannot hearth from the area"
    );

    {
        let mut sim = env.state().borrow_mut();
        sim.has_hearthstone = true;
        sim.can_teleport = false;
    }
    let without_teleport: bool = env.eval("return CanHearthAndResurrectFromArea()").unwrap();
    assert!(
        !without_teleport,
        "the probe should turn off when teleport/hearth travel is disabled"
    );
}

// ============================================================================
// String library aliases
// ============================================================================

#[test]
fn test_strlen_alias() {
    let env = env();
    let len: i32 = env.eval("return strlen('hello')").unwrap();
    assert_eq!(len, 5);
}

#[test]
fn test_strtrim() {
    let env = env();
    let result: String = env.eval("return strtrim('  hello  ')").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_string_trim_alias() {
    let env = env();
    let result: (String, String) = env
        .eval(r#"return string.trim('  hello  '), ('xxhellox'):trim('x')"#)
        .unwrap();
    assert_eq!(result.0, "hello");
    assert_eq!(result.1, "hello");
}

#[test]
fn test_strtrim_custom_chars() {
    let env = env();
    let result: String = env.eval("return strtrim('\\nhello\\r', '\\n\\r')").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_strjoin() {
    let env = env();
    let result: String = env.eval("return strjoin(',', 'a', 'b', 'c')").unwrap();
    assert_eq!(result, "a,b,c");
}

#[test]
fn test_strsplittable() {
    let env = env();
    let (a, b): (String, String) = env
        .eval(
            r#"
            local t = strsplittable(',', 'x,y')
            return t[1], t[2]
            "#,
        )
        .unwrap();
    assert_eq!(a, "x");
    assert_eq!(b, "y");
}

#[test]
fn test_string_split_method() {
    let env = env();
    let (a, b, c): (String, String, String) = env.eval(r#"return ("a-b-c"):split("-")"#).unwrap();
    assert_eq!((a.as_str(), b.as_str(), c.as_str()), ("a", "b", "c"));
}

#[test]
fn test_string_split_method_accepts_delimiter_receiver() {
    let env = env();
    let (major, minor, build): (String, String, String) =
        env.eval(r#"return ("."):split("12.0.5")"#).unwrap();
    assert_eq!(
        (major.as_str(), minor.as_str(), build.as_str()),
        ("12", "0", "5")
    );
}

#[test]
fn test_string_split_method_accepts_empty_delimiter_receiver_input() {
    let env = env();
    let value: String = env.eval(r#"return (" "):split("")"#).unwrap();
    assert_eq!(value, "");
}

#[test]
fn test_string_split_function_accepts_empty_delimiter_receiver_input() {
    let env = env();
    let value: String = env.eval(r#"return string.split(" ", "")"#).unwrap();
    assert_eq!(value, "");
}

#[test]
fn test_string_split_method_accepts_equal_length_delimiter_receiver_input() {
    let env = env();
    let value: String = env.eval(r#"return (" "):split("?")"#).unwrap();
    assert_eq!(value, "?");
}

#[test]
fn test_string_split_function_accepts_equal_length_delimiter_receiver_input() {
    let env = env();
    let value: String = env.eval(r#"return string.split(" ", "?")"#).unwrap();
    assert_eq!(value, "?");
}

#[test]
fn test_format_alias() {
    let env = env();
    let result: String = env.eval("return format('%d items', 5)").unwrap();
    assert_eq!(result, "5 items");
}

// ============================================================================
// Math library aliases
// ============================================================================

#[test]
fn test_math_aliases() {
    let env = env();
    let val: i32 = env.eval("return abs(-5)").unwrap();
    assert_eq!(val, 5);
    let val: i32 = env.eval("return ceil(1.2)").unwrap();
    assert_eq!(val, 2);
    let val: i32 = env.eval("return floor(1.9)").unwrap();
    assert_eq!(val, 1);
    let val: i32 = env.eval("return max(3, 7)").unwrap();
    assert_eq!(val, 7);
    let val: i32 = env.eval("return min(3, 7)").unwrap();
    assert_eq!(val, 3);
}

// ============================================================================
// Bitwise operations
// ============================================================================

#[test]
fn test_bit_band() {
    let env = env();
    let val: i32 = env.eval("return bit.band(12, 10)").unwrap();
    assert_eq!(val, 8); // 1100 & 1010 = 1000
}

#[test]
fn test_bit_bor() {
    let env = env();
    let val: i32 = env.eval("return bit.bor(12, 10)").unwrap();
    assert_eq!(val, 14); // 1100 | 1010 = 1110
}

#[test]
fn test_bit_bxor() {
    let env = env();
    let val: i32 = env.eval("return bit.bxor(12, 10)").unwrap();
    assert_eq!(val, 6); // 1100 ^ 1010 = 0110
}

#[test]
fn test_bit_bnot() {
    let env = env();
    let val: i64 = env.eval("return bit.bnot(0)").unwrap();
    assert_eq!(val, 4294967295); // 32-bit not of 0
}

#[test]
fn test_bit_shifts() {
    let env = env();
    let val: i32 = env.eval("return bit.lshift(1, 3)").unwrap();
    assert_eq!(val, 8);
    let val: i32 = env.eval("return bit.rshift(16, 2)").unwrap();
    assert_eq!(val, 4);
}

// ============================================================================
// Table aliases
// ============================================================================

#[test]
fn test_sort_alias() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local t = {3, 1, 2}
            sort(t)
            return t[1]
            "#,
        )
        .unwrap();
    assert_eq!(val, 1);
}

#[test]
fn test_getn_alias() {
    let env = env();
    let val: i32 = env.eval("return getn({10, 20, 30})").unwrap();
    assert_eq!(val, 3);
}

// ============================================================================
// Mixin system
// ============================================================================

#[test]
fn test_mixin() {
    let env = env();
    let (a, b): (i32, i32) = env
        .eval(
            r#"
            local obj = {a = 1}
            local mixin = {b = 2}
            Mixin(obj, mixin)
            return obj.a, obj.b
            "#,
        )
        .unwrap();
    assert_eq!((a, b), (1, 2));
}

#[test]
fn test_mixin_multiple() {
    let env = env();
    let (a, b, c): (i32, i32, i32) = env
        .eval(
            r#"
            local obj = {}
            Mixin(obj, {a = 1}, {b = 2}, {c = 3})
            return obj.a, obj.b, obj.c
            "#,
        )
        .unwrap();
    assert_eq!((a, b, c), (1, 2, 3));
}

#[test]
fn test_create_from_mixins() {
    let env = env();
    let (a, b): (i32, i32) = env
        .eval(
            r#"
            local m1 = {a = 1}
            local m2 = {b = 2}
            local obj = CreateFromMixins(m1, m2)
            return obj.a, obj.b
            "#,
        )
        .unwrap();
    assert_eq!((a, b), (1, 2));
}

#[test]
fn test_create_and_init_from_mixin() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local MyMixin = {}
            function MyMixin:Init(x)
                self.value = x
            end
            local obj = CreateAndInitFromMixin(MyMixin, 42)
            return obj.value
            "#,
        )
        .unwrap();
    assert_eq!(val, 42);
}
