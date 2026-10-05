//! Tests for `C_Club` probes backed by `WorldState.guild_*` fields.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::GuildMember;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn is_enabled_returns_true() {
    let env = env();
    let enabled: bool = env.eval("return C_Club.IsEnabled()").unwrap();
    assert!(enabled);
}

#[test]
fn get_subscribed_clubs_returns_guild_entry() {
    let env = env();
    let (count, name, club_type): (i32, String, i32) = env
        .eval(
            r#"
            local clubs = C_Club.GetSubscribedClubs()
            return #clubs, clubs[1].name, clubs[1].clubType
            "#,
        )
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(name, "Heroes of Azeroth");
    assert_eq!(club_type, 2);
}

#[test]
fn get_subscribed_clubs_club_id_is_string() {
    let env = env();
    let club_id: String = env
        .eval("return C_Club.GetSubscribedClubs()[1].clubId")
        .unwrap();
    assert!(!club_id.is_empty(), "clubId should be a non-empty string");
}

#[test]
fn get_club_info_returns_guild_shape() {
    let env = env();
    let (name, club_type, member_count): (String, i32, i32) = env
        .eval(
            r#"
            local club = C_Club.GetClubInfo("guild-0")
            return club.name, club.clubType, club.memberCount
            "#,
        )
        .unwrap();
    assert_eq!(name, "Heroes of Azeroth");
    assert_eq!(club_type, 2);
    assert!(member_count > 0);
}

#[test]
fn get_subscribed_clubs_empty_when_no_guild() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.world.guild_name = None;
    }
    let count: i32 = env.eval("return #C_Club.GetSubscribedClubs()").unwrap();
    assert_eq!(count, 0, "no guild → empty subscribed clubs list");
}

#[test]
fn get_club_members_returns_guild_roster() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.world.guild_members = vec![GuildMember {
            name: "Uther".into(),
            rank_index: 1,
            online: true,
        }];
    }
    let count: i32 = env
        .eval("return #C_Club.GetClubMembers('guild-0')")
        .unwrap();
    assert_eq!(count, 1, "should return seeded guild member");
}

#[test]
fn get_club_members_entry_has_required_fields() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.world.guild_members = vec![GuildMember {
            name: "Uther".into(),
            rank_index: 1,
            online: true,
        }];
    }
    let (member_id, name, is_self, presence): (String, String, bool, i32) = env
        .eval(
            r#"
            local members = C_Club.GetClubMembers('guild-0')
            local memberId = members[1]
            local m = C_Club.GetMemberInfo('guild-0', memberId)
            return memberId, m.name, m.isSelf, m.presence
            "#,
        )
        .unwrap();
    assert!(!member_id.is_empty());
    assert_eq!(name, "Uther");
    assert!(is_self, "first member should be isSelf=true");
    assert_eq!(presence, 1, "online presence = 1");
}

#[test]
fn get_club_members_returns_member_ids_for_member_info_lookup() {
    let env = env();
    let (count, first_id, first_name, second_id, second_name): (i32, String, String, String, String) =
        env.eval(
            r#"
            local members = C_Club.GetClubMembers('guild-0')
            local first = C_Club.GetMemberInfo('guild-0', members[1])
            local second = C_Club.GetMemberInfo('guild-0', members[2])
            return #members, members[1], first.name, members[2], second.name
            "#,
        )
        .unwrap();

    assert_eq!(count, 2);
    assert!(!first_id.is_empty());
    assert_eq!(first_name, "Uther");
    assert_ne!(second_id, first_id);
    assert_eq!(second_name, "Jaina");
}

#[test]
fn get_member_info_for_self_has_role() {
    let env = env();
    let (name, is_self, role): (String, bool, i32) = env
        .eval(
            r#"
            local member = C_Club.GetMemberInfoForSelf("guild-0")
            return member.name, member.isSelf, member.role
            "#,
        )
        .unwrap();
    assert!(!name.is_empty());
    assert!(is_self);
    assert_eq!(role, 4);
}

#[test]
fn get_club_privileges_returns_flags_table() {
    let env = env();
    let (kind, can_invite): (String, bool) = env
        .eval(
            r#"
            local privileges = C_Club.GetClubPrivileges("guild-0")
            return type(privileges), privileges.canSendInvitation
            "#,
        )
        .unwrap();
    assert_eq!(kind, "table");
    assert!(!can_invite);
}

#[test]
fn get_club_members_reflects_state_mutation() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.world.guild_members = vec![
            GuildMember {
                name: "Arthas".into(),
                rank_index: 1,
                online: true,
            },
            GuildMember {
                name: "Jaina".into(),
                rank_index: 2,
                online: false,
            },
        ];
    }
    let (count, first_name, first_presence, second_name, second_presence): (
        i32,
        String,
        i32,
        String,
        i32,
    ) = env
        .eval(
            r#"
            local members = C_Club.GetClubMembers('guild-0')
            local first = C_Club.GetMemberInfo('guild-0', members[1])
            local second = C_Club.GetMemberInfo('guild-0', members[2])
            return #members, first.name, first.presence, second.name, second.presence
            "#,
        )
        .unwrap();
    assert_eq!(count, 2);
    assert_eq!(first_name, "Arthas");
    assert_eq!(first_presence, 1);
    assert_eq!(second_name, "Jaina");
    assert_eq!(second_presence, 3);
}

