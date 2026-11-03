//! Tests for `C_CurrencyInfo` probes backed by `SimState.currency_info`:
//!
//! - `C_CurrencyInfo.GetCurrencyInfo(currencyID)` — returns the retail
//!   `CurrencyInfo` structure, or nothing when the currency id is not
//!   seeded.
//! - `C_CurrencyInfo.GetCurrencyInfoFromLink(link)` — parses the
//!   currency id out of a `|Hcurrency:<id>:...` hyperlink and returns
//!   the same structure.
//! - `C_CurrencyInfo.GetBasicCurrencyInfo(currencyID, quantity?)`
//!   — returns the retail `CurrencyDisplayInfo` structure used by
//!   addon hover tooltips.
//! - `C_CurrencyInfo.GetCurrencyContainerInfo(currencyID, quantity)`
//!   — returns the 6-field `CurrencyDisplayInfo` structure with
//!   `actualAmount` / `displayAmount` set to the passed quantity.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::CurrencyInfo;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn get_currency_info_returns_seeded_valorstones_row() {
    let env = env();
    let (name, quantity, quality, icon, is_show_in_backpack): (String, i32, i32, i32, bool) = env
        .eval(
            r#"
            local info = C_CurrencyInfo.GetCurrencyInfo(2245)
            return info.name,
                   info.quantity,
                   info.quality,
                   info.iconFileID,
                   info.isShowInBackpack
            "#,
        )
        .unwrap();
    assert_eq!(name, "Valorstones");
    assert_eq!(quantity, 1847);
    assert_eq!(quality, 3);
    assert_eq!(icon, 5868905);
    assert!(is_show_in_backpack, "Valorstones is a backpack currency");
}

#[test]
fn get_currency_info_covers_saved_instances_currency_ids() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local ids = {
                81, 515, 2588, 3363, 241, 391, 416, 402, 697, 738,
                752, 776, 777, 789, 823, 824, 994, 1101, 1129, 1149,
                1155, 1166, 1220, 1226, 1273, 1275, 1299, 1314, 1342,
                1501, 1508, 1533, 1710, 1580, 1560, 1587, 1716, 1717, 1718,
                1721, 1719, 1755, 1803, 1754, 1191, 1602, 1792, 1822,
                1767, 1828, 1810, 1813, 1816, 1819, 1820, 1885, 1889,
                1904, 1906, 1931, 1977, 1979, 2009, 2000, 2003, 2245,
                2123, 2797, 2045, 2118, 2122, 2409, 2410, 2411, 2412,
                2413, 2533, 2594, 2650, 2651, 2777, 2796, 2706, 2707,
                2708, 2709, 2774, 2657, 2912, 2806, 2807, 2809, 2812,
                2800, 3010, 2778, 3089, 2803, 2815, 3056, 3008, 2813,
                2914, 2915, 2916, 2917, 3023, 3100, 3090, 3218, 3220,
                3226, 3116, 3107, 3108, 3109, 3110, 3132, 3149, 3278,
                3303, 3356, 3269, 3284, 3286, 3288, 3290, 3141, 3319,
                3316, 3376, 3377, 3379, 3385, 3392, 3400, 3373, 3393,
                3405, 3256, 3257, 3258, 3259, 3260, 3261, 3262, 3263,
                3264, 3265, 3266, 3028, 3310, 3212, 3378, 3383, 3341,
                3343, 3345, 3347, 3418, 3508, 3442, 3443, 3444,
                3445, 3446, 3448, 3465, 3509, 3546,
            }
            for _, id in ipairs(ids) do
                local info = C_CurrencyInfo.GetCurrencyInfo(id)
                if not info or type(info.name) ~= "string" or info.name == "" then
                    return "missing:" .. tostring(id)
                end
            end
            table.sort(ids, function(a, b)
                return C_CurrencyInfo.GetCurrencyInfo(a).name < C_CurrencyInfo.GetCurrencyInfo(b).name
            end)
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "SavedInstances currency sort needs every tracked currency to expose a name: {result}"
    );
}

#[derive(serde::Deserialize)]
struct ExpectedCurrencyMetadata {
    id: i32,
    name: String,
    description: String,
    icon: i32,
    max_quantity: i32,
    max_weekly_quantity: i32,
    quality: i32,
    transfer_percentage: Option<f64>,
}

