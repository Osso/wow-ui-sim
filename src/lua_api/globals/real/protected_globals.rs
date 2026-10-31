//! Protected-call policy based on the community API tags; native membership is unknown.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, create_string, registry_get, registry_set};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const PROTECTED: &[&str] = &[
    "AscendStop",
    "AssistUnit",
    "AttackTarget",
    "C_CharacterServices.AssignPCTDistribution",
    "C_CharacterServices.AssignUpgradeDistribution",
    "C_CharacterServices.GetActiveCharacterUpgradeBoostType",
    "C_CharacterServices.GetActiveClassTrialBoostType",
    "C_CharacterServices.GetAutomaticBoost",
    "C_CharacterServices.GetAutomaticBoostCharacter",
    "C_CharacterServices.GetCharacterServiceDisplayData",
    "C_CharacterServices.GetCharacterServiceDisplayDataByVASType",
    "C_CharacterServices.GetCharacterServiceDisplayInfo",
    "C_CharacterServices.GetVASDistributions",
    "C_CharacterServices.HasRequiredBoostForClassTrial",
    "C_CharacterServices.HasRequiredBoostForUnrevoke",
    "C_CharacterServices.SetAutomaticBoost",
    "C_CharacterServices.SetAutomaticBoostCharacter",
    "C_Club.CreateClub",
    "C_Club.KickMember",
    "C_Club.SetClubMemberNote",
    "C_CovenantSanctumUI.DepositAnima",
    "C_Debug.DashboardIsEnabled",
    "C_Debug.GetAllPortLocsForMap",
    "C_Debug.GetMapDebugObjects",
    "C_Debug.TeleportToMapDebugObject",
    "C_Debug.TeleportToMapLocation",
    "C_GuildInfo.Demote",
    "C_GuildInfo.Disband",
    "C_GuildInfo.Leave",
    "C_GuildInfo.Promote",
    "C_GuildInfo.SetLeader",
    "C_GuildInfo.SetNote",
    "C_GuildInfo.Uninvite",
    "C_Item.ConfirmBindOnUse",
    "C_Item.ConfirmNoRefundOnUse",
    "C_Item.UseItemByName",
    "C_ItemUpgrade.UpgradeItem",
    "C_MountJournal.ApplyMountEquipment",
    "C_Ping.SendMacroPing",
    "C_Ping.TogglePingListener",
    "C_ReportSystem.SendReport",
    "CameraOrSelectOrMoveStart",
    "CameraOrSelectOrMoveStop",
    "CancelShapeshiftForm",
    "CastPetAction",
    "CastShapeshiftForm",
    "CastSpell",
    "CastSpellByID",
    "CastSpellByName",
    "ClearFocus",
    "ClearTarget",
    "CopyToClipboard",
    "DeleteGMTicket",
    "DescendStop",
    "DestroyTotem",
    "FocusUnit",
    "ForceLogout",
    "ForceQuit",
    "GuildControlSetRank",
    "GuildControlSetRankFlag",
    "JoinRatedBattlefield",
    "JumpOrAscendStart",
    "LaunchURL",
    "Logout",
    "MoveAndSteerStart",
    "MoveAndSteerStop",
    "MoveBackwardStart",
    "MoveBackwardStop",
    "MoveForwardStart",
    "MoveForwardStop",
    "PetAggressiveMode",
    "PetAttack",
    "PetDefensiveMode",
    "PetFollow",
    "PetMoveTo",
    "PetPassiveMode",
    "PetWait",
    "PitchDownStart",
    "PitchDownStop",
    "PitchUpStart",
    "PitchUpStop",
    "PlaceRaidMarker",
    "PortGraveyard",
    "Quit",
    "RunMacro",
    "SetMoveEnabled",
    "SetMultiCastSpell",
    "SetPartyAssignment",
    "SetRaidTarget",
    "SetSendMailCOD",
    "SetSendMailMoney",
    "SetTurnEnabled",
    "SitStandOrDescendStart",
    "SpellStopCasting",
    "SpellStopTargeting",
    "SpellTargetItem",
    "SpellTargetUnit",
    "StartAttack",
    "StartAutoRun",
    "StopAutoRun",
    "StopMacro",
    "StrafeLeftStart",
    "StrafeLeftStop",
    "StrafeRightStart",
    "StrafeRightStop",
    "Stuck",
    "TargetDirectionEnemy",
    "TargetDirectionFinished",
    "TargetDirectionFriend",
    "TargetLastEnemy",
    "TargetLastFriend",
    "TargetLastTarget",
    "TargetNearest",
    "TargetNearestEnemy",
    "TargetNearestEnemyPlayer",
    "TargetNearestFriend",
    "TargetNearestFriendPlayer",
    "TargetNearestPartyMember",
    "TargetNearestRaidMember",
    "TargetPriorityHighlightEnd",
    "TargetPriorityHighlightStart",
    "TargetUnit",
    "ToggleAutoRun",
    "TogglePetAutocast",
    "ToggleRun",
    "TurnLeftStart",
    "TurnLeftStop",
    "TurnOrActionStart",
    "TurnOrActionStop",
    "TurnRightStart",
    "TurnRightStop",
    "UseAction",
    "UseInventoryItem",
];

