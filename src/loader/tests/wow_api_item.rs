//! C_Item API coverage extracted from `wow_api.rs`.

use super::*;

#[test]
fn test_c_item_get_sub_class_info() {
    let env = WowLuaEnv::new().unwrap();
    let name: String = env
        .eval("return C_Item.GetItemSubClassInfo(Enum.ItemClass.Tradegoods, 4)")
        .unwrap();
    assert_eq!(name, "Jewelcrafting");
}

#[test]
fn test_c_item_get_sub_class_info_returns_nil_for_unknown_subclass() {
    let env = WowLuaEnv::new().unwrap();
    let is_nil: bool = env
        .eval("return C_Item.GetItemSubClassInfo(Enum.ItemClass.Tradegoods, 999) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_c_item_functions_exist() {
    let env = WowLuaEnv::new().unwrap();
    for f in &[
        "GetItemIconByID",
        "GetDetailedItemLevelInfo",
        "GetItemInfoInstant",
        "GetItemInfo",
    ] {
        let expr = format!("return type(C_Item.{})", f);
        let ty: String = env.eval(&expr).unwrap();
        assert_eq!(ty, "function", "C_Item.{} should be function", f);
    }
}

#[test]
fn test_c_item_get_item_info_returns_multi_value() {
    let env = WowLuaEnv::new().unwrap();
    // GetItemInfo returns 17 values. Item 6948 = Hearthstone.
    let name: String = env
        .eval("local n = C_Item.GetItemInfo(6948); return n")
        .unwrap();
    assert_eq!(name, "Hearthstone");

    let quality: i32 = env
        .eval("local _,_,q = C_Item.GetItemInfo(6948); return q")
        .unwrap();
    assert_eq!(quality, 1, "Hearthstone quality should be 1 (Common)");

    let bind_type: i32 = env
        .eval("return select(14, C_Item.GetItemInfo(6948))")
        .unwrap();
    assert!(bind_type >= 0, "bindType should be a valid number");
}

#[test]
fn test_get_item_info_accepts_bare_item_strings() {
    let env = WowLuaEnv::new().unwrap();
    // AllTheThings queries items by bare item strings, not full links.
    let results: String = env
        .eval(
            r#"
            local out = {}
            for _, query in ipairs({ "item:6948", "item:6948:::::::::::1:1:3524:" }) do
                local name, link, quality = GetItemInfo(query)
                out[#out + 1] = tostring(name) .. "|" .. tostring(quality) .. "|" .. tostring(link == select(2, GetItemInfo(6948)))
            end
            return table.concat(out, ",")
            "#,
        )
        .unwrap();
    assert_eq!(results, "Hearthstone|1|true,Hearthstone|1|true");
}

#[test]
#[cfg(feature = "client-mists")]
fn test_c_item_get_classic_tradegoods_subclass_info() {
    let env = WowLuaEnv::new().unwrap();
    let (explosives, devices): (String, String) = env
        .eval(
            r#"
            return C_Item.GetItemSubClassInfo(Enum.ItemClass.Tradegoods, 2),
                C_Item.GetItemSubClassInfo(Enum.ItemClass.Tradegoods, 3)
            "#,
        )
        .unwrap();
    assert_eq!(explosives, "Explosives");
    assert_eq!(devices, "Devices");
}

#[test]
#[cfg(feature = "client-mists")]
fn test_c_item_get_classic_consumable_subclass_info() {
    let env = WowLuaEnv::new().unwrap();
    let matches_classic_names: bool = env
        .eval(
            r#"
            local expected = {
                [1] = "Potion",
                [2] = "Elixir",
                [3] = "Flask",
                [4] = "Scroll",
                [6] = "Item Enhancement",
                [7] = "Bandage",
            }
            for subclassID, expectedName in pairs(expected) do
                if C_Item.GetItemSubClassInfo(Enum.ItemClass.Consumable, subclassID) ~= expectedName then
                    return false
                end
            end
            return true
            "#,
        )
        .unwrap();
    assert!(matches_classic_names);
}

#[test]
#[cfg(feature = "client-mists")]
fn test_c_item_get_tradegoods_class_info() {
    let env = WowLuaEnv::new().unwrap();
    let name: String = env
        .eval("return C_Item.GetItemClassInfo(Enum.ItemClass.Tradegoods)")
        .unwrap();
    assert_eq!(name, "Trade Goods");
}

#[test]
#[cfg(not(feature = "client-mists"))]
fn test_c_item_get_tradegoods_class_info() {
    let env = WowLuaEnv::new().unwrap();
    let name: String = env
        .eval("return C_Item.GetItemClassInfo(Enum.ItemClass.Tradegoods)")
        .unwrap();
    assert_eq!(name, "Tradeskill");
}

#[test]
fn test_c_item_subclass_names_match_en_us_keywords() {
    let env = WowLuaEnv::new().unwrap();
    let weapon_11 = if cfg!(feature = "client-mists") {
        "one-handed exotics"
    } else {
        "bear claws"
    };
    let weapon_12 = if cfg!(feature = "client-mists") {
        "two-handed exotics"
    } else {
        "catclaws"
    };
    let lua = format!(
        r#"
        local expected = {{
            {{Enum.ItemClass.Armor, 6, "shields"}},
            {{Enum.ItemClass.Weapon, 0, "one-handed axes"}},
            {{Enum.ItemClass.Weapon, 1, "two-handed axes"}},
            {{Enum.ItemClass.Weapon, 4, "one-handed maces"}},
            {{Enum.ItemClass.Weapon, 5, "two-handed maces"}},
            {{Enum.ItemClass.Weapon, 6, "polearms"}},
            {{Enum.ItemClass.Weapon, 11, "{weapon_11}"}},
            {{Enum.ItemClass.Weapon, 12, "{weapon_12}"}},
            {{Enum.ItemClass.Weapon, 13, "fist weapons"}},
        }}
        for _, entry in ipairs(expected) do
            if C_Item.GetItemSubClassInfo(entry[1], entry[2]):lower() ~= entry[3] then
                return false
            end
        end
        return true
        "#
    );
    let names_match_keywords: bool = env.eval(&lua).unwrap();
    assert!(names_match_keywords);
}
