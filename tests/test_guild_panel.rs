use crate::common;

use std::path::PathBuf;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::GuildMember;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Eager guild/Communities publishers in dependency order.
const GUILD_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    "Blizzard_Menu",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLBase",
    "Blizzard_FrameEffects",
    "Blizzard_LoadLocale",
    "Blizzard_Fonts_Shared",
    "Blizzard_HelpPlate",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_GameMenuEsc",
    "Blizzard_UIParentUtil",
    "Blizzard_Game",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_GuildControlUI",
    "Blizzard_TimerunningUtil",
    "Blizzard_ChatFrameBase",
    "Blizzard_ChatFrame",
    "Blizzard_Communities",
];

fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    let ui = blizzard_ui_dir();
    for addon_name in GUILD_ADDONS {
        common::load_required_blizzard_addon(&env, &ui, addon_name);
    }

    env.apply_post_load_workarounds();
    install_dropdown_test_helpers(&env);
    fire_startup_events(&env);
    env
}

fn install_dropdown_test_helpers(env: &WowLuaEnv) {
    env.exec(
        r#"
        function GuildDropdownTestVisibleLabels(dropdown)
            local labels = {}
            if dropdown.menu then
                for _, child in ipairs({ dropdown.menu:GetChildren() }) do
                    if child:GetObjectType() == "Button" and child:IsShown() then
                        local text = child:GetText()
                        if (text == nil or text == "") and child.fontString then
                            text = child.fontString:GetText()
                        end
                        if (text == nil or text == "") and child.Text then
                            text = child.Text:GetText()
                        end
                        if text ~= nil and text ~= "" then
                            labels[#labels + 1] = text
                        end
                    end
                end
            end
            return labels
        end

        function GuildDropdownTestLabels(dropdown)
            dropdown:OpenMenu()
            return GuildDropdownTestVisibleLabels(dropdown)
        end
        "#,
    )
    .expect("guild dropdown test helpers should install");
}

fn load_guild_control_ui(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    common::load_required_blizzard_addon(env, &ui, "Blizzard_GuildControlUI");
}

fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

#[test]
fn guild_panel_opens_without_unavailable_error() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            -- Capture error messages
            local errors = {}
            local origAddMessage = UIErrorsFrame.AddMessage
            UIErrorsFrame.AddMessage = function(self, msg, ...)
                table.insert(errors, msg)
                if origAddMessage then pcall(origAddMessage, self, msg, ...) end
            end

            -- Try to open guild frame via ToggleGuildFrame
            local ok, err = pcall(ToggleGuildFrame)
            if not ok then return "error: " .. tostring(err) end

            -- Check for the unavailable message
            for _, msg in ipairs(errors) do
                if msg and msg:find("unavailable") then
                    return "unavailable_error: " .. msg
                end
            end
            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "Guild panel should open without 'unavailable' error: {result}");
    }
}

#[test]
fn bn_connected_returns_true() {
    let env = WowLuaEnv::new().unwrap();
    let connected: bool = env.eval("return BNConnected()").unwrap();
    assert!(
        connected,
        "BNConnected should return true for Communities to work"
    );
}

#[test]
fn c_club_is_enabled() {
    let env = WowLuaEnv::new().unwrap();
    let enabled: bool = env.eval("return C_Club.IsEnabled()").unwrap();
    assert!(enabled, "C_Club.IsEnabled should return true");
}

#[test]
fn c_club_returns_guild() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
        local clubs = C_Club.GetSubscribedClubs()
        if #clubs == 0 then return "no_clubs" end
        local club = clubs[1]
        if club.clubType ~= 2 then return "type=" .. tostring(club.clubType) end
        if club.name ~= "Heroes of Azeroth" then return "name=" .. tostring(club.name) end
        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "C_Club should return guild: {result}");
}

