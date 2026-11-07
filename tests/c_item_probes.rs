//! Tests for `C_Item` probe methods backed by the static `ITEM_DB`
//! (seeded item-db table in `crates/sim-data/data/items.rs`):
//!
//! - `C_Item.GetItemIconByID(itemID)` — returns the icon fileDataID
//!   or the generic-placeholder `134400` when unknown.
//! - `C_Item.GetItemNameByID(itemID)` — returns the item name or
//!   `"Unknown"` when unknown.
//! - `C_Item.GetItemQualityByID(itemID)` — returns the quality index
//!   (0–7) or `0` (Poor) when unknown.
//!
//! All three were listed in `NAMESPACE_NIL_STUBS` even though the real
//! implementations already ran (the stub registrar skips non-nil
//! namespace methods). Drop the dead entries.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

/// An item id known to be present in `crates/sim-data/data/items.rs` (Aqirite, seeded
/// at the top of the db).
const KNOWN_ITEM_ID: u32 = 210935;

#[test]
fn get_item_name_by_id_returns_db_name_for_known_item() {
    let env = env();
    let name: String = env
        .eval(&format!("return C_Item.GetItemNameByID({KNOWN_ITEM_ID})"))
        .unwrap();
    assert_eq!(name, "Aqirite");
}

#[test]
fn get_item_name_by_id_falls_back_to_unknown_for_missing_item() {
    let env = env();
    let name: String = env
        .eval("return C_Item.GetItemNameByID(999999999)")
        .unwrap();
    assert_eq!(name, "Unknown");
}

#[test]
fn item_existence_by_id_matches_seeded_item_database() {
    let env = env();
    let exists: bool = env
        .eval(&format!("return C_Item.DoesItemExistByID({KNOWN_ITEM_ID})"))
        .unwrap();
    let positive_id_may_exist: bool = env
        .eval("return C_Item.DoesItemExistByID(999999999)")
        .unwrap();
    assert!(exists);
    assert!(positive_id_may_exist);
}

#[test]
fn item_data_cached_by_id_matches_seeded_item_database() {
    let env = env();
    let cached: bool = env
        .eval(&format!(
            "return C_Item.IsItemDataCachedByID({KNOWN_ITEM_ID})"
        ))
        .unwrap();
    let missing_cached: bool = env
        .eval("return C_Item.IsItemDataCachedByID(999999999)")
        .unwrap();
    assert!(cached);
    assert!(missing_cached);
}

#[test]
fn get_item_info_returns_placeholder_for_missing_positive_item_id() {
    let env = env();
    let item_info: (String, String, i32) = env
        .eval(
            r#"
            local name, link, quality = C_Item.GetItemInfo(999999999)
            return name, link, quality
            "#,
        )
        .unwrap();
    assert_eq!(item_info.0, "Unknown");
    assert!(item_info.1.contains("Hitem:999999999"));
    assert_eq!(item_info.2, 0);
}

#[test]
fn request_load_item_data_fires_for_synthetic_item_data() {
    let env = env();
    let callbacks: (bool, bool) = env
        .eval(&format!(
            r#"
            local knownLoaded = false
            local missingLoaded = false
            ItemEventListener = {{
                FireCallbacks = function(_, itemID)
                    if itemID == {KNOWN_ITEM_ID} then
                        knownLoaded = true
                    elseif itemID == 999999999 then
                        missingLoaded = true
                    end
                end
            }}
            C_Item.RequestLoadItemDataByID({KNOWN_ITEM_ID})
            C_Item.RequestLoadItemDataByID(999999999)
            return knownLoaded, missingLoaded
            "#
        ))
        .unwrap();
    assert!(callbacks.0);
    assert!(callbacks.1);
}

#[test]
fn get_item_quality_by_id_returns_db_quality() {
    let env = env();
    let quality: i32 = env
        .eval(&format!(
            "return C_Item.GetItemQualityByID({KNOWN_ITEM_ID})"
        ))
        .unwrap();
    // Aqirite is quality 3 (Rare) in the seeded item database.
    assert_eq!(quality, 3);
}

#[test]
fn get_item_quality_by_id_returns_zero_for_missing_item() {
    let env = env();
    let quality: i32 = env
        .eval("return C_Item.GetItemQualityByID(999999999)")
        .unwrap();
    assert_eq!(quality, 0, "unknown items default to Poor (0)");
}

