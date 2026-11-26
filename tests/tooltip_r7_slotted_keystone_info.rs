#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::WowLuaEnv;

const ADDONS_LUA: &str = r#"
for _, addon in ipairs({"Blizzard_GroupFinder", "Blizzard_ChallengesUI"}) do
    local loaded, reason = C_AddOns.LoadAddOn(addon)
    assert(loaded, addon .. " load failed: " .. tostring(reason))
end
assert(ChallengesKeystoneFrame.KeystoneSlot)
CHALLENGE_MODE_POWER_LEVEL = "Level %d"
"#;

fn load_challenges_frame() -> WowLuaEnv {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(ADDONS_LUA).expect("original ChallengesUI frame should load");
    env
}

fn set_map_info(env: &WowLuaEnv, map_id: u32, name: &str) {
    env.exec(&format!(
        "A_Admin.SetChallengeModeMapInfo({map_id}, {{name={name:?}, id={map_id}, timeLimit=1800, texture=0, backgroundTexture=0, mapID=960}})"
    ))
    .unwrap();
}

fn slot_keystone(env: &WowLuaEnv, bag_slot: u32, item_id: u32, map_id: u32, level: u32, a: u32, b: u32) {
    env.exec(&format!(
        "A_Admin.AddBagItem(0, {bag_slot}, {item_id}, 1); A_Admin.SetKeystoneItemModifiers(0, {bag_slot}, {item_id}, 960, {{{{17, {map_id}}}, {{18, {level}}}, {{19, {a}}}, {{20, {b}}}}}); C_Container.PickupContainerItem(0, {bag_slot}); C_ChallengeMode.SlotKeystone()"
    ))
    .unwrap();
}

fn assert_slotted_info(env: &WowLuaEnv, map_id: u32, name: &str, level: u32, a: u32, b: u32) {
    env.exec(&format!(
        r#"
        local selectedMap, affixes, selectedLevel = C_ChallengeMode.GetSlottedKeystoneInfo()
        assert(selectedMap == {map_id} and selectedLevel == {level})
        assert(affixes[1] == {a} and affixes[2] == {b} and affixes[3] == nil)
        assert(ChallengesKeystoneFrame.DungeonName:GetText() == {name:?})
        assert(ChallengesKeystoneFrame.PowerLevel:GetText() == "Level {level}")
        assert(ChallengesKeystoneFrame.Affixes[3].affixID == {a})
        assert(ChallengesKeystoneFrame.Affixes[4].affixID == {b})
        "#
    ))
    .unwrap();
}

#[test]
fn challenges_ui_slotted_event_reads_and_clears_selected_keystone_info() {
    let env = load_challenges_frame();
    set_map_info(&env, 2, "Keystone Map Alpha");
    set_map_info(&env, 3, "Keystone Map Beta");
    for (slot, item, map, name, level, a, b) in [
        (3, 180_653, 2, "Keystone Map Alpha", 5, 6, 9),
        (4, 158_923, 3, "Keystone Map Beta", 8, 7, 10),
    ] {
        slot_keystone(&env, slot, item, map, level, a, b);
        assert_slotted_info(&env, map, name, level, a, b);
        env.exec("assert(C_ChallengeMode.RemoveKeystone()); assert(not C_ChallengeMode.HasSlottedKeystone()); assert(select('#', C_ChallengeMode.GetSlottedKeystoneInfo()) == 0)")
            .unwrap();
    }
}
