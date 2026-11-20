#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use wow_ui_sim::c_api::pet_battle_runtime_model::{
    PetBattleAbility, PetBattleAbilityState, PetBattleAura, PetBattleMatchmaking, PetBattleRequest,
    PetBattleRuntime,
};
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
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    seed_runtime_inputs(&mut sim.pet_battles.runtime);
    drop(sim);
    env.exec("A_Admin.SetPetBattleState(1)").unwrap();
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    env.exec("C_PetBattles._state.isWildBattle = true").unwrap();
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
        display_id: None,
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
        display_id: None,
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
        display_id: None,
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
            display_id: None,
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
            display_id: None,
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
    assert_attack_modifier(&env);
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

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
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

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn seed_runtime_inputs(runtime: &mut PetBattleRuntime) {
    runtime.is_wild = true;
    runtime.qualities = [
        ((1, 1), 3),
        ((1, 2), 3),
        ((1, 3), 3),
        ((2, 1), 3),
        ((2, 2), 3),
    ]
    .into();
    runtime.auras.insert(
        (1, 1),
        vec![PetBattleAura {
            ability_id: 1002,
            instance_id: 9001,
            turns_remaining: 2,
            is_buff: true,
        }],
    );
    runtime.parser_states.insert("Stat_Power".into(), 18);
    seed_ability_inputs(runtime);
    seed_ability_states(runtime);
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn seed_ability_inputs(runtime: &mut PetBattleRuntime) {
    for (id, name, cooldown, description, pet_type) in [
        (1001, "Arcane Bite", 2, "Arcane bite.", 7),
        (1002, "Blink Ward", 1, "Blink ward.", 7),
        (1003, "Spring-Loaded", 2, "Jump forward.", 9),
        (1004, "Snowball", 1, "Throw snowball.", 8),
        (1101, "Pebble Toss", 1, "Pebble toss.", 9),
        (1102, "Bog Kick", 1, "Bog kick.", 9),
    ] {
        runtime.abilities.insert(
            id,
            PetBattleAbility {
                name: name.into(),
                icon: 0,
                max_cooldown: cooldown,
                description: description.into(),
                num_turns: 1,
                pet_type,
                no_strong_weak_hints: false,
            },
        );
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn seed_ability_states(runtime: &mut PetBattleRuntime) {
    for (owner, slot, ability, cooldown) in [
        (1, 1, 1, 0),
        (1, 1, 2, 1),
        (1, 2, 1, 0),
        (1, 3, 1, 0),
        (2, 1, 1, 0),
        (2, 2, 1, 0),
    ] {
        runtime.ability_states.insert(
            (owner, slot, ability),
            PetBattleAbilityState {
                usable: true,
                cooldown,
                lockdown: 0,
            },
        );
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const QUEUED_ASSERTIONS: &str = r#"
    local queueState, estimatedTime, queuedTime = C_PetBattles.GetPVPMatchmakingInfo()
    if queueState ~= "queued" then return "matchmaking_should_project_host_queue_state" end
    if estimatedTime ~= 12 or queuedTime ~= 4 then return "queue_timers_should_match_input" end
    return "ok"
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const ACCEPT_ASSERTIONS: &str = r#"
    if not C_PetBattles.CanAcceptQueuedPVPMatch() then
        return "queue_should_be_accepting_before_accept"
    end
    C_PetBattles.AcceptQueuedPVPMatch()
    if C_PetBattles.GetPVPMatchmakingInfo() ~= "entry" then
        return "accept_should_update_queue_status"
    end
    if C_PetBattles.CanAcceptQueuedPVPMatch() then
        return "accept_should_clear_accept_flag"
    end
    return "ok"
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_matchmaking_boundary(env: &WowLuaEnv) {
    env.exec("C_PetBattles.StartPVPMatchmaking()").unwrap();
    env.exec("assert(C_PetBattles.GetPVPMatchmakingInfo() == nil, 'request is not a queue acknowledgement')").unwrap();
    env.state().borrow_mut().pet_battles.runtime.matchmaking = PetBattleMatchmaking {
        status: 1,
        estimated_time: 12,
        queued_time_ms: 4000,
        can_accept: false,
    };
    assert_eq!(env.eval::<String>(QUEUED_ASSERTIONS).unwrap(), "ok");
    {
        let mut sim = env.state().borrow_mut();
        sim.pet_battles.runtime.matchmaking.status = 3;
        sim.pet_battles.runtime.matchmaking.can_accept = true;
    }
    assert_eq!(env.eval::<String>(ACCEPT_ASSERTIONS).unwrap(), "ok");
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_duel_and_report_requests(env: &WowLuaEnv) {
    let _ = env.state().borrow_mut().pet_battles.runtime.take_requests();
    env.exec(
        r#"
        C_PetBattles.StartPVPDuel("target", true)
        C_PetBattles.AcceptPVPDuel()
        C_PetBattles.SetPendingReportBattlePetTarget(3)
        C_PetBattles.SetPendingReportTargetFromUnit("mouseover")
    "#,
    )
    .unwrap();
    let requests = env.state().borrow_mut().pet_battles.runtime.take_requests();
    assert_eq!(
        requests,
        vec![
            PetBattleRequest::StartDuel {
                unit: "target".into(),
                exact_match: true
            },
            PetBattleRequest::AcceptDuel,
            PetBattleRequest::ReportPet(3),
            PetBattleRequest::ReportUnit("mouseover".into()),
        ]
    );
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const PUBLIC_ACTION_ASSERTIONS: &str = r#"
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

    C_PetBattles.ForfeitGame()
    if C_PetBattles.GetBattleState() ~= Enum.PetbattleState.Finished then
        return "forfeit_should_update_battle_state"
    end

    return "ok"
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn pet_battles_actions_and_queue_state_are_mutable() {
    let env = env();
    seed_sample_pet_battle(&env);
    assert_matchmaking_boundary(&env);
    assert_duel_and_report_requests(&env);
    assert_eq!(
        env.eval::<String>(PUBLIC_ACTION_ASSERTIONS).unwrap(),
        "ok",
        "Pet battle actions should mutate seeded state"
    );
}

fn assert_attack_modifier(env: &WowLuaEnv) {
    // Main #1968: selected chart uses Elemental -> Mechanical; legacy unchanged.
    let defender = if cfg!(any(
        feature = "client-retail",
        feature = "client-wowforever"
    )) {
        10
    } else {
        9
    };
    let modifier: f64 = env
        .eval(&format!(
            "return C_PetBattles.GetAttackModifier(7, {defender})"
        ))
        .unwrap();
    assert!(modifier > 1.0, "expected_seeded_attack_modifier");
}
