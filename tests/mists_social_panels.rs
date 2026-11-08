#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::path::PathBuf;
use std::process::Command;

fn wow_sim_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_wow-sim")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join("wow-sim")
        })
}

#[test]
fn mists_social_panels_support_friends_who_guild_and_communities() {
    let output = Command::new("timeout")
        .arg("90")
        .arg(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleFriendsFrame(1)
            if not (FriendsFrame and FriendsFrame:IsShown()) then
                error("FriendsFrame did not open")
            end

            FriendsFrame_ShowSubFrame("FriendsListFrame")
            FriendsFrame_Update()
            if C_FriendList.GetNumFriends() < 1 then
                error("friend list has no seeded rows")
            end

            FriendsFrame_ShowSubFrame("WhoFrame")
            WhoList_Update()
            if not WhoFrame:IsShown() then
                error("WhoFrame did not show")
            end

            local shownWhos, totalWhos = C_FriendList.GetNumWhoResults()
            if shownWhos < 1 or totalWhos < shownWhos then
                error("who counts invalid")
            end

            local whoInfo = C_FriendList.GetWhoInfo(1)
            if not whoInfo or not whoInfo.fullName then
                error("who row missing")
            end

            PanelTemplates_SetTab(FriendsFrame, FRIEND_TAB_GUILD)
            FriendsFrame_Update()
            if not GuildFrame:IsShown() then
                error("GuildFrame did not show")
            end
            if GetGuildRosterSize() < 1 then
                error("guild roster has no rows")
            end

            local ok, reason = LoadAddOn("Blizzard_Communities")
            if ok == false then
                error("Blizzard_Communities failed to load: " .. tostring(reason))
            end
            if not CommunitiesFrame then
                error("CommunitiesFrame missing")
            end

            CommunitiesFrame:Show()
            CommunitiesFrame.CommunitiesList:Update()
            local guildFinderEntry
            CommunitiesFrame.CommunitiesList.ScrollBox:ForEachFrame(function(frame, elementData)
                if elementData and elementData.setGuildFinder then
                    guildFinderEntry = frame
                end
            end)
            if not guildFinderEntry then
                error("guild finder list entry did not render")
            end
            guildFinderEntry:Click("LeftButton")
            if not CommunitiesFrame.GuildFinderFrame:IsShown()
                or CommunitiesFrame:GetDisplayMode() ~= COMMUNITIES_FRAME_DISPLAY_MODES.GUILD_FINDER
                or CommunitiesFrame.GuildFinderFrame.isGuildType ~= true then
                error("guild finder list entry click did not open guild finder mode")
            end

            CommunitiesFrame:SetDisplayMode(COMMUNITIES_FRAME_DISPLAY_MODES.CHAT)
            local toggled, toggleReason = ToggleGuildFinder()
            if toggled == false then
                error("ToggleGuildFinder failed: " .. tostring(toggleReason))
            end
            if not CommunitiesFrame:IsShown()
                or not CommunitiesFrame.GuildFinderFrame:IsShown()
                or CommunitiesFrame:GetDisplayMode() ~= COMMUNITIES_FRAME_DISPLAY_MODES.GUILD_FINDER
                or CommunitiesFrame.GuildFinderFrame.isGuildType ~= true then
                error("ToggleGuildFinder did not open guild finder mode")
            end

            local guildFinder = CommunitiesFrame.GuildFinderFrame
            if not guildFinder then
                error("GuildFinderFrame missing")
            end

            guildFinder:Show()
            guildFinder.isGuildType = true
            guildFinder:OnEvent("CLUB_FINDER_PLAYER_PENDING_LIST_RECIEVED", Enum.ClubFinderRequestType.Guild)
            guildFinder:UpdateType()
            guildFinder:OnEvent("CLUB_FINDER_CLUB_LIST_RETURNED", Enum.ClubFinderRequestType.Guild)

            if not guildFinder.GuildCards:IsShown() then
                error("guild finder search cards did not show")
            end
            if #guildFinder.GuildCards.CardList < 1 then
                error("guild finder search has no rows")
            end
            local guildCard = guildFinder.GuildCards.Cards and guildFinder.GuildCards.Cards[1]
            if not (guildCard and guildCard:IsShown() and guildCard.cardInfo) then
                error("guild finder first search row did not render")
            end
            if guildCard.Name:GetText() ~= guildCard.cardInfo.name then
                error("guild finder search row name did not bind")
            end
            if not guildCard.RequestJoin:IsShown() then
                error("guild finder search row did not expose request state")
            end

            guildFinder.ClubFinderPendingTab:OnClick()
            if not guildFinder.PendingGuildCards:IsShown() or guildFinder.GuildCards:IsShown() then
                error("guild finder pending tab did not switch visible card set")
            end
            if #guildFinder.PendingGuildCards.CardList < 1 then
                error("guild finder pending list has no rows")
            end
            local pendingGuildCard = guildFinder.PendingGuildCards.Cards and guildFinder.PendingGuildCards.Cards[1]
            if not (pendingGuildCard and pendingGuildCard:IsShown()) then
                error("guild finder first pending row did not render")
            end
            if not pendingGuildCard.RequestStatus:IsShown() then
                error("guild finder pending row did not show status")
            end

            guildFinder.isGuildType = false
            guildFinder.selectedTab = 1
            guildFinder:OnEvent("CLUB_FINDER_PLAYER_PENDING_LIST_RECIEVED", Enum.ClubFinderRequestType.Community)
            guildFinder:UpdateType()
            guildFinder:OnEvent("CLUB_FINDER_CLUB_LIST_RETURNED", Enum.ClubFinderRequestType.Community)

            if not guildFinder.CommunityCards:IsShown() or guildFinder.GuildCards:IsShown() then
                error("community finder search mode did not switch visible card set")
            end
            if #guildFinder.CommunityCards.CardList < 1 then
                error("community finder search has no rows")
            end
            if guildFinder.CommunityCards.showingCards ~= true then
                error("community finder search rows did not render")
            end

            guildFinder.ClubFinderPendingTab:OnClick()
            if not guildFinder.PendingCommunityCards:IsShown() or guildFinder.CommunityCards:IsShown() then
                error("community finder pending tab did not switch visible card set")
            end
            if #guildFinder.PendingCommunityCards.CardList < 1 then
                error("community finder pending list has no rows")
            end
            if guildFinder.PendingCommunityCards.showingCards ~= true then
                error("community finder pending rows did not render")
            end
            "#,
            "lua-errors",
        ])
        .output()
        .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("social panel flow", &stdout, &stderr);
}

