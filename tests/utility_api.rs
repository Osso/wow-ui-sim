//! Tests for utility API functions (utility_api.rs).

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// strsplit
// ============================================================================

#[test]
fn test_strsplit_basic() {
    let env = env();
    let result: (String, String, String) = env
        .eval("return strsplit(',', 'a,b,c')")
        .unwrap();
    assert_eq!(result, ("a".into(), "b".into(), "c".into()));
}

#[test]
fn test_strsplit_with_limit() {
    let env = env();
    let result: (String, String) = env
        .eval("return strsplit(',', 'a,b,c', 2)")
        .unwrap();
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
    let found: bool = env
        .eval("return tContains({10, 20, 30}, 20)")
        .unwrap();
    assert!(found);
}

#[test]
fn test_tcontains_not_found() {
    let env = env();
    let found: bool = env
        .eval("return tContains({10, 20, 30}, 99)")
        .unwrap();
    assert!(!found);
}

// ============================================================================
// tIndexOf
// ============================================================================

#[test]
fn test_tindexof_found() {
    let env = env();
    let idx: i32 = env
        .eval("return tIndexOf({10, 20, 30}, 20)")
        .unwrap();
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
// SecureCmdOptionParse
// ============================================================================

#[test]
fn test_securecmdoptionparse_returns_last() {
    let env = env();
    let result: String = env
        .eval("return SecureCmdOptionParse('[mod:shift] action1; action2')")
        .unwrap();
    assert_eq!(result, "action2");
}

#[test]
fn test_securecmdoptionparse_single_option() {
    let env = env();
    let result: String = env
        .eval("return SecureCmdOptionParse('just_this')")
        .unwrap();
    assert_eq!(result, "just_this");
}

// ============================================================================
// hooksecurefunc
// ============================================================================

#[test]
fn test_hooksecurefunc_global() {
    let env = env();
    let (orig_ran, hook_ran): (bool, bool) = env
        .eval(
            r#"
            HOOK_TEST_ORIG = false
            HOOK_TEST_HOOK = false
            function MyTestFunc() HOOK_TEST_ORIG = true end
            hooksecurefunc("MyTestFunc", function() HOOK_TEST_HOOK = true end)
            MyTestFunc()
            return HOOK_TEST_ORIG, HOOK_TEST_HOOK
            "#,
        )
        .unwrap();
    assert!(orig_ran);
    assert!(hook_ran);
}

#[test]
fn test_hooksecurefunc_table() {
    let env = env();
    let (orig_ran, hook_ran): (bool, bool) = env
        .eval(
            r#"
            local t = {}
            HOOK_TABLE_ORIG = false
            HOOK_TABLE_HOOK = false
            function t.Foo() HOOK_TABLE_ORIG = true end
            hooksecurefunc(t, "Foo", function() HOOK_TABLE_HOOK = true end)
            t.Foo()
            return HOOK_TABLE_ORIG, HOOK_TABLE_HOOK
            "#,
        )
        .unwrap();
    assert!(orig_ran);
    assert!(hook_ran);
}

// ============================================================================
// securecallfunction
// ============================================================================

#[test]
fn test_securecallfunction() {
    let env = env();
    let result: i32 = env
        .eval("return securecallfunction(function(a, b) return a + b end, 3, 4)")
        .unwrap();
    assert_eq!(result, 7);
}

// ============================================================================
// secureexecuterange
// ============================================================================

#[test]
fn test_secureexecuterange() {
    let env = env();
    let total: i32 = env
        .eval(
            r#"
            SECURE_TOTAL = 0
            local t = {10, 20, 30}
            secureexecuterange(t, function(key, value) SECURE_TOTAL = SECURE_TOTAL + value end)
            return SECURE_TOTAL
            "#,
        )
        .unwrap();
    assert_eq!(total, 60);
}

// ============================================================================
// Security stubs
// ============================================================================

#[test]
fn test_issecure_returns_true() {
    let env = env();
    let val: bool = env.eval("return issecure()").unwrap();
    assert!(val);
}

#[test]
fn test_issecurevariable_returns_true() {
    let env = env();
    let val: bool = env
        .eval("local s, t = issecurevariable(nil, 'print'); return s")
        .unwrap();
    assert!(val);
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
    let (a, b, c): (String, String, String) = env
        .eval(
            r#"
            local t = ("a-b-c"):split("-")
            return t[1], t[2], t[3]
            "#,
        )
        .unwrap();
    assert_eq!((a.as_str(), b.as_str(), c.as_str()), ("a", "b", "c"));
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

// ============================================================================
// Error handler functions
// ============================================================================

#[test]
fn test_geterrorhandler_returns_function() {
    let env = env();
    let is_func: bool = env
        .eval("return type(geterrorhandler()) == 'function'")
        .unwrap();
    assert!(is_func);
}

#[test]
fn test_seterrorhandler_accepts_function() {
    let env = env();
    env.eval::<()>("seterrorhandler(function() end)").unwrap();
}


// ============================================================================
// SecureHandler stubs
// ============================================================================

#[test]
fn test_secure_handler_stubs_exist() {
    let env = env();
    for func in &[
        "SecureHandlerSetFrameRef",
        "SecureHandlerExecute",
        "SecureHandlerWrapScript",
        "RegisterStateDriver",
        "UnregisterStateDriver",
        "RegisterAttributeDriver",
        "UnregisterAttributeDriver",
    ] {
        let is_func: bool = env
            .eval(&format!("return type({}) == 'function'", func))
            .unwrap();
        assert!(is_func, "{} should be a function", func);
    }
}

// ============================================================================
// GetCurrentEnvironment
// ============================================================================

#[test]
fn test_get_current_environment() {
    let env = env();
    let is_global: bool = env
        .eval("return GetCurrentEnvironment() == _G")
        .unwrap();
    assert!(is_global);
}

// ============================================================================
// securecall
// ============================================================================

#[test]
fn test_securecall() {
    let env = env();
    let result: i32 = env
        .eval("return securecall(function(a) return a * 2 end, 5)")
        .unwrap();
    assert_eq!(result, 10);
}

// ============================================================================
// forceinsecure
// ============================================================================

#[test]
fn test_forceinsecure_is_noop() {
    let env = env();
    env.eval::<()>("forceinsecure()").unwrap();
}
