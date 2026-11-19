//! Pure utility globals: locale/build info, string helpers, hooksecurefunc,
//! Mixin, and presence checks for assorted global functions.

use super::super::*;

#[test]
fn test_get_build_info() {
    let env = WowLuaEnv::new().unwrap();
    let (version, toc): (String, i32) = env
        .eval("local v,_,_,t = GetBuildInfo(); return v, t")
        .unwrap();
    assert!(!version.is_empty());
    assert!(toc > 0);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_get_build_option_restricted_aura_api() {
    let env = WowLuaEnv::new().unwrap();
    let (function_type, restricted_aura_api, unknown): (String, bool, Option<bool>) = env
        .eval(
            "return type(GetBuildOption), GetBuildOption('RestrictedAuraAPI'), GetBuildOption('UnknownOption')",
        )
        .unwrap();

    assert_eq!(function_type, "function");
    assert!(restricted_aura_api);
    assert_eq!(unknown, None);
}

#[test]
fn test_get_locale() {
    let env = WowLuaEnv::new().unwrap();
    let locale: String = env.eval("return GetLocale()").unwrap();
    assert!(!locale.is_empty());
}

#[test]
fn test_unit_name_player() {
    let env = WowLuaEnv::new().unwrap();
    let name: String = env.eval("return UnitName('player')").unwrap();
    assert!(!name.is_empty());
}

#[test]
fn test_get_money() {
    let env = WowLuaEnv::new().unwrap();
    let money: i64 = env.eval("return GetMoney()").unwrap();
    assert!(money >= 0);
}

#[test]
fn test_get_inventory_item_texture_reagent_bag_slot_fallback() {
    let env = WowLuaEnv::new().unwrap();
    let texture: String = env
        .eval("return GetInventoryItemTexture('player', 25)")
        .unwrap();
    assert_eq!(texture, "Interface\\Icons\\INV_Misc_Bag_08");
}

#[test]
fn test_in_combat_lockdown_false() {
    let env = WowLuaEnv::new().unwrap();
    let in_combat: bool = env.eval("return InCombatLockdown()").unwrap();
    assert!(!in_combat);
}

#[test]
fn test_wipe_function() {
    let (t, _) = load_test_lua(
        "test-wipe",
        r#"
        local t = {1, 2, 3, a = "b"}
        wipe(t)
        WIPE_LEN = #t
        WIPE_A_NIL = (t.a == nil)
    "#,
    );
    let len: i32 = t.env.eval("return WIPE_LEN").unwrap();
    assert_eq!(len, 0);
    t.assert_lua_true("return WIPE_A_NIL", "wipe should clear named keys");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_global_security_helpers() {
    let env = WowLuaEnv::new().unwrap();
    let (copied_value, copied_nested, same_nested, table_security): (i32, i32, bool, String) = env
        .eval(
            r#"
            local source = { value = 7, nested = { value = 9 } }
            local copy = securecopy(source)
            settablesecurity({}, "secure")
            return copy.value,
                copy.nested.value,
                copy.nested == source.nested,
                type(settablesecurity)
            "#,
        )
        .unwrap();

    assert_eq!(copied_value, 7);
    assert_eq!(copied_nested, 9);
    assert!(!same_nested, "securecopy should deep-copy nested tables");
    assert_eq!(table_security, "function");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_load_addon_with_error_handling_uses_vendor_owner() {
    let env = WowLuaEnv::new().unwrap();
    let root = crate::paths::default_blizzard_ui_addons_path().unwrap();
    let source = std::fs::read_to_string(root.join("Blizzard_SharedXML/AddOnUtil.lua")).unwrap();
    // PRESERVATION: execute the original owner, not the retired PTR wrapper.
    env.exec(&source).unwrap();
    env.exec(
        r#"
        C_AddOns.LoadAddOn = function(name) return "loaded:" .. name end
        UIParentLoadAddOn = function() error("retired wrapper called") end
    "#,
    )
    .unwrap();
    let result: String = env
        .eval("return LoadAddOnWithErrorHandling('Blizzard_EditMode')")
        .unwrap();
    assert_eq!(result, "loaded:Blizzard_EditMode");
}

#[test]
fn test_copy_table_deep() {
    let (t, _) = load_test_lua(
        "test-copytable",
        r#"
        local orig = {a = 1, b = {c = 2}}
        local copy = CopyTable(orig)
        COPY_A = copy.a
        COPY_BC = copy.b.c
        copy.a = 99
        ORIG_A = orig.a
    "#,
    );
    let copy_a: i32 = t.env.eval("return COPY_A").unwrap();
    assert_eq!(copy_a, 1);
    let copy_bc: i32 = t.env.eval("return COPY_BC").unwrap();
    assert_eq!(copy_bc, 2);
    let orig_a: i32 = t.env.eval("return ORIG_A").unwrap();
    assert_eq!(orig_a, 1, "original should be unmodified");
}

#[test]
fn test_copy_table_shallow() {
    let (t, _) = load_test_lua(
        "test-copytable-shallow",
        r#"
        local orig = {a = {value = 1}}
        local copy = CopyTable(orig, true)
        copy.a.value = 99
        SHALLOW_SHARED = (orig.a == copy.a)
        SHALLOW_VALUE = orig.a.value
    "#,
    );
    let shared: bool = t.env.eval("return SHALLOW_SHARED").unwrap();
    assert!(
        shared,
        "shallow CopyTable should preserve nested table identity"
    );
    let value: i32 = t.env.eval("return SHALLOW_VALUE").unwrap();
    assert_eq!(
        value, 99,
        "shallow CopyTable should share nested table mutations"
    );
}

#[test]
fn test_strsplit() {
    let (t, _) = load_test_lua(
        "test-strsplit",
        r#"
        local a, b, c = strsplit(",", "one,two,three")
        SS_A, SS_B, SS_C = a, b, c
    "#,
    );
    t.assert_lua_str("return SS_A", "one");
    t.assert_lua_str("return SS_B", "two");
    t.assert_lua_str("return SS_C", "three");
}

#[test]
fn test_strtrim() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env.eval(r#"return strtrim("  hello  ")"#).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_get_player_info_by_guid_returns_player_identity() {
    let env = WowLuaEnv::new().unwrap();
    let (localized_class, english_class, localized_race, english_race, sex, name, realm): (
        String,
        String,
        String,
        String,
        i32,
        String,
        String,
    ) = env
        .eval(
            r#"
            return GetPlayerInfoByGUID(UnitGUID("player"))
        "#,
        )
        .unwrap();

    assert_eq!(localized_class, "Paladin");
    assert_eq!(english_class, "PALADIN");
    assert_eq!(localized_race, "Human");
    assert_eq!(english_race, "Human");
    assert_eq!(sex, 2);
    assert!(!name.is_empty());
    assert!(!realm.is_empty());
}

#[test]
fn test_geterrorhandler() {
    let env = WowLuaEnv::new().unwrap();
    let ty: String = env.eval("return type(geterrorhandler())").unwrap();
    assert_eq!(ty, "function");
}

#[test]
fn test_hooksecurefunc() {
    let (t, _) = load_test_lua(
        "test-hooksecure",
        r#"
        local obj = { MyMethod = function() end }
        HOOK_CALLED = false
        hooksecurefunc(obj, "MyMethod", function() HOOK_CALLED = true end)
        obj:MyMethod()
    "#,
    );
    t.assert_lua_true("return HOOK_CALLED", "hook should fire");
}

#[test]
fn test_hooksecurefunc_multiple_hooks_share_stable_wrapper() {
    let (t, _) = load_test_lua(
        "test-hooksecure-multiple",
        r#"
        local obj = { calls = {}, MyMethod = function(self, value) table.insert(self.calls, "orig:" .. value) end }
        hooksecurefunc(obj, "MyMethod", function(self, value) table.insert(self.calls, "first:" .. value) end)
        hooksecurefunc(obj, "MyMethod", function(self, value) table.insert(self.calls, "second:" .. value) end)
        collectgarbage()
        obj:MyMethod("ok")
        HOOK_CALLS = table.concat(obj.calls, ",")
    "#,
    );
    t.assert_lua_str("return HOOK_CALLS", "orig:ok,first:ok,second:ok");
}

#[test]
fn test_hooksecurefunc_on_frame_userdata() {
    let (t, _) = load_test_lua(
        "test-hooksecure-ud",
        r#"
        local f = CreateFrame("Frame", "HookSecureUDTest", UIParent)
        HOOK_CALLED = false
        hooksecurefunc(f, "SetAlpha", function() HOOK_CALLED = true end)
        f:SetAlpha(0.5)
    "#,
    );
    t.assert_lua_true("return HOOK_CALLED", "hook should fire on userdata frame");
}

#[test]
fn test_issecurevariable_on_frame_userdata() {
    let (t, _) = load_test_lua(
        "test-issecurevar-ud",
        r#"
        local f = CreateFrame("Frame", "IssecureVarUDTest", UIParent)
        -- issecurevariable(frame, "method") should not error on userdata
        local secure, taint = issecurevariable(f, "Show")
        SECURE_RESULT = secure
    "#,
    );
    t.assert_lua_true("return SECURE_RESULT", "native method should be secure");
}

#[test]
fn test_mixin() {
    let (t, _) = load_test_lua(
        "test-mixin",
        r#"
        local target = {}
        Mixin(target, {foo = 1, bar = "hello"})
        MIX_FOO = target.foo
        MIX_BAR = target.bar
    "#,
    );
    let foo: i32 = t.env.eval("return MIX_FOO").unwrap();
    assert_eq!(foo, 1);
    t.assert_lua_str("return MIX_BAR", "hello");
}

#[cfg(feature = "client-retail")]
#[test]
// These globals are test-only fixtures for unrelated UnitUtil APIs, not
// native claims. They let the consumer call complete; no assertion checks
// their values, and assertions inspect only the wrapped selector.
fn test_retail_get_specialization_system_via_cached_unit_util() {
    let env = WowLuaEnv::new().unwrap();
    let path =
        crate::client_profile::blizzard_ui_addons_dir().join("Blizzard_SharedXML/UnitUtil.lua");
    let source = std::fs::read_to_string(path).unwrap();
    env.exec(&source).unwrap();
    let passed: bool = env
        .eval(
            r##"
            local getter = GetSpecializationSystem
            local function verify(...)
                local count, value = select("#", getter(...)), getter(...)
                assert(count == 1 and type(value) == "number" and value == 1)
                return value
            end
            GetSpecializationSystem = verify
            UnitGroupRolesAssigned = function() return "NONE" end
            C_SpecializationInfo = { GetSpecialization = function() return 1 end }
            GetSpecializationRole = function() return "TANK" end
            PlayerUtil.IsPlayerEffectivelyTank()
            verify()
            verify(nil)
            verify(false, {}, "extra")
            return true
            "##,
        )
        .unwrap();
    assert!(passed);
}

#[test]
fn test_global_functions_callable() {
    let env = WowLuaEnv::new().unwrap();
    for f in &[
        "BreakUpLargeNumbers",
        "PlaySound",
        "ReloadUI",
        "GetBindingKey",
        "SetOverrideBinding",
        "ClearOverrideBindings",
        "GetInventoryItemLink",
        "GetInventoryItemTexture",
        "GetInventoryItemsForSlot",
        #[cfg(not(feature = "retail-12-1-0"))]
        "GetInventorySlotInfo",
        "GetFramerate",
        "format",
        "strjoin",
    ] {
        let expr = format!("return type({})", f);
        let ty: String = env.eval(&expr).unwrap();
        assert_eq!(ty, "function", "{} should be function", f);
    }
}
