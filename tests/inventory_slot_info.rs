use crate::common::inventory_slot_info_chunk;
use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn reagent_bag_slot_uses_bag_file_data_id() {
    let (id, texture, _check_relic): (i32, i32, bool) = env()
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("ReagentBag0Slot")"#))
        .unwrap();
    assert_eq!(id, 25);
    assert_eq!(texture, 136511);
}

#[test]
fn wrist_slot_maps_to_wrists_file_data_id() {
    let (_, texture, _): (i32, i32, bool) = env()
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("WristSlot")"#))
        .unwrap();
    assert_eq!(texture, 136530);
}

#[test]
fn main_hand_slot_uses_main_hand_file_data_id() {
    let (_, texture, _): (i32, i32, bool) = env()
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("MainHandSlot")"#))
        .unwrap();
    assert_eq!(texture, 136518);
}

#[test]
fn bag_slots_share_single_bag_file_data_id() {
    let env = env();
    for slot in ["Bag0Slot", "Bag1Slot", "Bag2Slot", "Bag3Slot", "Bag4Slot"] {
        let code = format!(r#"return GetInventorySlotInfo("{slot}")"#);
        let (_, texture, _): (i32, i32, bool) = env.eval(&inventory_slot_info_chunk(&code)).unwrap();
        assert_eq!(texture, 136511, "slot {slot} should share Bag fileDataID");
    }
}

#[test]
fn profession_equipment_slots_resolve_to_retail_slot_ids() {
    let env = env();
    for (slot, expected_id) in [
        ("Prof0ToolSlot", 20),
        ("Prof0Gear0Slot", 21),
        ("Prof0Gear1Slot", 22),
        ("Prof1ToolSlot", 23),
        ("Prof1Gear0Slot", 24),
        ("Prof1Gear1Slot", 25),
        ("CookingToolSlot", 26),
        ("CookingGear0Slot", 27),
        ("FishingToolSlot", 28),
    ] {
        let code = format!(r#"return GetInventorySlotInfo("{slot}")"#);
        let (id, texture, check_relic): (i32, i32, bool) = env.eval(&inventory_slot_info_chunk(&code)).unwrap();
        assert_eq!(id, expected_id, "slot {slot} should resolve");
        assert_ne!(texture, 0, "slot {slot} should have an empty-slot texture");
        assert!(!check_relic, "slot {slot} checkRelic should be false");
    }
}

#[test]
fn main_hand_slot_id_is_16() {
    let id: i32 = env()
        .eval(&inventory_slot_info_chunk(r#"return (GetInventorySlotInfo("MainHandSlot"))"#))
        .unwrap();
    assert_eq!(id, 16);
}

#[test]
fn get_inventory_slot_info_returns_three_values() {
    let count: i32 = env()
        .eval(&inventory_slot_info_chunk(r#"return select('#', GetInventorySlotInfo("HeadSlot"))"#))
        .unwrap();
    assert_eq!(
        count, 3,
        "canonical shape is (slotId, textureFileID, checkRelic)"
    );
}

#[test]
fn check_relic_is_always_false_on_retail() {
    // Third return is the legacy classic-era relic-slot flag. Retail has
    // no relic slot so every call returns false.
    let env = env();
    for slot in ["HeadSlot", "MainHandSlot", "TrinketSlot0", "Finger1Slot"] {
        let (_, _, check_relic): (i32, i32, bool) = env
            .eval(&inventory_slot_info_chunk(&format!(r#"return GetInventorySlotInfo("{slot}")"#)))
            .unwrap_or((0, 0, false));
        assert!(!check_relic, "slot {slot} checkRelic should be false");
    }
}

#[test]
fn case_insensitive_slot_names() {
    let env = env();
    let upper: i32 = env
        .eval(&inventory_slot_info_chunk(r#"return (GetInventorySlotInfo("HEADSLOT"))"#))
        .unwrap();
    let mixed: i32 = env
        .eval(&inventory_slot_info_chunk(r#"return (GetInventorySlotInfo("HeadSlot"))"#))
        .unwrap();
    let lower: i32 = env
        .eval(&inventory_slot_info_chunk(r#"return (GetInventorySlotInfo("headslot"))"#))
        .unwrap();
    assert_eq!(upper, 1);
    assert_eq!(mixed, 1);
    assert_eq!(lower, 1);
}

#[test]
fn unknown_slot_returns_nil() {
    let ok: bool = env()
        .eval(&inventory_slot_info_chunk(r#"return GetInventorySlotInfo("NotASlot") == nil"#))
        .unwrap();
    assert!(ok);
}

#[test]
fn non_string_arg_returns_nil() {
    let ok: bool = env()
        .eval(&inventory_slot_info_chunk(
            r#"
            return GetInventorySlotInfo(nil) == nil
               and GetInventorySlotInfo(42) == nil
               and GetInventorySlotInfo(true) == nil
            "#,
        ))
        .unwrap();
    assert!(ok);
}

#[test]
fn slot_id_works_as_table_key() {
    // Real-world callsite: CANCELABLE_ITEMS[GetInventorySlotInfo(...)] = 1
    let ok: bool = env()
        .eval(&inventory_slot_info_chunk(
            r#"
            local id = GetInventorySlotInfo("MainHandSlot")
            local t = {}
            t[id] = "marker"
            return t[16] == "marker"
            "#,
        ))
        .unwrap();
    assert!(ok);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn deprecated_paper_doll_startup_addon_publishes_secure_global_alias() {
    use wow_ui_sim::screen::ScreenKind;

    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available");
    let (_, toc_path) =
        wow_ui_sim::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
            .into_iter()
            .find(|(name, _)| name == "Deprecated_PaperDoll")
            .expect("built-in Deprecated_PaperDoll must be a Game startup addon");
    let env = env();
    let before: bool = env.eval("return GetInventorySlotInfo == nil").unwrap();
    assert!(before, "12.1.0 has no native GetInventorySlotInfo global");

    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc_path)
        .expect("Deprecated_PaperDoll should load");

    let (aliased, secure, slot_id): (bool, bool, i32) = env
        .eval(
            r#"
            return GetInventorySlotInfo == C_PaperDollInfo.GetInventorySlotInfo,
                   issecurevariable("GetInventorySlotInfo"),
                   (GetInventorySlotInfo("MainHandSlot"))
            "#,
        )
        .unwrap();
    assert!(aliased, "Deprecated_PaperDoll aliases the C_PaperDollInfo producer");
    assert!(secure, "built-in addon assignments must not carry addon taint");
    assert_eq!(slot_id, 16);
}
