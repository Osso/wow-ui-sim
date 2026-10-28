//! Temporary `C_Reputation` seeded state surface.
//!
//! Reputation data is not backed by a simulator model yet. Keep the seeded
//! faction list explicit as temporary compatibility behavior.

const REPUTATION_STATE_LUA: &str = r#"
if type(C_Reputation) ~= "table" then
    C_Reputation = {}
end

local state = rawget(_G, "__wow_reputation_state")
if type(state) ~= "table" then
    state = {
        selectedFaction = 0,
        watchedFactionID = seedRetailReputationFixture and 2590 or 0,
        sortType = Enum.ReputationSortType and Enum.ReputationSortType.None or 0,
        showLegacy = true,
        factions = seedRetailReputationFixture and {
            { factionID = 0, name = "The War Within", description = "", reaction = 0, standing = 0, bottom = 0, top = 0, isHeader = true, isCollapsed = false, isChild = false, isAccountWide = false, isLegacy = false },
            { factionID = 2590, name = "Council of Dornogal", description = "The governing body of Dornogal.", reaction = 6, standing = 8200, bottom = 0, top = 12000, isHeader = false, isCollapsed = false, isChild = true, isAccountWide = fixtureAccountWide, isLegacy = false },
            { factionID = 2570, name = "Hallowfall Arathi", description = "The Arathi settlers of Hallowfall.", reaction = 7, standing = 4500, bottom = 0, top = 21000, isHeader = false, isCollapsed = false, isChild = true, isAccountWide = fixtureAccountWide, isLegacy = false },
            { factionID = 2600, name = "The Assembly of the Deeps", description = "United denizens of the deep.", reaction = 6, standing = 11000, bottom = 0, top = 12000, isHeader = false, isCollapsed = false, isChild = true, isAccountWide = fixtureAccountWide, isLegacy = false },
            { factionID = 2605, name = "The Severed Threads", description = "A coalition of Nerubian outcasts.", reaction = 5, standing = 4800, bottom = 0, top = 6000, isHeader = false, isCollapsed = false, isChild = true, isAccountWide = fixtureAccountWide, isLegacy = false },
            { factionID = 0, name = "Dragonflight", description = "", reaction = 0, standing = 0, bottom = 0, top = 0, isHeader = true, isCollapsed = false, isChild = false, isAccountWide = false, isLegacy = false },
            { factionID = 2507, name = "Dragonscale Expedition", description = "Explorers of the Dragon Isles.", reaction = 8, standing = 999, bottom = 0, top = 1000, isHeader = false, isCollapsed = false, isChild = true, isAccountWide = fixtureAccountWide, isLegacy = false },
            { factionID = 2510, name = "Valdrakken Accord", description = "The united dragonflights.", reaction = 8, standing = 999, bottom = 0, top = 1000, isHeader = false, isCollapsed = false, isChild = true, isAccountWide = fixtureAccountWide, isLegacy = false },
            { factionID = 0, name = "Classic", description = "", reaction = 0, standing = 0, bottom = 0, top = 0, isHeader = true, isCollapsed = false, isChild = false, isAccountWide = false, isLegacy = true },
            { factionID = 72, name = "Stormwind", description = "The Kingdom of Stormwind.", reaction = 8, standing = 999, bottom = 0, top = 1000, isHeader = false, isCollapsed = false, isChild = true, isAccountWide = false, isLegacy = true },
            { factionID = 47, name = "Ironforge", description = "The Dwarven capital.", reaction = 8, standing = 999, bottom = 0, top = 1000, isHeader = false, isCollapsed = false, isChild = true, isAccountWide = false, isLegacy = true },
        } or {},
    }
    rawset(_G, "__wow_reputation_state", state)
end

local function NormalizeFaction(faction)
    if type(faction) ~= "table" then
        return faction
    end
    faction.currentReactionThreshold = faction.currentReactionThreshold or faction.bottom or 0
    faction.nextReactionThreshold = faction.nextReactionThreshold or faction.top or faction.currentReactionThreshold
    faction.currentStanding = faction.currentStanding or faction.standing or faction.currentReactionThreshold
    if faction.isAccountWide == nil then
        faction.isAccountWide = false
    end
    if faction.isLegacy == nil then
        faction.isLegacy = false
    end
    return faction