#[test]
fn get_currency_info_matches_new_saved_instances_currency_metadata() {
    let env = env();
    let expected: Vec<ExpectedCurrencyMetadata> =
        serde_json::from_str(include_str!("fixtures/currency-types-12.1.0.69497.json"))
            .expect("parse pinned CurrencyTypes rows");
    for row in expected {
        let actual: (String, String, i32, i32, i32, i32, Option<f64>, i32) = env
            .eval(&format!(
                r#"
                local info = C_CurrencyInfo.GetCurrencyInfo({})
                assert(info, "missing currency {}")
                assert(info.currencyID == {})
                return info.name, info.description, info.iconFileID, info.maxQuantity,
                       info.maxWeeklyQuantity, info.quality, info.transferPercentage, info.quantity
                "#,
                row.id, row.id, row.id
            ))
            .unwrap_or_else(|error| panic!("currency {}: {error}", row.id));
        assert_eq!(
            actual,
            (
                row.name,
                row.description,
                row.icon,
                row.max_quantity,
                row.max_weekly_quantity,
                row.quality,
                row.transfer_percentage,
                0,
            ),
            "currency {} must retain its pinned DB2 metadata",
            row.id
        );
    }
}

#[test]
fn get_currency_info_exposes_all_retail_fields() {
    let env = env();
    let (has_id, has_name, has_desc, has_qty, has_max_qty, has_quality, has_icon): (
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local info = C_CurrencyInfo.GetCurrencyInfo(2245)
            return info.currencyID ~= nil,
                   info.name ~= nil,
                   info.description ~= nil,
                   info.quantity ~= nil,
                   info.maxQuantity ~= nil,
                   info.quality ~= nil,
                   info.iconFileID ~= nil
            "#,
        )
        .unwrap();
    assert!(has_id);
    assert!(has_name);
    assert!(has_desc);
    assert!(has_qty);
    assert!(has_max_qty);
    assert!(has_quality);
    assert!(has_icon);
}

#[test]
fn get_basic_currency_info_returns_display_info_for_saved_instances_hover() {
    let env = env();
    let (name, description, icon, quality, display, actual): (String, String, i32, i32, i32, i32) =
        env.eval(
            r#"
            local info = C_CurrencyInfo.GetBasicCurrencyInfo(2245, 17)
            return info.name,
                   info.description,
                   info.icon,
                   info.quality,
                   info.displayAmount,
                   info.actualAmount
            "#,
        )
        .unwrap();

    assert_eq!(name, "Valorstones");
    assert!(!description.is_empty());
    assert_eq!(icon, 5868905);
    assert_eq!(quality, 3);
    assert_eq!(display, 17);
    assert_eq!(actual, 17);
}

#[test]
fn get_currency_info_returns_nothing_for_unknown_id() {
    let env = env();
    let nret: i32 = env
        .eval("return select('#', C_CurrencyInfo.GetCurrencyInfo(999999))")
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_currency_info_reflects_sim_state_mutation() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.currency_info.insert(
            42,
            CurrencyInfo {
                currency_id: 42,
                name: "Bonus Currency".into(),
                quantity: 7,
                quality: 4,
                ..CurrencyInfo::default()
            },
        );
    }

    let (name, qty, quality): (String, i32, i32) = env
        .eval(
            r#"
            local info = C_CurrencyInfo.GetCurrencyInfo(42)
            return info.name, info.quantity, info.quality
            "#,
        )
        .unwrap();
    assert_eq!(name, "Bonus Currency");
    assert_eq!(qty, 7);
    assert_eq!(quality, 4);
}

