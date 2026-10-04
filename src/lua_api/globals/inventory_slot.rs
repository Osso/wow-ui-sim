//! `GetInventorySlotInfo(slotName)` — canonical WoW slot id + icon fileDataID.
//!
//! Real signature: `(slotId, textureFileID, checkRelic)`. The `checkRelic`
//! third return is a legacy classic-era flag for the Relic equipment slot,
//! which retail no longer has — always `false`. Callsites typically use the
//! numeric `slotId` as a TABLE KEY (e.g.
//! `CANCELABLE_ITEMS[GetInventorySlotInfo("MainHandSlot")] = 1`), so returning
//! `nil` for unknown names would crash the chunk with "table index is nil";
//! we therefore only answer the real retail slot names.
//!
//! The icon fileDataIDs mirror `PaperDollItemFrame.SlotIconFileID` — the DB
//! maps ItemButtonName to a specific icon, not a naive
//! `UI-PaperDoll-Slot-<slotName>` concat. Mismatches (WristSlot→Wrists,
//! BackSlot→Rear, Bag*Slot→Bag, ReagentBag0Slot→Bag, AmmoSlot→Ammo) cause
//! "Not found" warnings for visible slots in the paperdoll UI.

use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const EMPTY_PROFESSION_SLOT_ICON: i32 = 130766;

/// `(slot_id, icon_file_id)` for every named equipment/bag slot the sim
/// recognises. Keys are lowercase for case-insensitive lookup.
const INVENTORY_SLOTS: &[(&'static str, i32, i32)] = &[
    ("headslot", 1, 136516),
    ("neckslot", 2, 136519),
    ("shoulderslot", 3, 136526),
    ("shirtslot", 4, 136525),
    ("chestslot", 5, 136512),
    ("waistslot", 6, 136529),
    ("legsslot", 7, 136517),
    ("feetslot", 8, 136513),
    ("wristslot", 9, 136530),
    ("handsslot", 10, 136515),
    ("finger0slot", 11, 136514),
    ("finger1slot", 12, 136514),
    ("trinket0slot", 13, 136528),
    ("trinket1slot", 14, 136528),
    ("backslot", 15, 136521),
    ("mainhandslot", 16, 136518),
    ("secondaryhandslot", 17, 136524),
    ("rangedslot", 18, 136520),
    ("tabardslot", 19, 136527),
    ("prof0toolslot", 20, EMPTY_PROFESSION_SLOT_ICON),
    ("prof0gear0slot", 21, EMPTY_PROFESSION_SLOT_ICON),
    ("prof0gear1slot", 22, EMPTY_PROFESSION_SLOT_ICON),
    ("prof1toolslot", 23, EMPTY_PROFESSION_SLOT_ICON),
    ("prof1gear0slot", 24, EMPTY_PROFESSION_SLOT_ICON),
    ("prof1gear1slot", 25, EMPTY_PROFESSION_SLOT_ICON),
    ("cookingtoolslot", 26, EMPTY_PROFESSION_SLOT_ICON),
    ("cookinggear0slot", 27, EMPTY_PROFESSION_SLOT_ICON),
    ("fishingtoolslot", 28, EMPTY_PROFESSION_SLOT_ICON),
    ("ammoslot", 0, 136510),
    ("bag0slot", 20, 136511),
    ("bag1slot", 21, 136511),
    ("bag2slot", 22, 136511),
    ("bag3slot", 23, 136511),
    ("bag4slot", 24, 136511),
    ("reagentbag0slot", 25, 136511),
    ("reagentbagslot", 25, 136511),
];

pub(crate) fn lookup_slot(name: &str) -> Option<(i32, i32)> {
    let needle = name.to_ascii_lowercase();
    INVENTORY_SLOTS
        .iter()
        .find(|(key, _, _)| *key == needle)
        .map(|(_, id, icon)| (*id, *icon))
}

pub(crate) fn get_inventory_slot_info(state: &mut LuaState) -> LuaResult<u32> {
    let name_val = crate::lua_bridge::stack_val(state, 1);
    let name = match name_val {
        Val::Str(s) => state
            .gc
            .string_arena
            .get(s)
            .and_then(|lua_str| std::str::from_utf8(lua_str.data()).ok())
            .map(str::to_owned),
        _ => None,
    };
    let Some(name) = name else {
        // Non-string or nil arg: return nil, matching WoW's "unknown slot" path.
        state.push(Val::Nil);
        return Ok(1);
    };
    let Some((slot_id, icon)) = lookup_slot(&name) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    // Canonical return shape: (slotId, textureFileID, checkRelic).
    // Texture is a numeric fileDataID (retail convention — SetTexture accepts
    // the fileDataID form directly). checkRelic is false on retail (Relic
    // slot doesn't exist anymore).
    state.push(Val::Num(slot_id as f64));
    state.push(Val::Num(icon as f64));
    state.push(Val::Bool(false));
    Ok(3)
}

#[cfg(not(feature = "retail-12-1-0"))]
pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use crate::lua_bridge::table_set_rust_fn_static;
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    table_set_rust_fn_static(
        state,
        state.global,
        "GetInventorySlotInfo",
        get_inventory_slot_info,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::lookup_slot;

    #[test]
    fn head_slot_resolves() {
        assert_eq!(lookup_slot("HeadSlot"), Some((1, 136516)));
    }

    #[test]
    fn ammo_slot_is_zero_id() {
        // Ammo slot legitimately has slot_id = 0 (classic holdover).
        assert_eq!(lookup_slot("AmmoSlot"), Some((0, 136510)));
    }

    #[test]
    fn reagent_bag_aliases_share_id() {
        assert_eq!(lookup_slot("ReagentBag0Slot"), Some((25, 136511)));
        assert_eq!(lookup_slot("ReagentBagSlot"), Some((25, 136511)));
    }

    #[test]
    fn finger_slots_share_icon_but_differ_in_id() {
        assert_eq!(lookup_slot("Finger0Slot"), Some((11, 136514)));
        assert_eq!(lookup_slot("Finger1Slot"), Some((12, 136514)));
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(lookup_slot("MAINHANDSLOT"), Some((16, 136518)));
        assert_eq!(lookup_slot("mainhandslot"), Some((16, 136518)));
        assert_eq!(lookup_slot("MainHandSlot"), Some((16, 136518)));
    }

    #[test]
    fn unknown_slots_return_none() {
        assert_eq!(lookup_slot("NotASlot"), None);
        assert_eq!(lookup_slot(""), None);
        assert_eq!(lookup_slot("finger2slot"), None);
    }
}
