//! Temporary Lua-owned `C_PetBattles` runtime state.
//!
//! Rust owns the core `SimState.pet_battles` probes, but this compatibility
//! layer still seeds sample pets and models selection, queue, and duel state
//! that are not yet represented in the Rust subsystem.

const PET_BATTLE_RUNTIME_STATE_LUA: &str = r#"
if C_PetBattles == nil then
  C_PetBattles = __wow_namespace()
end

local __wow_pet_battle_state = rawget(_G, "__wow_pet_battle_state")
if type(__wow_pet_battle_state) ~= "table" then
  __wow_pet_battle_state = {
    battleState = 0,
    numPetsPlayer = 0,
    numPetsEnemy = 0,
    isWildBattle = false,
    queueStatus = Enum.PetBattleQueueStatus and Enum.PetBattleQueueStatus.None or 0,
    queueEstimatedTime = 12,
    queueTime = 4,
    canAcceptQueuedPVPMatch = false,
    selectedActionType = nil,
    selectedActionIndex = nil,
    pendingReportBattlePetTarget = nil,
    pendingReportTargetUnit = nil,
    pvpDuel = {
      pending = false,
      challengedUnit = nil,
      exactMatch = false,
      accepted = false,
    },
    sampleSeeded = false,
  }
  rawset(_G, "__wow_pet_battle_state", __wow_pet_battle_state)
end

local __wow_pet_battle_waiting_state = Enum.PetbattleState and Enum.PetbattleState.WaitingPreBattle or 1
local __wow_pet_battle_finished_state = Enum.PetbattleState and Enum.PetbattleState.Finished or 7

local function __wow_pet_battle_seed_sample()
  if __wow_pet_battle_state.sampleSeeded then
    return
  end

  __wow_pet_battle_state.sampleSeeded = true
  __wow_pet_battle_state.numPetsPlayer = 3
  __wow_pet_battle_state.numPetsEnemy = 2
  __wow_pet_battle_state.isWildBattle = true
  __wow_pet_battle_state.playerPets = {
    {
      name = "Arcane Familiar",
      level = 25,
      health = 1120,
      maxHealth = 1420,
      power = 18,
      speed = 21,
      petType = 7,
      breedQuality = 3,
      xp = 45,
      maxXP = 100,
      abilities = {
        [1] = { id = 1001, name = "Arcane Bite", icon = 0, maxCooldown = 2, description = "Arcane bite.", numTurns = 1, petType = 7, usable = true, cooldown = 0, lockdown = 0 },
        [2] = { id = 1002, name = "Blink Ward", icon = 0, maxCooldown = 1, description = "Blink ward.", numTurns = 1, petType = 7, usable = true, cooldown = 1, lockdown = 0 },
      },
      auras = {
        { auraID = 1002, instanceID = 9001, turnsRemaining = 2, isBuff = true },
      },
    },
    {
      name = "Clockwork Hopper",
      level = 24,
      health = 910,
      maxHealth = 1180,
      power = 15,
      speed = 17,
      petType = 9,
      breedQuality = 3,
      xp = 15,
      maxXP = 100,
      abilities = {
        [1] = { id = 1003, name = "Spring-Loaded", icon = 0, maxCooldown = 2, description = "Jump forward.", numTurns = 1, petType = 9, usable = true, cooldown = 0, lockdown = 0 },
      },
      auras = {},
    },
    {
      name = "Frost Pup",
      level = 23,
      health = 870,
      maxHealth = 1110,
      power = 14,
      speed = 19,
      petType = 8,
      breedQuality = 3,
      xp = 10,
      maxXP = 100,
      abilities = {
        [1] = { id = 1004, name = "Snowball", icon = 0, maxCooldown = 1, description = "Throw snowball.", numTurns = 1, petType = 8, usable = true, cooldown = 0, lockdown = 0 },
      },
      auras = {},
    },
  }
  __wow_pet_battle_state.enemyPets = {
    {
      name = "Stone Lurker",
      level = 24,
      health = 980,
      maxHealth = 1320,
      power = 16,
      speed = 14,
      petType = 9,
      breedQuality = 3,
      xp = 0,
      maxXP = 100,
      abilities = {
        [1] = { id = 1101, name = "Pebble Toss", icon = 0, maxCooldown = 1, description = "Pebble toss.", numTurns = 1, petType = 9, usable = true, cooldown = 0, lockdown = 0 },
      },
      auras = {},
    },
    {
      name = "Bog Hopper",
      level = 24,
      health = 930,
      maxHealth = 1210,
      power = 13,
      speed = 20,
      petType = 9,
      breedQuality = 3,
      xp = 0,
      maxXP = 100,
      abilities = {
        [1] = { id = 1102, name = "Bog Kick", icon = 0, maxCooldown = 1, description = "Bog kick.", numTurns = 1, petType = 9, usable = true, cooldown = 0, lockdown = 0 },
      },
      auras = {},
    },
  }
  __wow_pet_battle_state.abilitiesByID = {
    [1001] = __wow_pet_battle_state.playerPets[1].abilities[1],
    [1002] = __wow_pet_battle_state.playerPets[1].abilities[2],
    [1003] = __wow_pet_battle_state.playerPets[2].abilities[1],
    [1004] = __wow_pet_battle_state.playerPets[3].abilities[1],
    [1101] = __wow_pet_battle_state.enemyPets[1].abilities[1],
    [1102] = __wow_pet_battle_state.enemyPets[2].abilities[1],
  }
