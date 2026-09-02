use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::PetBattlePet;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn seed_sample_pet_battle(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    seed_sample_pet_battle_state(&mut sim);
    sim.pet_battles.player_pets = seeded_player_pets();
    sim.pet_battles.enemy_pets = seeded_enemy_pets();
    drop(sim);
    env.exec(
        r#"
        A_Admin.SetPetBattleState(1)
        C_PetBattles._state.isWildBattle = true
        "#,
    )
    .unwrap();
}

fn seed_sample_pet_battle_state(sim: &mut wow_ui_sim::lua_api::state::SimState) {
    sim.pet_battles.num_pets_player = 3;
    sim.pet_battles.num_pets_enemy = 2;
    sim.pet_battles.battle_state = 1;
    sim.pet_battles.active_pet_player = 1;
    sim.pet_battles.active_pet_enemy = 1;
}

fn seeded_player_pets() -> Vec<PetBattlePet> {
    vec![arcane_familiar(), clockwork_hopper(), frost_pup()]
}

fn arcane_familiar() -> PetBattlePet {
    PetBattlePet {
        name: "Arcane Familiar".into(),
        species_id: 1,
        level: 25,
        max_health: 1420,
        current_health: 1120,
        power: 18,
        speed: 21,
        pet_type: 7,
        ability_ids: vec![1001, 1002],
        xp: 45,
        max_xp: 100,
    }
}

fn clockwork_hopper() -> PetBattlePet {
    PetBattlePet {
        name: "Clockwork Hopper".into(),
        species_id: 2,
        level: 24,
        max_health: 1180,
        current_health: 910,
        power: 15,
        speed: 17,
        pet_type: 9,
        ability_ids: vec![1003],
        xp: 15,
        max_xp: 100,
    }
}

fn frost_pup() -> PetBattlePet {
    PetBattlePet {
        name: "Frost Pup".into(),
        species_id: 3,
        level: 23,
        max_health: 1110,
        current_health: 870,
        power: 14,
        speed: 19,
        pet_type: 8,
        ability_ids: vec![1004],
        xp: 10,
        max_xp: 100,
    }
}

fn seeded_enemy_pets() -> Vec<PetBattlePet> {
    vec![
        PetBattlePet {
            name: "Stone Lurker".into(),
            species_id: 4,
            level: 24,
            max_health: 1320,
            current_health: 980,
            power: 16,
            speed: 14,
            pet_type: 9,
            ability_ids: vec![1101],
            xp: 0,
            max_xp: 100,
        },
        PetBattlePet {
            name: "Bog Hopper".into(),
            species_id: 5,
            level: 24,
            max_health: 1210,
            current_health: 930,
            power: 13,
            speed: 20,
            pet_type: 9,
            ability_ids: vec![1102],
            xp: 0,
            max_xp: 100,
        },
    ]
}

#[test]
fn pet_battles_seeded_pet_state_and_abilities_are_exposed() {
    let env = env();
    seed_sample_pet_battle(&env);
    let result: String = env
        .eval(
            r#"
            if not C_PetBattles.IsInBattle() then
                return "battle_should_start_open"
            end
            if not C_PetBattles.IsWildBattle() then
                return "battle_should_start_wild"
            end
            if C_PetBattles.GetNumPets(Enum.BattlePetOwner.Ally) ~= 3 then
                return "expected_three_ally_pets"
            end
            if C_PetBattles.GetNumPets(Enum.BattlePetOwner.Enemy) ~= 2 then
                return "expected_two_enemy_pets"
            end

            local activePet = C_PetBattles.GetActivePet(Enum.BattlePetOwner.Ally)
            if activePet ~= 1 then
                return "expected_first_ally_pet_active"
            end

            local abilityID, abilityName, abilityIcon, maxCooldown, description, numTurns, petType = C_PetBattles.GetAbilityInfo(Enum.BattlePetOwner.Ally, activePet, 1)
            if abilityID ~= 1001 or abilityName ~= "Arcane Bite" then
                return "expected_seeded_ability_info"
            end

            local _, sameName, _, sameCooldown, _, sameTurns, samePetType = C_PetBattles.GetAbilityInfoByID(abilityID)
            if sameName ~= abilityName or sameCooldown ~= maxCooldown or sameTurns ~= numTurns or samePetType ~= petType then
                return "ability_lookup_by_id_should_match"
            end

            local isUsable, cooldown, lockdown = C_PetBattles.GetAbilityState(Enum.BattlePetOwner.Ally, activePet, 2)
            if not isUsable or cooldown ~= 1 or lockdown ~= 0 then
                return "expected_seeded_ability_state"
            end

            local auraID, auraInstanceID, turnsRemaining, isBuff = C_PetBattles.GetAuraInfo(Enum.BattlePetOwner.Ally, activePet, 1)
            if auraID ~= 1002 or auraInstanceID ~= 9001 or turnsRemaining ~= 2 or not isBuff then
                return "expected_seeded_aura_info"
            end

            if C_PetBattles.GetNumAuras(Enum.BattlePetOwner.Ally, activePet) ~= 1 then
                return "expected_one_seeded_aura"
            end
            if C_PetBattles.GetHealth(Enum.BattlePetOwner.Ally, activePet) <= 0 then
                return "health_should_be_positive"
            end
            if C_PetBattles.GetMaxHealth(Enum.BattlePetOwner.Ally, activePet) <= C_PetBattles.GetHealth(Enum.BattlePetOwner.Ally, activePet) then
                return "max_health_should_exceed_health"
            end
            if C_PetBattles.GetPower(Enum.BattlePetOwner.Ally, activePet) <= 0 then
                return "power_should_be_positive"
            end
            if C_PetBattles.GetSpeed(Enum.BattlePetOwner.Ally, activePet) <= 0 then
                return "speed_should_be_positive"
            end
            if C_PetBattles.GetLevel(Enum.BattlePetOwner.Ally, activePet) <= 0 then
                return "level_should_be_positive"
            end

            local xp, maxXP = C_PetBattles.GetXP(Enum.BattlePetOwner.Ally, activePet)
            if xp <= 0 or maxXP <= xp then
                return "xp_should_round_trip"
            end

            if C_PetBattles.GetAttackModifier(7, 9) <= 1 then
                return "expected_seeded_attack_modifier"
            end

            local parserEnv = {}
            C_PetBattles.GetAllStates(parserEnv)
            if parserEnv.STATE_Stat_Power ~= 18 then
                return "GetAllStates_should_populate_parser_env"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "Pet battle getters should expose seeded state"
    );
}

