#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::c_stable_info::forever::{PetInfo, StableReadState};
use wow_ui_sim::lua_api::WowLuaEnv;

const DEFAULT_READS: &str = r#"
    assert(C_StableInfo.GetNumStableSlots() == 2, "default owned slots")
    assert(C_StableInfo.GetNextStableSlotCost() == 0, "unavailable purchase cost")
    assert(C_StableInfo.GetNumStablePets() == 0, "empty stable count")
    for slot = 1, 3 do assert(type(C_StableInfo.GetStablePetInfo(slot)) == "table") end
"#;

const MONEY_EVENT: &str = r#"
    assert(PetStableFrame:IsEventRegistered("PLAYER_MONEY"))
    A_Admin.SetMoney(12345)
    A_Admin.FireEvent("PLAYER_MONEY")
    assert(not PetStableFrame.purchaseButton:IsShown(), "fully unlocked purchase hidden")
    for slot = 1, 3 do
        local frame = PetStableFrame:GetSlotFrame(slot)
        assert(frame.tooltip == EMPTY_STABLE_SLOT, "unlocked empty slot")
    end
    print("STABLE_READ_MONEY_DONE")
"#;

#[test]
fn stable_reads_default_and_restore() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(DEFAULT_READS).unwrap();
    env.loader_env().restore_post_cleanup_globals().unwrap();
    env.exec(DEFAULT_READS).unwrap();
}

#[test]
fn stable_reads_cached_money_event() {
    let root = tempfile::tempdir().unwrap();
    let output = std::process::Command::new("timeout")
        .args(["90", env!("CARGO_BIN_EXE_wow-sim")])
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            MONEY_EVENT,
            "lua-errors",
        ])
        .env("XDG_DATA_HOME", root.path().join("data"))
        .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("STABLE_READ_MONEY_DONE"), "{stdout}");
    assert!(stdout.trim_end().ends_with("[]"), "{stdout}");
}

fn wolf() -> PetInfo {
    PetInfo {
        native_record_id: 17,
        native_flags: 2,
        icon: 132203,
        name: "Fang".into(),
        level: 42,
        family_name: "Wolf".into(),
        specialization: "Ferocity".into(),
        pet_type: "Beast".into(),
        pet_abilities: vec![17253, 2649],
        spec_abilities: vec![61684],
        display_id: 1166,
        is_favorite: true,
        is_exotic: false,
        ui_model_scene_id: 718,
        pet_number: 17,
        creature_id: 299,
        spec_id: 1,
        loyalty_level: 6,
        loyalty_name: "Best Friend".into(),
        happiness_level: 3,
        experience: 1200,
        experience_needed: 4000,
    }
}