end

local function __wow_pet_battle_ensure_active()
  if not __wow_pet_battle_state.sampleSeeded then
    __wow_pet_battle_seed_sample()
  end
end

local function __wow_pet_battle_get_pet(owner, petIndex)
  __wow_pet_battle_ensure_active()
  local pets
  if owner == (Enum.BattlePetOwner and Enum.BattlePetOwner.Ally or 1) then
    pets = __wow_pet_battle_state.playerPets
  elseif owner == (Enum.BattlePetOwner and Enum.BattlePetOwner.Enemy or 2) then
    pets = __wow_pet_battle_state.enemyPets
  else
    return nil
  end

  return pets and pets[petIndex] or nil
end

local function __wow_pet_battle_get_ability(owner, petIndex, abilityIndex)
  local pet = __wow_pet_battle_get_pet(owner, petIndex)
  return pet and pet.abilities and pet.abilities[abilityIndex] or nil
end

C_PetBattles._state = __wow_pet_battle_state

local function __wow_pet_battle_install_default(name, fn)
  if rawget(C_PetBattles, name) == nil then
    C_PetBattles[name] = fn
  end
end

__wow_pet_battle_install_default("GetAllEffectNames", function()
end)

__wow_pet_battle_install_default("GetPetInfoByPetID", function(_petID)
end)

__wow_pet_battle_install_default("IsTrapAvailable", function()
  return false, 0
end)

__wow_pet_battle_install_default("IsPlayerNPC", function(_owner)
  return false
end)

__wow_pet_battle_install_default("ShouldShowPetSelect", function()
  return false
end)

C_PetBattles.IsInBattle = function()
  local battleState = C_PetBattles.GetBattleState()
  return battleState ~= 0 and battleState ~= __wow_pet_battle_finished_state
end
C_PetBattles.IsWildBattle = function()
  return C_PetBattles.IsInBattle() and __wow_pet_battle_state.isWildBattle == true
end
C_PetBattles.GetAbilityInfo = function(owner, petIndex, abilityIndex)
  local ability = __wow_pet_battle_get_ability(owner, petIndex, abilityIndex)
  if not ability then
    return nil
  end
  return ability.id, ability.name, ability.icon, ability.maxCooldown, ability.description, ability.numTurns, ability.petType
end
C_PetBattles.GetAbilityInfoByID = function(abilityID)
  __wow_pet_battle_ensure_active()
  local ability = __wow_pet_battle_state.abilitiesByID and __wow_pet_battle_state.abilitiesByID[abilityID]
  if not ability then
    return nil
  end
  return ability.id, ability.name, ability.icon, ability.maxCooldown, ability.description, ability.numTurns, ability.petType
end
C_PetBattles.GetAbilityState = function(owner, petIndex, abilityIndex)
  local ability = __wow_pet_battle_get_ability(owner, petIndex, abilityIndex)
  if not ability then
    return false, 0, 0
  end
  return ability.usable ~= false, ability.cooldown or 0, ability.lockdown or 0
