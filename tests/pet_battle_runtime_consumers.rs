#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::c_api::pet_battle_runtime_model::{PetBattleAbility, PetBattleAura};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::PetBattlePet;

fn loaded_game() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![common::blizzard_ui_dir()];
    common::supply_empty_forever_spellbook_outfits(&env);
    common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    env.exec("assert(C_AddOns.LoadAddOn('Blizzard_PetBattleUI')); assert(C_AddOns.LoadAddOn('Blizzard_QueueStatusFrame'))")
        .expect("load original consumer addons");
    env
}

fn pet_queue_entry(env: &WowLuaEnv) -> (bool, f64, bool, bool, bool) {
    env.eval(
        r#"
        for entry in QueueStatusFrame.statusEntriesPool:EnumerateActive() do
            if entry.Title:GetText() == PET_BATTLE_PVP_QUEUE then
                return true, entry.queuedTime or 0, entry.AverageWait:IsShown(),
                    entry.TimeInQueue:IsShown(),
                    entry.Status:GetText() == QUEUED_STATUS_PROPOSAL
            end
        end
        return false, 0, false, false, false
        "#,
    )
    .expect("read original pet-battle queue entry")
}

#[test]
fn original_queue_frame_uses_admin_supplied_matchmaking_records() {
    let env = loaded_game();
    env.exec(
        "A_Admin.SetPetBattleMatchmakingInfo('queued', 90, 2500, false); A_Admin.FireEvent('PET_BATTLE_QUEUE_STATUS')",
    )
    .expect("supply queued matchmaking state");
    let queued = pet_queue_entry(&env);
    assert!(queued.0 && queued.1 == 2.5 && queued.2 && queued.3 && !queued.4);

    env.exec(
        "A_Admin.SetPetBattleMatchmakingInfo('proposal', 70, 3000, true); A_Admin.FireEvent('PET_BATTLE_QUEUE_STATUS')",
    )
    .expect("supply proposal matchmaking state");
    let proposal = pet_queue_entry(&env);
    assert!(proposal.0 && !proposal.2 && !proposal.3 && proposal.4);
}

fn supply_queue(env: &WowLuaEnv, status: u8) {
    let mut sim = env.state().borrow_mut();
    let queue = &mut sim.pet_battles.runtime.matchmaking;
    queue.status = status;
    queue.estimated_time = 90;
    queue.queued_time_ms = 2500;
}

fn assert_queue_entry_count(env: &WowLuaEnv, expected: i32) {
    let count: i32 = env
        .eval(
            r#"
        QueueStatusFrame:Update()
        local count = 0
        for entry in QueueStatusFrame.statusEntriesPool:EnumerateActive() do
            if entry.Title:GetText() == PET_BATTLE_PVP_QUEUE then count = count + 1 end
        end
        return count
    "#,
        )
        .expect("original queue frame update");
    assert_eq!(count, expected, "original frame's pet-battle queue entries");
}

fn assert_queued_display(env: &WowLuaEnv) {
    env.exec(
        r#"
        RuntimeQueueEntry = CreateFrame('Frame', nil, UIParent, 'QueueStatusEntryTemplate')
        QueueStatusEntry_SetUpPetBattlePvP(RuntimeQueueEntry)
        local entry = RuntimeQueueEntry
        assert(entry.Title:GetText() == PET_BATTLE_PVP_QUEUE)
        assert(not entry.Status:IsShown())
        assert(entry.AverageWait:IsShown() and entry.TimeInQueue:IsShown())
        assert(entry.queuedTime == 2.5, 'supplied milliseconds reach original time display')
        assert(entry.AverageWait:GetText() ==
            LFG_STATISTIC_AVERAGE_WAIT:format(SecondsToTime(90, false, false, 1)))
    "#,
    )
    .expect("original queue entry displays supplied timers");
}

