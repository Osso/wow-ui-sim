#![cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::lua_api::WowLuaEnv;

type TalentStateSnapshot = (bool, bool, i32, i32, i32, Option<i32>, Option<i32>);

#[cfg(feature = "client-retail")]
#[test]
fn retail_supplied_pvp_talent_state_reaches_open_player_spells_panel() {
    with_player_spells_panel(|env| {
        supply_selected_talent_state(env);
        assert_eq!(
            read_talent_state(env),
            (true, true, 2, 123, 456, Some(456), None)
        );
    });
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_supplied_pvp_talent_state_reaches_open_player_spells_panel() {
    with_player_spells_panel(|env| {
        supply_selected_talent_state(env);
        env.exec("A_Admin.SetPvpTalentSlotUnlockLevel(2, 30)")
            .expect("supply the Forever slot unlock level");
        assert_eq!(
            read_talent_state(env),
            (true, true, 2, 123, 456, Some(456), Some(30))
        );
    });
}

fn with_player_spells_panel(assert_state: impl FnOnce(&WowLuaEnv)) {
    with_blizzard_addon_closure(
        &["Blizzard_PlayerSpells", "Blizzard_InspectUI"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_PlayerSpells"));
            assert!(loaded.iter().any(|name| name == "Blizzard_InspectUI"));
            env.exec("PlayerSpellsFrame_LoadUI(); ShowUIPanel(PlayerSpellsFrame)")
                .expect("open the original Player Spells panel");
            assert_state(env);
        },
    );
}

fn supply_selected_talent_state(env: &WowLuaEnv) {
    env.exec(
        r#"
        A_Admin.SetSelectedPvpTalentIDs({ 123, 456 })
        A_Admin.SetInspectSelectedPvpTalent("target", 2, 456)
        C_SpecializationInfo.SetPvpTalentLocked(123, true)
        "#,
    )
    .expect("supply selected, inspected and locked PvP talents");
}

fn read_talent_state(env: &WowLuaEnv) -> TalentStateSnapshot {
    env.eval(
        r#"
        local selected = C_SpecializationInfo.GetAllSelectedPvpTalentIDs()
        return PlayerSpellsFrame:IsShown(), C_SpecializationInfo.IsPvpTalentLocked(123),
            #selected, selected[1], selected[2],
            C_SpecializationInfo.GetInspectSelectedPvpTalent("target", 2),
            C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(2)
        "#,
    )
    .expect("read supplied PvP state through client APIs")
}