end
C_PetBattles.GetAuraInfo = function(owner, petIndex, auraIndex)
  local pet = __wow_pet_battle_get_pet(owner, petIndex)
  local aura = pet and pet.auras and pet.auras[auraIndex]
  if not aura then
    return nil
  end
  return aura.auraID, aura.instanceID, aura.turnsRemaining, aura.isBuff
end
C_PetBattles.GetNumAuras = function(owner, petIndex)
  local pet = __wow_pet_battle_get_pet(owner, petIndex)
  return pet and pet.auras and #pet.auras or 0
end
C_PetBattles.GetHealth = function(owner, petIndex)
  local pet = __wow_pet_battle_get_pet(owner, petIndex)
  return pet and pet.health or 0
end
C_PetBattles.GetMaxHealth = function(owner, petIndex)
  local pet = __wow_pet_battle_get_pet(owner, petIndex)
  return pet and pet.maxHealth or 0
end
C_PetBattles.GetPower = function(owner, petIndex)
  local pet = __wow_pet_battle_get_pet(owner, petIndex)
  return pet and pet.power or 0
end
C_PetBattles.GetSpeed = function(owner, petIndex)
  local pet = __wow_pet_battle_get_pet(owner, petIndex)
  return pet and pet.speed or 0
end
C_PetBattles.GetLevel = function(owner, petIndex)
  local pet = __wow_pet_battle_get_pet(owner, petIndex)
  return pet and pet.level or 0
end
C_PetBattles.GetBreedQuality = function(owner, petIndex)
  local pet = __wow_pet_battle_get_pet(owner, petIndex)
  return pet and pet.breedQuality or 0
end
if C_PetBattles.GetXP == nil then
  C_PetBattles.GetXP = function(owner, petIndex)
    local pet = __wow_pet_battle_get_pet(owner, petIndex)
    if not pet then
      return 0, 0
    end
    return pet.xp or 0, pet.maxXP or 0
  end
end
C_PetBattles.GetAttackModifier = function(attackerType, defenderType)
  if attackerType == 7 and defenderType == 9 then
    return 1.5
  end
  return 1.0
end
C_PetBattles.GetAllStates = function(parserEnv)
  if type(parserEnv) ~= "table" then
    return
  end
  parserEnv.STATE_Stat_Power = 18
end
-- The client returns nothing while not queued (QueueStatusFrame lists a
-- pet-battle entry for any non-nil status) and a status string otherwise.
C_PetBattles.GetPVPMatchmakingInfo = function()
  local status = __wow_pet_battle_state.queueStatus
  local matchmaking = Enum.PetBattleQueueStatus and Enum.PetBattleQueueStatus.Matchmaking or 1
  local accepted = Enum.PetBattleQueueStatus and Enum.PetBattleQueueStatus.MatchAccepted or 2
  if status == matchmaking then
    return "queued", __wow_pet_battle_state.queueEstimatedTime, __wow_pet_battle_state.queueTime
  elseif status == accepted then
    return "proposal", __wow_pet_battle_state.queueEstimatedTime, __wow_pet_battle_state.queueTime
  end
  return nil
end
C_PetBattles.CanAcceptQueuedPVPMatch = function()
  return __wow_pet_battle_state.canAcceptQueuedPVPMatch == true
end
if C_PetBattles.StartPVPMatchmaking == nil then
  C_PetBattles.StartPVPMatchmaking = function()
    __wow_pet_battle_ensure_active()
    __wow_pet_battle_state.queueStatus = Enum.PetBattleQueueStatus and Enum.PetBattleQueueStatus.Matchmaking or 1
    __wow_pet_battle_state.canAcceptQueuedPVPMatch = true
  end
end
C_PetBattles.AcceptQueuedPVPMatch = function()
  __wow_pet_battle_state.queueStatus = Enum.PetBattleQueueStatus and Enum.PetBattleQueueStatus.MatchAccepted or 2
  __wow_pet_battle_state.canAcceptQueuedPVPMatch = false
end
C_PetBattles.GetSelectedAction = function()
  return __wow_pet_battle_state.selectedActionType, __wow_pet_battle_state.selectedActionIndex
end
C_PetBattles.UseAbility = function(abilityIndex)
  __wow_pet_battle_state.selectedActionType = Enum.BattlePetAction and Enum.BattlePetAction.Ability or 1
  __wow_pet_battle_state.selectedActionIndex = abilityIndex