#[test]
fn stable_reads_configured_pet_fields_and_snapshot() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().stable_reads = StableReadState {
        owned_slots: 1,
        slot_costs: [(2, 50_000)].into(),
        pets: [(2, wolf())].into(),
    };
    env.exec(
        r#"
        assert(C_StableInfo.GetNumStableSlots() == 1)
        assert(C_StableInfo.GetNextStableSlotCost() == 50000)
        assert(C_StableInfo.GetNumStablePets() == 1)
        local p = assert(C_StableInfo.GetStablePetInfo(2))
        assert(p.slotID == 2 and p.icon == 132203 and p.name == "Fang")
        assert(p.level == 42 and p.familyName == "Wolf")
        assert(p.specialization == "Ferocity" and p.type == "Beast")
        assert(#p.petAbilities == 2 and p.petAbilities[1] == 17253 and p.petAbilities[2] == 2649)
        assert(#p.specAbilities == 1 and p.specAbilities[1] == 61684)
        assert(p.displayID == 1166 and p.isFavorite == true and p.isExotic == false)
        assert(p.uiModelSceneID == 718 and p.petNumber == 17 and p.creatureID == 299)
        assert(p.specID == 1 and p.loyaltyLevel == 6 and p.loyaltyName == "Best Friend")
        assert(p.happinessLevel == 3 and p.experience == 1200 and p.experienceNeeded == 4000)
        assert(type(C_StableInfo.GetStablePetInfo(1)) == "table")
        assert(type(C_StableInfo.GetStablePetInfo(3)) == "table")
        assert(type(C_StableInfo.GetStablePetInfo(0)) == "table")
        assert(type(C_StableInfo.GetStablePetInfo(-1)) == "table")
        assert(type(C_StableInfo.GetStablePetInfo(99)) == "table")
        p.name = "changed"; p.petAbilities[1] = 0
        local again = C_StableInfo.GetStablePetInfo(2)
        assert(again.name == "Fang" and again.petAbilities[1] == 17253)
        assert(not pcall(C_StableInfo.GetStablePetInfo))
        local ok, err = pcall(C_StableInfo.GetStablePetInfo, {})
        assert(not ok and err:find("Usage: local petInfo = C_StableInfo.GetStablePetInfo(index)", 1, true))
    "#,
    )
    .unwrap();
}

#[test]
fn stable_reads_count_tracks_pets_and_environments_are_independent() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    {
        let mut state = first.state().borrow_mut();
        state.stable_reads.owned_slots = 0;
        state.stable_reads.slot_costs.insert(1, 10_000);
        state.stable_reads.pets.insert(1, wolf());
        state.pet_stables_open = true;
    }
    first
        .exec(
            r#"
        assert(C_StableInfo.GetNumStableSlots() == 0)
        assert(C_StableInfo.GetNextStableSlotCost() == 10000)
        assert(C_StableInfo.GetNumStablePets() == 1)
        assert(C_StableInfo.GetStablePetInfo(1).slotID == 1)
        assert(C_StableInfo.IsAtPetStable())
    "#,
        )
        .unwrap();
    second.exec(DEFAULT_READS).unwrap();
    second
        .exec("assert(not C_StableInfo.IsAtPetStable())")
        .unwrap();
    first.state().borrow_mut().stable_reads.pets.clear();
    first.exec("assert(C_StableInfo.GetNumStablePets() == 0); assert(type(C_StableInfo.GetStablePetInfo(1)) == 'table')").unwrap();
}

#[test]
fn forever_native_empty_pet_and_original_purchase_consumer() {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    use wow_ui_sim::loader::find_toc_file;
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::toc::TocFile;

    let ui = blizzard_ui_dir();
    let toc = TocFile::from_file(&find_toc_file(&ui.join("Blizzard_StableUI")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    assert!(
        toc.files
            .iter()
            .any(|file| file.ends_with("Camelot/Blizzard_StableUI.lua"))
    );
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_StableUI"], &[]);
    env.exec(r#"
        assert(debug.getinfo(PetStablePurchaseButtonMixin.Update).source:find("Camelot/Blizzard_StableUI.lua", 1, true))
        local value = C_StableInfo.GetStablePetInfo(99)
        assert(type(value) == "table", "absent stable pet must return a table")
        PetStablePurchaseButtonMixin.Update(PetStableFrame.purchaseButton)
    "#).unwrap();
}

#[test]
fn forever_stable_native_record_flags_and_cost_keys() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        let stable = &mut state.stable_reads;
        stable.owned_slots = 0;
        stable.slot_costs = [(1, 123), (2, 456)].into();
        let mut no_id = wolf();
        no_id.native_record_id = 0;
        let mut no_flag = wolf();
        no_flag.native_flags = 1;
        stable.pets = [(1, wolf()), (2, no_id), (3, no_flag)].into();
    }
    env.exec(
        r#"
        assert(select('#', C_StableInfo.GetNumStablePets()) == 1)
        assert(C_StableInfo.GetNumStablePets() == 1)
        assert(C_StableInfo.GetNextStableSlotCost() == 123)
        assert(select('#', C_StableInfo.GetStablePetInfo(99)) == 1)
        assert(type(C_StableInfo.GetStablePetInfo(99)) == 'table')
    "#,
    )
    .unwrap();
    env.state().borrow_mut().stable_reads.owned_slots = 1;
    env.exec("assert(C_StableInfo.GetNextStableSlotCost() == 456)")
        .unwrap();
    env.state().borrow_mut().stable_reads.owned_slots = 255;
    env.exec("assert(C_StableInfo.GetNumStableSlots() == 255); assert(C_StableInfo.GetNextStableSlotCost() == 0)").unwrap();
}