fn assert_queue_label(env: &WowLuaEnv, status: u8, label: &str) {
    supply_queue(env, status);
    env.exec(&format!(
        r#"
        QueueStatusEntry_SetUpPetBattlePvP(RuntimeQueueEntry)
        assert(RuntimeQueueEntry.Status:IsShown())
        assert(RuntimeQueueEntry.Status:GetText() == {label})
        assert(not RuntimeQueueEntry.TimeInQueue:IsShown())
    "#
    ))
    .expect("original queue status label");
}

#[test]
fn original_queue_entry_tracks_supplied_pet_matchmaking() {
    test_timeout! {
        let env = loaded_game();
        supply_queue(&env, 0);
        assert_queue_entry_count(&env, 0);
        supply_queue(&env, 1);
        assert_queue_entry_count(&env, 1);
        assert_queued_display(&env);
        assert_queue_label(&env, 3, "QUEUED_STATUS_PROPOSAL");
        assert_queue_label(&env, 4, "QUEUED_STATUS_WAITING");
        supply_queue(&env, 0);
        assert_queue_entry_count(&env, 0);
    }
}

fn supplied_pet() -> PetBattlePet {
    PetBattlePet {
        name: "Supplied battle pet".into(),
        display_id: None,
        species_id: 39,
        level: 5,
        max_health: 100,
        current_health: 40,
        power: 11,
        speed: 13,
        pet_type: 7,
        ability_ids: vec![73001],
        xp: 4,
        max_xp: 100,
    }
}

fn supply_pet_data(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.pet_battles.player_pets = vec![supplied_pet()];
    sim.pet_battles.enemy_pets = vec![supplied_pet()];
    sim.pet_battles.battle_state = 1;
    sim.pet_battles.num_pets_player = 1;
    sim.pet_battles.num_pets_enemy = 1;
    sim.pet_battles.active_pet_player = 1;
    sim.pet_battles.active_pet_enemy = 1;
    supply_ability_and_aura(&mut sim.pet_battles.runtime);
}

fn supply_ability_and_aura(
    runtime: &mut wow_ui_sim::c_api::pet_battle_runtime_model::PetBattleRuntime,
) {
    runtime.abilities.insert(
        73001,
        PetBattleAbility {
            name: "Supplied ability".into(),
            icon: 136116,
            max_cooldown: 2,
            description: "Supplied scenario ability".into(),
            num_turns: 1,
            pet_type: 7,
            no_strong_weak_hints: true,
        },
    );
    runtime.auras.insert(
        (1, 1),
        vec![PetBattleAura {
            ability_id: 73001,
            instance_id: 81001,
            turns_remaining: 3,
            is_buff: true,
        }],
    );
}