#[test]
fn guild_member_rank_dropdown_generates_rank_options() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            A_Admin.SetGuildRanks({
                { name = "Guild Leader", flags = {} },
                { name = "Officer", flags = {} },
                { name = "Member", flags = {} },
            })

            function CanGuildPromote() return true end
            function CanGuildDemote() return true end
            C_GuildInfo.IsGuildRankAssignmentAllowed = function() return true end
            C_GuildInfo.SetGuildRankOrder = function() end

            local dropdown = CreateFrame("DropdownButton", "GuildRankDropdownProbe", UIParent)
            Mixin(dropdown, DropdownButtonMixin)

            local detail = {
                RankDropdown = dropdown,
                GetClubId = function() return "guild-0" end,
                GetMemberInfo = function()
                    return { guid = "member-2", guildRankOrder = 2 }
                end,
            }
            setmetatable(detail, { __index = CommunitiesGuildMemberDetailMixin })

            detail:SetupRankDropdown()
            dropdown:GenerateMenu()
            if not dropdown:HasElements() then
                return "missing_elements"
            end

            local labels = GuildDropdownTestLabels(dropdown)
            return table.concat(labels, ",")
        "#).unwrap();
        assert_eq!(
            result,
            "Officer,Member",
            "rank dropdown should show visible assignable guild ranks: {result}"
        );
    }
}

