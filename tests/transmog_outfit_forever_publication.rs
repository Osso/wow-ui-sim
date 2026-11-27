//! Forever 1.60.1 TransmogOutfitInfoDocumentation.lua declares the Retail outfit
//! model, with `GetOptionsForSlot`/`SetViewedOptionForSlot` (option field `type`,
//! event `VIEWED_TRANSMOG_OUTFIT_SLOT_OPTION_CHANGED`) in place of the Retail
//! weapon-option pair. The shared Rust model now serves those members on Forever.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const OUTFIT_ROUND_TRIP: &str = r#"
local unlocked, name, icon, renamed, option = ...
local events = {}
RegisterEventCallback("VIEWED_TRANSMOG_OUTFIT_SLOT_OPTION_CHANGED", function(_, slot, value)
    events[#events + 1] = slot .. ":" .. value
end)
A_Admin.SetUnlockedOutfitSlots(1, unlocked)
C_TransmogOutfitInfo.AddNewOutfit(name, icon)
local id = C_TransmogOutfitInfo.GetOutfitInfoByName(name).outfitID
C_TransmogOutfitInfo.CommitOutfitInfo(id, renamed, icon + 1)
C_TransmogOutfitInfo.ChangeViewedOutfit(id)
C_TransmogOutfitInfo.SetSecondarySlotState(1, option == 2)
C_TransmogOutfitInfo.SetViewedOptionForSlot(12, option)
C_TransmogOutfitInfo.ChangeDisplayedOutfit(id, 0, false, false)
local options = C_TransmogOutfitInfo.GetOptionsForSlot(12)
return C_TransmogOutfitInfo.GetNumberOfOutfitsUnlockedForSource(1),
    C_TransmogOutfitInfo.GetMaxNumberOfUsableOutfits(),
    C_TransmogOutfitInfo.GetOutfitInfoByName(renamed).icon,
    C_TransmogOutfitInfo.GetOutfitInfoByName(name) == nil,
    tostring(C_TransmogOutfitInfo.GetSecondarySlotState(2)),
    table.concat(events, ","), options[option].type,
    tostring(C_TransmogOutfitInfo.IsEquippedGearOutfitDisplayed())
"#;

type RoundTrip = (i32, i32, i32, bool, String, String, i32, String);

fn round_trip(unlocked: i32, name: &str, icon: i32, renamed: &str, option: i32) -> RoundTrip {
    let env = WowLuaEnv::new().expect("create Forever Lua environment");
    let chunk = format!(
        "return (function(...) {OUTFIT_ROUND_TRIP} end)({unlocked}, '{name}', {icon}, '{renamed}', {option})"
    );
    env.eval(&chunk)
        .expect("the outfit model should follow its inputs")
}

#[test]
fn forever_outfit_model_follows_supplied_slots_names_and_options() {
    assert_eq!(
        round_trip(2, "Copper", 123, "Bronze", 1),
        (2, 22, 124, true, "false".into(), "12:1".into(), 1, "false".into())
    );
    assert_eq!(
        round_trip(3, "Silver", 456, "Gold", 2),
        (3, 23, 457, true, "true".into(), "12:2".into(), 2, "false".into())
    );
}