fn assert_currency_membership_preserved(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        assert(C_CurrencyInfo.GetCurrencyListSize() == 19)
        local header = C_CurrencyInfo.GetCurrencyListInfo(1)
        assert(header.currencyTypesID == 0)
        assert(header.name == "The War Within")
        assert(header.quantity == 0 and header.maxQuantity == 0)
        assert(header.iconFileID == 0 and header.quality == 0)
        assert(header.isHeader == true and header.isHeaderExpanded == true)
        assert(header.isShowInBackpack == false)

        local listed = C_CurrencyInfo.GetCurrencyListInfo(6)
        assert(listed.currencyTypesID == 3089)
        assert(listed.name == "Resonance Crystals" and listed.quantity == 620)
        assert(listed.maxQuantity == 4000)
        assert(listed.iconFileID == 3528287 and listed.quality == 3)
        assert(listed.isHeader == false and listed.isHeaderExpanded == false)
        assert(listed.isShowInBackpack == true)
        local watched = C_CurrencyInfo.GetBackpackCurrencyInfo(2)
        assert(watched.currencyTypesID == 3089)
        assert(watched.name == "Resonance Crystals" and watched.quantity == 620)
        assert(watched.iconFileID == 3528287)
        assert(C_CurrencyInfo.GetBackpackCurrencyInfo(3).currencyTypesID == 1792)

        for _, index in ipairs({0, 20}) do
            assert(select('#', C_CurrencyInfo.GetCurrencyListInfo(index)) == 1)
            assert(C_CurrencyInfo.GetCurrencyListInfo(index) == nil)
        end
        for _, index in ipairs({0, 4}) do
            assert(select('#', C_CurrencyInfo.GetBackpackCurrencyInfo(index)) == 0)
        end
        "#,
    )
    .unwrap();
}

#[test]
fn get_currency_list_info_reflects_successive_existing_currency_updates() {
    let env = env();
    for (name, quantity, max_quantity, icon, quality) in [
        ("Updated Valorstones", 73, 250, 134400, 4),
        ("Revised Valorstones", 0, 0, 134401, 0),
    ] {
        {
            let mut state = env.state().borrow_mut();
            let info = state.currency_info.get_mut(&2245).unwrap();
            info.name = name.into();
            info.quantity = quantity;
            info.max_quantity = max_quantity;
            info.icon_file_id = icon;
            info.quality = quality;
        }
        let actual: (String, i32, i32, i64, i32) = env
            .eval(&format!(
                r#"
                local listed = C_CurrencyInfo.GetCurrencyListInfo(2)
                assert(listed.currencyTypesID == 2245)
                if {} then assert(listed.currencyID == 2245) end
                assert(listed.isHeader == false)
                assert(listed.isHeaderExpanded == false)
                assert(listed.isShowInBackpack == true)
                local fieldCount = 0
                for _ in pairs(listed) do fieldCount = fieldCount + 1 end
                assert(fieldCount == {})
                local detail = C_CurrencyInfo.GetCurrencyInfo(2245)
                assert(detail.currencyID == 2245)
                assert(listed.name == detail.name)
                assert(listed.quantity == detail.quantity)
                assert(listed.maxQuantity == detail.maxQuantity)
                assert(listed.iconFileID == detail.iconFileID)
                assert(listed.quality == detail.quality)
                return listed.name, listed.quantity, listed.maxQuantity,
                       listed.iconFileID, listed.quality
                "#,
                if cfg!(feature = "client-retail") {
                    "true"
                } else {
                    "false"
                },
                if cfg!(feature = "client-retail") {
                    13
                } else {
                    9
                }
            ))
            .unwrap();
        assert_eq!(
            actual,
            (
                name.into(),
                quantity,
                max_quantity,
                i64::from(icon),
                quality
            )
        );
        assert_currency_membership_preserved(&env);
    }
}

#[test]
fn get_backpack_currency_info_reflects_successive_existing_currency_updates() {
    let env = env();
    for (name, quantity, max_quantity, icon, quality) in [
        ("Updated Valorstones", 73, 250, 134400, 4),
        ("Revised Valorstones", 0, 0, 134401, 0),
    ] {
        {
            let mut state = env.state().borrow_mut();
            let info = state.currency_info.get_mut(&2245).unwrap();
            info.name = name.into();
            info.quantity = quantity;
            info.max_quantity = max_quantity;
            info.icon_file_id = icon;
            info.quality = quality;
        }
        let actual: (String, i32, i64, i32, i32) = env
            .eval(
                r#"
                local watched = C_CurrencyInfo.GetBackpackCurrencyInfo(1)
                assert(watched.currencyTypesID == 2245)
                assert(watched.maxQuantity == nil and watched.quality == nil)
                assert(watched.isHeader == nil and watched.isHeaderExpanded == nil)
                assert(watched.isShowInBackpack == nil and watched.currencyID == nil)
                local fieldCount = 0
                for _ in pairs(watched) do fieldCount = fieldCount + 1 end
                assert(fieldCount == 4)
                local detail = C_CurrencyInfo.GetCurrencyInfo(2245)
                assert(detail.currencyID == 2245)
                assert(watched.name == detail.name)
                assert(watched.quantity == detail.quantity)
                assert(watched.iconFileID == detail.iconFileID)
                return watched.name, watched.quantity, watched.iconFileID,
                       detail.maxQuantity, detail.quality
                "#,
            )
            .unwrap();
        assert_eq!(
            actual,
            (
                name.into(),
                quantity,
                i64::from(icon),
                max_quantity,
                quality
            )
        );
        assert_currency_membership_preserved(&env);
    }
}

