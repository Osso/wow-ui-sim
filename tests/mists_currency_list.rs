#![cfg(feature = "client-mists")]

#[path = "common/blizzard_addon_closure.rs"]
mod blizzard_addon_closure;

use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;

fn bounded_field(value: &str) -> String {
    value.chars().take(96).collect()
}

fn log_error_boundary(case: &str, error: &str) {
    // Inspect only a bounded first line; never emit error text or source excerpts.
    let first_line: String = error
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(512)
        .collect();
    let nil_access = first_line.contains("nil value");
    let class = if nil_access {
        "nil-access"
    } else if first_line.contains("convert") || first_line.contains("conversion") {
        "conversion"
    } else {
        "other"
    };
    eprintln!("[currency-diagnostic] case={case} error_class={class}");
}

fn load_mists_token_ui_env() -> WowLuaEnv {
    // TokenUI's bare TOC omits its implicit Game startup dependencies.
    let (env, _) = blizzard_addon_closure::build_blizzard_addon_closure_env(
        &blizzard_addon_closure::blizzard_ui_dir(),
        &["Blizzard_TokenUI"],
        &[BlizzardAddonOverride {
            addon: "Blizzard_TokenUI",
            extra_roots: &["Blizzard_UIPanels_Game", "Blizzard_MoneyFrame"],
        }],
    );
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_TokenUI"))
        assert(TokenFrame:GetObjectType() == "Frame")
        assert(TokenFrameContainer:GetObjectType() == "ScrollFrame")
        assert(type(HybridScrollFrame_GetOffset) == "function")
        assert(type(HybridScrollFrame_Update) == "function")
        assert(#TokenFrameContainer.buttons > 0)
        "#,
    )
    .expect("profile addon closure should create TokenUI and HybridScroll widgets");
    env
}

#[test]
fn token_frame_update_reproduces_missing_currency_list_size() {
    let env = load_mists_token_ui_env();

    // Exclude bootstrap/addon-load observations from the fault-injection interval.
    let previous_accesses = std::mem::take(&mut env.state().borrow_mut().nil_symbol_accesses);
    let result = env.eval::<(bool, String)>(
        r#"
        local handler = TokenFrame_Update
        local previousSize = rawget(_G, "GetCurrencyListSize")
        local previousDedupe = rawget(_G, "__wow_logged_nil_symbols")
        rawset(_G, "GetCurrencyListSize", nil)
        rawset(_G, "__wow_logged_nil_symbols", {})
        local ok, err = pcall(handler)
        rawset(_G, "GetCurrencyListSize", previousSize)
        rawset(_G, "__wow_logged_nil_symbols", previousDedupe)
        return ok, tostring(err)
        "#,
    );
    let accesses = std::mem::replace(
        &mut env.state().borrow_mut().nil_symbol_accesses,
        previous_accesses,
    );
    let (ok, err) = result
        .inspect_err(|err| log_error_boundary("missing-size-eval", &err.to_string()))
        .expect("TokenFrame_Update pcall should return a status");

    eprintln!("[currency-diagnostic] case=missing-size pcall_ok={ok}");
    if !ok {
        log_error_boundary("missing-size-pcall", &err);
    }
    assert!(!ok, "TokenFrame_Update should reproduce the nil global");
    assert!(
        err.contains("attempt to call a nil value"),
        "expected a nil-call failure"
    );
    assert_eq!(
        accesses.len(),
        1,
        "expected only the injected global lookup"
    );
    assert_eq!(accesses[0].container, "_G");
    assert_eq!(accesses[0].key, "GetCurrencyListSize");
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
            assert(type(watched.isShowInBackpack) == "boolean")
            assert(type(unwatched.isShowInBackpack) == "boolean")
            assert(type(legacyWatched) == "boolean")
            assert(type(legacyUnwatched) == "boolean")
            return watched.name, legacyWatchedName,
                tostring(watched.isShowInBackpack), tostring(legacyWatched),
                unwatched.name, legacyUnwatchedName,
                tostring(unwatched.isShowInBackpack), tostring(legacyUnwatched)
            "#,
        )
        .inspect_err(|err| log_error_boundary("watched-eval", &err.to_string()))
        .expect("currency list info should expose watched flags at both API boundaries");

    eprintln!(
        "[currency-diagnostic] case=watched namespace_names={:?}/{:?} legacy_names={:?}/{:?}",
        bounded_field(&watched_name),
        bounded_field(&unwatched_name),
        bounded_field(&legacy_watched_name),
        bounded_field(&legacy_unwatched_name),
    );
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
            assert(type(info.maxQuantity) == "number")
            assert(type(legacyMaxQuantity) == "number")
            return info.name, legacyName,
                tostring(info.maxQuantity), tostring(legacyMaxQuantity)
            "#,
        )
        .inspect_err(|err| log_error_boundary("cap-eval", &err.to_string()))
        .expect("currency list info should expose maximum quantity at both API boundaries");

    eprintln!(
        "[currency-diagnostic] case=cap namespace_name={:?} legacy_name={:?}",
        bounded_field(&name),
        bounded_field(&legacy_name),
    );
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
    let env = load_mists_token_ui_env();

    let (legacy_size, namespaced_size, update_ok, err): (i32, i32, bool, String) = env
        .eval(
            r#"
            local legacySize = GetCurrencyListSize()
            local namespacedSize = C_CurrencyInfo.GetCurrencyListSize()
            local ok, err = pcall(TokenFrame_Update)
            if ok then
                local populatedRows = 0
                for _, button in ipairs(TokenFrameContainer.buttons) do
                    local name = button.name:GetText()
                    if button:IsShown() and name and name ~= "" then
                        populatedRows = populatedRows + 1
                    end
                end
                assert(populatedRows > 0, "TokenFrame should populate currency rows")
                assert(TokenFrameContainer.totalHeight > 0)
            end
            return legacySize, namespacedSize, ok, tostring(err)
            "#,
        )
        .inspect_err(|err| log_error_boundary("wrapper-eval", &err.to_string()))
        .expect("legacy currency wrapper should support TokenFrame_Update");

    eprintln!("[currency-diagnostic] case=wrapper pcall_ok={update_ok}");
    if !update_ok {
        log_error_boundary("wrapper-pcall", &err);
    }
    assert_eq!(
        legacy_size, namespaced_size,
        "legacy GetCurrencyListSize should delegate to C_CurrencyInfo.GetCurrencyListSize"
    );
    assert!(
        update_ok,
        "TokenFrame_Update should complete with the legacy compatibility wrapper"
    );
}
