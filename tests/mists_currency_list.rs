#![cfg(feature = "client-mists")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn read_mists_token_ui_lua() -> String {
    std::fs::read_to_string(
        wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
            "CARGO_MANIFEST_DIR"
        )))
        .join("Blizzard_TokenUI/Blizzard_TokenUI.lua"),
    )
    .expect("Mists TokenUI Lua should be available in the profile UI source")
}

#[test]
fn token_frame_update_reproduces_missing_currency_list_size() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    env.exec(
        r#"
        rawset(_G, "GetCurrencyListSize", nil)
        UIPanelWindows = {}
        CharacterFrameTab4 = {
            Hide = function() end,
            Show = function() end,
        }
        TokenFrameContainer = {}
        "#,
    )
    .expect("install TokenFrame reproduction fixtures");

    let source = read_mists_token_ui_lua();
    env.exec(&source)
        .expect("Mists TokenUI Lua should define TokenFrame helpers");

    let (ok, err): (bool, String) = env
        .eval(
            r#"
            local ok, err = pcall(TokenFrame_Update)
            return ok, tostring(err)
            "#,
        )
        .expect("TokenFrame_Update pcall should return a status");

    assert!(!ok, "TokenFrame_Update should reproduce the nil global");
    assert!(
        err.contains("GetCurrencyListSize"),
        "expected GetCurrencyListSize nil failure, got: {err}"
    );
}

#[test]
fn legacy_currency_list_size_returns_one_numeric_backing_count() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let (size_type, arity, legacy_size, namespaced_size): (String, i32, i32, i32) = env
        .eval(
            r#"
            return type(GetCurrencyListSize()),
                select('#', GetCurrencyListSize()),
                GetCurrencyListSize(), C_CurrencyInfo.GetCurrencyListSize()
            "#,
        )
        .expect("legacy currency size should return the backing count");

    assert_eq!(size_type, "number");
    assert_eq!(
        arity, 1,
        "legacy currency size must return exactly one value"
    );
    assert_eq!(legacy_size, namespaced_size);
    assert_eq!(
        legacy_size,
        wow_ui_sim::lua_api::globals::currency_data::currency_list_size(),
        "legacy currency size must match the existing currency backing"
    );
}

#[test]
fn legacy_currency_list_size_wraps_c_currency_info() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    env.exec(
        r#"
        UIPanelWindows = {}
        local tab_visible = nil
        CharacterFrameTab4 = {
            Hide = function() tab_visible = false end,
            Show = function() tab_visible = true end,
        }
        TokenFrameContainer = {}
        "#,
    )
    .expect("install TokenFrame compatibility fixtures");

    let source = read_mists_token_ui_lua();
    env.exec(&source)
        .expect("Mists TokenUI Lua should define TokenFrame helpers");

    let (legacy_size, namespaced_size, update_ok, err): (i32, i32, bool, String) = env
        .eval(
            r#"
            local legacySize = GetCurrencyListSize()
            local namespacedSize = C_CurrencyInfo.GetCurrencyListSize()
            local ok, err = pcall(TokenFrame_Update)
            return legacySize, namespacedSize, ok, tostring(err)
            "#,
        )
        .expect("legacy currency wrapper should support TokenFrame_Update");

    assert_eq!(
        legacy_size, namespaced_size,
        "legacy GetCurrencyListSize should delegate to C_CurrencyInfo.GetCurrencyListSize"
    );
    assert!(
        update_ok,
        "TokenFrame_Update should use the legacy compatibility wrapper: {err}"
    );
}
