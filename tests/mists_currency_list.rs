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
    // TokenUI's bare TOC omits its implicit Game startup dependencies, including
    // the chat and item-button addons that receive its currency links.
    let (env, _) = blizzard_addon_closure::build_blizzard_addon_closure_env(
        &blizzard_addon_closure::blizzard_ui_dir(),
        &["Blizzard_TokenUI"],
        &[BlizzardAddonOverride {
            addon: "Blizzard_TokenUI",
            extra_roots: &[
                "Blizzard_UIPanels_Game",
                "Blizzard_MoneyFrame",
                "Blizzard_ChatFrame",
                "Blizzard_ItemButton",
            ],
        }],
    );
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_TokenUI"))
        assert(TokenFrame:GetObjectType() == "Frame")
        assert(TokenFrameContainer:GetObjectType() == "ScrollFrame")
        assert(type(HybridScrollFrame_GetOffset) == "function")
        assert(type(HybridScrollFrame_Update) == "function")
        -- Mists loads Cata/Blizzard_TokenUI.lua, whose TokenFrame_OnShow creates the
        -- rows (lines 41-57); TokenFrame starts hidden, so none exist after load.
        assert(TokenFrameContainer.buttons == nil)
        ToggleCharacter("TokenFrame")
        assert(TokenFrame:IsVisible())
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
        err.contains("attempt to call global 'GetCurrencyListSize' (a nil value)"),
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
fn legacy_currency_display_updates_preserve_static_rows_and_backpack_membership() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        currencyPreservationBaseline = {
            size = GetCurrencyListSize(),
            header = { GetCurrencyListInfo(1) },
            unwatched = { GetCurrencyListInfo(3) },
            neighbor = { GetBackpackCurrencyInfo(2) },
        }
        assert(currencyPreservationBaseline.neighbor[4] == 3089)
        "#,
    )
    .expect("static currency rows should be available without loading addons");

    for (name, quantity, max_quantity, icon_file_id) in [
        ("Updated Valorstones", 123, 456, 134400),
        ("Zero Valorstones", 0, 0, 134401),
    ] {
        {
            let state = env.state();
            let mut state = state.borrow_mut();
            let info = state
                .currency_info
                .get_mut(&2245)
                .expect("Valorstones should already exist in the currency map");
            info.name = name.to_string();
            info.quantity = quantity;
            info.max_quantity = max_quantity;
            info.icon_file_id = icon_file_id;
            info.is_show_in_backpack = false;
        }

        env.exec(&format!(
            r#"
            local info = C_CurrencyInfo.GetCurrencyListInfo(2)
            local legacy = {{ GetCurrencyListInfo(2) }}
            assert(select('#', GetCurrencyListInfo(2)) == 12)
            assert(legacy[1] == "{name}" and legacy[1] == info.name)
            assert(legacy[5] == true and info.isShowInBackpack == true)
            assert(legacy[6] == {quantity} and legacy[6] == info.quantity)
            assert(legacy[7] == {icon_file_id} and legacy[7] == info.iconFileID)
            assert(legacy[8] == {max_quantity} and legacy[8] == info.maxQuantity)
            assert(legacy[12] == 2245 and legacy[12] == info.currencyTypesID)
            assert(legacy[2] == false and info.isHeader == false)
            assert(legacy[3] == false and info.isHeaderExpanded == false)
            assert(legacy[4] == false and legacy[11] == false)

            local backpack = C_CurrencyInfo.GetBackpackCurrencyInfo(1)
            local legacyBackpack = {{ GetBackpackCurrencyInfo(1) }}
            assert(select('#', GetBackpackCurrencyInfo(1)) == 4)
            assert(legacyBackpack[1] == "{name}" and legacyBackpack[1] == backpack.name)
            assert(legacyBackpack[2] == {quantity} and legacyBackpack[2] == backpack.quantity)
            assert(legacyBackpack[3] == {icon_file_id} and legacyBackpack[3] == backpack.iconFileID)
            assert(legacyBackpack[4] == 2245 and legacyBackpack[4] == backpack.currencyTypesID)

            local baseline = currencyPreservationBaseline
            assert(GetCurrencyListSize() == baseline.size)
            for _, row in ipairs({{ {{1, baseline.header}}, {{3, baseline.unwatched}} }}) do
                local unchanged = {{ GetCurrencyListInfo(row[1]) }}
                assert(select('#', GetCurrencyListInfo(row[1])) == 12)
                for slot = 1, 12 do
                    assert(unchanged[slot] == row[2][slot])
                end
                local unchangedInfo = C_CurrencyInfo.GetCurrencyListInfo(row[1])
                assert(unchangedInfo.isHeader == unchanged[2])
                assert(unchangedInfo.isHeaderExpanded == unchanged[3])
                assert(unchangedInfo.isShowInBackpack == unchanged[5])
            end
            local neighbor = {{ GetBackpackCurrencyInfo(2) }}
            assert(select('#', GetBackpackCurrencyInfo(2)) == 4)
            for slot = 1, 4 do
                assert(neighbor[slot] == baseline.neighbor[slot])
            end
            local neighborInfo = C_CurrencyInfo.GetBackpackCurrencyInfo(2)
            assert(neighborInfo.currencyTypesID == 3089)
            assert(neighborInfo.name == neighbor[1])
            assert(neighborInfo.quantity == neighbor[2])
            assert(neighborInfo.iconFileID == neighbor[3])
            "#,
        ))
        .expect("legacy display tuples should update without changing static currency rows");
    }
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

