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
    register_secure_namespaces(lua, &g)?;
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
    isi.set("IsArtifactRelicItem", lua.create_function(|_, _item: Value| Ok(false))?)?;
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
    uap.set("SetPrivateRaidBossMessageCallback", lua.create_function(|_, _cb: Value| Ok(()))?)?;
    g.set("C_UnitAurasPrivate", uap)?;

    // C_LevelLink - level-gated spell/action locking (nothing locked in simulator)
    let ll = lua.create_table()?;
    ll.set("IsActionLocked", lua.create_function(|_, _action_id: Value| Ok(false))?)?;
    ll.set("IsSpellLocked", lua.create_function(|_, _spell_id: Value| Ok(false))?)?;
    g.set("C_LevelLink", ll)?;

    Ok(())
}

/// Global functions referenced during addon loading.
fn register_missing_global_functions(lua: &Lua, g: &mlua::Table) -> Result<()> {
    // IsPlayerInWorld - always true in the simulator
    g.set("IsPlayerInWorld", lua.create_function(|_, ()| Ok(true))?)?;

    // ActionBarController_GetCurrentActionBarState - returns LE_ACTIONBAR_STATE_MAIN (1)
    g.set("ActionBarController_GetCurrentActionBarState", lua.create_function(|_, ()| Ok(1i32))?)?;

    // GetMaxLevelForLatestExpansion - max player level for current expansion (The War Within = 80)
    g.set("GetMaxLevelForLatestExpansion", lua.create_function(|_, ()| Ok(80i32))?)?;

    // Glyph functions
    g.set("HasAttachedGlyph", lua.create_function(|_, _spell_id: Value| Ok(false))?)?;
    g.set("IsSpellValidForPendingGlyph", lua.create_function(|_, _spell_id: Value| Ok(false))?)?;

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

    // SpellIsSelfBuff - returns whether a spell is a self-buff (used by AuraUtil)
    g.set("SpellIsSelfBuff", lua.create_function(|_, _spell_id: i32| Ok(false))?)?;

    // CombatLog C++ API functions used by Blizzard_CombatLog
    g.set("CombatLogResetFilter", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("CombatLogAddFilter", lua.create_function(|_, (_event_list, _source_flags, _dest_flags): (Value, Value, Value)| Ok(()))?)?;
    g.set("CombatLogSetCurrentEntry", lua.create_function(|_, _index: Value| Ok(()))?)?;
    g.set("CombatLogGetCurrentEntry", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("CombatLogGetNumEntries", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("CombatLogShowCurrentEntry", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("CombatLogAdvanceEntry", lua.create_function(|_, _delta: Value| Ok(false))?)?;
    g.set("CombatLogClearEntries", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("CombatLogGetCurrentEventInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    g.set("CombatLogGetRetentionTime", lua.create_function(|_, ()| Ok(300.0f64))?)?;
    g.set("CombatLogSetRetentionTime", lua.create_function(|_, _time: Value| Ok(()))?)?;
    // CombatLog_Object_IsA: checks if unitFlags match a filter mask (bitwise AND)
    g.set("CombatLog_Object_IsA", lua.create_function(|_, (unit_flags, mask): (i64, i64)| {
        Ok(unit_flags & mask != 0)
    })?)?;

    // GetExpansionDisplayInfo - returns expansion logo data (nil = no data)
    g.set("GetExpansionDisplayInfo", lua.create_function(|_, _expansion_level: Value| Ok(Value::Nil))?)?;

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
    if g.get::<Value>("UIFrameManager_ManagedFrameMixin")?.is_nil() {
        // Stub with OnLoad/UpdateFrameState — Blizzard_UIFrameManager loads after
        // Blizzard_Tutorials (alphabetical order), so RPETutorialInterruptMixin:OnLoad
        // calls UIFrameManager_ManagedFrameMixin.OnLoad(self) before the real definition.
        // The real OnLoad registers frames with UIFrameManager; our stub is a no-op.
        lua.load(r#"
            UIFrameManager_ManagedFrameMixin = {}
            function UIFrameManager_ManagedFrameMixin:OnLoad()
                if UIFrameManager and UIFrameManager.RegisterFrameForFrameType then
                    UIFrameManager:RegisterFrameForFrameType(self, self.frameType)
                end
            end
            function UIFrameManager_ManagedFrameMixin:UpdateFrameState(show)
                self:SetShown(show)
            end
        "#).exec()?;
    }
    // ActionButtonSpellAlertManager - referenced by PetBattleUI OnLoad
    // before ActionBar workarounds run. Provide stub with ShowAlert/HideAlert.
    if g.get::<Value>("ActionButtonSpellAlertManager")?.is_nil() {
        lua.load(r#"
            ActionButtonSpellAlertManager = {
                ShowAlert = function() end,
                HideAlert = function() end,
            }
        "#).exec()?;
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

/// Secure/premium/niche C_* namespaces referenced during addon loading.
fn register_secure_namespaces(lua: &Lua, g: &mlua::Table) -> Result<()> {
    register_auth_ping_store(lua, g)?;
    register_trial_raf_token(lua, g)?;
    register_shop_who_auras(lua, g)?;
    register_guild_bank_pet_battles(lua, g)?;
    Ok(())
}

/// C_AuthChallenge, C_PingSecure, C_StoreSecure stubs.
fn register_auth_ping_store(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let auth_challenge = lua.create_table()?;
    auth_challenge.set("SetFrame", lua.create_function(|_, _frame: Value| Ok(()))?)?;
    g.set("C_AuthChallenge", auth_challenge)?;

    // C_PingSecure - uses noop metatable for all callback setters
    lua.load(r#"
        C_PingSecure = setmetatable({}, {
            __index = function() return function() end end,
        })
    "#).exec()?;

    // C_WowTokenSecure - secure token operations (noop metatable)
    lua.load(r#"
        C_WowTokenSecure = setmetatable({}, {
            __index = function() return function() end end,
        })
    "#).exec()?;

    // C_Ping - ping system (non-secure side)
    let ping = lua.create_table()?;
    ping.set("GetDefaultPingOptions", lua.create_function(|lua, ()| lua.create_table())?)?;
    ping.set("GetTextureKitForType", lua.create_function(|_, _ping_type: Value| Ok(Value::Nil))?)?;
    g.set("C_Ping", ping)?;

    // C_StoreSecure - uses noop metatable for ~40 methods
    lua.load(r#"
        C_StoreSecure = setmetatable({
            IsStoreAvailable = function() return false end,
            IsAvailable = function() return false end,
            HasPurchaseInProgress = function() return false end,
            HasPurchaseList = function() return false end,
            HasProductList = function() return false end,
        }, { __index = function() return function() end end })
    "#).exec()?;
    Ok(())
}

/// C_ClassTrial, C_RecruitAFriend, C_WowTokenPublic, C_FriendList stubs.
fn register_trial_raf_token(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let class_trial = lua.create_table()?;
    class_trial.set("IsClassTrialCharacter", lua.create_function(|_, ()| Ok(false))?)?;
    class_trial.set("GetClassTrialLogoutTimeSeconds", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("C_ClassTrial", class_trial)?;

    let raf = lua.create_table()?;
    raf.set("GetRecruitInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    raf.set("IsEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    raf.set("IsRecruitingEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    // GetRAFInfo returns nil when there's no active RAF relationship
    raf.set("GetRAFInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    // GetRAFSystemInfo returns a table with RAF system configuration
    raf.set("GetRAFSystemInfo", lua.create_function(|lua, ()| {
        let info = lua.create_table()?;
        info.set("maxRecruits", 0i32)?;
        info.set("maxRecruitMonths", 0i32)?;
        info.set("maxRewardMonths", 0i32)?;
        info.set("daysInCycle", 30i32)?;
        Ok(info)
    })?)?;
    g.set("C_RecruitAFriend", raf)?;

    let wow_token = lua.create_table()?;
    wow_token.set("GetCurrentMarketPrice", lua.create_function(|_, ()| Ok(0i32))?)?;
    wow_token.set("GetGuaranteedPrice", lua.create_function(|_, ()| Ok(0i32))?)?;
    wow_token.set("UpdateTokenCount", lua.create_function(|_, ()| Ok(()))?)?;
    // GetCommerceSystemStatus returns (purchaseAvailable, listAvailable, balanceEnabled)
    wow_token.set("GetCommerceSystemStatus", lua.create_function(|_, ()| {
        Ok((false, false, false))
    })?)?;
    wow_token.set("UpdateMarketPrice", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("C_WowTokenPublic", wow_token)?;

    // C_FriendList - friend list / who system
    let friend_list = lua.create_table()?;
    friend_list.set("SetWhoToUi", lua.create_function(|_, _flag: bool| Ok(()))?)?;
    friend_list.set("SendWho", lua.create_function(|_, _msg: String| Ok(()))?)?;
    friend_list.set("GetNumWhoResults", lua.create_function(|_, ()| Ok(0i32))?)?;
    friend_list.set("GetNumFriends", lua.create_function(|_, ()| Ok(0i32))?)?;
    friend_list.set("GetNumOnlineFriends", lua.create_function(|_, ()| Ok(0i32))?)?;
    friend_list.set("GetFriendInfoByIndex", lua.create_function(|_, _idx: i32| Ok(Value::Nil))?)?;
    friend_list.set("ShowFriends", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("C_FriendList", friend_list)?;

    Ok(())
}

/// C_CatalogShop, C_Who, C_PrivateAuras stubs.
fn register_shop_who_auras(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let catalog_shop = lua.create_table()?;
    catalog_shop.set("GetAvailableCategoryIDs", lua.create_function(|lua, ()| lua.create_table())?)?;
    catalog_shop.set("IsShop2Enabled", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("C_CatalogShop", catalog_shop)?;

    let who = lua.create_table()?;
    who.set("SetWhoToUi", lua.create_function(|_, _flag: bool| Ok(()))?)?;
    who.set("SendWho", lua.create_function(|_, _msg: String| Ok(()))?)?;
    who.set("GetWhoInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    g.set("C_Who", who)?;

    let private_auras = lua.create_table()?;
    private_auras.set("SetPrivateRaidBossMessageCallback", lua.create_function(|_, _cb: Value| Ok(()))?)?;
    g.set("C_PrivateAuras", private_auras)?;
    Ok(())
}

/// C_GuildBank, C_PetBattles stubs.
fn register_guild_bank_pet_battles(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let guild_bank = lua.create_table()?;
    guild_bank.set("IsGuildBankEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    guild_bank.set("GetCurrentBankTab", lua.create_function(|_, ()| Ok(1i32))?)?;
    guild_bank.set("FetchNumTabs", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("C_GuildBank", guild_bank)?;

    // C_PetBattles - methods return 0 by default (many are numeric: GetHealth, GetLevel, etc.)
    // Returning nil would cause "attempt to compare nil with number" in PetBattle Lua code.
    // Methods that return non-numeric types must be listed explicitly.
    lua.load(r#"
        C_PetBattles = setmetatable({
            IsInBattle = function() return false end,
            IsWildBattle = function() return false end,
            IsPlayerNPC = function() return false end,
            GetAllEffectNames = function() end,
            GetAllStates = function() return {} end,
            GetBattleState = function() return nil end,
            GetPVPMatchmakingInfo = function() return nil end,
        }, { __index = function() return function() return 0 end end })
    "#).exec()?;
    Ok(())
}

/// C_DelvesUI namespace - Delves companion data.
fn register_c_delves_ui(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetTraitTreeForCompanion", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetRoleNodeForCompanion", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetRoleSubtreeForCompanion", lua.create_function(|_, _role_type: Value| Ok(0i32))?)?;
    t.set("GetCreatureDisplayInfoForCompanion", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetCurioNodeForCompanion", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
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

/// Achievement category API stubs needed by Blizzard_AchievementUI at parse time.
pub fn register_achievement_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("GetCategoryList", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set("GetGuildCategoryList", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set("GetStatisticsCategoryList", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set(
        "GetCategoryInfo",
        lua.create_function(|_, _id: Value| Ok((Value::Nil, -1i32, -1i32)))?,
    )?;
    g.set(
        "GetCategoryNumAchievements",
        lua.create_function(|_, _id: Value| Ok((0i32, 0i32, 0i32)))?,
    )?;
    g.set(
        "GetTotalAchievementPoints",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(0i32))?,
    )?;
    g.set(
        "GetLatestCompletedAchievements",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(mlua::MultiValue::new()))?,
    )?;
    g.set("GetAchievementInfo", lua.create_function(stub_get_achievement_info)?)?;
    g.set(
        "GetTrackedAchievements",
        lua.create_function(|_, ()| Ok(mlua::MultiValue::new()))?,
    )?;
    g.set(
        "GetNumCompletedAchievements",
        lua.create_function(|_, _for_guild: Option<bool>| Ok((0i32, 0i32)))?,
    )?;
    Ok(())
}

/// Stub for GetAchievementInfo — returns 14 values matching WoW's signature.
fn stub_get_achievement_info(lua: &Lua, id: Value) -> Result<mlua::MultiValue> {
    let aid = match &id {
        Value::Integer(n) => *n,
        Value::Number(n) => *n as i64,
        _ => return Ok(mlua::MultiValue::from_vec(vec![Value::Nil])),
    };
    Ok(mlua::MultiValue::from_vec(vec![
        Value::Integer(aid),
        Value::String(lua.create_string("Achievement")?),
        Value::Integer(10),
        Value::Boolean(false),
        Value::Integer(1),
        Value::Integer(1),
        Value::Integer(2025),
        Value::String(lua.create_string("Achievement description")?),
        Value::Integer(0),
        Value::Integer(136243),
        Value::String(lua.create_string("")?),
        Value::Boolean(false),
        Value::Boolean(false),
        Value::Nil,
    ]))
}

/// Loot, content-tracking, and achievement telemetry namespace stubs.
pub fn register_tracking_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();

    let cl = lua.create_table()?;
    cl.set("GetLootRollDuration", lua.create_function(|_, _id: Value| Ok(0i32))?)?;
    g.set("C_Loot", cl)?;

    let ct = lua.create_table()?;
    ct.set("GetTrackedIDs", lua.create_function(|lua, _type: Value| lua.create_table())?)?;
    ct.set("IsTracking", lua.create_function(|_, (_type, _id): (Value, Value)| Ok(false))?)?;
    ct.set("GetCollectableSourceTrackingEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("C_ContentTracking", ct)?;

    let at = lua.create_table()?;
    at.set("ShowAchievements", lua.create_function(|_, ()| Ok(()))?)?;
    at.set("LinkAchievementInWhisper", lua.create_function(|_, _id: Value| Ok(()))?)?;
    at.set("LinkAchievementInClub", lua.create_function(|_, _id: Value| Ok(()))?)?;
    g.set("C_AchievementTelemetry", at)?;

    Ok(())
}
