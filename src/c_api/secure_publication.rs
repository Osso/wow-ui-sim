//! Bounded native secure-publication routes for Retail C_* namespace members.

use super::{global_val, register_namespace_value_with_flags};
use crate::lua_api::methods::table_get;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

// R88n.11b, Retail 12.1.5.70688; the verified TSV SHA-256 is
// bd06ccc2a4606f359ee5d430a2eb4d6a1f648946495b968f61cb2419e96621a9.
// Flags are listed per member; this inventory does not cover every C_* route.
const VERIFIED_R88N_11B_MEMBERS: [(&str, &str, u8); 48] = [
    ("C_ClassColor", "GetClassColor", 3),
    ("C_ColorUtil", "ConvertHSLToHSV", 3),
    ("C_ColorUtil", "ConvertHSVToHSL", 3),
    ("C_ColorUtil", "ConvertHSVToRGB", 3),
    ("C_ColorUtil", "ConvertRGBToHSV", 3),
    ("C_ColorUtil", "GenerateTextColorCode", 3),
    ("C_ColorUtil", "WrapTextInColor", 3),
    ("C_ColorUtil", "WrapTextInColorCode", 3),
    ("C_CurveUtil", "CreateColorCurve", 3),
    ("C_CurveUtil", "CreateCurve", 3),
    ("C_CurveUtil", "EvaluateColorFromBoolean", 3),
    ("C_CurveUtil", "EvaluateColorValueFromBoolean", 3),
    ("C_CurveUtil", "EvaluateGameCurve", 3),
    ("C_FunctionContainers", "CreateCallback", 3),
    ("C_GameRules", "AutoConnectToGameModeRealm", 3),
    ("C_GameRules", "DoesGameModeHavePromo", 3),
    ("C_GameRules", "GetActiveGameMode", 3),
    ("C_GameRules", "GetCurrentEventRealmQueues", 3),
    ("C_GameRules", "GetCurrentGameModeDisplayInfo", 3),
    ("C_GameRules", "GetCurrentGameModeRecordID", 3),
    ("C_GameRules", "GetDisplayedGameModeRecordIDAtIndex", 3),
    ("C_GameRules", "GetGameModeDisplayInfoByRecordID", 3),
    ("C_GameRules", "GetGameModeGlueScreenName", 3),
    ("C_GameRules", "GetGameModePromoGlobalString", 3),
    ("C_GameRules", "GetGameRuleAsFloat", 3),
    ("C_GameRules", "GetGameRuleAsFrameStrata", 3),
    ("C_GameRules", "GetNumDisplayedGameModes", 3),
    ("C_GameRules", "IsCharacterlessLoginActive", 3),
    ("C_GameRules", "IsClassAllowedForGameMode", 3),
    ("C_GameRules", "IsGameModeEnabled", 3),
    ("C_GameRules", "IsGameRuleActive", 3),
    ("C_GameRules", "IsMultiActionBarVisibilityForced", 3),
    ("C_GameRules", "IsPersonalResourceDisplayEnabled", 3),
    ("C_GameRules", "IsPlunderstorm", 3),
    ("C_GameRules", "IsStandard", 3),
    ("C_GameRules", "IsWoWHack", 3),
    ("C_ScriptedAnimations", "GetAllScriptedAnimationEffects", 3),
    ("C_Texture", "ClearTitleIconTexture", 3),
    ("C_Texture", "GetAtlasElementID", 3),
    ("C_Texture", "GetAtlasElements", 3),
    ("C_Texture", "GetAtlasExists", 3),
    ("C_Texture", "GetAtlasID", 3),
    ("C_Texture", "GetAtlasInfo", 3),
    ("C_Texture", "GetFilenameFromFileDataID", 3),
    ("C_Texture", "GetTitleIconTexture", 3),
    ("C_Texture", "IsTitleIconTextureReady", 3),
    ("C_Texture", "SetTitleIconTexture", 3),
    ("C_Texture", "SetURLTexture", 3),
];