const NO_COMBAT: &[&str] = &[
    "C_EquipmentSet.UseEquipmentSet",
    "C_Item.IsItemInRange",
    "C_Map.OpenWorldMap",
    "C_PetJournal.SummonPetByGUID",
    "C_PetJournal.SummonRandomPet",
    "CancelSpellByName",
    "CancelUnitBuff",
    "CheckInteractDistance",
    "ClearOverrideBindings",
    "CreateMacro",
    "EditMacro",
    "LearnTalents",
    "PickupInventoryItem",
    "RemoveTalent",
    "SetBinding",
    "SetBindingClick",
    "SetBindingItem",
    "SetBindingMacro",
    "SetBindingSpell",
    "SetOverrideBinding",
    "SetOverrideBindingClick",
    "SetOverrideBindingItem",
    "SetOverrideBindingMacro",
    "SetOverrideBindingSpell",
    "SetRaidSubgroup",
    "SwapRaidSubgroup",
    "UnlearnSpecialization",
];

const SECURECALL_ORIGIN: &str = "__sim_protected_securecall_origin";

fn securecall_with_origin(state: &mut LuaState) -> LuaResult<u32> {
    let previous = registry_get(state, SECURECALL_ORIGIN);
    let origin = state.call_stack[..=state.ci]
        .iter()
        .rev()
        .find_map(|frame| frame.taint.clone());
    if previous == Val::Nil {
        if let Some(origin) = origin {
            let origin = create_string(state, &origin);
            registry_set(state, SECURECALL_ORIGIN, origin);
        }
    }
    let arguments: Vec<_> = (state.base..state.top)
        .map(|i| state.stack_get(i))
        .collect();
    let start = state.top;
    for argument in arguments {
        state.push(argument);
    }
    let result = state.call_function(start, rilua::vm::callinfo::LUA_MULTRET);
    registry_set(state, SECURECALL_ORIGIN, previous);
    result?;
    Ok((state.top - start) as u32)
}

fn refuse_call(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let combat_only = bool::from_stack(state, 2)?;
    let origin = state.call_stack[..=state.ci]
        .iter()
        .rev()
        .find_map(|frame| frame.taint.clone());
    let origin = match origin {
        Some(origin) => create_string(state, &origin),
        None => registry_get(state, SECURECALL_ORIGIN),
    };
    let blocked = origin != Val::Nil && (!combat_only || borrow_state(state)?.in_combat_lockdown());
    if blocked {
        let function = create_string(state, &format!("{name}()"));
        let event = if combat_only {
            "ADDON_ACTION_BLOCKED"
        } else {
            "ADDON_ACTION_FORBIDDEN"
        };
        dispatch_event_now(state, event, &[origin, function])?;
    }
    state.push(Val::Bool(blocked));
    Ok(1)
}

pub(crate) fn install(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "__sim_protected_global_refused", refuse_call)?;
    LuaApiMut::register_function(lua, "__sim_securecall_with_origin", securecall_with_origin)?;
    let mut source = String::from(
        r#"
        local refused = __sim_protected_global_refused
        local securecallWithOrigin = __sim_securecall_with_origin
        __sim_protected_global_refused = nil
        __sim_securecall_with_origin = nil
        local function wrap(name, combatOnly)
            local namespace, member = string.match(name, "^([^.]+)%.(.+)$")
            local target, key = _G, name
            if namespace then target, key = _G[namespace], member end
            if type(target) == "table" and type(target[key]) == "function" then
                local original = target[key]
                target[key] = function(...)
                    if refused(name, combatOnly) then return end
                    return original(...)
                end
            end
        end
    "#,
    );
    for (names, combat_only) in [(PROTECTED, false), (NO_COMBAT, true)] {
        for name in names {
            source.push_str(&format!("wrap({name:?}, {combat_only})\n"));
        }
    }
    source.push_str(
        r#"
        local function wrapSecureCall(original)
            return function(fn, ...)
                return securecallWithOrigin(original, fn, ...)
            end
        end
        securecall = wrapSecureCall(securecall)
        securecallfunction = wrapSecureCall(securecallfunction)
    "#,
    );
    lua.exec(&source)?;
    Ok(())
}