fn assert_populated_health(env: &WowLuaEnv) {
    env.exec(r#"
        local frame = PetBattleFrame.ActiveAlly
        frame.petOwner, frame.petIndex = Enum.BattlePetOwner.Ally, 1
        PetBattleUnitFrame_UpdateHealthInstant(frame)
        assert(frame.ActualHealthBar:IsShown(), "positive supplied health must show bar")
        assert(math.abs(frame.ActualHealthBar:GetWidth() - frame.healthBarWidth * 0.4) < 0.001, "original bar must reflect supplied 40/100")
        assert(frame.HealthText:GetText() ==
            (frame.healthTextFormat or PET_BATTLE_CURRENT_HEALTH_FORMAT):format(40, 100), "original health text must show supplied values")
    "#).expect("original health display reads supplied pet");
}

fn assert_populated_ability_and_aura(env: &WowLuaEnv) {
    env.exec(
        r#"
        PetBattleFrame_UpdateActionBarLayout(PetBattleFrame)
        local button = PetBattleFrame.BottomFrame.abilityButtons[1]
        PetBattleAbilityButton_UpdateIcons(button)
        assert(button.abilityID == 73001 and button:IsShown() and button:IsEnabled())
        assert(button.Icon:GetTexture() == 136116)
        local holder = PetBattleFrame.AllyBuffFrame
        PetBattleAuraHolder_SetUnit(holder, Enum.BattlePetOwner.Ally, 1)
        assert(holder:IsShown() and holder.frames[1]:IsShown())
        local aura = holder.frames[1]
        assert(aura.Icon:GetTexture() == 136116)
        assert(aura.Duration:GetText() == PET_BATTLE_AURA_TURNS_REMAINING:format(3))
        assert(not aura.DebuffBorder:IsShown())
    "#,
    )
    .expect("original ability and aura display supplied records");
}

fn change_pet_data(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.pet_battles.player_pets[0].current_health = 0;
    sim.pet_battles
        .runtime
        .abilities
        .get_mut(&73001)
        .unwrap()
        .icon = 136048;
    sim.pet_battles.runtime.auras.clear();
}

fn assert_changed_pet_display(env: &WowLuaEnv) {
    env.exec(
        r#"
        local frame = PetBattleFrame.ActiveAlly
        PetBattleUnitFrame_UpdateHealthInstant(frame)
        assert(not frame.ActualHealthBar:IsShown())
        local button = PetBattleFrame.BottomFrame.abilityButtons[1]
        PetBattleAbilityButton_UpdateIcons(button)
        assert(button.Icon:GetTexture() == 136048)
        local holder = PetBattleFrame.AllyBuffFrame
        PetBattleAuraHolder_Update(holder)
        assert(not holder:IsShown() and not holder.frames[1]:IsShown())
    "#,
    )
    .expect("original consumers refresh changed host inputs");
}

fn assert_original_action_callbacks(env: &WowLuaEnv) {
    env.exec(
        r#"
        local button = PetBattleFrame.BottomFrame.abilityButtons[1]
        PetBattleAbilityButton_OnClick(button)
        local action, slot = C_PetBattles.GetSelectedAction()
        assert(action == Enum.BattlePetAction.Ability and slot == button:GetID())
        StaticPopupDialogs['PET_BATTLE_FORFEIT'].OnAccept()
        assert(C_PetBattles.GetBattleState() == Enum.PetbattleState.Finished)
        assert(not C_PetBattles.IsInBattle())
    "#,
    )
    .expect("original ability and forfeit callbacks preserve local actions");
}

#[test]
fn original_pet_frames_track_supplied_health_abilities_and_auras() {
    test_timeout! {
        let env = loaded_game();
        supply_pet_data(&env);
        assert_populated_health(&env);
        assert_populated_ability_and_aura(&env);
        change_pet_data(&env);
        assert_changed_pet_display(&env);
        assert_original_action_callbacks(&env);
    }
}

#[test]
fn original_pet_duel_callbacks_emit_host_requests() {
    test_timeout! {
        use wow_ui_sim::c_api::pet_battle_runtime_model::PetBattleRequest;
        let env = loaded_game();
        env.exec(r#"
            C_AddOns.LoadAddOn('Blizzard_UnitPopupShared')
            UnitPopupPetBattleDuelButtonMixin:OnClick({unit = 'target'})
            StaticPopupDialogs['PET_BATTLE_PVP_DUEL_REQUESTED'].OnAccept()
        "#).expect("original duel menu and invitation callbacks");
        let requests = env.state().borrow_mut().pet_battles.runtime.take_requests();
        assert_eq!(requests, vec![
            PetBattleRequest::StartDuel { unit: "target".into(), exact_match: true },
            PetBattleRequest::AcceptDuel,
        ]);
    }
}

fn assert_type_hint(env: &WowLuaEnv, defender_type: i32, suffix: Option<&str>) {
    env.state().borrow_mut().pet_battles.enemy_pets[0].pet_type = defender_type;
    env.exec("PetBattleAbilityButton_UpdateIcons(PetBattleFrame.BottomFrame.abilityButtons[1])")
        .expect("original type hint refresh");
    let shown: bool = env
        .eval("return PetBattleFrame.BottomFrame.abilityButtons[1].BetterIcon:IsShown()")
        .unwrap();
    assert_eq!(shown, suffix.is_some());
    if let Some(suffix) = suffix {
        let texture: String = env
            .eval("return PetBattleFrame.BottomFrame.abilityButtons[1].BetterIcon:GetTextureFilePath()")
            .unwrap();
        assert_eq!(
            texture,
            format!("Interface\\PetBattles\\BattleBar-AbilityBadge-{suffix}")
        );
    }
}

#[test]
fn original_ability_hint_tracks_type_chart() {
    test_timeout! {
        let env = loaded_game();
        supply_pet_data(&env);
        env.state().borrow_mut().pet_battles.runtime.abilities.get_mut(&73001)
            .unwrap().no_strong_weak_hints = false;
        env.exec("PetBattleFrame_UpdateActionBarLayout(PetBattleFrame)").unwrap();
        assert_type_hint(&env, 10, Some("Strong"));
        assert_type_hint(&env, 5, Some("Weak"));
        assert_type_hint(&env, 9, None);
    }
}

fn set_battle_runtime_inputs(env: &WowLuaEnv, phase: i32, npc: bool, selecting: bool, trap: bool) {
    env.exec(&format!(
        "A_Admin.SetPetBattleCounts(3, 2); A_Admin.SetPetBattleState({phase}); \
         A_Admin.SetPetBattleAvailability({}, true, 0); \
         A_Admin.SetPetBattleTeamState(true, false, {selecting}, {npc})",
        i32::from(trap),
    ))
    .expect("supplied battle state");
}

fn assert_action_button_state(env: &WowLuaEnv, selecting: bool, npc: bool, trap: bool) {
    let actual: String = env
        .eval(r#"
            local switch = PetBattleFrame.BottomFrame.SwitchPetButton
            local catch = PetBattleFrame.BottomFrame.CatchButton
            local available = C_PetBattles.IsTrapAvailable()
            return tostring(switch:GetAlpha()) .. "," .. tostring(switch:IsEnabled()) .. "," ..
                tostring(catch:IsEnabled()) .. "," .. tostring(C_PetBattles.ShouldShowPetSelect()) ..
                "," .. tostring(C_PetBattles.IsPlayerNPC(2)) .. "," .. tostring(available)
        "#)
        .expect("original PetBattleUI action button state");
    assert_eq!(
        actual,
        format!("{},{},{},{},{},{}", if selecting { 0 } else { 1 }, false, trap, selecting, npc, trap),
        "original action buttons follow supplied battle state"
    );
}

#[test]
fn original_pet_battle_ui_consumes_supplied_runtime_predicates() {
    test_timeout! {
        let env = loaded_game();
        set_battle_runtime_inputs(&env, 1, true, true, false);
        env.exec("A_Admin.FireEvent('PET_BATTLE_ACTION_SELECTED')")
            .expect("original action button event for forced pet selection");
        assert_action_button_state(&env, true, true, false);

        set_battle_runtime_inputs(&env, 2, true, false, true);
        env.exec("A_Admin.FireEvent('PET_BATTLE_ACTION_SELECTED')")
            .expect("original button event for NPC trap availability");
        assert_action_button_state(&env, false, true, true);

        set_battle_runtime_inputs(&env, 2, false, false, true);
        env.exec("A_Admin.FireEvent('PET_BATTLE_ACTION_SELECTED')")
            .expect("original button event for player-opponent trap availability");
        assert_action_button_state(&env, false, false, false);
    }
}

#[test]
fn empty_pet_queries_do_not_create_sample_rosters() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_PetBattles.GetNumPets(1) == 0 and C_PetBattles.GetNumPets(2) == 0)
        assert(C_PetBattles.GetHealth(1, 1) == 100)
        assert(C_PetBattles.GetMaxHealth(1, 1) == 100)
        assert(C_PetBattles.GetLevel(1, 1) == 1)
        assert(C_PetBattles.GetAbilityInfo(1, 1, 1) == nil)
        assert(C_PetBattles.GetAuraInfo(1, 1, 1) == nil)
        assert(C_PetBattles.GetNumAuras(1, 1) == 0)
        assert(C_PetBattles.GetNumPets(1) == 0 and C_PetBattles.GetNumPets(2) == 0)
    "#,
    )
    .expect("approved empty shapes remain empty after queries");
}
