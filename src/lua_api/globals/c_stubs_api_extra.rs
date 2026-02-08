//! Extra C_* namespace stubs and global tables split from c_stubs_api.rs.
//!
//! Contains:
//! - C_DelvesUI - Delves companion data
//! - C_ZoneAbility - Zone ability data
//! - C_ItemSocketInfo, C_PetInfo, C_UnitAurasPrivate, C_Sound
//! - Missing global functions, constants, and utility tables

use mlua::{Lua, Result, Value};

/// Register all extra stubs (called from c_stubs_api::register_c_stubs_api).
pub fn register_extra_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    register_missing_c_namespaces(lua, &g)?;
    register_missing_global_functions(lua, &g)?;
    register_missing_constants(lua, &g)?;
    register_missing_global_tables(lua, &g)?;
    register_c_delves_ui(lua)?;
    register_c_zone_ability(lua)?;
    Ok(())
}

/// C_* namespace stubs that are referenced during addon loading.
fn register_missing_c_namespaces(lua: &Lua, g: &mlua::Table) -> Result<()> {
    // C_ItemSocketInfo
    let isi = lua.create_table()?;
    isi.set("GetCurrUIType", lua.create_function(|_, ()| Ok(0i32))?)?;
    isi.set("GetExistingSocketInfo", lua.create_function(|_, _idx: i32| Ok(Value::Nil))?)?;
    isi.set("AcceptSockets", lua.create_function(|_, ()| Ok(()))?)?;
    isi.set("CloseSocketInfo", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("C_ItemSocketInfo", isi)?;

    // C_PetInfo
    let pi = lua.create_table()?;
    pi.set("GetPetTamersForMap", lua.create_function(|lua, _map_id: Value| lua.create_table())?)?;
    pi.set("GetSpellForPetAction", lua.create_function(|_, _action: Value| Ok(Value::Nil))?)?;
    pi.set("IsPetActionPassive", lua.create_function(|_, _action: Value| Ok(false))?)?;
    g.set("C_PetInfo", pi)?;

    // C_UnitAurasPrivate
    let uap = lua.create_table()?;
    uap.set("GetAuraDataBySlot", lua.create_function(|_, (_unit, _slot): (Value, Value)| Ok(Value::Nil))?)?;
    uap.set("SetPrivateAuraAnchorAddedCallback", lua.create_function(|_, _cb: Value| Ok(()))?)?;
    uap.set("SetPrivateAuraAnchorRemovedCallback", lua.create_function(|_, _cb: Value| Ok(()))?)?;
    uap.set("GetPrivateAuraAnchors", lua.create_function(|lua, _unit: Value| lua.create_table())?)?;
    uap.set("SetPrivateWarningTextFrame", lua.create_function(|_, _frame: Value| Ok(()))?)?;
    g.set("C_UnitAurasPrivate", uap)?;

    // C_Sound
    let snd = lua.create_table()?;
    snd.set("GetSoundScaledVolume", lua.create_function(|_, _id: Value| Ok(1.0f64))?)?;
    snd.set("IsPlaying", lua.create_function(|_, _handle: Value| Ok(false))?)?;
    snd.set("PlayItemSound", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    snd.set("PlayVocalErrorSound", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    g.set("C_Sound", snd)?;

    Ok(())
}

/// Global functions referenced during addon loading.
fn register_missing_global_functions(lua: &Lua, g: &mlua::Table) -> Result<()> {
    // RegisterUIPanel - registers a frame for panel layout management
    g.set("RegisterUIPanel", lua.create_function(|_, (_frame, _attrs): (Value, Option<Value>)| Ok(()))?)?;

    // GetScenariosChoiceOrder - returns table of scenario ordering
    g.set("GetScenariosChoiceOrder", lua.create_function(|lua, ()| lua.create_table())?)?;

    // NUM_LE_LFG_CATEGORYS - number of LFG categories
    g.set("NUM_LE_LFG_CATEGORYS", 7i32)?;

    // LE_AUTOCOMPLETE_PRIORITY_* constants
    g.set("LE_AUTOCOMPLETE_PRIORITY_OTHER", 1i32)?;
    g.set("LE_AUTOCOMPLETE_PRIORITY_INTERACTED", 2i32)?;
    g.set("LE_AUTOCOMPLETE_PRIORITY_IN_GROUP", 3i32)?;
    g.set("LE_AUTOCOMPLETE_PRIORITY_GUILD", 4i32)?;
    g.set("LE_AUTOCOMPLETE_PRIORITY_FRIEND", 5i32)?;
    g.set("LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER", 6i32)?;
    g.set("LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER_SAME_REALM", 7i32)?;

    // AUTOCOMPLETE_LABEL_* strings used alongside priority constants
    g.set("AUTOCOMPLETE_LABEL_INTERACTED", "Interacted")?;
    g.set("AUTOCOMPLETE_LABEL_GROUP", "Group")?;
    g.set("AUTOCOMPLETE_LABEL_GUILD", "Guild")?;
    g.set("AUTOCOMPLETE_LABEL_FRIEND", "Friend")?;

    // LE_PARTY_CATEGORY_* used by VoiceUtils.lua
    g.set("LE_PARTY_CATEGORY_HOME", 1i32)?;
    g.set("LE_PARTY_CATEGORY_INSTANCE", 2i32)?;

    Ok(())
}

/// Constants tables referenced during addon loading.
fn register_missing_constants(lua: &Lua, g: &mlua::Table) -> Result<()> {
    register_bag_constants(lua, g)?;
    register_chat_constants(lua, g)?;
    register_pet_inventory_constants(lua, g)?;
    Ok(())
}

fn register_bag_constants(_lua: &Lua, g: &mlua::Table) -> Result<()> {
    // BACKPACK_CONTAINER = Enum.BagIndex.Backpack = 0
    g.set("BACKPACK_CONTAINER", 0i32)?;
    // NUM_BAG_SLOTS + NUM_REAGENTBAG_SLOTS
    g.set("NUM_BAG_SLOTS", 4i32)?;
    g.set("NUM_REAGENTBAG_SLOTS", 1i32)?;
    g.set("NUM_TOTAL_EQUIPPED_BAG_SLOTS", 5i32)?;
    Ok(())
}

fn register_chat_constants(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let cfc = lua.create_table()?;
    cfc.set("MaxCharacterNameBytes", 305i32)?;
    cfc.set("MaxChatChannels", 20i32)?;
    cfc.set("MaxChatWindows", 10i32)?;
    cfc.set("ScrollToBottomFlashInterval", 0.5f64)?;
    cfc.set("WhisperSoundAlertCooldown", 3.0f64)?;
    cfc.set("TruncatedCommunityNameLength", 20i32)?;
    cfc.set("TruncatedCommunityNameWithoutChannelLength", 15i32)?;
    cfc.set("MaxRememberedWhisperTargets", 10i32)?;
    g.set("ChatFrameConstants", cfc)?;

    let mfsb = lua.create_table()?;
    mfsb.set("InitialScrollDelay", 0.4f64)?;
    mfsb.set("HeldScrollDelay", 0.04f64)?;
    g.set("MessageFrameScrollButtonConstants", mfsb)?;
    Ok(())
}

fn register_pet_inventory_constants(lua: &Lua, g: &mlua::Table) -> Result<()> {
    // Constants.PetConsts_PostCata.STABLED_PETS_FIRST_SLOT_INDEX
    let constants: mlua::Table = match g.get("Constants")? {
        Value::Table(t) => t,
        _ => {
            let t = lua.create_table()?;
            g.set("Constants", t.clone())?;
            t
        }
    };
    let pet_consts = lua.create_table()?;
    pet_consts.set("STABLED_PETS_FIRST_SLOT_INDEX", 5i32)?;
    pet_consts.set("EXTRA_PET_STABLE_SLOT", 5i32)?;
    pet_consts.set("NUM_PET_SLOTS_THAT_NEED_LEARNED_SPELL", 5i32)?;
    constants.set("PetConsts_PostCata", pet_consts)?;

    // Constants.InventoryConstants
    let inv = lua.create_table()?;
    inv.set("NumBagSlots", 4i32)?;
    inv.set("NumReagentBagSlots", 1i32)?;
    constants.set("InventoryConstants", inv)?;

    Ok(())
}

/// Global Lua tables that are referenced by addon code.
fn register_missing_global_tables(lua: &Lua, g: &mlua::Table) -> Result<()> {
    if g.get::<Value>("QuestUtil")?.is_nil() {
        g.set("QuestUtil", lua.create_table()?)?;
    }
    if g.get::<Value>("ChatFrameMixin")?.is_nil() {
        g.set("ChatFrameMixin", lua.create_table()?)?;
    }
    if g.get::<Value>("ChatFrameEditBoxMixin")?.is_nil() {
        g.set("ChatFrameEditBoxMixin", lua.create_table()?)?;
    }
    if g.get::<Value>("TalentButtonUtil")?.is_nil() {
        g.set("TalentButtonUtil", build_talent_button_util(lua)?)?;
    }
    if g.get::<Value>("SpellSearchUtil")?.is_nil() {
        g.set("SpellSearchUtil", build_spell_search_util(lua)?)?;
    }
    if g.get::<Value>("Dispatcher")?.is_nil() {
        g.set("Dispatcher", build_dispatcher_stub(lua)?)?;
    }
    Ok(())
}

/// TalentButtonUtil - utility table for talent button rendering.
fn build_talent_button_util(lua: &Lua) -> Result<mlua::Table> {
    let tbu = lua.create_table()?;
    tbu.set("CircleEdgeDiameterOffset", 1.2f64)?;
    tbu.set("SquareEdgeMinDiameterOffset", 1.2f64)?;
    tbu.set("SquareEdgeMaxDiameterOffset", 1.5f64)?;
    tbu.set("ChoiceEdgeMinDiameterOffset", 1.2f64)?;
    tbu.set("ChoiceEdgeMaxDiameterOffset", 1.5f64)?;
    let bvs = lua.create_table()?;
    for (i, name) in ["Normal", "Gated", "Disabled", "Locked", "Selectable",
                       "Maxed", "Invisible", "RefundInvalid", "DisplayError"]
        .iter().enumerate()
    {
        bvs.set(*name, (i + 1) as i32)?;
    }
    tbu.set("BaseVisualState", bvs)?;
    Ok(tbu)
}

/// SpellSearchUtil - spell search utility tables.
fn build_spell_search_util(lua: &Lua) -> Result<mlua::Table> {
    let ssu = lua.create_table()?;
    let mt = lua.create_table()?;
    for (i, name) in ["DescriptionMatch", "NameMatch", "RelatedMatch", "ExactMatch",
                       "NotOnActionBar", "OnInactiveBonusBar", "OnDisabledActionBar",
                       "AssistedCombat"].iter().enumerate()
    {
        mt.set(*name, (i + 1) as i32)?;
    }
    ssu.set("MatchType", mt)?;
    let st = lua.create_table()?;
    for (i, name) in ["Trait", "PvPTalent", "SpellBookItem"].iter().enumerate() {
        st.set(*name, (i + 1) as i32)?;
    }
    ssu.set("SourceType", st)?;
    let ft = lua.create_table()?;
    for (i, name) in ["Text", "ActionBar", "Name", "AssistedCombat"].iter().enumerate() {
        ft.set(*name, (i + 1) as i32)?;
    }
    ssu.set("FilterType", ft)?;
    ssu.set("ActionBarStatusTooltips", lua.create_table()?)?;
    Ok(ssu)
}

/// Dispatcher - event dispatch system (real impl: Blizzard_Dispatcher addon).
fn build_dispatcher_stub(lua: &Lua) -> Result<mlua::Table> {
    let d = lua.create_table()?;
    d.set("Events", lua.create_table()?)?;
    d.set("Functions", lua.create_table()?)?;
    d.set("Scripts", lua.create_table()?)?;
    d.set("NextEventID", 1i32)?;
    d.set("NextFunctionID", 1i32)?;
    d.set("NextScriptID", 1i32)?;
    let noop = lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?;
    for name in ["Initialize", "OnEvent", "RegisterEvent", "UnregisterEvent",
                  "UnregisterAllEvents", "UnregisterAll", "RegisterFunction",
                  "UnregisterFunction", "UnregisterAllFunctions",
                  "RegisterScript", "UnregisterScript", "UnregisterAllScripts"]
    {
        d.set(name, noop.clone())?;
    }
    Ok(d)
}

/// C_DelvesUI namespace - Delves companion data.
fn register_c_delves_ui(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetTraitTreeForCompanion", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetRoleNodeForCompanion", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetRoleSubtreeForCompanion", lua.create_function(|_, _role_type: Value| Ok(0i32))?)?;
    t.set("GetCreatureDisplayInfoForCompanion", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetCurioNodeForCompanion", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetCurrentDelvesSeasonNumber", lua.create_function(|_, ()| Ok(1i32))?)?;
    t.set("GetDelvesMinRequiredLevel", lua.create_function(|_, ()| Ok(80i32))?)?;
    t.set("GetFactionForCompanion", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("HasActiveDelve", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetUnseenCuriosBySlotType", lua.create_function(|lua, _slot_type: Value| lua.create_table())?)?;
    t.set("GetDelvesFactionForSeason", lua.create_function(|_, _season: Value| Ok(Value::Nil))?)?;
    t.set("RequestPartyEligibilityForDelveTiers", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("SaveSeenCuriosBySlotType", lua.create_function(|_, (_slot_type, _table): (Value, Value)| Ok(()))?)?;
    lua.globals().set("C_DelvesUI", t)?;
    Ok(())
}

/// C_ZoneAbility namespace - zone ability data.
fn register_c_zone_ability(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetActiveAbilities", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetZoneAbilityIcon", lua.create_function(|_, _spell_id: Value| Ok(Value::Nil))?)?;
    lua.globals().set("C_ZoneAbility", t)?;
    Ok(())
}