#[test]
fn members_are_ready_for_seeded_guild_club() {
    let env = env();
    let (club_id, ready): (String, bool) = env
        .eval(
            r#"
            local clubId = C_Club.GetGuildClubId()
            C_Club.FocusMembers(clubId)
            return clubId, C_Club.AreMembersReady(clubId)
            "#,
        )
        .unwrap();

    assert_eq!(club_id, "guild-0");
    assert!(ready);
}

#[test]
fn get_club_capacity_returns_number() {
    let env = env();
    let capacity: i32 = env
        .eval("return C_Club.GetClubCapacity('guild-0')")
        .unwrap();
    assert!(capacity > 0, "guild capacity should be positive");
}

#[test]
fn get_streams_returns_sortable_table() {
    let env = env();
    let (stream_type, count, guild_id, guild_name, officer_id, officer_name): (
        String,
        i32,
        i32,
        String,
        i32,
        String,
    ) = env
        .eval(
            r#"
            local streams = C_Club.GetStreams('guild-0')
            table.sort(streams, function(lhs, rhs)
                return lhs.creationTime < rhs.creationTime
            end)
            return type(streams), #streams,
                streams[1].streamId, streams[1].name,
                streams[2].streamId, streams[2].name
            "#,
        )
        .unwrap();
    assert_eq!(stream_type, "table");
    assert_eq!(count, 2);
    assert_eq!(guild_id, 1);
    assert_eq!(guild_name, "Guild");
    assert_eq!(officer_id, 2);
    assert_eq!(officer_name, "Officer");
}

#[test]
fn guild_stream_returns_generated_message_history() {
    let env = env();
    let (range_count, message_count, first_author, first_content, last_author, last_content): (
        i32,
        i32,
        String,
        String,
        String,
        String,
    ) = env
        .eval(
            r#"
            local ranges = C_Club.GetMessageRanges('guild-0', 1)
            local newest = ranges[1].newestMessageId
            local messages = C_Club.GetMessagesBefore('guild-0', 1, newest, 20)
            return #ranges, #messages,
                messages[1].author.name, messages[1].content,
                messages[#messages].author.name, messages[#messages].content
            "#,
        )
        .unwrap();

    assert_eq!(range_count, 1);
    assert_eq!(message_count, 4);
    assert_eq!(first_author, "Uther");
    assert!(first_content.contains("Welcome"));
    assert_eq!(last_author, "Uther");
    assert!(last_content.contains("Transmog"));
}

#[test]
fn guild_message_info_round_trips_by_message_id() {
    let env = env();
    let (stream_name, subscribed, content, beginning): (String, bool, String, bool) = env
        .eval(
            r#"
            local stream = C_Club.GetStreamInfo('guild-0', 1)
            local ranges = C_Club.GetMessageRanges('guild-0', 1)
            local oldest = ranges[1].oldestMessageId
            local message = C_Club.GetMessageInfo('guild-0', 1, oldest)
            return stream.name, C_Club.IsSubscribedToStream('guild-0', 1),
                message.content, C_Club.IsBeginningOfStream('guild-0', 1, oldest)
            "#,
        )
        .unwrap();

    assert_eq!(stream_name, "Guild");
    assert!(subscribed);
    assert!(content.contains("Welcome"));
    assert!(beginning);
}

#[test]
fn club_unread_message_queries_have_safe_defaults() {
    let env = env();
    let (any_unread, stream_marker_type, settings_count): (bool, String, i32) = env
        .eval(
            r#"
            local settings = C_Club.GetClubStreamNotificationSettings('guild-0')
            return
                C_Club.DoesAnyCommunityHaveUnreadMessages(),
                type(C_Club.GetStreamViewMarker('guild-0', 1)),
                #settings
            "#,
        )
        .unwrap();
    assert!(!any_unread);
    assert_eq!(stream_marker_type, "nil");
    assert_eq!(settings_count, 0);
}

#[test]
fn send_message_appends_to_history_and_fires_event() {
    let env = env();
    let (message_count, last_content, event_fired, event_club, event_stream, retrieved_content): (
        i32,
        String,
        bool,
        String,
        i32,
        String,
    ) = env
        .eval(
            r#"
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("CLUB_MESSAGE_ADDED")
            local fired = false
            local capturedClub, capturedStream, capturedMessageId
            listener:SetScript("OnEvent", function(self, event, clubId, streamId, messageId)
                fired = true
                capturedClub = clubId
                capturedStream = streamId
                capturedMessageId = messageId
            end)

            C_Club.SendMessage("guild-0", 1, "Pulling in 30 seconds")

            local ranges = C_Club.GetMessageRanges("guild-0", 1)
            local newest = ranges[1].newestMessageId
            local messages = C_Club.GetMessagesBefore("guild-0", 1, newest, 20)
            local last = messages[#messages]
            local lookup = C_Club.GetMessageInfo("guild-0", 1, capturedMessageId)
            return #messages, last.content, fired,
                capturedClub, capturedStream, lookup.content
            "#,
        )
        .unwrap();
    assert_eq!(message_count, 5, "static seed (4) + 1 sent message");
    assert_eq!(last_content, "Pulling in 30 seconds");
    assert!(event_fired, "CLUB_MESSAGE_ADDED should fire");
    assert_eq!(event_club, "guild-0");
    assert_eq!(event_stream, 1);
    assert_eq!(
        retrieved_content, "Pulling in 30 seconds",
        "GetMessageInfo should return the sent message via the event's messageId"
    );
}

#[test]
fn send_message_ignores_empty_text() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            C_Club.SendMessage("guild-0", 1, "")
            local ranges = C_Club.GetMessageRanges("guild-0", 1)
            local messages = C_Club.GetMessagesBefore("guild-0", 1, ranges[1].newestMessageId, 20)
            return #messages
            "#,
        )
        .unwrap();
    assert_eq!(count, 4, "empty text should not append a message");
}

