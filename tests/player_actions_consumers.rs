#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! Original consumers: SlashCommands.lua (/follow), ZoneText.lua AutoFollowStatus, GuildInviteFrame and the DEATH /
//! RESURRECT dialogs of Blizzard_StaticPopup_Game.

use crate::common::blizzard_addon_harness as harness;
use wow_ui_sim::lua_api::WowLuaEnv;

const FOLLOW_AND_DISMOUNT: &str = r#"
    A_Admin.SetPartySize(2)
    local leader = UnitName("party1")
    local begins, ends, displays = {}, 0, 0
    local watcher = CreateFrame("Frame")
    watcher:RegisterEvent("AUTOFOLLOW_BEGIN")
    watcher:RegisterEvent("AUTOFOLLOW_END")
    watcher:RegisterEvent("PLAYER_MOUNT_DISPLAY_CHANGED")
    watcher:SetScript("OnEvent", function(_, event, name)
        if event == "AUTOFOLLOW_BEGIN" then begins[#begins + 1] = name
        elseif event == "AUTOFOLLOW_END" then ends = ends + 1
        else displays = displays + 1 end
    end)
    local follow = SlashCmdList[SLASH_COMMAND.FOLLOW]
    follow("party1")
    assert(begins[1] == leader and AutoFollowStatus:IsShown())
    assert(AutoFollowStatusText:GetText() == AUTOFOLLOWSTART:format(leader))
    A_Admin.EndAutoFollow()
    assert(ends == 1 and AutoFollowStatusText:GetText() == AUTOFOLLOWSTOP:format(leader))
    A_Admin.EndAutoFollow()
    assert(ends == 1, "no follow to end")
    follow("nobody")
    follow("player")
    assert(#begins == 1, "unknown names and the player follow nobody")
    follow(leader:upper())
    FollowUnit(leader:upper(), true)
    assert(#begins == 2 and begins[2] == leader, "names match case-insensitively unless exact")

    A_Admin.SetMounted(true)
    Dismount()
    assert(not IsMounted() and displays == 1)
    Dismount()
    assert(displays == 1, "dismounting on foot changes nothing")
    return true
"#;

const SHEATH_AND_RAID_OPTION: &str = r#"
    assert(GetSheathState() == 1, "weapons start sheathed")
    ToggleSheath()
    assert(GetSheathState() == 2)
    ToggleSheath()
    assert(GetSheathState() == 1)
    assert(GetAllowLowLevelRaid() == false)
    SetAllowLowLevelRaid(true)
    assert(GetAllowLowLevelRaid() == true)
    SetAllowLowLevelRaid()
    assert(GetAllowLowLevelRaid() == false, "documented default is false")
    assert(not pcall(SetAllowLowLevelRaid, 1))
    return true
"#;

const GUILD_INVITE: &str = r#"
    A_Admin.SetManualClock(5)
    A_Admin.ClearGuild()
    local function offer(guild, rank, members)
        A_Admin.OfferGuildInvite({ inviter = "Thrall", guildName = guild,
            guildAchievementPoints = 1250, oldGuildName = "", isNewGuild = false,
            rankName = rank, numMembers = members })
    end
    offer("Horde Heroes", "Initiate", 42)
    assert(GuildInviteFrame:IsShown() and GuildInviteFrameGuildName:GetText() == "Horde Heroes")
    assert(GuildInviteFrameInviterName:GetText() == "Thrall")
    GuildInviteFrameJoinButton:Click()
    local name, rank = GetGuildInfo("player")
    assert(not GuildInviteFrame:IsShown() and name == "Horde Heroes" and rank == "Initiate")
    A_Admin.LeaveGuild()
    offer("Alliance Aces", "Recruit", 7)
    GuildInviteFrameDeclineButton:Click()
    AcceptGuild()
    assert(not IsInGuild(), "declining withdraws the invite")
    offer("Slow Guild", "Member", 3)
    A_Admin.AdvanceTime(61)
    assert(not GuildInviteFrame:IsShown())
    AcceptGuild()
    assert(not IsInGuild(), "the frame timeout declines")
    SetAutoDeclineGuildInvites(true)
    offer("Ignored", "Member", 3)
    assert(not GuildInviteFrame:IsShown())
    AcceptGuild()
    assert(not IsInGuild(), "auto-decline refuses the invite")
    assert(not pcall(A_Admin.OfferGuildInvite, { inviter = "x" }))
    return true
"#;

const DEATH_DIALOG_CONTROL: &str = r#"
    A_Admin.SetManualClock(5)
    A_Admin.SetPlayerHealth(0, 100)
    A_Admin.SetReleaseTimeRemaining(360)
    StaticPopup_Show("DEATH")
    local death = StaticPopup_FindVisible("DEATH")
    local release = death:GetButton1()
    A_Admin.SetFalling(true)
    A_Admin.AdvanceTime(0.1)
    assert(not release:IsEnabled(), "falling in bounds blocks release")
    A_Admin.SetOutOfBounds(true)
    A_Admin.AdvanceTime(0.1)
    assert(release:IsEnabled() and IsOutOfBounds(), "falling out of bounds allows release")

    A_Admin.SetControllingUnit(true)
    assert(UnitIsControlling("player") and not UnitIsControlling("party1"))
    A_Admin.OfferResurrect({ offerer = "Healer", hasSickness = true, hasTimer = true })
    A_Admin.AdvanceTime(1.5)
    StaticPopup_FindVisible("RESURRECT"):GetButton2():Click()
    assert(StaticPopup_FindVisible("DEATH") == nil, "a controlling player gets no DEATH dialog")
    A_Admin.SetControllingUnit(false)
    A_Admin.OfferResurrect({ offerer = "Healer", hasSickness = true, hasTimer = true })
    A_Admin.AdvanceTime(1.5)
    StaticPopup_FindVisible("RESURRECT"):GetButton2():Click()
    assert(StaticPopup_FindVisible("DEATH"), "declining while dead reopens DEATH")
    return true
"#;

fn run(addons: &[&str], script: &str) {
    harness::with_blizzard_addon_startup_shape(addons, &[], |env, _| {
        let result: bool = env.eval(script).expect("original consumers run");
        assert!(result);
    });
}

#[test]
fn follow_slash_command_and_dismount_drive_follow_status_and_mount_events() {
    run(&["Blizzard_FrameXML", "Blizzard_ChatFrameBase"], FOLLOW_AND_DISMOUNT);
}

#[test]
fn sheath_toggle_and_low_level_raid_option_follow_calls() {
    run(&["Blizzard_FrameXML"], SHEATH_AND_RAID_OPTION);
}

#[test]
fn guild_invite_frame_accepts_declines_and_times_out() {
    run(&["Blizzard_FrameXML"], GUILD_INVITE);
}

#[test]
fn death_dialog_reads_out_of_bounds_and_unit_control() {
    run(&["Blizzard_Game", "Blizzard_StaticPopup_Game"], DEATH_DIALOG_CONTROL);
}

#[test]
fn percent_health_from_guid_tracks_visible_units() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        A_Admin.SetPartySize(2)
        A_Admin.SetPartyMemberHealth(1, 30, 120)
        A_Admin.SetPlayerHealth(50, 200)
        assert(UnitPercentHealthFromGUID(UnitGUID("party1")) == 0.25)
        assert(UnitPercentHealthFromGUID(UnitGUID("player")) == 0.25 * 1)
        A_Admin.SetPartyMemberHealth(1, 120, 120)
        assert(UnitPercentHealthFromGUID(UnitGUID("party1")) == 1)
        assert(UnitPercentHealthFromGUID("Player-0-00000000") == nil)
    "#,
        )
        .unwrap();
}

/// Retail GameTooltip health bar (Forever's tooltip does not watch GUIDs).
#[cfg(feature = "client-retail")]
#[test]
fn tooltip_health_bar_watches_guid_health() {
    run(
        &["Blizzard_GameTooltip"],
        r#"
        A_Admin.SetPartySize(2)
        A_Admin.SetPartyMemberHealth(1, 45, 180)
        local bar = GameTooltip.StatusBar
        bar:SetWatch(UnitGUID("party1"))
        assert(bar:IsShown() and bar:GetValue() == 0.25)
        A_Admin.SetPartyMemberHealth(1, 90, 180)
        bar:UpdateUnitHealth()
        assert(bar:GetValue() == 0.5)
        return true
    "#,
    );
}