// R88n.11b marks these Retail 12.1.5.70688 members secure-only (flag 2).
const VERIFIED_R88N_11B_PRIVATE_AURA_MEMBERS: [(&str, u8); 13] = [
    ("AddPrivateAuraUpdateCallback", 2),
    ("AnchorPrivateAura", 2),
    ("GetAllPrivateAuraInstanceIDs", 2),
    ("GetAllPrivateAuras", 2),
    ("GetAuraDataByAuraInstanceIDPrivate", 2),
    ("GetPrivateAuraAnchors", 2),
    ("GetPrivateAuraCasterGUID", 2),
    ("IsPrivateAuraFilteredOutByInstanceID", 2),
    ("RemovePrivateAuraUpdateCallback", 2),
    ("SetPrivateAuraAnchorAddedCallback", 2),
    ("SetPrivateAuraAnchorRemovedCallback", 2),
    ("SetPrivateRaidBossMessageCallback", 2),
    ("SetPrivateWarningTextFrame", 2),
];

pub(crate) fn publish_verified_secure_namespace_members(
    state: &mut LuaState,
) -> LuaResult<Vec<String>> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(Vec::new());
    }

    let mut unmodeled = publish_verified_namespace_members(state, &VERIFIED_R88N_11B_MEMBERS)?;
    unmodeled.extend(publish_verified_private_aura_members(state)?);
    Ok(unmodeled)
}

fn publish_verified_namespace_members(
    state: &mut LuaState,
    members: &[(&str, &str, u8)],
) -> LuaResult<Vec<String>> {
    let mut unmodeled = Vec::new();
    for &(namespace, member, flags) in members {
        let namespace_value = global_val(state, namespace);
        let Val::Table(_) = namespace_value else {
            return Err(runtime_error(format!(
                "R88n.11b ordinary namespace {namespace} is not registered"
            )));
        };
        let member_value = table_get(state, namespace_value, member);
        match member_value {
            Val::Function(_) => {
                register_namespace_value_with_flags(state, namespace, member, member_value, flags)?;
            }
            Val::Nil => unmodeled.push(format!("{namespace}.{member}")),
            value => {
                return Err(runtime_error(format!(
                    "R88n.11b member {namespace}.{member} is {}, not a function",
                    value.type_name()
                )));
            }
        }
    }
    Ok(unmodeled)
}

const PRIVATE_AURA_NAMESPACE: &str = "C_UnitAurasPrivate";

fn publish_verified_private_aura_members(state: &mut LuaState) -> LuaResult<Vec<String>> {
    let namespace_value = global_val(state, PRIVATE_AURA_NAMESPACE);
    let Val::Table(ordinary_namespace) = namespace_value else {
        return Err(runtime_error(format!(
            "R88n.11b ordinary namespace {PRIVATE_AURA_NAMESPACE} is not registered"
        )));
    };

    let mut unmodeled = Vec::new();
    for (member, flags) in VERIFIED_R88N_11B_PRIVATE_AURA_MEMBERS {
        if let Some(name) =
            publish_verified_private_aura_member(state, ordinary_namespace, member, flags)?
        {
            unmodeled.push(name);
        }
    }
    Ok(unmodeled)
}

fn publish_verified_private_aura_member(
    state: &mut LuaState,
    ordinary_namespace: GcRef<rilua::vm::table::Table>,
    member: &str,
    flags: u8,
) -> LuaResult<Option<String>> {
    let namespace_value = Val::Table(ordinary_namespace);
    let member_value = table_get(state, namespace_value, member);
    match member_value {
        Val::Function(_) => {
            register_namespace_value_with_flags(
                state,
                PRIVATE_AURA_NAMESPACE,
                member,
                member_value,
                flags,
            )?;
            crate::lua_api::methods::table_set(state, namespace_value, member, Val::Nil);
            Ok(None)
        }
        Val::Nil => Ok(Some(format!("{PRIVATE_AURA_NAMESPACE}.{member}"))),
        value => Err(runtime_error(format!(
            "R88n.11b member {PRIVATE_AURA_NAMESPACE}.{member} is {}, not a function",
            value.type_name()
        ))),
    }
}
