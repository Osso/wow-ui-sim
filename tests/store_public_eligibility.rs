//! Eligibility input proof, not a working storefront. Outbound sinks are test-only.
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

fn supply(env: &WowLuaEnv, subscription: bool, game_time: bool) {
    let mut state = env.state().borrow_mut();
    state.store_public_eligibility.insert(156, subscription);
    state.store_public_eligibility.insert(37, game_time);
}

fn load_original_helper() -> WowLuaEnv {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    use wow_ui_sim::{loader::find_toc_file, screen::ScreenKind, toc::TocFile};
    let ui = blizzard_ui_dir();
    let toc_path = find_toc_file(&ui.join("Blizzard_SharedXML")).unwrap();
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    assert!(toc.files.iter().any(|file| file == "InterfaceUtil.lua"));
    eprintln!("selected SharedXML TOC: {}", toc_path.display());
    let env = new_blizzard_addon_env(&ui);
    env.set_server_mirrored_cvar("shop2Enabled", "0")
        .expect("legacy store backend should be selected for this fixture");
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SharedXML"], &[]);
    assert!(loaded.iter().any(|addon| addon == "Blizzard_SharedXML"));
    eprintln!("setup diagnostics: {:?}", env.state().borrow().lua_errors);
    env.exec(r#"
        assert(WOW_SUBSCRIPTION_CATEGORY_ID == 156 and WOW_GAME_TIME_CATEGORY_ID == 37)
        assert(not C_CatalogShop.IsShop2Enabled(), 'legacy branch prerequisite')
        local info = debug.getinfo(StoreInterfaceUtil.OpenToSubscriptionProduct, 'S')
        local suffix = 'Blizzard_SharedXML/InterfaceUtil.lua'
        local source = info.source:gsub('\\', '/')
        assert(info.what == 'Lua' and source:sub(-#suffix) == suffix, 'original helper provenance')
        print('original helper', source, info.linedefined, info.lastlinedefined)
        -- Test-only recording sinks at the original helper's outbound boundary.
        -- These prove helper control flow, NOT a working storefront.
        storeCalls = {}
        StoreFrame_SelectSubscriptionProduct = function() table.insert(storeCalls, 'subscription') end
        StoreFrame_SelectGameTimeProduct = function() table.insert(storeCalls, 'gametime') end
        ToggleStoreUI = function() table.insert(storeCalls, 'toggle') end
        PlaySound = function(id)
            assert(id == SOUNDKIT.GS_LOGIN_NEW_ACCOUNT)
            table.insert(storeCalls, 'sound')
        end
        LoadURLIndex = function(id)
            assert(id == 22)
            table.insert(storeCalls, 'url:22')
        end
        -- Observe original query lines without replacing the helper or query.
        function observeStoreHelper()
            storeCalls = {}
            local lines = {}
            debug.sethook(function(_, line)
                local caller = debug.getinfo(2, 'S')
                local path = caller.source:gsub('\\', '/')
                if path:sub(-#suffix) == suffix then lines[line] = true end
            end, 'l')
            local ok, result = pcall(StoreInterfaceUtil.OpenToSubscriptionProduct)
            debug.sethook()
            if not ok then error(result, 0) end
            return result, table.concat(storeCalls, ','), lines
        end
    "#).expect("selected original helper and recording sinks");
    env
}

fn assert_choice(env: &WowLuaEnv, subscription: bool, game_time: bool, expected: &str) {
    supply(env, subscription, game_time);
    let errors_before = env.state().borrow().lua_errors.len();
    let line = if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        44
    } else {
        32
    };
    let script = format!(
        r#"
        local result, calls, lines = observeStoreHelper()
        print('original query lines', {line}, lines[{line}], {second}, lines[{second}])
        assert(result == {has_product}, 'helper return for supplied observations')
        assert(calls == '{expected}', 'outbound choice: '..calls)
        assert(lines[{line}], 'original subscription query line')
        assert(not not lines[{second}] == {second_reached}, 'original game-time query reachability')
    "#,
        has_product = subscription || game_time,
        second = line + 4,
        second_reached = !subscription
    );
    env.exec(&script)
        .expect("original helper must consume supplied eligibility");
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}

#[test]
fn original_subscription_helper_tracks_supplied_group_eligibility() {
    if ACTIVE == ClientProfile::Wrath {
        return;
    }
    let env = load_original_helper();
    // Fixed secure lists deliberately contradict the supplied observations.
    env.exec("__wow_store_state.productsByGroup[156] = {}; __wow_store_state.productsByGroup[37] = {1003}").unwrap();
    assert_choice(&env, true, false, "subscription,toggle");
    assert_choice(&env, false, true, "gametime,toggle");
    assert_choice(&env, false, false, "sound,url:22");
    assert_choice(&env, true, false, "subscription,toggle");
    assert_choice(&env, true, true, "subscription,toggle");
    env.state()
        .borrow_mut()
        .store_public_eligibility
        .remove(&37);
    env.exec("local result, calls = observeStoreHelper(); assert(result and calls == 'subscription,toggle')")
        .expect("subscription priority must not require a game-time observation");
    env.state()
        .borrow_mut()
        .store_public_eligibility
        .insert(156, false);
    let missing = env
        .exec("observeStoreHelper()")
        .expect_err("original second query must expose missing game-time input");
    assert!(
        missing
            .to_string()
            .contains("groupID=37 has no supplied eligibility observation"),
        "{missing}"
    );
    // Restore and observe the tuple before changing membership alone.
    assert_choice(&env, true, false, "subscription,toggle");
    // Membership changes cannot change the same supplied eligibility result.
    env.exec("__wow_store_state.productsByGroup[156] = {1003}; __wow_store_state.productsByGroup[37] = {}").unwrap();
    assert_choice(&env, true, false, "subscription,toggle");
    assert_choice(&env, false, true, "gametime,toggle");
    assert_choice(&env, false, false, "sound,url:22");
    assert_choice(&env, true, false, "subscription,toggle");
}

#[test]
fn supplied_group_shape_updates_missing_input_and_environment_isolation() {
    let first = WowLuaEnv::new().unwrap();
    if ACTIVE == ClientProfile::Wrath {
        first
            .exec("assert(rawget(C_StorePublic, 'DoesGroupHavePurchaseableProducts') == nil)")
            .unwrap();
        return;
    }
    supply(&first, true, false);
    first
        .exec(
            r#"
        assert(type(C_StorePublic.DoesGroupHavePurchaseableProducts) == 'function')
        local function checkResult(expected, ...)
            assert(select('#', ...) == 1, 'one result')
            assert(type((...)) == 'boolean', 'non-null boolean')
            assert((...) == expected, 'supplied observation')
        end
        checkResult(true, C_StorePublic.DoesGroupHavePurchaseableProducts(156))
        checkResult(false, C_StorePublic.DoesGroupHavePurchaseableProducts(37))
    "#,
        )
        .unwrap();
    let second = WowLuaEnv::new().unwrap();
    assert_missing(&second, 156);
    supply(&second, false, true);
    supply(&first, false, true);
    first.exec("assert(not C_StorePublic.DoesGroupHavePurchaseableProducts(156)); assert(C_StorePublic.DoesGroupHavePurchaseableProducts(37))").unwrap();
    supply(&first, true, false);
    second.exec("assert(not C_StorePublic.DoesGroupHavePurchaseableProducts(156)); assert(C_StorePublic.DoesGroupHavePurchaseableProducts(37))").unwrap();
    first
        .state()
        .borrow_mut()
        .store_public_eligibility
        .remove(&156);
    assert_missing(&first, 156);
    assert_missing(&first, 999);
    supply(&first, false, false);
    first
        .exec("assert(not C_StorePublic.DoesGroupHavePurchaseableProducts(156))")
        .unwrap();
}

fn assert_missing(env: &WowLuaEnv, group: i32) {
    let error = env
        .exec(&format!(
            "C_StorePublic.DoesGroupHavePurchaseableProducts({group})"
        ))
        .expect_err("missing input must not silently return false");
    let expected = format!(
        "missing-input: C_StorePublic.DoesGroupHavePurchaseableProducts groupID={group} has no supplied eligibility observation"
    );
    assert!(error.to_string().contains(&expected), "{error}");
}