#[test]
fn pet_battles_actions_and_queue_state_are_mutable() {
    let env = env();
    seed_sample_pet_battle(&env);
    let result: String = env
        .eval(
            r#"
            C_PetBattles.StartPVPMatchmaking()
            -- QueueStatusFrame.lua compares the status against the strings
            -- "queued" / "proposal" / "suspended" / "entry", not the enum.
            local queueState, estimatedTime, queuedTime = C_PetBattles.GetPVPMatchmakingInfo()
            if queueState ~= "queued" then
                return "matchmaking_should_seed_queue_state"
            end
            if estimatedTime <= 0 or queuedTime <= 0 then
                return "queue_timers_should_be_positive"
            end
            if not C_PetBattles.CanAcceptQueuedPVPMatch() then
                return "queue_should_be_accepting_before_accept"
            end

            C_PetBattles.AcceptQueuedPVPMatch()
            if C_PetBattles.GetPVPMatchmakingInfo() ~= "proposal" then
                return "accept_should_update_queue_status"
            end
            if C_PetBattles.CanAcceptQueuedPVPMatch() then
                return "accept_should_clear_accept_flag"
            end

            C_PetBattles.UseAbility(2)
            local actionType, actionIndex = C_PetBattles.GetSelectedAction()
            if actionType ~= Enum.BattlePetAction.Ability or actionIndex ~= 2 then
                return "use_ability_should_select_action"
            end

            C_PetBattles.ChangePet(2)
            actionType, actionIndex = C_PetBattles.GetSelectedAction()
            if actionType ~= Enum.BattlePetAction.SwitchPet or actionIndex ~= 2 then
                return "change_pet_should_select_swap_action"
            end

            C_PetBattles.UseTrap()
            actionType = C_PetBattles.GetSelectedAction()
            if actionType ~= Enum.BattlePetAction.Trap then
                return "use_trap_should_select_trap_action"
            end

            C_PetBattles.SkipTurn()
            actionType = C_PetBattles.GetSelectedAction()
            if actionType ~= Enum.BattlePetAction.Skip then
                return "skip_turn_should_select_skip_action"
            end

            C_PetBattles.StartPVPDuel("target", true)
            if not C_PetBattles._state.pvpDuel.pending or C_PetBattles._state.pvpDuel.challengedUnit ~= "target" or not C_PetBattles._state.pvpDuel.exactMatch then
                return "pvp_duel_should_seed_pending_state"
            end
            C_PetBattles.AcceptPVPDuel()
            if C_PetBattles._state.pvpDuel.pending or not C_PetBattles._state.pvpDuel.accepted then
                return "accept_pvp_duel_should_flip_state"
            end

            C_PetBattles.SetPendingReportBattlePetTarget(3)
            C_PetBattles.SetPendingReportTargetFromUnit("mouseover")
            if C_PetBattles._state.pendingReportBattlePetTarget ~= 3 or C_PetBattles._state.pendingReportTargetUnit ~= "mouseover" then
                return "report_targets_should_round_trip"
            end

            C_PetBattles.ForfeitGame()
            if C_PetBattles.GetBattleState() ~= Enum.PetbattleState.Finished then
                return "forfeit_should_update_battle_state"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "Pet battle actions should mutate seeded state"
    );
}