#[test]
fn token_row_chat_link_click_inserts_supplied_currency_link_into_chat() {
    let env = load_mists_token_ui_env();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state
            .currency_info
            .get_mut(&2245)
            .expect("Valorstones should already exist in the currency map")
            .name = "Seeded Valorstones".to_string();
        state.modifier_keys.shift = true;
    }

    // Consumers: TokenButton_OnClick (Cata/Blizzard_TokenUI.lua:242) and the row's
    // LinkButton OnClick (Cata/Blizzard_TokenUI.xml:109) pass
    // C_CurrencyInfo.GetCurrencyListLink(index) to HandleModifiedItemClick, which
    // inserts it into the active chat edit box.
    let (row_text, link_button_text, other_text, other_id): (String, String, String, i32) = env
        .eval(
            r#"
            TokenFrame_Update()
            ChatFrameUtil.OpenChat("")
            local editBox = ChatFrameUtil.GetActiveWindow()
            assert(editBox, "chat edit box should be active")
            assert(IsModifiedClick("CHATLINK"))
            local rows = TokenFrameContainer.buttons
            assert(rows[2].name:GetText() == "Seeded Valorstones")
            rows[2]:Click("LeftButton")
            local rowText = editBox:GetText()
            editBox:SetText("")
            rows[2].LinkButton:Click("LeftButton")
            local linkButtonText = editBox:GetText()
            editBox:SetText("")
            rows[3]:Click("LeftButton")
            return rowText, linkButtonText, editBox:GetText(),
                C_CurrencyInfo.GetCurrencyListInfo(3).currencyTypesID
            "#,
        )
        .inspect_err(|err| log_error_boundary("chat-link-eval", &err.to_string()))
        .expect("modified TokenUI row clicks should reach the chat edit box");

    assert!(
        row_text.contains("|Hcurrency:2245|h[Seeded Valorstones]|h"),
        "row click should insert the seeded Valorstones link: {row_text:?}"
    );
    assert_eq!(
        link_button_text, row_text,
        "LinkButton click should insert the same row link"
    );
    assert_ne!(other_id, 2245);
    assert!(
        other_text.contains(&format!(
            "|Hcurrency:{other_id}|h[Weathered Harbinger Crest]|h"
        )),
        "third row click should insert its own currency link: {other_text:?}"
    );
}