end
C_PetBattles.ChangePet = function(petIndex)
  __wow_pet_battle_state.selectedActionType = Enum.BattlePetAction and Enum.BattlePetAction.SwitchPet or 2
  __wow_pet_battle_state.selectedActionIndex = petIndex
end
C_PetBattles.UseTrap = function()
  __wow_pet_battle_state.selectedActionType = Enum.BattlePetAction and Enum.BattlePetAction.Trap or 3
  __wow_pet_battle_state.selectedActionIndex = nil
end
C_PetBattles.SkipTurn = function()
  __wow_pet_battle_state.selectedActionType = Enum.BattlePetAction and Enum.BattlePetAction.Skip or 4
  __wow_pet_battle_state.selectedActionIndex = nil
end
C_PetBattles.StartPVPDuel = function(unitToken, exactMatch)
  __wow_pet_battle_state.pvpDuel.pending = true
  __wow_pet_battle_state.pvpDuel.challengedUnit = unitToken
  __wow_pet_battle_state.pvpDuel.exactMatch = exactMatch == true
  __wow_pet_battle_state.pvpDuel.accepted = false
end
C_PetBattles.AcceptPVPDuel = function()
  __wow_pet_battle_state.pvpDuel.pending = false
  __wow_pet_battle_state.pvpDuel.accepted = true
end
C_PetBattles.SetPendingReportBattlePetTarget = function(petIndex)
  __wow_pet_battle_state.pendingReportBattlePetTarget = petIndex
end
C_PetBattles.SetPendingReportTargetFromUnit = function(unitToken)
  __wow_pet_battle_state.pendingReportTargetUnit = unitToken
end
C_PetBattles.ForfeitGame = function()
  __wow_pet_battle_state.battleState = __wow_pet_battle_finished_state
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PET_BATTLE_RUNTIME_STATE_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_static_pet_battle_fallbacks() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r##"
                local trapReady, trapError = C_PetBattles.IsTrapAvailable()
                if select("#", C_PetBattles.GetAllEffectNames()) ~= 0 then return "effects" end
                if select("#", C_PetBattles.GetPetInfoByPetID("BattlePet-0-000000000000")) ~= 0 then return "pet-info" end
                if trapReady ~= false or trapError ~= 0 then return "trap" end
                if C_PetBattles.IsPlayerNPC() ~= false then return "player-npc" end
                if C_PetBattles.ShouldShowPetSelect() ~= false then return "pet-select" end
                return "ok"
                "##,
            )
            .expect("static pet-battle fallbacks should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn pvp_matchmaking_info_is_nil_until_queued() {
        // QueueStatusFrame lists a pet-battle entry for ANY non-nil status,
        // so a numeric "None" showed the LFG eye with a phantom queue.
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let (idle, queued): (String, String) = env
            .eval(
                r#"
                local idle = tostring(C_PetBattles.GetPVPMatchmakingInfo())
                C_PetBattles.StartPVPMatchmaking()
                return idle, tostring(C_PetBattles.GetPVPMatchmakingInfo())
                "#,
            )
            .expect("matchmaking probe");
        assert_eq!(idle, "nil", "not queued: no status at all");
        assert_eq!(
            queued, "queued",
            "queued: the status string the UI compares against"
        );
    }

    #[test]
    fn preserves_existing_static_pet_battle_providers() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_PetBattles = C_PetBattles or __wow_namespace()

            function C_PetBattles.IsTrapAvailable()
                return true, 9
            end
            function C_PetBattles.IsPlayerNPC(_owner)
                return true
            end
            function C_PetBattles.ShouldShowPetSelect()
                return true
            end
            "#,
        )
        .expect("fixture should install existing C_PetBattles providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                local trapReady, trapError = C_PetBattles.IsTrapAvailable()
                return tostring(trapReady) .. ":" .. trapError .. ":" ..
                    tostring(C_PetBattles.IsPlayerNPC()) .. ":" ..
                    tostring(C_PetBattles.ShouldShowPetSelect())
                "#,
            )
            .expect("existing static pet-battle providers should remain callable");

        assert_eq!(result, "true:9:true:true");
    }
}
