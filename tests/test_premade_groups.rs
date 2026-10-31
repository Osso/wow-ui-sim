use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().unwrap()
}

#[test]
fn clear_search_cancels_pending_delivery_and_preserves_catalog() {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        return;
    }
    let env = env();
    // GroupFinder/Mainline/LFGList.lua:2778 clears, and :2817 searches.
    env.exec(r#"
        observed = 0
        local f = CreateFrame('Frame')
        f:RegisterEvent('LFG_LIST_SEARCH_RESULTS_RECEIVED')
        f:SetScript('OnEvent', function() observed = observed + 1 end)
        C_LFGList.Search(2)
        local n, ids = C_LFGList.GetSearchResults()
        assert(n == 8 and C_LFGList.HasSearchResultInfo(ids[1]))
        C_LFGList.ClearSearchResults()
        assert(select(1, C_LFGList.GetSearchResults()) == 0)
        assert(not C_LFGList.HasSearchResultInfo(ids[1]))
        assert(C_LFGList.GetSearchResultInfo(ids[1]) == nil)
        assert(C_LFGList.GetSearchResultMemberCounts(ids[1]) == nil)
        assert(select(1, C_LFGList.GetFilteredSearchResults()) == 0)
    "#).unwrap();
    env.process_timers().unwrap();
    env.exec("assert(observed == 0); C_LFGList.Search(2); assert(select(1, C_LFGList.GetSearchResults()) == 8)").unwrap();
    env.process_timers().unwrap();
    env.exec("assert(observed == 1)").unwrap();
}

#[test]
fn supplied_quest_search_filters_results() {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        return;
    }
    let env = env();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state.world.premade_listings[0].metadata.quest_id = Some(987654);
        state.world.premade_listings[1].metadata.quest_id = Some(987655);
    }
    // Retail GroupFinder/Mainline/LFGList.lua:796,2779,2817.
    env.exec(r#"
        C_LFGList.SetSearchToQuestID(987654)
        C_LFGList.Search(2)
        local count, ids = C_LFGList.GetSearchResults()
        assert(count == 1 and ids[1] == 1)
        assert(C_LFGList.GetSearchResultInfo(1).questID == 987654)
        C_LFGList.SetSearchToQuestID(987655)
        C_LFGList.Search(2)
        count, ids = C_LFGList.GetSearchResults()
        assert(count == 1 and ids[1] == 2)
        C_LFGList.ClearSearchTextFields()
        C_LFGList.Search(2)
        assert(select(1, C_LFGList.GetSearchResults()) == 8)
    "#).unwrap();
}

#[test]
fn get_search_result_info_returns_table() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local info = C_LFGList.GetSearchResultInfo(1)
            if type(info) ~= "table" then return "type=" .. type(info) end
            if info.name ~= "+15 Mists chill run" then return "name=" .. tostring(info.name) end
            if info.leaderName ~= "Thrallx" then return "leader=" .. tostring(info.leaderName) end
            if info.numMembers ~= 3 then return "num=" .. tostring(info.numMembers) end
            if info.maxMembers ~= 5 then return "max=" .. tostring(info.maxMembers) end
            if info.activityID ~= 1195 then return "activity=" .. tostring(info.activityID) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetSearchResultInfo: {result}");
}

#[test]
fn get_search_result_info_invalid_returns_nil() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local info = C_LFGList.GetSearchResultInfo(9999)
            return info == nil and "nil" or "not_nil"
            "#,
        )
        .unwrap();
    assert_eq!(result, "nil");
}

#[test]
fn get_search_results_returns_ids() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local numResults, results = C_LFGList.GetSearchResults()
            if numResults ~= 8 then return "count=" .. tostring(numResults) end
            if type(results) ~= "table" then return "type=" .. type(results) end
            if results[1] ~= 1 then return "first=" .. tostring(results[1]) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetSearchResults: {result}");
}

#[test]
fn search_fires_results_event_on_next_tick() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            __lfg_search_results_fired = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("LFG_LIST_SEARCH_RESULTS_RECEIVED")
            f:SetScript("OnEvent", function(self, event)
                if event == "LFG_LIST_SEARCH_RESULTS_RECEIVED" then __lfg_search_results_fired = true end
            end)
            C_LFGList.Search(2)
            return __lfg_search_results_fired and "fired_sync" or "pending"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "pending",
        "Search event timing before tick: {result}"
    );

    env.process_timers().unwrap();
    let result: String = env
        .eval("return __lfg_search_results_fired and 'ok' or 'not_fired'")
        .unwrap();
    assert_eq!(
        result, "ok",
        "Search should fire event on timer tick: {result}"
    );
}

