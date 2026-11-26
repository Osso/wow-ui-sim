//! Tests for `C_PetBattles` probes backed by `SimState.pet_battles`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::PetBattlePet;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

/// The populated scenario belongs to this test, not production initialization.
fn seeded_env() -> WowLuaEnv {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.pet_battles.player_pets = vec![PetBattlePet::default_player()];
        sim.pet_battles.enemy_pets = vec![PetBattlePet::default_enemy()];
        sim.pet_battles.num_pets_player = 1;
        sim.pet_battles.num_pets_enemy = 1;
        #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
        {
            sim.pet_battles.runtime.qualities.insert((1, 1), 3);
            sim.pet_battles.runtime.qualities.insert((2, 1), 3);
        }
    }
    env
}

// ── Defaults ─────────────────────────────────────────────────────────────────

#[test]
fn get_num_pets_default_roster_matches_profile() {
    let env = env();
    let (player, enemy): (i32, i32) = env
        .eval(
            r#"
            return C_PetBattles.GetNumPets(1), C_PetBattles.GetNumPets(2)
            "#,
        )
        .unwrap();
    let expected = if cfg!(any(
        feature = "client-retail",
        feature = "client-wowforever"
    )) {
        0
    } else {
        1 // INFERRED legacy sample roster remains unchanged.
    };
    assert_eq!(player, expected);
    assert_eq!(enemy, expected);
}

#[test]
fn get_battle_state_default_zero() {
    let env = env();
    let state: i32 = env.eval("return C_PetBattles.GetBattleState()").unwrap();
    assert_eq!(state, 0);
}

// ── GetActivePet ──────────────────────────────────────────────────────────────

#[test]
fn get_active_pet_returns_default_one_for_both_sides() {
    let env = env();
    let (p, e): (i32, i32) = env
        .eval("return C_PetBattles.GetActivePet(1), C_PetBattles.GetActivePet(2)")
        .unwrap();
    assert_eq!(p, 1);
    assert_eq!(e, 1);
}

#[test]
fn get_active_pet_reflects_mutation() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.pet_battles.active_pet_player = 2;
    }
    let slot: i32 = env.eval("return C_PetBattles.GetActivePet(1)").unwrap();
    assert_eq!(slot, 2);
}

// ── GetPetInfo ────────────────────────────────────────────────────────────────

#[test]
fn get_pet_info_returns_seeded_player_pet_name() {
    let env = seeded_env();
    let name: String = env
        .eval("local n = C_PetBattles.GetPetInfo(1, 1); return n")
        .unwrap();
    assert_eq!(name, "Squirrel");
}

#[test]
fn get_pet_info_enemy_returns_rabbit() {
    let env = seeded_env();
    let name: String = env
        .eval("local n = C_PetBattles.GetPetInfo(2, 1); return n")
        .unwrap();
    assert_eq!(name, "Rabbit");
}

#[test]
fn get_pet_info_out_of_range_returns_nil() {
    let env = seeded_env();
    let result: Option<String> = env.eval("return C_PetBattles.GetPetInfo(1, 99)").unwrap();
    assert!(result.is_none(), "out-of-range slot should return nil");
}

#[test]
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn get_pet_info_by_pet_id_returns_nil() {
    let env = env();
    let result_count: i32 = env
        .eval("return select('#', C_PetBattles.GetPetInfoByPetID('BattlePet-0-000000000000'))")
        .unwrap();
    assert_eq!(result_count, 0);
}

#[test]
fn get_ability_info_by_id_unknown_returns_nil() {
    let env = env();
    let ability_name: Option<String> = env
        .eval("local _, name = C_PetBattles.GetAbilityInfoByID(987654); return name")
        .unwrap();
    assert!(ability_name.is_none());
}

// ── GetPetSpeciesID ───────────────────────────────────────────────────────────