end

for _, faction in ipairs(state.factions) do
    NormalizeFaction(faction)
end

local function VisibleFactions()
    local visible = {}
    local hideChildren = false
    for _, faction in ipairs(state.factions) do
        if faction.isHeader then
            hideChildren = faction.isCollapsed == true
            table.insert(visible, faction)
        elseif not hideChildren then
            table.insert(visible, faction)
        end
    end
    return visible
end

local function FindVisible(index)
    return VisibleFactions()[index]
end

local function FindByID(factionID)
    for _, faction in ipairs(state.factions) do
        if not faction.isHeader and faction.factionID == factionID then
            return faction
        end
    end
    return nil
end

local function FindHeader(index)
    local faction = VisibleFactions()[index]
    if faction and faction.isHeader then
        return faction
    end
    return nil
end

C_Reputation.GetNumFactions = function()
    return #VisibleFactions()
end

C_Reputation.GetFactionDataByIndex = function(index)
    return FindVisible(index)
end

C_Reputation.GetFactionDataByID = function(factionID)
    return FindByID(factionID)
end

C_Reputation.GetSelectedFaction = function()
    return state.selectedFaction or 0
end

C_Reputation.SetSelectedFaction = function(index)
    state.selectedFaction = tonumber(index) or 0
end

C_Reputation.GetWatchedFactionData = function()
    local faction = FindByID(state.watchedFactionID) or FindVisible(1)
    if faction == nil then
        return nil
    end
    local info = {}
    for key, value in pairs(faction) do
        info[key] = value
    end
    info.factionID = info.factionID or 0
    info.reaction = info.reaction or info.standing or 0
    info.currentReactionThreshold = info.currentReactionThreshold or 0
    info.nextReactionThreshold = info.nextReactionThreshold or info.topValue or 3000
    info.currentStanding = info.currentStanding or 0
    return info
end

C_Reputation.SetWatchedFactionByIndex = function(index)
    local faction = FindVisible(index)
    state.watchedFactionID = faction and faction.factionID or 0
end

C_Reputation.SetWatchedFactionByID = function(factionID)
    state.watchedFactionID = tonumber(factionID) or 0
end

C_Reputation.CollapseFactionHeader = function(index)
    local header = FindHeader(index)
    if header then
        header.isCollapsed = true
    end
end

C_Reputation.ExpandFactionHeader = function(index)
    local header = FindHeader(index)
    if header then
        header.isCollapsed = false
    end
end

C_Reputation.CollapseAllFactionHeaders = function()
    for _, faction in ipairs(state.factions) do
        if faction.isHeader then
            faction.isCollapsed = true
        end
    end
end

C_Reputation.ExpandAllFactionHeaders = function()
    for _, faction in ipairs(state.factions) do
        if faction.isHeader then
            faction.isCollapsed = false
        end
    end
end

C_Reputation.GetReputationSortType = function()
    return state.sortType
end

C_Reputation.SetReputationSortType = function(sortType)
    state.sortType = tonumber(sortType) or 0
end

C_Reputation.AreLegacyReputationsShown = function()
    return state.showLegacy == true
end

C_Reputation.SetLegacyReputationsShown = function(shown)
    state.showLegacy = shown ~= false
end

C_Reputation.GetGuildFactionData = function()
    return NormalizeFaction({
        factionID = 1168,
        name = "Guild",
        description = "Guild reputation",
        reaction = 8,
        standing = 1000,
        bottom = 0,
        top = 1000,
        isHeader = false,
        isCollapsed = false,
        isChild = false,
    })
end

C_Reputation.IsAccountWideReputation = function(factionID)
    local faction = FindByID(tonumber(factionID) or 0)
    return faction ~= nil and faction.isAccountWide == true
end

if C_Reputation.IsFactionParagonForCurrentPlayer == nil then
    C_Reputation.IsFactionParagonForCurrentPlayer = function()
        return false
    end
end

if C_Reputation.IsFactionParagon == nil then
    C_Reputation.IsFactionParagon = function()
        return false
    end
end

if C_Reputation.IsMajorFaction == nil then
    C_Reputation.IsMajorFaction = function()
        return false
    end
end