#[test]
fn get_currency_list_info_returns_nil_when_existing_currency_row_is_removed() {
    let env = env();
    assert!(
        env.state()
            .borrow_mut()
            .currency_info
            .remove(&2245)
            .is_some()
    );
    env.eval::<()>(
        r#"
        assert(select('#', C_CurrencyInfo.GetCurrencyListInfo(2)) == 1)
        assert(C_CurrencyInfo.GetCurrencyListInfo(2) == nil)
        assert(select('#', C_CurrencyInfo.GetCurrencyInfo(2245)) == 0)
        "#,
    )
    .unwrap();
    assert_currency_membership_preserved(&env);
}

#[test]
fn get_backpack_currency_info_returns_nothing_when_existing_currency_row_is_removed() {
    let env = env();
    assert!(
        env.state()
            .borrow_mut()
            .currency_info
            .remove(&2245)
            .is_some()
    );
    env.eval::<()>(
        r#"
        assert(select('#', C_CurrencyInfo.GetBackpackCurrencyInfo(1)) == 0)
        assert(C_CurrencyInfo.GetBackpackCurrencyInfo(1) == nil)
        assert(select('#', C_CurrencyInfo.GetCurrencyInfo(2245)) == 0)
        "#,
    )
    .unwrap();
    assert_currency_membership_preserved(&env);
}

#[test]
fn get_currency_info_from_link_parses_currency_hyperlink() {
    let env = env();
    let (name, qty): (String, i32) = env
        .eval(
            r#"
            local link = "|cffa335ee|Hcurrency:2245:1847|h[Valorstones]|h|r"
            local info = C_CurrencyInfo.GetCurrencyInfoFromLink(link)
            return info.name, info.quantity
            "#,
        )
        .unwrap();
    assert_eq!(name, "Valorstones");
    assert_eq!(qty, 1847);
}

#[test]
fn get_currency_info_from_link_returns_nothing_for_non_currency_link() {
    let env = env();
    let nret: i32 = env
        .eval(
            r#"
            return select('#', C_CurrencyInfo.GetCurrencyInfoFromLink("|Hitem:12345::::|h[Some Item]|h"))
            "#,
        )
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_currency_info_from_link_returns_nothing_for_unknown_currency_id() {
    let env = env();
    let nret: i32 = env
        .eval(
            r#"
            return select('#', C_CurrencyInfo.GetCurrencyInfoFromLink("|Hcurrency:999999:10|h[X]|h"))
            "#,
        )
        .unwrap();
    assert_eq!(nret, 0);
}

#[test]
fn get_currency_container_info_returns_display_info_with_passed_quantity() {
    let env = env();
    let (actual, display, name, icon, quality): (i32, i32, String, i32, i32) = env
        .eval(
            r#"
            local info = C_CurrencyInfo.GetCurrencyContainerInfo(2245, 250)
            return info.actualAmount,
                   info.displayAmount,
                   info.name,
                   info.icon,
                   info.quality
            "#,
        )
        .unwrap();
    assert_eq!(actual, 250);
    assert_eq!(display, 250);
    assert_eq!(name, "Valorstones");
    assert_eq!(icon, 5868905);
    assert_eq!(quality, 3);
}

#[test]
fn get_currency_container_info_returns_nothing_for_unknown_id() {
    let env = env();
    let nret: i32 = env
        .eval("return select('#', C_CurrencyInfo.GetCurrencyContainerInfo(999999, 1))")
        .unwrap();
    assert_eq!(nret, 0);
}