#[test]
fn get_pet_species_id_matches_journal_model_scenes_for_default_battle_pets() {
    let env = seeded_env();
    let result: String = env
        .eval(
            r#"
            local ally = C_PetBattles.GetPetSpeciesID(1, 1)
            local enemy = C_PetBattles.GetPetSpeciesID(2, 1)
            local missing = C_PetBattles.GetPetSpeciesID(1, 99)
            if ally ~= 39 then return "ally:" .. tostring(ally) end
            if enemy ~= 87 then return "enemy:" .. tostring(enemy) end
            if missing ~= nil then return "missing:" .. tostring(missing) end

            local allyCard, allyLoadout = C_PetJournal.GetPetModelSceneInfoBySpeciesID(ally)
            local enemyCard, enemyLoadout = C_PetJournal.GetPetModelSceneInfoBySpeciesID(enemy)
            if type(allyCard) ~= "number" or type(allyLoadout) ~= "number" then return "ally-scenes" end
            if type(enemyCard) ~= "number" or type(enemyLoadout) ~= "number" then return "enemy-scenes" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

// ── GetPetStats ───────────────────────────────────────────────────────────────

#[test]
fn get_pet_stats_returns_seeded_values() {
    let env = seeded_env();
    let (hp, max_hp, power, speed, pet_type): (i32, i32, i32, i32, i32) =
        env.eval("return C_PetBattles.GetPetStats(1, 1)").unwrap();
    assert_eq!(hp, 289);
    assert_eq!(max_hp, 289);
    assert_eq!(power, 10);
    assert_eq!(speed, 20);
    assert_eq!(pet_type, 1);
}

#[test]
fn get_pet_stats_mutation_reflects() {
    let env = seeded_env();
    {
        let mut sim = env.state().borrow_mut();
        sim.pet_battles.player_pets[0].current_health = 150;
        sim.pet_battles.player_pets[0].power = 25;
    }
    let (hp, _max_hp, power, _speed, _pt): (i32, i32, i32, i32, i32) =
        env.eval("return C_PetBattles.GetPetStats(1, 1)").unwrap();
    assert_eq!(hp, 150);
    assert_eq!(power, 25);
}

// ── GetPetAbilityList ─────────────────────────────────────────────────────────

#[test]
fn get_pet_ability_list_returns_ability_count() {
    let env = seeded_env();
    let count: i32 = env
        .eval(
            r#"
            local ids, enabled = C_PetBattles.GetPetAbilityList(1, 1)
            return #ids
            "#,
        )
        .unwrap();
    assert_eq!(count, 3, "default player pet has 3 abilities");
}

#[test]
fn get_pet_ability_list_first_id_correct() {
    let env = seeded_env();
    let first_id: i32 = env
        .eval("local ids = C_PetBattles.GetPetAbilityList(1, 1); return ids[1]")
        .unwrap();
    assert_eq!(first_id, 110);
}

// ── GetAllEffectiveAbilityIDs ─────────────────────────────────────────────────

#[test]
fn get_all_effective_ability_ids_returns_array() {
    let env = seeded_env();
    let (count, id): (i32, i32) = env
        .eval(
            r#"
            local ids = C_PetBattles.GetAllEffectiveAbilityIDs(2, 1)
            return #ids, ids[1]
            "#,
        )
        .unwrap();
    assert_eq!(count, 3);
    assert_eq!(id, 120, "first enemy ability id");
}

// ── GetRoundTimingInfo ────────────────────────────────────────────────────────

#[test]
fn get_round_timing_info_defaults() {
    let env = env();
    let (remaining, turn_time): (f64, f64) = env
        .eval("return C_PetBattles.GetRoundTimingInfo()")
        .unwrap();
    assert_eq!(remaining, 0.0, "no round active by default");
    assert_eq!(turn_time, 30.0, "default 30s turn time");
}

#[test]
fn get_round_timing_info_reflects_mutation() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.pet_battles.round_time_left_ms = 15_000.0;
    }
    let remaining: f64 = env
        .eval("local r, _ = C_PetBattles.GetRoundTimingInfo(); return r")
        .unwrap();
    assert_eq!(remaining, 15.0);
}

// ── GetTurnResult ─────────────────────────────────────────────────────────────

#[test]
fn get_turn_result_default_zero() {
    let env = env();
    let result: i32 = env.eval("return C_PetBattles.GetTurnResult(1)").unwrap();
    assert_eq!(result, 0);
}

// ── GetBreedQuality ───────────────────────────────────────────────────────────

