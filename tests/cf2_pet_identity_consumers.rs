//! Selected original pet-name consumers. Edit-only staging: runtime proof is pending.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use sha2::{Digest, Sha256};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn assert_selected_consumer() {
    let ui = blizzard_ui_dir();
    let addon = ui.join("Blizzard_PetBattleUI");
    let toc_path = wow_ui_sim::loader::find_toc_file(&addon).expect("selected pet-battle TOC");
    let toc = wow_ui_sim::toc::TocFile::from_file(&toc_path).unwrap();
    assert!(!toc.is_game_type_restricted());
    assert!(!toc.is_ptr_only());
    assert!(toc.allows_screen(ScreenKind::Game));
    for file in [
        "Shared/Blizzard_PetBattleUI.lua",
        "Shared/Blizzard_PetBattleUI.xml",
    ] {
        assert!(
            toc.files
                .iter()
                .any(|entry| entry == std::path::Path::new(file))
        );
        let source = std::fs::read(addon.join(file)).unwrap();
        eprintln!(
            "CF2 selected {} {file} sha256={:x}",
            toc_path.display(),
            Sha256::digest(source)
        );
    }
}

fn supply_names(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let catalog_pet = sim.world.pets[0].clone();
    sim.world.pets = [
        (70001, "Amber Moth"),
        (70002, "Ivory Owl"),
        (70003, "Copper Beetle"),
    ]
    .into_iter()
    .map(|(species_id, name)| {
        let mut pet = catalog_pet.clone();
        pet.species_id = species_id;
        pet.name = name.into();
        pet.custom_name = Some("Journal-only alias".into());
        pet
    })
    .collect();
    let battle_pet = wow_ui_sim::lua_api::state::PetBattlePet::default_player();
    let battle_pets = [("Spark", 70001), ("Hush", 70002), ("Glimmer", 70003)]
        .into_iter()
        .map(|(name, species_id)| {
            let mut pet = battle_pet.clone();
            pet.name = name.into();
            pet.species_id = species_id;
            pet
        })
        .collect::<Vec<_>>();
    sim.pet_battles.player_pets = battle_pets[..2].to_vec();
    sim.pet_battles.enemy_pets = battle_pets[2..].to_vec();
    sim.pet_battles.num_pets_player = 2;
    sim.pet_battles.num_pets_enemy = 1;
}

fn original_consumer() -> WowLuaEnv {
    assert_selected_consumer();
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[
            "Blizzard_SharedXMLGame",
            "Blizzard_FrameXML",
            "Blizzard_TextStatusBar",
            "Blizzard_ActionBar",
        ],
        &[],
    );
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_PetBattleUI"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_PetBattleUI"));
    env.apply_post_load_workarounds();
    supply_names(&env);
    env.exec("assert(PetBattlePrimaryUnitTooltip and PetBattlePrimaryUnitTooltip.Name and PetBattlePrimaryUnitTooltip.SpeciesName)")
        .expect("original XML tooltip and its name fields");
    env
}

fn assert_consumer_errors_absent(env: &WowLuaEnv) {
    let errors = env.state().borrow().lua_errors.clone();
    eprintln!("CF2 complete error array: {errors:#?}");
    assert!(
        !errors
            .iter()
            .any(|error| error.contains("Blizzard_PetBattleUI.lua")),
        "original pet consumer must not retain or expose an error: {errors:#?}"
    );
}

fn refresh_display(env: &WowLuaEnv, owner: i32, slot: i32) {
    env.exec(&format!(
        "PetBattlePrimaryUnitTooltip.petOwner = {owner}; PetBattlePrimaryUnitTooltip.petIndex = {slot}"
    )).unwrap();
    // Actual XML OnLoad registers this event on the actual tooltip frame.
    env.fire_event("PET_BATTLE_PET_CHANGED").unwrap();
}

fn assert_display(env: &WowLuaEnv, name: &str, species: &str, show_species: bool) {
    let (actual_name, actual_species, shown): (String, String, bool) = env.eval(
        "local f = PetBattlePrimaryUnitTooltip; return f.Name:GetText(), f.SpeciesName:GetText(), f.SpeciesName:IsShown()"
    ).unwrap();
    assert_eq!(actual_name, name);
    if show_species {
        assert_eq!(actual_species, species);
    }
    assert_eq!(shown, show_species);
}

fn name_transitions(env: &WowLuaEnv) {
    refresh_display(env, 1, 1);
    assert_display(env, "Spark", "Amber Moth", true);
    refresh_display(env, 1, 2);
    assert_display(env, "Hush", "Ivory Owl", true);
    refresh_display(env, 2, 1);
    assert_display(env, "Glimmer", "Copper Beetle", true);
    {
        let mut sim = env.state().borrow_mut();
        sim.pet_battles.enemy_pets[0].name = "Copper Beetle".into();
    }
    refresh_display(env, 2, 1);
    assert_display(env, "Copper Beetle", "Copper Beetle", false);
    {
        let mut sim = env.state().borrow_mut();
        sim.pet_battles.enemy_pets[0].name = "Flare".into();
        sim.world.pets[2].name = "Bronze Beetle".into();
    }
    refresh_display(env, 2, 1);
    assert_display(env, "Flare", "Bronze Beetle", true);
    assert_optional_results(env);
    assert_consumer_errors_absent(env);
}

fn assert_optional_results(env: &WowLuaEnv) {
    env.state().borrow_mut().pet_battles.enemy_pets.clear();
    env.exec(
        r#"
        assert(select('#', C_PetBattles.GetName(2, 1)) == 0, 'removed pet has no results')
        assert(select('#', C_PetBattles.GetName(1, 0)) == 0, 'zero slot has no results')
        assert(select('#', C_PetBattles.GetName(1, -1)) == 0, 'negative slot has no results')
        assert(select('#', C_PetBattles.GetName(1, 3)) == 0, 'missing slot has no results')
        assert(select('#', C_PetBattles.GetName(99, 1)) == 0, 'unknown owner has no results')
        local name, species = C_PetBattles.GetName(1, 1)
        assert(name == 'Spark' and species == 'Amber Moth', 'other owner retained')
    "#,
    )
    .unwrap();
    env.state().borrow_mut().world.pets.clear();
    env.exec("assert(select('#', C_PetBattles.GetName(1, 1)) == 0, 'missing species catalog is not a fabricated pair')")
        .unwrap();
}

#[test]
fn original_pet_name_owner_slot_and_refresh() {
    test_timeout! { name_transitions(&original_consumer()); }
}

#[test]
fn stock_pet_name_consumer_refresh() {
    test_timeout! {
        assert_selected_consumer();
        let env = crate::spell_casting::env_with_full_blizzard_ui();
        assert_consumer_errors_absent(&env);
        supply_names(&env);
        name_transitions(&env);
    }
}
