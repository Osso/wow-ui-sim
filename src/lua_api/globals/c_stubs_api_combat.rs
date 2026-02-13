//! Combat, color, curve, and encounter-related C_* namespace stubs.
//!
//! Split from c_stubs_api_extra.rs to keep file sizes manageable.
//! Contains: C_ColorUtil, C_CombatLog, C_CurveUtil, C_RestrictedActions,
//! C_TransmogOutfitInfo, Constants.EncounterTimelineIconMasks.

use mlua::{Lua, Result, Value};

/// Register all combat/encounter-related stubs.
pub fn register_combat_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    register_c_curve_util(lua)?;
    register_c_color_util(lua, &g)?;
    register_c_combat_log(lua, &g)?;
    register_c_restricted_transmog(lua, &g)?;
    register_encounter_timeline_constants(lua, &g)?;
    register_c_damage_meter(lua, &g)?;
    register_c_combat_text(lua, &g)?;
    register_c_combat_audio_alert(lua, &g)?;
    register_c_housing_photo_sharing(lua, &g)?;
    register_nameplate_constants(lua)?;
    register_c_death_recap(lua, &g)?;
    Ok(())
}

/// C_CurveUtil - creates curve objects for interpolation (used by CurveConstants.lua).
fn register_c_curve_util(lua: &Lua) -> Result<()> {
    lua.load(r#"
        local CurveMT = {}
        CurveMT.__index = CurveMT
        function CurveMT:AddPoint(x, y) table.insert(self._points, {x = x, y = y}) end
        function CurveMT:SetType(t) self._type = t end
        function CurveMT:GetValue(x)
            local pts = self._points
            if #pts == 0 then return 0 end
            if #pts == 1 then return pts[1].y end
            if x <= pts[1].x then return pts[1].y end
            if x >= pts[#pts].x then return pts[#pts].y end
            for i = 1, #pts - 1 do
                if x >= pts[i].x and x <= pts[i+1].x then
                    local t = (x - pts[i].x) / (pts[i+1].x - pts[i].x)
                    return pts[i].y + t * (pts[i+1].y - pts[i].y)
                end
            end
            return pts[#pts].y
        end
        C_CurveUtil = {
            CreateCurve = function()
                return setmetatable({_points = {}, _type = 0}, CurveMT)
            end,
            CreateColorCurve = function()
                return setmetatable({_points = {}, _type = 0}, CurveMT)
            end,
        }
    "#).exec()
}

/// C_ColorUtil - hex color formatting for ColorMixin.
fn register_c_color_util(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let cu = lua.create_table()?;
    cu.set("GenerateTextColorCode", lua.create_function(|_, color: mlua::Table| {
        let r: f64 = color.get("r").unwrap_or(1.0);
        let g: f64 = color.get("g").unwrap_or(1.0);
        let b: f64 = color.get("b").unwrap_or(1.0);
        let a: f64 = color.get("a").unwrap_or(1.0);
        Ok(format!("{:02X}{:02X}{:02X}{:02X}",
            (a * 255.0) as u8, (r * 255.0) as u8,
            (g * 255.0) as u8, (b * 255.0) as u8))
    })?)?;
    cu.set("WrapTextInColor", lua.create_function(|_, (text, color): (String, mlua::Table)| {
        let r: f64 = color.get("r").unwrap_or(1.0);
        let g: f64 = color.get("g").unwrap_or(1.0);
        let b: f64 = color.get("b").unwrap_or(1.0);
        let a: f64 = color.get("a").unwrap_or(1.0);
        let hex = format!("{:02X}{:02X}{:02X}{:02X}",
            (a * 255.0) as u8, (r * 255.0) as u8,
            (g * 255.0) as u8, (b * 255.0) as u8);
        Ok(format!("|c{hex}{text}|r"))
    })?)?;
    g.set("C_ColorUtil", cu)?;
    Ok(())
}

/// C_CombatLog - combat log API (relocated from global functions in modern WoW).
fn register_c_combat_log(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let cl = lua.create_table()?;
    cl.set("DoesObjectMatchFilter", lua.create_function(|_, (unit_flags, mask): (i64, i64)| {
        Ok(unit_flags & mask != 0)
    })?)?;
    cl.set("AddEventFilter", lua.create_function(|_, (_ev, _src, _dst): (Value, Value, Value)| Ok(()))?)?;
    cl.set("ClearEntries", lua.create_function(|_, ()| Ok(()))?)?;
    cl.set("GetCurrentEntryInfo", lua.create_function(|_, ()| Ok(0i32))?)?;
    cl.set("GetCurrentEventInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    cl.set("GetEntryCount", lua.create_function(|_, ()| Ok(0i32))?)?;
    cl.set("ShowCurrentEntry", lua.create_function(|_, ()| Ok(false))?)?;
    cl.set("AdvanceEntry", lua.create_function(|_, _delta: Value| Ok(false))?)?;
    cl.set("GetRetentionTime", lua.create_function(|_, ()| Ok(300.0f64))?)?;
    cl.set("SetRetentionTime", lua.create_function(|_, _time: Value| Ok(()))?)?;
    cl.set("ResetFilter", lua.create_function(|_, ()| Ok(()))?)?;
    cl.set("SetCurrentEntry", lua.create_function(|_, _index: Value| Ok(()))?)?;
    cl.set("ApplyFilterSettings", lua.create_function(|_, _settings: Value| Ok(()))?)?;
    cl.set("RefilterEntries", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("C_CombatLog", cl)?;
    Ok(())
}

/// C_RestrictedActions, C_TransmogOutfitInfo stubs.
fn register_c_restricted_transmog(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let ra = lua.create_table()?;
    ra.set("CheckAllowProtectedFunctions", lua.create_function(|_, ()| Ok(true))?)?;
    g.set("C_RestrictedActions", ra)?;

    let toi = lua.create_table()?;
    toi.set("GetOutfitInfoList", lua.create_function(|lua, ()| lua.create_table())?)?;
    toi.set("GetSlotSourceID", lua.create_function(|_, (_id, _slot): (Value, Value)| Ok(0i32))?)?;
    toi.set("GetAllSlotLocationInfo", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set("C_TransmogOutfitInfo", toi)?;
    Ok(())
}

/// C_DamageMeter - damage/healing meter API.
fn register_c_damage_meter(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsDamageMeterAvailable", lua.create_function(|_, ()| Ok((false, Value::Nil)))?)?;
    t.set("GetAvailableCombatSessions", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetCombatSessionFromID", lua.create_function(|_, _a: mlua::MultiValue| Ok(Value::Nil))?)?;
    t.set("GetCombatSessionFromType", lua.create_function(|_, _a: mlua::MultiValue| Ok(Value::Nil))?)?;
    t.set("GetCombatSessionSourceFromID", lua.create_function(|_, _a: mlua::MultiValue| Ok(Value::Nil))?)?;
    t.set("GetCombatSessionSourceFromType", lua.create_function(|_, _a: mlua::MultiValue| Ok(Value::Nil))?)?;
    t.set("GetSessionDurationSeconds", lua.create_function(|_, _st: Value| Ok(0.0f64))?)?;
    t.set("ResetAllCombatSessions", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("C_DamageMeter", t)?;
    Ok(())
}

/// C_CombatText - combat floating text API.
fn register_c_combat_text(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetCurrentEventInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("SetActiveUnit", lua.create_function(|_, _unit: Value| Ok(()))?)?;
    g.set("C_CombatText", t)?;
    Ok(())
}

/// C_CombatAudioAlert - combat audio alert system.
fn register_c_combat_audio_alert(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetCategoryVoice", lua.create_function(|_, _cat: Value| Ok(0i32))?)?;
    t.set("GetCategoryVolume", lua.create_function(|_, _cat: Value| Ok(1.0f64))?)?;
    t.set("GetFormatSetting", lua.create_function(|_, _a: mlua::MultiValue| Ok(0i32))?)?;
    t.set("GetSpeakerSpeed", lua.create_function(|_, ()| Ok(1.0f64))?)?;
    t.set("GetSpecSetting", lua.create_function(|_, _s: Value| Ok(0i32))?)?;
    t.set("GetThrottle", lua.create_function(|_, _t: Value| Ok(0.0f64))?)?;
    t.set("SetCategoryVoice", lua.create_function(|_, _a: mlua::MultiValue| Ok(()))?)?;
    t.set("SetCategoryVolume", lua.create_function(|_, _a: mlua::MultiValue| Ok(()))?)?;
    t.set("SetFormatSetting", lua.create_function(|_, _a: mlua::MultiValue| Ok(()))?)?;
    t.set("SetSpeakerSpeed", lua.create_function(|_, _s: Value| Ok(()))?)?;
    t.set("SetSpecSetting", lua.create_function(|_, _a: mlua::MultiValue| Ok(()))?)?;
    t.set("SetThrottle", lua.create_function(|_, _a: mlua::MultiValue| Ok(()))?)?;
    t.set("SpeakText", lua.create_function(|_, _text: Value| Ok(()))?)?;
    g.set("C_CombatAudioAlert", t)?;
    Ok(())
}

/// C_HousingPhotoSharing - housing screenshot sharing.
fn register_c_housing_photo_sharing(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsAuthorized", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("BeginAuthorizationFlow", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("ClearAuthorization", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("CompleteAuthorizationFlow", lua.create_function(|_, _url: Value| Ok(()))?)?;
    t.set("GetCropRatio", lua.create_function(|_, ()| Ok(1.0f64))?)?;
    t.set("GetPhotoSharingAuthURL", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("SetScreenshotPreviewTexture", lua.create_function(|_, _tex: Value| Ok(()))?)?;
    t.set("UploadPhotoToService", lua.create_function(|_, _a: mlua::MultiValue| Ok(()))?)?;
    g.set("C_HousingPhotoSharing", t)?;
    Ok(())
}

/// NamePlateConstants - global constant table for nameplate system.
fn register_nameplate_constants(lua: &Lua) -> Result<()> {
    lua.load(r#"
        NamePlateConstants = {
            INFO_DISPLAY_CVAR = "nameplateInfoDisplay",
            CAST_BAR_DISPLAY_CVAR = "nameplateCastBarDisplay",
            THREAT_DISPLAY_CVAR = "nameplateThreatDisplay",
            ENEMY_NPC_AURA_DISPLAY_CVAR = "nameplateEnemyNpcAuraDisplay",
            ENEMY_PLAYER_AURA_DISPLAY_CVAR = "nameplateEnemyPlayerAuraDisplay",
            FRIENDLY_PLAYER_AURA_DISPLAY_CVAR = "nameplateFriendlyPlayerAuraDisplay",
            SHOW_DEBUFFS_ON_FRIENDLY_CVAR = "nameplateShowDebuffsOnFriendly",
            DEBUFF_PADDING_CVAR = "nameplateDebuffPadding",
            AURA_SCALE_CVAR = "nameplateAuraScale",
            SIZE_CVAR = "nameplateSize",
            STYLE_CVAR = "nameplateStyle",
            SIMPLIFIED_TYPES_CVAR = "nameplateSimplifiedTypes",
            SOFT_TARGET_NAMEPLATE_SIZE_CVAR = "SoftTargetNameplateSize",
            SOFT_TARGET_ICON_ENEMY_CVAR = "SoftTargetIconEnemy",
            SOFT_TARGET_ICON_FRIEND_CVAR = "SoftTargetIconFriend",
            SOFT_TARGET_ICON_INTERACT_CVAR = "SoftTargetIconInteract",
            SHOW_FRIENDLY_NPCS_CVAR = "nameplateShowFriendlyNpcs",
            SHOW_ONLY_NAME_FOR_FRIENDLY_PLAYER_UNITS_CVAR =
                "nameplateShowOnlyNameForFriendlyPlayerUnits",
            USE_CLASS_COLOR_FOR_FRIENDLY_PLAYER_UNIT_NAMES_CVAR =
                "nameplateUseClassColorForFriendlyPlayerUnitNames",
            PREVIEW_UNIT_TOKEN = "preview",
            AURA_ITEM_HEIGHT = 25,
            LARGE_HEALTH_BAR_HEIGHT = 20,
            SMALL_HEALTH_BAR_HEIGHT = 10,
            HEALTH_BAR_FONT_HEIGHT = 12,
            LARGE_CAST_BAR_HEIGHT = 16,
            SMALL_CAST_BAR_HEIGHT = 10,
            CAST_BAR_FONT_HEIGHT = 10,
            CAST_BAR_ICON_HEIGHT = 12,
            NAME_PLATE_SCALES = {
                [1] = 0.75, [2] = 1.0, [3] = 1.25, [4] = 1.5, [5] = 2.0,
            },
        }
    "#).exec()
}

/// C_DeathRecap - death recap data.
fn register_c_death_recap(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let t = lua.create_table()?;
    t.set("HasRecapEvents", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetRecapEvents", lua.create_function(|lua, _id: Value| lua.create_table())?)?;
    t.set("GetRecapMaxHealth", lua.create_function(|_, _id: Value| Ok(0i32))?)?;
    t.set("GetRecapLink", lua.create_function(|_, _id: Value| Ok(Value::Nil))?)?;
    g.set("C_DeathRecap", t)?;
    Ok(())
}

/// Constants.EncounterTimelineIconMasks - bitmask constants for timeline icon filtering.
fn register_encounter_timeline_constants(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let constants: mlua::Table = match g.get("Constants")? {
        Value::Table(t) => t,
        _ => {
            let t = lua.create_table()?;
            g.set("Constants", t.clone())?;
            t
        }
    };
    let masks = lua.create_table()?;
    masks.set("EncounterTimelineTankAlertIcons", 1i32)?;
    masks.set("EncounterTimelineHealerAlertIcons", 2i32)?;
    masks.set("EncounterTimelineDamageAlertIcons", 4i32)?;
    masks.set("EncounterTimelineDeadlyIcons", 8i32)?;
    masks.set("EncounterTimelineDispelIcons", 16i32)?;
    masks.set("EncounterTimelineEnrageIcons", 32i32)?;
    masks.set("EncounterTimelineAllIcons", 63i32)?;
    constants.set("EncounterTimelineIconMasks", masks)?;
    Ok(())
}