#[test]
fn get_breed_quality_returns_numeric_seeded_and_missing_values() {
    let env = seeded_env();
    let result: String = env
        .eval(
            r#"
            local ally = C_PetBattles.GetBreedQuality(Enum.BattlePetOwner.Ally, 1)
            local enemy = C_PetBattles.GetBreedQuality(Enum.BattlePetOwner.Enemy, 1)
            local missing = C_PetBattles.GetBreedQuality(Enum.BattlePetOwner.Ally, 99)
            if type(ally) ~= "number" or type(enemy) ~= "number" or type(missing) ~= "number" then
                return "type"
            end
            if ally ~= Enum.BattlePetBreedQuality.Rare or enemy ~= Enum.BattlePetBreedQuality.Rare then
                return "seeded"
            end
            if missing ~= 0 then return "missing" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

// ── GetXP ─────────────────────────────────────────────────────────────────────

#[test]
fn get_xp_returns_zero_by_default() {
    let env = seeded_env();
    let (xp, max_xp): (i32, i32) = env.eval("return C_PetBattles.GetXP(1, 1)").unwrap();
    assert_eq!(xp, 0);
    assert_eq!(max_xp, 100);
}

// ── IsPlayerNPC ───────────────────────────────────────────────────────────────

#[test]
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn is_player_npc_returns_false() {
    let env = env();
    let is_npc: bool = env.eval("return C_PetBattles.IsPlayerNPC()").unwrap();
    assert!(!is_npc);
}

#[test]
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn static_fallbacks_return_safe_default_shapes() {
    let env = env();
    let (effect_count, trap_ready, trap_error, should_select): (i32, bool, i32, bool) = env
        .eval(
            r##"
            local trapReady, trapError = C_PetBattles.IsTrapAvailable()
            return
                select("#", C_PetBattles.GetAllEffectNames()),
                trapReady,
                trapError,
                C_PetBattles.ShouldShowPetSelect()
            "##,
        )
        .unwrap();
    assert_eq!(effect_count, 0);
    assert!(!trap_ready);
    assert_eq!(trap_error, 0);
    assert!(!should_select);
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn retail_pet_battle_runtime_queries_follow_admin_inputs() {
    let env = env();
    env.exec(
        "A_Admin.SetPetBattleEffectName('Supplied effect'); \
         A_Admin.SetPetBattleState(1); \
         A_Admin.SetPetBattleTeamState(true, false, true, true); \
         A_Admin.SetPetBattleAvailability(0, true, 4)",
    )
    .unwrap();
    let (effects, effect, trap, error, select, npc): (i32, String, bool, i32, bool, bool) = env
        .eval("local e = {C_PetBattles.GetAllEffectNames()}; local t, x = C_PetBattles.IsTrapAvailable(); return #e, e[1], t, x, C_PetBattles.ShouldShowPetSelect(), C_PetBattles.IsPlayerNPC(2)")
        .unwrap();
    assert_eq!((effects, effect), (1, "Supplied effect".into()));
    assert!(!trap);
    assert_eq!(error, 4);
    assert!(select);
    assert!(npc);
}

// ── StartPVPMatchmaking ───────────────────────────────────────────────────────

#[test]
fn start_pvp_matchmaking_sets_flag() {
    let env = env();
    assert!(
        !env.state().borrow().pet_battles.is_matchmaking,
        "default false"
    );
    env.eval::<()>("C_PetBattles.StartPVPMatchmaking()")
        .unwrap();
    assert!(
        env.state().borrow().pet_battles.is_matchmaking,
        "flag set after call"
    );
}

// ── GetPetAbilityInfo ─────────────────────────────────────────────────────────

#[test]
fn get_pet_ability_info_returns_name_and_charges() {
    let env = seeded_env();
    let (name, _icon, max_charges): (String, i32, i32) = env
        .eval("return C_PetBattles.GetPetAbilityInfo(1, 1, 1)")
        .unwrap();
    assert!(name.contains("110"), "name mentions ability id 110");
    assert_eq!(max_charges, 1);
}

// ── custom seed ───────────────────────────────────────────────────────────────

#[test]
fn custom_seeded_pet_reflects_in_get_pet_info() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.pet_battles.player_pets = vec![PetBattlePet {
            name: "Lil' Ragnaros".into(),
            display_id: None,
            species_id: 117,
            level: 25,
            max_health: 1725,
            current_health: 1725,
            power: 276,
            speed: 244,
            pet_type: 10, // Elemental
            ability_ids: vec![500, 501, 502],
            xp: 500,
            max_xp: 1000,
        }];
        sim.pet_battles.num_pets_player = 1;
    }
    let name: String = env.eval("return C_PetBattles.GetPetInfo(1, 1)").unwrap();
    assert_eq!(name, "Lil' Ragnaros");
}
