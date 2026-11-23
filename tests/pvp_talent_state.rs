#![cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn supplied_pvp_talent_state_reaches_the_open_player_spells_panel() {
    assert_player_spells_state();
}

fn assert_player_spells_state() {
    with_blizzard_addon_closure(
        &["Blizzard_PlayerSpells", "Blizzard_InspectUI"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_PlayerSpells"));
            assert!(loaded.iter().any(|name| name == "Blizzard_InspectUI"));

            let result = read_supplied_state(env);
            assert_eq!(result, (true, true, 2, 123, 456, Some(456), Some(30)));
        },
    );
}

fn read_supplied_state(env: &WowLuaEnv) -> (bool, bool, i32, i32, i32, Option<i32>, Option<i32>) {
    env.exec(
        r#"
        A_Admin.SetSelectedPvpTalentIDs({ 123, 456 })
        A_Admin.SetInspectSelectedPvpTalent("target", 2, 456)
        A_Admin.SetPvpTalentSlotUnlockLevel(2, 30)
        C_SpecializationInfo.SetPvpTalentLocked(123, true)
        PlayerSpellsFrame_LoadUI()
        ShowUIPanel(PlayerSpellsFrame)
        "#,
    )
    .expect("supply PvP state and open the original Player Spells panel");
    env.eval(
        r#"
        local selected = C_SpecializationInfo.GetAllSelectedPvpTalentIDs()
        return PlayerSpellsFrame:IsShown(), C_SpecializationInfo.IsPvpTalentLocked(123),
            #selected, selected[1], selected[2],
            C_SpecializationInfo.GetInspectSelectedPvpTalent("target", 2),
            C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(2)
        "#,
    )
    .expect("read PvP state through the loaded client APIs")
}