if C_Reputation.GetFactionParagonInfo == nil then
    C_Reputation.GetFactionParagonInfo = function()
        return nil
    end
end

C_Reputation.RequestFactionParagonPreloadRewardData = function()
end

C_Reputation.IsFactionActive = function()
    return true
end

C_Reputation.SetFactionActive = function()
end

C_Reputation.ToggleFactionAtWar = function()
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    // Era/Anniversary have no selected Retail faction fixture. Keep existing
    // supplied Lua state intact; only a new state's population is scoped.
    let seed_retail_fixture = !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Era
            | crate::client_profile::ClientProfile::Anniversary
    );
    let fixture_account_wide = crate::c_api::c_reputation::fixture_account_wide_default();
    lua.exec(&format!(
        "local seedRetailReputationFixture = {seed_retail_fixture}\nlocal fixtureAccountWide = {fixture_account_wide}\n{REPUTATION_STATE_LUA}"
    ))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_seeded_reputation_state_and_visibility_controls() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                &r#"
                if C_Reputation.GetNumFactions() ~= 11 then
                    return "bad_initial_count"
                end
                if C_Reputation.GetFactionDataByIndex(2).name ~= "Council of Dornogal" then
                    return "bad_visible_index"
                end
                if C_Reputation.GetFactionDataByID(2590).currentStanding ~= 8200 then
                    return "bad_lookup"
                end
                C_Reputation.SetSelectedFaction(2)
                if C_Reputation.GetSelectedFaction() ~= 2 then
                    return "bad_selected"
                end
                if C_Reputation.GetWatchedFactionData().factionID ~= 2590 then
                    return "bad_default_watch"
                end
                C_Reputation.SetWatchedFactionByID(72)
                if C_Reputation.GetWatchedFactionData().name ~= "Stormwind" then
                    return "bad_watch_id"
                end
                C_Reputation.SetWatchedFactionByIndex(3)
                if C_Reputation.GetWatchedFactionData().factionID ~= 2570 then
                    return "bad_watch_index"
                end
                C_Reputation.CollapseFactionHeader(1)
                if C_Reputation.GetNumFactions() ~= 7 then
                    return "bad_collapse"
                end
                C_Reputation.ExpandFactionHeader(1)
                if C_Reputation.GetNumFactions() ~= 11 then
                    return "bad_expand"
                end
                C_Reputation.CollapseAllFactionHeaders()
                if C_Reputation.GetNumFactions() ~= 3 then
                    return "bad_collapse_all"
                end
                C_Reputation.ExpandAllFactionHeaders()
                if C_Reputation.GetNumFactions() ~= 11 then
                    return "bad_expand_all"
                end
                C_Reputation.SetReputationSortType(4)
                if C_Reputation.GetReputationSortType() ~= 4 then
                    return "bad_sort"
                end
                C_Reputation.SetLegacyReputationsShown(false)
                if C_Reputation.AreLegacyReputationsShown() then
                    return "bad_legacy_flag"
                end
                if C_Reputation.IsAccountWideReputation(2590) ~= EXPECT_ACCOUNT_WIDE or C_Reputation.IsAccountWideReputation(72) then
                    return "bad_account_wide"
                end
                if C_Reputation.GetGuildFactionData().factionID ~= 1168 then
                    return "bad_guild"
                end
                if type(C_Reputation.IsFactionParagonForCurrentPlayer) ~= "function"
                    or type(C_Reputation.IsFactionParagon) ~= "function"
                    or type(C_Reputation.IsMajorFaction) ~= "function"
                    or type(C_Reputation.GetFactionParagonInfo) ~= "function" then
                    return "missing_existing_reputation_methods"
                end
                if not C_Reputation.IsFactionActive(2590) then
                    return "bad_active"
                end
                C_Reputation.RequestFactionParagonPreloadRewardData(2590)
                C_Reputation.SetFactionActive(2590)
                C_Reputation.ToggleFactionAtWar(2590)
                return "ok"
                "#
                .replace(
                    "EXPECT_ACCOUNT_WIDE",
                    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Mists {
                        "true"
                    } else {
                        "false"
                    },
                ),
            )
            .expect("reputation state probe should run");

        assert_eq!(result, "ok");
    }
}
