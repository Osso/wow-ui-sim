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
fn currency_list_info_preserves_watched_flags_in_namespace_and_legacy_tuple() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let (
        watched_name,
        legacy_watched_name,
        watched,
        legacy_watched,
        unwatched_name,
        legacy_unwatched_name,
        unwatched,
        legacy_unwatched,
    ): (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ) = env
        .eval(
            r#"
            local watched = C_CurrencyInfo.GetCurrencyListInfo(2)
            local unwatched = C_CurrencyInfo.GetCurrencyListInfo(3)
            local legacyWatchedName, _, _, _, legacyWatched = GetCurrencyListInfo(2)
            local legacyUnwatchedName, _, _, _, legacyUnwatched = GetCurrencyListInfo(3)
            return watched.name, legacyWatchedName,
                tostring(watched.isShowInBackpack), tostring(legacyWatched),
                unwatched.name, legacyUnwatchedName,
                tostring(unwatched.isShowInBackpack), tostring(legacyUnwatched)
            "#,
        )
        .expect("currency list info should expose watched flags at both API boundaries");

    assert_eq!(watched_name, "Valorstones");
    assert_eq!(legacy_watched_name, "Valorstones");
    assert_eq!(unwatched_name, "Weathered Harbinger Crest");
    assert_eq!(legacy_unwatched_name, "Weathered Harbinger Crest");
    assert_eq!(
        (watched.as_str(), legacy_watched.as_str()),
        ("true", "true"),
        "Valorstones must be watched in namespace isShowInBackpack and legacy position 5"
    );
    assert_eq!(
        (unwatched.as_str(), legacy_unwatched.as_str()),
        ("false", "false"),
        "Weathered Harbinger Crest must not be watched at either API boundary"
    );
}

#[test]
fn currency_list_info_preserves_max_quantity_in_namespace_and_legacy_tuple() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let (name, legacy_name, max_quantity, legacy_max_quantity): (String, String, String, String) =
        env.eval(
            r#"
            local info = C_CurrencyInfo.GetCurrencyListInfo(3)
            local legacyName, _, _, _, _, _, _, legacyMaxQuantity = GetCurrencyListInfo(3)
            return info.name, legacyName,
                tostring(info.maxQuantity), tostring(legacyMaxQuantity)
            "#,
        )
        .expect("currency list info should expose maximum quantity at both API boundaries");

    assert_eq!(name, "Weathered Harbinger Crest");
    assert_eq!(legacy_name, "Weathered Harbinger Crest");
    assert_eq!(
        (max_quantity.as_str(), legacy_max_quantity.as_str()),
        ("90", "90"),
        "crest cap must be 90 in namespace maxQuantity and legacy position 8"
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