#[test]
fn communities_member_detail_rank_dropdown_shows_rank_rows() {
    test_timeout! {
        let env = setup_env();
        env.state().borrow_mut().world.guild_members = vec![
            GuildMember {
                name: "Uther".to_string(),
                rank_index: 1,
                online: true,
            },
            GuildMember {
                name: "Jaina".to_string(),
                rank_index: 2,
                online: true,
            },
        ];
        let result: String = env.eval(r#"
            A_Admin.SetGuildRanks({
                { name = "Guild Leader", flags = {} },
                { name = "Officer", flags = {} },
                { name = "Member", flags = {} },
            })

            function CanGuildPromote() return true end
            function CanGuildDemote() return true end
            C_GuildInfo.IsGuildRankAssignmentAllowed = function() return true end
            C_GuildInfo.SetGuildRankOrder = function() end

            local frame = CommunitiesFrame and CommunitiesFrame.GuildMemberDetailFrame
            if frame == nil then
                return "missing_detail_frame"
            end

            local memberInfo = C_Club.GetMemberInfo("guild-0", 2)
            frame:DisplayMember("guild-0", memberInfo)
            frame:SetupRankDropdown()

            local dropdown = frame.RankDropdown
            if dropdown == nil then
                return "missing_rank_dropdown"
            end
            if not dropdown:IsShown() then
                return "rank_dropdown_hidden"
            end

            dropdown:GenerateMenu()
            if not dropdown:HasElements() then
                return "missing_elements"
            end

            local labels = GuildDropdownTestLabels(dropdown)
            if #labels == 0 then
                return "empty_rank_frames"
            end

            local closedText = dropdown.Text and dropdown.Text:GetText() or dropdown:GetText()
            if closedText == nil or closedText == "" then
                return "empty_closed_text:frames=" .. table.concat(labels, ",")
            end
            return "ok:" .. closedText .. ":" .. table.concat(labels, ",")
        "#).unwrap();
        assert_eq!(
            result,
            "ok:Officer:Officer,Member",
            "member detail rank dropdown should show rank rows, not the guild selector: {result}"
        );
    }
}

#[test]
fn guild_control_rank_settings_dropdown_shows_rank_rows() {
    test_timeout! {
        let env = setup_env();
        load_guild_control_ui(&env);
        let result: String = env.eval(r#"
            A_Admin.SetGuildRanks({
                { name = "Guild Leader", flags = {} },
                { name = "Officer", flags = {} },
                { name = "Member", flags = {} },
            })

            local dropdown = GuildControlUIRankSettingsFrame
                and GuildControlUIRankSettingsFrame.dropdown
            if dropdown == nil then
                return "missing_dropdown"
            end

            dropdown:GenerateMenu()
            if not dropdown:HasElements() then
                return "missing_elements"
            end
            local labels = GuildDropdownTestLabels(dropdown)
            local textWidth = dropdown.Text and dropdown.Text:GetWidth() or 0
            if textWidth <= 0 then
                return "zero_text_width:" .. tostring(textWidth)
            end
            return (dropdown:GetText() or "") .. "/" .. (dropdown.Text and dropdown.Text:GetText() or "") .. "/" .. tostring(textWidth) .. "|" .. table.concat(labels, ",")
        "#).unwrap();
        assert!(result.starts_with("Officer/Officer/"), "guild control rank dropdown should show selected rank and visible rank rows: {result}");
        assert!(result.ends_with("|Officer,Member"), "guild control rank dropdown should materialize visible rank rows: {result}");
    }
}

#[test]
fn guild_control_tab_dropdown_shows_initial_selection() {
    test_timeout! {
        let env = setup_env();
        load_guild_control_ui(&env);
        let result: String = env.eval(r#"
            local dropdown = GuildControlUI and GuildControlUI.dropdown
            if dropdown == nil then
                return "missing_dropdown"
            end
            return (dropdown:GetText() or "") .. "/" .. (dropdown.Text and dropdown.Text:GetText() or "")
        "#).unwrap();
        assert_eq!(result, "Guild Ranks/Guild Ranks", "guild control tab dropdown should show its initial selected tab: {result}");
    }
}

#[test]
fn guild_control_rank_dropdown_omits_unassignable_leader_rank() {
    test_timeout! {
        let env = setup_env();
        load_guild_control_ui(&env);
        let result: String = env.eval(r#"
            local dropdown = GuildControlUIRankSettingsFrame
                and GuildControlUIRankSettingsFrame.dropdown
            if dropdown == nil then
                return "missing_dropdown"
            end
            GuildControlSetRank(1)
            GuildControlUI.currentRank = 1
            dropdown:GenerateMenu()
            if not dropdown:HasElements() then
                return "missing_elements"
            end
            return table.concat(GuildDropdownTestLabels(dropdown), ",")
        "#).unwrap();
        assert_eq!(
            result,
            "Officer,Member",
            "rank dropdown should omit the unassignable guild-leader rank: {result}"
        );
    }
}

#[test]
fn guild_dropdown_descriptions_have_elements() {
    test_timeout! {
        let env = setup_env();
        load_guild_control_ui(&env);
        let result: String = env.eval(r#"
            local checks = {
                { "guild_control_tabs", GuildControlUI and GuildControlUI.dropdown },
                { "rank_settings", GuildControlUIRankSettingsFrame and GuildControlUIRankSettingsFrame.dropdown },
                { "rank_bank", GuildControlUIRankBankFrame and GuildControlUIRankBankFrame.dropdown },
            }

            local failures = {}
            for _, check in ipairs(checks) do
                local name, dropdown = check[1], check[2]
                if dropdown == nil then
                    failures[#failures + 1] = name .. ":missing_dropdown"
                else
                    dropdown:GenerateMenu()
                    if not dropdown:HasElements() then
                        failures[#failures + 1] = name .. ":empty"
                    end
                end
            end

            if #failures > 0 then
                return "fail:" .. table.concat(failures, ";")
            end
            return "ok"
        "#).unwrap();
        assert_eq!(
            result, "ok",
            "guild dropdown descriptions must expose elements: {result}"
        );
    }
}

#[test]
fn guild_dropdown_materialized_buttons_have_labels() {
    test_timeout! {
        let env = setup_env();
        load_guild_control_ui(&env);
        let result: String = env.eval(r#"
            local checks = {
                { "guild_control_tabs", GuildControlUI and GuildControlUI.dropdown },
                { "rank_settings", GuildControlUIRankSettingsFrame and GuildControlUIRankSettingsFrame.dropdown },
                { "rank_bank", GuildControlUIRankBankFrame and GuildControlUIRankBankFrame.dropdown },
            }

            local failures = {}
            local summaries = {}
            for _, check in ipairs(checks) do
                local name, dropdown = check[1], check[2]
                if dropdown == nil then
                    failures[#failures + 1] = name .. ":missing_dropdown"
                else
                    local labels = GuildDropdownTestLabels(dropdown)
                    if #labels == 0 then
                        failures[#failures + 1] = name .. ":empty_buttons"
                    else
                        summaries[#summaries + 1] = name .. "=" .. table.concat(labels, ",")
                    end
                end
            end

            if #failures > 0 then
                return "fail:" .. table.concat(failures, ";") .. "|buttons:" .. table.concat(summaries, ";")
            end
            return "ok:" .. table.concat(summaries, ";")
        "#).unwrap();
        assert!(
            result.starts_with("ok:"),
            "guild dropdown materialized buttons must have labels: {result}"
        );
    }
}

#[test]
fn communities_list_dropdown_materializes_subscribed_club_rows() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            local dropdown = CommunitiesFrame and CommunitiesFrame.CommunitiesListDropdown
            if dropdown == nil then
                return "missing_dropdown"
            end

            dropdown:SetupMenu()
            return table.concat(GuildDropdownTestLabels(dropdown), ",")
        "#).unwrap();
        assert_eq!(
            result,
            "Heroes of Azeroth",
            "communities list dropdown should materialize the subscribed guild row: {result}"
        );
    }
}

#[test]
fn communities_stream_dropdown_shows_guild_officer_and_notification_settings() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            local frame = CommunitiesFrame
            if frame == nil then
                return "missing_communities_frame"
            end
            local dropdown = frame.StreamDropdown
            if dropdown == nil then
                return "missing_stream_dropdown"
            end

            frame.selectedClubId = "guild-0"
            frame.selectedClubInfo = C_Club.GetClubInfo("guild-0")
            frame.privilegesForClub["guild-0"] = {
                canCreateStream = false,
                canDestroyStream = false,
            }
            frame.selectedStreamForClub["guild-0"] = C_Club.GetStreamInfo("guild-0", 1)
            dropdown:SetupMenu()
            local labels = GuildDropdownTestLabels(dropdown)

            if #labels == 0 then
                return "empty_stream_frames"
            end
            return "ok:" .. table.concat(labels, "|")
        "#).unwrap();
        assert!(
            result.contains("Guild")
                && result.contains("Officer")
                && result.contains("Notification Settings"),
            "stream dropdown should show guild, officer, and notification settings rows: {result}"
        );
    }
}

#[test]
fn communities_stream_dropdown_click_opens_menu() {
    test_timeout! {
        let env = setup_env();
        env.exec(r#"
            local frame = CommunitiesFrame
            frame.selectedClubId = "guild-0"
            frame.selectedClubInfo = C_Club.GetClubInfo("guild-0")
            frame.privilegesForClub["guild-0"] = {
                canCreateStream = false,
                canDestroyStream = false,
            }
            frame.selectedStreamForClub["guild-0"] = C_Club.GetStreamInfo("guild-0", 1)
            frame.StreamDropdown:SetupMenu()
        "#).unwrap();

        let dropdown_id = {
            let state = env.state().borrow();
            let communities_id = state
                .widgets
                .get_id_by_name("CommunitiesFrame")
                .expect("CommunitiesFrame should exist");
            state
                .widgets
                .get(communities_id)
                .and_then(|frame| frame.children_keys.get("StreamDropdown"))
                .copied()
                .expect("CommunitiesFrame.StreamDropdown should exist")
        };
        let left_button = env.lua_string("LeftButton");
        env.fire_script_handler(dropdown_id, "OnMouseDown", vec![left_button])
            .expect("stream dropdown OnMouseDown should dispatch");

        let result: String = env.eval(r#"
            local dropdown = CommunitiesFrame.StreamDropdown
            if not dropdown:IsMenuOpen() then
                return "menu_closed"
            end
            if dropdown.menu == nil then
                return "missing_menu"
            end
            local labels = GuildDropdownTestVisibleLabels(dropdown)
            if #labels == 0 then
                return "empty_stream_buttons"
            end
            return "ok:" .. table.concat(labels, "|")
        "#).unwrap();
        assert!(
            result.contains("Guild")
                && result.contains("Officer")
                && result.contains("Notification Settings"),
            "clicking stream dropdown should open visible menu rows: {result}"
        );
    }
}

#[test]
fn guild_info_panel_populates_motd_details_and_challenges() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            if CommunitiesFrame == nil then return "missing_communities" end
            local details = CommunitiesFrame.GuildDetailsFrame
            if details == nil then return "missing_guild_details" end
            local frame = details.Info
            if frame == nil then return "missing_info_frame" end

            -- Officer rights so the edit buttons reveal themselves.
            A_Admin.SetGuildIsOfficer(true)
            frame:Show()
            CommunitiesGuildInfoFrame_OnShow(frame)
            -- OnShow only requests challenge data; drive the update directly so
            -- the row labels/counts populate without waiting on the event pump.
            CommunitiesGuildInfoFrame_UpdateChallenges(frame)

            local motd = frame.MOTDScrollFrame.MOTD:GetText() or ""
            local detailsText = frame.DetailsFrame:GetScrollChild().Details:GetText() or ""

            local challengeRows = {}
            for i, row in ipairs(frame.Challenges) do
                if row:IsShown() then
                    local label = (row.label and row.label:GetText()) or ""
                    local countShown = row.count and row.count:IsShown()
                    local count = (countShown and row.count:GetText()) or ""
                    local checkShown = row.check and row.check:IsShown()
                    table.insert(challengeRows, label .. "=" .. count .. (checkShown and "[done]" or ""))
                end
            end

            local editMotd = frame.EditMOTDButton:IsShown() and "yes" or "no"
            local editDetails = frame.EditDetailsButton:IsShown() and "yes" or "no"

            return table.concat({
                "motd:" .. motd,
                "details:" .. detailsText,
                "rows:" .. table.concat(challengeRows, "|"),
                "editMOTD:" .. editMotd,
                "editDetails:" .. editDetails,
            }, ";")
        "#).unwrap();

        assert!(
            result.contains("motd:Raid invites tonight at 20:00 server."),
            "MOTD should be populated from world.guild_motd: {result}"
        );
        assert!(
            result.contains("details:Mythic-focused guild recruiting"),
            "Details should be populated from world.guild_info_text: {result}"
        );
        // GUILD_CHALLENGE_ORDER = {1, 4, 2, 3} — type 1 Dungeon, type 4 (no
        // global string), type 2 Raid (1/1 → check), type 3 Rated BG.
        assert!(
            result.contains("Dungeon=5 / 7"),
            "Dungeon challenge row should show 5/7 progress: {result}"
        );
        assert!(
            result.contains("Raid=[done]"),
            "Raid challenge (1/1) should display the completed check: {result}"
        );
        assert!(
            result.contains("Rated Battleground=1 / 3"),
            "RBG challenge row should show 1/3 progress: {result}"
        );
        assert!(
            result.contains("editMOTD:yes;editDetails:yes"),
            "Officer edit buttons should be shown when guild_is_officer: {result}"
        );
    }
}

#[test]
fn c_guild_info_set_motd_round_trips_through_world_state() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let result: String = env
        .eval(
            r#"
        local original = C_GuildInfo.GetMOTD()
        C_GuildInfo.SetMOTD("Mythic raid Friday 21:00 server.")
        local updated = C_GuildInfo.GetMOTD()
        local roster = GetGuildRosterMOTD()
        return original .. "|" .. updated .. "|" .. roster
    "#,
        )
        .expect("SetMOTD should round-trip via world state");

    let parts: Vec<&str> = result.split('|').collect();
    assert_eq!(parts.len(), 3, "expected 3 parts, got {result}");
    assert_eq!(
        parts[0], "Raid invites tonight at 20:00 server. Repairs are on for progression.",
        "initial MOTD should be seeded"
    );
    assert_eq!(parts[1], "Mythic raid Friday 21:00 server.");
    assert_eq!(
        parts[2], parts[1],
        "GetGuildRosterMOTD must return the same world.guild_motd as C_GuildInfo.GetMOTD"
    );
}

#[test]
fn admin_set_guild_challenges_replaces_world_state_rows() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.eval::<()>(
        r#"
        A_Admin.SetGuildChallenges({
            { challengeType = 1, current = 3, max = 7, gold = 100, maxGold = 700 },
            { challengeType = 5, current = 1, max = 1, gold = 999, maxGold = 999 },
        })
    "#,
    )
    .expect("SetGuildChallenges should accept a list");

    let count: f64 = env.eval("return GetNumGuildChallenges()").unwrap();
    assert_eq!(count, 2.0);

    let dungeon: String = env
        .eval(
            r#"
        local id, current, max, gold, maxGold = GetGuildChallengeInfo(1)
        return tostring(id) .. ":" .. tostring(current) .. "/" .. tostring(max) .. " " .. tostring(gold) .. "/" .. tostring(maxGold)
    "#,
        )
        .unwrap();
    assert_eq!(dungeon, "1:3/7 100/700");

    let mythic: String = env
        .eval(
            r#"
        local id, current, max, gold, maxGold = GetGuildChallengeInfo(5)
        return tostring(id) .. ":" .. tostring(current) .. "/" .. tostring(max) .. " " .. tostring(gold) .. "/" .. tostring(maxGold)
    "#,
        )
        .unwrap();
    assert_eq!(mythic, "5:1/1 999/999");

    let missing: bool = env.eval("return GetGuildChallengeInfo(2) == nil").unwrap();
    assert!(missing, "challenge type 2 was not seeded, expected nil");
}