#[test]
fn get_activity_info_table() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local info = C_LFGList.GetActivityInfoTable(1195)
            if type(info) ~= "table" then return "type=" .. type(info) end
            if info.activityID ~= 1195 then return "id=" .. tostring(info.activityID) end
            if info.maxPlayers ~= 5 then return "max=" .. tostring(info.maxPlayers) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetActivityInfoTable: {result}");
}

#[test]
fn admin_add_premade_listing() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local before = select(1, C_LFGList.GetSearchResults())
            A_Admin.AddPremadeListing("Test Group", "Testing", 1195, 2, 5)
            assert(select(1, C_LFGList.GetSearchResults()) == before)
            C_LFGList.Search(2)
            local after = select(1, C_LFGList.GetSearchResults())
            if after ~= before + 1 then return "count=" .. after end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "AddPremadeListing: {result}");
}

#[test]
fn admin_update_premade_listing() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            A_Admin.UpdatePremadeListing(1, "numMembers", 5)
            local info = C_LFGList.GetSearchResultInfo(1)
            if info.numMembers ~= 5 then return "num=" .. tostring(info.numMembers) end
            A_Admin.UpdatePremadeListing(1, "isDelisted", true)
            local info2 = C_LFGList.GetSearchResultInfo(1)
            if info2.isDelisted ~= true then return "delisted=" .. tostring(info2.isDelisted) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "UpdatePremadeListing: {result}");
}

#[test]
fn admin_clear_premade_listings() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            A_Admin.ClearPremadeListings()
            C_LFGList.Search(2)
            local count = select(1, C_LFGList.GetSearchResults())
            return tostring(count)
            "#,
        )
        .unwrap();
    assert_eq!(result, "0");
}

#[test]
fn get_filtered_search_results_mirrors_get_search_results() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local total, results = C_LFGList.GetFilteredSearchResults()
            if total ~= 8 then return "total=" .. tostring(total) end
            if type(results) ~= "table" or #results ~= 8 then
                return "results=" .. tostring(results and #results)
            end
            if results[1] ~= 1 or results[8] ~= 8 then
                return "ids=" .. tostring(results[1]) .. "," .. tostring(results[8])
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetFilteredSearchResults: {result}");
}

#[test]
fn search_result_info_has_full_blizzard_schema() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local info = C_LFGList.GetSearchResultInfo(1)
            if type(info.activityIDs) ~= "table" or info.activityIDs[1] ~= 1195 then
                return "activityIDs=" .. tostring(info.activityIDs and info.activityIDs[1])
            end
            if type(info.voiceChat) ~= "string" then
                return "voiceChat type=" .. type(info.voiceChat)
            end
            if type(info.partyGUID) ~= "string" or info.partyGUID == "" then
                return "partyGUID=" .. tostring(info.partyGUID)
            end
            if info.numBNetFriends ~= 0 or info.numCharFriends ~= 0 or info.numGuildMates ~= 0 then
                return "friend counts wrong"
            end
            if type(info.generalPlaystyle) ~= "number" then
                return "generalPlaystyle=" .. type(info.generalPlaystyle)
            end
            if type(info.crossFactionListing) ~= "boolean" then
                return "crossFactionListing=" .. type(info.crossFactionListing)
            end
            if type(info.leaderFactionGroup) ~= "number" then
                return "leaderFactionGroup=" .. type(info.leaderFactionGroup)
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "schema: {result}");
}

#[test]
fn search_result_leader_factions_use_player_faction_group_indexes() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local _, results = C_LFGList.GetSearchResults()
            local valid = { [0] = true, [1] = true }
            for _, resultID in ipairs(results) do
                local info = C_LFGList.GetSearchResultInfo(resultID)
                if not valid[info.leaderFactionGroup] then
                    return "bad faction " .. tostring(resultID) .. "=" .. tostring(info.leaderFactionGroup)
                end
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "leader factions: {result}");
}

#[test]
fn get_playstyle_string_returns_text_for_seeded_general_playstyles() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local _, results = C_LFGList.GetSearchResults()
            for _, resultID in ipairs(results) do
                local info = C_LFGList.GetSearchResultInfo(resultID)
                local activityInfo = C_LFGList.GetActivityInfoTable(info.activityIDs[1])
                local text = C_LFGList.GetPlaystyleString(
                    Enum.LFGEntryPlaystyle.None,
                    info.generalPlaystyle,
                    activityInfo
                )
                if type(text) ~= "string" then
                    return "type " .. tostring(resultID) .. "=" .. type(text)
                end
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "playstyle string: {result}");
}

