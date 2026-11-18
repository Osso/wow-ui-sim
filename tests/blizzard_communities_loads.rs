#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use sha2::{Digest, Sha256};

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn communities_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Communities/Blizzard_Communities_Mainline.toc")
}

const GUILD_FACTION_GROUP_CONSTANT_ONE_OVERLAY: &str =
    r#"GetGuildFactionGroup = function() return 1 end"#;
const GUILD_FACTION_GROUP_HORDE_SHORTCUT_OVERLAY: &str = r#"
    GetGuildFactionGroup = function()
        return UnitFactionGroup("player") == "Horde" and 0 or 1
    end
"#;

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_communities_toc_declares_required_deps_and_per_character_savedvars() {
    let toc = TocFile::from_file(&communities_toc()).expect("Communities TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_Communities is a non-LOD addon (it should auto-load on Game screen)"
    );
    let deps = toc.dependencies();
    for required in [
        "Blizzard_SharedXML",
        "Blizzard_GuildControlUI",
        "Blizzard_TimerunningUtil",
        "Blizzard_HelpPlate",
    ] {
        assert!(
            deps.contains(&required.to_string()),
            "Blizzard_Communities should declare RequiredDep `{required}`, got {deps:?}"
        );
    }
    let per_char = toc.saved_variables_per_character();
    assert!(
        per_char.contains(&"g_clubIdToSeenApplicants".to_string()),
        "Blizzard_Communities declares `## SavedVariablesPerCharacter: g_clubIdToSeenApplicants` \
         (so the per-character seen-applicants set persists across reloads), got {per_char:?}"
    );
}

#[test]
fn blizzard_communities_appears_in_game_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_Communities");
    assert!(
        in_game,
        "Blizzard_Communities (`## AllowLoadGameType: mainline`, non-LOD) should appear in \
         Game-screen auto-discovery"
    );
}

prefork_full_ui_case! {
fn guild_news_pvp_texture_uses_player_faction_group(env: &WowLuaEnv) {
    env.exec(
        r#"
        local newsFrame = CommunitiesFrame.GuildDetailsFrame.News
        assert(newsFrame, "GuildNews frame must be instantiated")
        A_Admin.SetGuildFactionGroupFlags(2, 4)
        A_Admin.SetGuildFactionGroupFlags(1, 2)
        local function assertPvpTexture(raceIndex, expectedTexture, label, flags)
            if flags then
                A_Admin.SetGuildFactionGroupFlags(raceIndex + 1, flags)
            end
            A_Admin.SetPlayerRace(raceIndex)
            CommunitiesGuildNewsFrame_OnLoad(newsFrame)
            assert(
                GUILD_EVENT_TEXTURES[Enum.CalendarEventType.PvP] == expectedTexture,
                label .. ": unexpected PvP texture"
            )
        end

        assertPvpTexture(1, "Interface\\Calendar\\UI-Calendar-Event-PVP01", "Orc")
        assertPvpTexture(0, "Interface\\Calendar\\UI-Calendar-Event-PVP02", "Human")
        assertPvpTexture(99, "Interface\\Calendar\\UI-Calendar-Event-PVP02", "missing race row")
        assertPvpTexture(0, "Interface\\Calendar\\UI-Calendar-Event-PVP01", "flag 4 precedes flag 2", 6)
        "#,
    )
    .expect("GuildNews must select its PvP texture from GetGuildFactionGroup");
}
}

fn assert_wrong_guild_faction_group_producer_fails(
    env: &WowLuaEnv,
    overlay: &str,
    fixture: &str,
    label: &str,
) {
    env.exec(overlay)
        .expect("wrong producer overlay must install");
    env.exec(fixture)
        .unwrap_or_else(|error| panic!("{label}: {error}"));
    env.exec("GetGuildFactionGroup = GuildFactionOriginal")
        .expect("original producer must be restored");
}