#[test]
fn send_message_ignores_unknown_stream() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            C_Club.SendMessage("guild-0", 99, "Stream that does not exist")
            local ranges = C_Club.GetMessageRanges("guild-0", 1)
            local messages = C_Club.GetMessagesBefore("guild-0", 1, ranges[1].newestMessageId, 20)
            return #messages
            "#,
        )
        .unwrap();
    assert_eq!(count, 4, "unknown stream should not append to guild stream");
}

#[test]
fn club_finder_queries_have_seeded_card_defaults() {
    let env = env();
    let (
        enabled,
        invites,
        applicants,
        pending,
        status_flags,
        guild_total,
        recruitment_shape,
        applicant_shape,
        first_guild_name,
        pending_guilds,
        pending_status,
        community_total,
        first_community_name,
    ): (
        bool,
        i32,
        i32,
        i32,
        i32,
        i32,
        bool,
        bool,
        String,
        i32,
        i32,
        i32,
        String,
    ) = env
        .eval(
            r#"
            local recruitment = C_ClubFinder.GetClubRecruitmentSettings()
            local applicant = C_ClubFinder.GetPlayerApplicantSettings()
            local recruitmentShape =
                recruitment.playStyleDungeon == false and
                recruitment.playStyleRaids == false and
                recruitment.playStylePvp == false and
                recruitment.playStyleRP == false and
                recruitment.playStyleSocial == false and
                recruitment.maxLevelOnly == false and
                recruitment.enableListing == false
            local applicantShape =
                applicant.playStyleDungeon == false and
                applicant.playStyleRaids == false and
                applicant.playStylePvp == false and
                applicant.playStyleRP == false and
                applicant.playStyleSocial == false and
                applicant.roleTank == false and
                applicant.roleHealer == false and
                applicant.roleDps == false and
                applicant.sizeSmall == false and
                applicant.sizeMedium == false and
                applicant.sizeLarge == false and
                applicant.sortRelevance == true and
                applicant.sortMembers == false and
                applicant.sortNewest == false and
                applicant.crossFaction == false
            local guilds = C_ClubFinder.ReturnMatchingGuildList()
            local pendingGuilds = C_ClubFinder.PlayerReturnPendingGuildsList()
            local communities = C_ClubFinder.ReturnMatchingCommunityList()
            return
                C_ClubFinder.IsEnabled(),
                #C_ClubFinder.PlayerGetClubInvitationList(),
                #C_ClubFinder.ReturnClubApplicantList("guild-0"),
                #C_ClubFinder.ReturnPendingClubApplicantList("guild-0"),
                #C_ClubFinder.GetStatusOfPostingFromClubId("guild-0"),
                C_ClubFinder.GetTotalMatchingGuildListSize(),
                recruitmentShape,
                applicantShape,
                guilds[1].name,
                #pendingGuilds,
                C_ClubFinder.GetPlayerClubApplicationStatus(pendingGuilds[1].clubFinderGUID),
                C_ClubFinder.GetTotalMatchingCommunityListSize(),
                communities[1].name
            "#,
        )
        .unwrap();
    assert!(enabled);
    assert_eq!(invites, 0);
    assert_eq!(applicants, 0);
    assert_eq!(pending, 0);
    assert_eq!(status_flags, 0);
    assert_eq!(guild_total, 1);
    assert!(recruitment_shape);
    assert!(applicant_shape);
    assert_eq!(first_guild_name, "Heroes of Azeroth");
    assert_eq!(pending_guilds, 1);
    assert_eq!(pending_status, 1);
    assert_eq!(community_total, 1);
    assert_eq!(first_community_name, "Timeless Isle Raiders");
}
