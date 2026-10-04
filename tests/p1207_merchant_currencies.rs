#![cfg(feature = "retail-12-0-7")]
use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn merchant_rejects_secret_extras_for_every_caller() {
    let env = WowLuaEnv::new().unwrap();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        let value = wrap_host_secret_number(lua.state_mut(), 2815.0);
        lua.state_mut().push(value);
        lua.set_global_val("MerchantSecret", value).unwrap();
        lua.state_mut().pop();
    }
    env.exec(r#"
        local function check()
            assert(#C_MerchantFrame.GetMerchantCurrencies(false, 7) == 0)
            for _, call in ipairs({
                function() return C_MerchantFrame.GetMerchantCurrencies(MerchantSecret) end,
                function() return C_MerchantFrame.GetMerchantCurrencies(false, MerchantSecret) end,
            }) do
                local ok, err = pcall(call)
                assert(not ok and string.find(err, 'secret', 1, true))
            end
        end
        check(); collectgarbage('collect'); check()
        local function tainted()
            check(); assert(debug.getstacktaint() == 'MerchantProbe')
        end
        debug.setobjecttaint(tainted, 'MerchantProbe'); tainted()
        assert(issecure() and issecretvalue(MerchantSecret)); check()
    "#).unwrap();
}

#[test]
fn merchant_ids_are_live_ordered_and_detached() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().merchant_currencies = vec![2815, 3008];
    env.exec(
        r#"
        local function check(...)
            assert(select('#', ...) == 1)
            local ids = ...
            assert(#ids == 2 and ids[1] == 2815 and ids[2] == 3008)
            assert(type(ids[1]) == 'number' and not issecretvalue(ids[1]))
            CurrencySnapshot = ids
        end
        check(C_MerchantFrame.GetMerchantCurrencies())
        CurrencySnapshot[1] = 99; CurrencySnapshot[3] = 99
        local ids = C_MerchantFrame.GetMerchantCurrencies()
        assert(ids ~= CurrencySnapshot and #ids == 2 and ids[1] == 2815)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().merchant_currencies = vec![3010];
    env.exec(
        r#"
        local ids = C_MerchantFrame.GetMerchantCurrencies()
        assert(#ids == 1 and ids[1] == 3010)
        assert(CurrencySnapshot[1] == 99 and CurrencySnapshot[2] == 3008)
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().merchant_currencies, vec![3010]);
    env.state().borrow_mut().merchant_currencies.clear();
    env.exec("assert(#C_MerchantFrame.GetMerchantCurrencies() == 0)")
        .unwrap();
}

#[test]
fn merchant_empty_defaults_and_independent_environments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = C_MerchantFrame.GetMerchantCurrencies()
        first[1] = 2815
        local second = C_MerchantFrame.GetMerchantCurrencies()
        assert(first ~= second and #second == 0)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().merchant_currencies = vec![3008, 2815];
    let other = WowLuaEnv::new().unwrap();
    let count: f64 = other
        .eval("return #C_MerchantFrame.GetMerchantCurrencies()")
        .unwrap();
    assert_eq!(count, 0.0);
    env.exec(
        "local a = C_MerchantFrame.GetMerchantCurrencies(); assert(a[1] == 3008 and a[2] == 2815)",
    )
    .unwrap();
}

#[test]
fn merchant_cached_deprecated_wrapper_unpacks_exact_host_ids() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().merchant_currencies = vec![2815, 3008];
    env.exec("assert(GetMerchantCurrencies == nil)").unwrap();
    let path = std::path::PathBuf::from(std::env::var_os("HOME").unwrap())
        .join(".cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_0_7.lua");
    let source = std::fs::read_to_string(path).expect("required cached wrapper, no fallback");
    // Execute the actual cached wrapper alone; unrelated MenuUtil migrations need UI addons.
    let start = source.find("function GetMerchantCurrencies()").expect("cached wrapper declaration");
    env.exec(&source[start..]).unwrap();
    env.exec(
        r#"
        local function check(...)
            assert(select('#', ...) == 2)
            local a,b = ...; assert(a == 2815 and b == 3008)
        end
        check(GetMerchantCurrencies())
    "#,
    )
    .unwrap();
    env.state().borrow_mut().merchant_currencies.clear();
    let count: f64 = env
        .eval("return select('#', GetMerchantCurrencies())")
        .unwrap();
    assert_eq!(count, 0.0);
}