prefork_full_ui_case! {
fn wrong_guild_faction_group_producers_fail_guild_news_assertions(env: &WowLuaEnv) {
    let frame_exists: bool = env
        .eval("return CommunitiesFrame.GuildDetailsFrame.News ~= nil")
        .expect("GuildNews frame query must succeed");
    assert!(frame_exists, "GuildNews frame must be instantiated");
    assert_eq!(format!("{:x}", Sha256::digest(GUILD_FACTION_GROUP_CONSTANT_ONE_OVERLAY.as_bytes())), "aa5d9dcedf8a5d346b725ef9e8807927b2153dcd84efb69296756c4366591a35");
    assert_eq!(format!("{:x}", Sha256::digest(GUILD_FACTION_GROUP_HORDE_SHORTCUT_OVERLAY.as_bytes())), "73a30b8febf7866ff867e02acb54ce38815413d477771247522b798a99c85893");
    env.exec("GuildFactionOriginal = GetGuildFactionGroup; function GuildFactionAssertTexture(expected, label) CommunitiesGuildNewsFrame_OnLoad(CommunitiesFrame.GuildDetailsFrame.News); assert(GUILD_EVENT_TEXTURES[Enum.CalendarEventType.PvP] == expected, label) end")
        .expect("original GuildNews consumer assertion must be installed");
    assert_wrong_guild_faction_group_producer_fails(
        env,
        GUILD_FACTION_GROUP_CONSTANT_ONE_OVERLAY,
        "A_Admin.SetGuildFactionGroupFlags(2, 4); A_Admin.SetPlayerRace(1); local ok, err = pcall(GuildFactionAssertTexture, 'Interface\\\\Calendar\\\\UI-Calendar-Event-PVP01', 'constant-one'); assert(not ok and tostring(err):find('constant-one', 1, true))",
        "constant-one",
    );
    assert_wrong_guild_faction_group_producer_fails(
        env,
        GUILD_FACTION_GROUP_HORDE_SHORTCUT_OVERLAY,
        "A_Admin.SetGuildFactionGroupFlags(1, 6); A_Admin.SetPlayerRace(0); local ok, err = pcall(GuildFactionAssertTexture, 'Interface\\\\Calendar\\\\UI-Calendar-Event-PVP01', 'faction-shortcut'); assert(not ok and tostring(err):find('faction-shortcut', 1, true))",
        "faction shortcut",
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("Blizzard_Communities"))
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_Communities emitted Lua errors during load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_toplevel_frames_are_defined(env: &WowLuaEnv) {

    let frames_present: bool = env
        .eval(
            "return type(_G.CommunitiesFrame) == 'table' \
                and CommunitiesFrame:GetParent() == UIParent \
                and type(_G.CommunitiesAvatarPickerDialog) == 'table' \
                and type(_G.CommunitiesTicketManagerDialog) == 'table' \
                and type(_G.CommunitiesGuildTextEditFrame) == 'table' \
                and type(_G.CommunitiesGuildLogFrame) == 'table' \
                and type(_G.CommunitiesGuildNewsFiltersFrame) == 'table' \
                and type(_G.CommunitiesSettingsDialog) == 'table'",
        )
        .expect("toplevel frame query should succeed");
    assert!(
        frames_present,
        "Blizzard_Communities should define the seven top-level frames after load: \
         CommunitiesFrame (parent UIParent, ButtonFrameTemplateMinimizable), \
         CommunitiesAvatarPickerDialog, CommunitiesTicketManagerDialog, \
         CommunitiesGuildTextEditFrame, CommunitiesGuildLogFrame, \
         CommunitiesGuildNewsFiltersFrame, CommunitiesSettingsDialog"
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_core_mixins_are_defined(env: &WowLuaEnv) {

    let mixins_present: bool = env
        .eval(
            "return type(CommunitiesFrameMixin) == 'table' \
                and type(CommunitiesListMixin) == 'table' \
                and type(CommunitiesListEntryMixin) == 'table' \
                and type(CommunitiesListDropdownMixin) == 'table' \
                and type(CommunitiesChatMixin) == 'table' \
                and type(CommunitiesInvitationFrameMixin) == 'table' \
                and type(CommunitiesTicketFrameMixin) == 'table' \
                and type(CommunitiesSettingsDialogMixin) == 'table' \
                and type(CommunitiesAvatarPickerDialogMixin) == 'table' \
                and type(CommunitiesGuildMemberDetailMixin) == 'table' \
                and type(CommunitiesLanguageDropdownMixin) == 'table' \
                and type(CommunitiesGuildRewardsFrameMixin) == 'table' \
                and type(CommunitiesGuildFactionBarMixin) == 'table' \
                and type(GuildAchievementPointDisplayMixin) == 'table'",
        )
        .expect("core mixin query should succeed");
    assert!(
        mixins_present,
        "Blizzard_Communities should define its core 14 mixins (CommunitiesFrameMixin / List / \
         ListEntry / ListDropdown / Chat / InvitationFrame / TicketFrame / SettingsDialog / \
         AvatarPickerDialog / GuildMemberDetail / LanguageDropdown / GuildRewardsFrame / \
         GuildFactionBar / GuildAchievementPointDisplay)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_club_finder_mixins_are_defined(env: &WowLuaEnv) {

    let mixins_present: bool = env
        .eval(
            "return type(ClubFinderApplicantEntryMixin) == 'table' \
                and type(ClubFinderApplicantListMixin) == 'table' \
                and type(ClubFinderApplicantInviteButtonMixin) == 'table' \
                and type(ClubFinderApplicantCancelButtonMixin) == 'table' \
                and type(ClubFinderDropdownMixin) == 'table' \
                and type(ClubsRecruitmentDialogMixin) == 'table' \
                and type(ClubFinderRequestToJoinMixin) == 'table' \
                and type(ClubFinderFilterDropdownMixin) == 'table' \
                and type(ClubFinderSearchEditBoxMixin) == 'table' \
                and type(ClubFinderCardMixin) == 'table' \
                and type(ClubFinderGuildCardMixin) == 'table' \
                and type(ClubFinderCommunitiesCardMixin) == 'table' \
                and type(ClubFinderInvitationsFrameMixin) == 'table' \
                and type(ClubFinderTabMixin) == 'table' \
                and type(ClubFinderRoleMixin) == 'table'",
        )
        .expect("ClubFinder mixin query should succeed");
    assert!(
        mixins_present,
        "Blizzard_Communities should define its 15 ClubFinder.lua mixins (ApplicantEntry, \
         ApplicantList, ApplicantInviteButton, ApplicantCancelButton, Dropdown, \
         ClubsRecruitmentDialog, RequestToJoin, FilterDropdown, SearchEditBox, Card + \
         GuildCard/CommunitiesCard derivatives, InvitationsFrame, Tab, Role)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_frame_display_modes_are_populated(env: &WowLuaEnv) {

    let display_modes_present: bool = env
        .eval(
            "return type(COMMUNITIES_FRAME_DISPLAY_MODES) == 'table' \
                and type(COMMUNITIES_FRAME_DISPLAY_MODES.CHAT) == 'table' \
                and type(COMMUNITIES_FRAME_DISPLAY_MODES.ROSTER) == 'table' \
                and type(COMMUNITIES_FRAME_DISPLAY_MODES.COMMUNITY_APPLICANT_LIST) == 'table' \
                and type(COMMUNITIES_FRAME_DISPLAY_MODES.GUILD_FINDER) == 'table' \
                and type(COMMUNITIES_FRAME_DISPLAY_MODES.COMMUNITY_FINDER) == 'table' \
                and type(COMMUNITIES_FRAME_DISPLAY_MODES.GUILD_BENEFITS) == 'table' \
                and type(COMMUNITIES_FRAME_DISPLAY_MODES.GUILD_INFO) == 'table'",
        )
        .expect("COMMUNITIES_FRAME_DISPLAY_MODES query should succeed");
    assert!(
        display_modes_present,
        "COMMUNITIES_FRAME_DISPLAY_MODES should be populated with the per-mode child-name lists \
         (CHAT, ROSTER, COMMUNITY_APPLICANT_LIST, GUILD_FINDER, COMMUNITY_FINDER, \
         GUILD_BENEFITS, GUILD_INFO) used by SetDisplayMode to selectively show/hide \
         CommunitiesFrame children"
    );
}
}

prefork_full_ui_case! {
fn blizzard_communities_frame_methods_and_globals_are_defined(env: &WowLuaEnv) {

    let methods_present: bool = env
        .eval(
            "return type(CommunitiesFrameMixin.OnLoad) == 'function' \
                and type(CommunitiesFrameMixin.OnShow) == 'function' \
                and type(CommunitiesFrameMixin.OnHide) == 'function' \
                and type(CommunitiesFrameMixin.OnEvent) == 'function' \
                and type(CommunitiesFrameMixin.SelectClub) == 'function' \
                and type(CommunitiesFrameMixin.GetSelectedClubId) == 'function' \
                and type(CommunitiesFrameMixin.SetDisplayMode) == 'function' \
                and type(CommunitiesFrameMixin.GetDisplayMode) == 'function' \
                and type(CommunitiesFrameMixin.SelectStream) == 'function' \
                and type(CommunitiesFrameMixin.GetSelectedStreamId) == 'function' \
                and type(CommunitiesFrameMixin.UpdateCommunitiesButtons) == 'function' \
                and type(CommunitiesFrameMixin.HasNewClubApplications) == 'function' \
                and type(CommunitiesHyperlink) == 'table' \
                and type(GUILD_CHALLENGE_ORDER) == 'table' \
                and GUILD_CHALLENGE_ORDER[1] == 1 \
                and GUILD_CHALLENGE_ORDER[2] == 4 \
                and GUILD_CHALLENGE_ORDER[3] == 2 \
                and GUILD_CHALLENGE_ORDER[4] == 3",
        )
        .expect("CommunitiesFrameMixin method query should succeed");
    assert!(
        methods_present,
        "CommunitiesFrameMixin should expose its core 12 methods (OnLoad/OnShow/OnHide/OnEvent/\
         SelectClub/GetSelectedClubId/SetDisplayMode/GetDisplayMode/SelectStream/\
         GetSelectedStreamId/UpdateCommunitiesButtons/HasNewClubApplications) and the \
         CommunitiesHyperlink namespace + GUILD_CHALLENGE_ORDER=={{1,4,2,3}} (from \
         Mainline/GuildInfo.lua) should be populated"
    );
}
}