#[test]
fn get_search_result_member_counts_returns_role_breakdown() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local mc = C_LFGList.GetSearchResultMemberCounts(1)
            if type(mc) ~= "table" then return "type=" .. type(mc) end
            -- First seeded listing is 1 tank, 1 healer, 1 damager
            if mc.TANK ~= 1 or mc.HEALER ~= 1 or mc.DAMAGER ~= 1 or mc.NOROLE ~= 0 then
                return string.format("counts T=%s H=%s D=%s N=%s",
                    tostring(mc.TANK), tostring(mc.HEALER),
                    tostring(mc.DAMAGER), tostring(mc.NOROLE))
            end
            if type(mc.classesByRole) ~= "table" then
                return "classesByRole=" .. type(mc.classesByRole)
            end
            if type(mc.leaversByClass) ~= "table" then
                return "leaversByClass=" .. type(mc.leaversByClass)
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetSearchResultMemberCounts: {result}");
}

#[test]
fn get_application_info_for_unsubmitted_returns_none() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local id, status, pending, dur, role = C_LFGList.GetApplicationInfo(1)
            -- No application submitted yet — must return canonical "none",
            -- not nil, so the panel's `appStatus ~= "none"` check works.
            if status ~= "none" then return "status=" .. tostring(status) end
            if pending ~= nil then return "pending=" .. tostring(pending) end
            if dur ~= 0 then return "dur=" .. tostring(dur) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetApplicationInfo: {result}");
}

#[test]
fn apply_to_group_creates_application_and_fires_event() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            local fired = nil
            local f = CreateFrame("Frame")
            f:RegisterEvent("LFG_LIST_APPLICATION_STATUS_UPDATED")
            f:SetScript("OnEvent", function(self, event, rid, newStatus, oldStatus, name)
                if event == "LFG_LIST_APPLICATION_STATUS_UPDATED" then
                    fired = { rid = rid, newStatus = newStatus, oldStatus = oldStatus, name = name }
                end
            end)

            C_LFGList.ApplyToGroup(1, false, true, false)
            local apps = C_LFGList.GetApplications()
            if #apps ~= 1 or apps[1] ~= 1 then return "apps=" .. tostring(#apps) end

            local appID, status, pending, dur, role = C_LFGList.GetApplicationInfo(1)
            if status ~= "applied" then return "status=" .. tostring(status) end
            if role ~= "HEALER" then return "role=" .. tostring(role) end
            if dur ~= 120 then return "dur=" .. tostring(dur) end

            if not fired then return "event_not_fired" end
            if fired.rid ~= 1 then return "fired.rid=" .. tostring(fired.rid) end
            if fired.newStatus ~= "applied" then return "fired.new=" .. tostring(fired.newStatus) end
            if fired.oldStatus ~= "none" then return "fired.old=" .. tostring(fired.oldStatus) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "ApplyToGroup: {result}");
}

#[test]
fn apply_to_group_is_idempotent() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            C_LFGList.ApplyToGroup(2, true, false, false)
            C_LFGList.ApplyToGroup(2, false, true, false)
            local apps = C_LFGList.GetApplications()
            if #apps ~= 1 then return "apps=" .. tostring(#apps) end
            local _, _, _, _, role = C_LFGList.GetApplicationInfo(2)
            -- Role is whichever was set on first apply; second call is a
            -- no-op so role must still be the original "TANK".
            if role ~= "TANK" then return "role=" .. tostring(role) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "idempotent apply: {result}");
}

#[test]
fn cancel_application_removes_entry() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            C_LFGList.ApplyToGroup(3, false, false, true)
            if #C_LFGList.GetApplications() ~= 1 then return "before=" .. #C_LFGList.GetApplications() end
            C_LFGList.CancelApplication(3)
            if #C_LFGList.GetApplications() ~= 0 then return "after=" .. #C_LFGList.GetApplications() end
            local _, status = C_LFGList.GetApplicationInfo(3)
            if status ~= "none" then return "status=" .. tostring(status) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "CancelApplication: {result}");
}

#[test]
fn get_advanced_filter_default_is_permissive() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local f = C_LFGList.GetAdvancedFilter()
            if type(f) ~= "table" then return "type=" .. type(f) end
            -- All booleans must be false and minimumRating zero, otherwise
            -- the panel filters out every search result on first open.
            for _, k in ipairs({"needsTank","needsHealer","needsDamage","needsMyClass",
                                "hasTank","hasHealer","difficultyNormal","difficultyHeroic",
                                "difficultyMythic","difficultyMythicPlus",
                                "generalPlaystyle1","generalPlaystyle2",
                                "generalPlaystyle3","generalPlaystyle4"}) do
                if f[k] ~= false then return k .. "=" .. tostring(f[k]) end
            end
            if f.minimumRating ~= 0 then return "rating=" .. tostring(f.minimumRating) end
            if type(f.activities) ~= "table" or #f.activities ~= 0 then
                return "activities=" .. tostring(f.activities and #f.activities)
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetAdvancedFilter: {result}");
}