#[test]
fn get_item_icon_by_id_returns_seeded_icon() {
    let env = env();
    // Aqirite uses its seeded icon fileDataID.
    let icon: i32 = env
        .eval(&format!("return C_Item.GetItemIconByID({KNOWN_ITEM_ID})"))
        .unwrap();
    assert_eq!(icon, 134573);
}

#[test]
fn get_item_icon_by_id_returns_placeholder_for_missing_item() {
    let env = env();
    let icon: i32 = env
        .eval("return C_Item.GetItemIconByID(999999999)")
        .unwrap();
    assert_eq!(icon, 134400);
}

#[test]
fn c_item_probes_accept_item_hyperlinks() {
    let env = env();
    let (name, quality): (String, i32) = env
        .eval(&format!(
            r#"
            local link = "|cff1eff00|Hitem:{KNOWN_ITEM_ID}::::::::80:::::|h[Aqirite]|h|r"
            return C_Item.GetItemNameByID(link), C_Item.GetItemQualityByID(link)
            "#
        ))
        .unwrap();
    assert_eq!(name, "Aqirite");
    assert_eq!(quality, 3);
}

// Wrath pins no generated documentation; the reader is unpublished there.
#[cfg(not(feature = "client-wrath"))]
#[test]
fn get_item_num_sockets_counts_the_record_sockets() {
    let env = env();
    let (by_id, by_link, by_name, plain, unknown): (i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            local link = "|cnIQ4:|Hitem:268265::::::::90:::::::::|h[Aqirbane Reliquary]|h|r"
            return C_Item.GetItemNumSockets(128910),
                   C_Item.GetItemNumSockets(link),
                   C_Item.GetItemNumSockets("Aqirbane Reliquary"),
                   C_Item.GetItemNumSockets(212440),
                   C_Item.GetItemNumSockets(999999999)
            "#,
        )
        .unwrap();
    assert_eq!(by_id, 2, "Strom'kar declares two relic sockets");
    assert_eq!(by_link, 1);
    assert_eq!(by_name, 1);
    assert_eq!(plain, 0);
    assert_eq!(unknown, 0);
}

/// Round 2 of the native decomp (`pr46-49-decomp-answers.md`, Q2.e) counts the
/// non-zero entries of the record's three fixed socket slots, so a record can
/// never report more than three. The simulator stores exactly those three slots
/// (`crates/sim-data/data/items.rs`: `pub socket_types: [u8; 3]`), which makes a
/// wider record unrepresentable rather than countable-then-clamped.
#[cfg(not(feature = "client-wrath"))]
#[test]
fn get_item_num_sockets_caps_at_the_three_fixed_slots() {
    const FIXED_SLOTS: i32 = 3;
    let env = env();
    let (kind, count): (String, i32) = env
        .eval(
            r#"
            local count = C_Item.GetItemNumSockets(120978)
            return type(count), count
            "#,
        )
        .unwrap();
    assert_eq!(kind, "number");
    assert_eq!(count, 2, "Ashbringer declares two non-zero socket slots");
    assert!(count <= FIXED_SLOTS);
}

/// Q2.d: the bonus half indexes a `SpellItemEnchantment` entry by an `itemInfo`
/// dword that nothing writes in 70688, 70668 or 70707, so it is code-live and
/// data-dead. A link that carries bonus-id fields therefore still reports only
/// the record count.
#[cfg(not(feature = "client-wrath"))]
#[test]
fn get_item_num_sockets_ignores_the_unwritten_bonus_fields() {
    let env = env();
    let (without_bonus, with_bonus): (i32, i32) = env
        .eval(
            r#"
            local plain = "|cnIQ4:|Hitem:128910::::::::90:::::::::|h[Strom'kar, the Warbreaker]|h|r"
            local bonus = "|cnIQ4:|Hitem:128910::::::::90::::::::1:1234:5678|h[Strom'kar, the Warbreaker]|h|r"
            return C_Item.GetItemNumSockets(plain), C_Item.GetItemNumSockets(bonus)
            "#,
        )
        .unwrap();
    assert_eq!(without_bonus, 2);
    assert_eq!(with_bonus, 2, "the bonus addend stays zero in these builds");
}

/// Q2: an unknown or uncached item publishes the integer `0`, not nil.
#[cfg(not(feature = "client-wrath"))]
#[test]
fn get_item_num_sockets_publishes_a_number_for_an_unknown_item() {
    let env = env();
    let (kind, count): (String, i32) = env
        .eval(
            r#"
            local count = C_Item.GetItemNumSockets(999999999)
            return type(count), count
            "#,
        )
        .unwrap();
    assert_eq!(kind, "number");
    assert_eq!(count, 0);
}
