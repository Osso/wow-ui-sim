#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn friends_frame_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_FriendsFrame/Blizzard_FriendsFrame.toc")
}

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
fn blizzard_friends_frame_toc_declares_required_social_dependencies_and_allow_load_both() {
    let toc = TocFile::from_file(&friends_frame_toc()).expect("Blizzard_FriendsFrame TOC parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_FriendsFrame has no `## LoadOnDemand` line — the social UI frame is \
         eagerly loaded at startup so its event handlers (FRIENDLIST_UPDATE, \
         BN_FRIEND_INFO_CHANGED, WHO_LIST_UPDATE) are wired up before the user opens \
         the panel"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_FriendsFrame does not declare `## UseSecureEnvironment` — the social \
         panel runs in the standard taint environment"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_FriendsFrame declares `## AllowLoadGameType: mainline`, but \
         `is_game_type_restricted()` returns false because src/toc.rs:299 treats \
         `mainline` and `standard` as the unrestricted (retail) game type"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_FriendsFrame declares no `## SavedVariables` — friend / ignore / who \
         state lives server-side, not in the per-character SavedVars file"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_TimerunningUtil".to_string(),
            "Blizzard_AddFriend".to_string(),
        ],
        "Blizzard_FriendsFrame requires Blizzard_TimerunningUtil and Blizzard_AddFriend. \
         The latter now owns the BattleNet invite UI extracted from FriendsFrame. Got: \
         {:?}",
        deps
    );

    let toc_text = std::fs::read_to_string(friends_frame_toc())
        .expect("Blizzard_FriendsFrame TOC should read");
    assert!(
        toc_text.contains(
            "## OptionalDeps: Blizzard_GlueStubs, Blizzard_ActionBar, Blizzard_SocialUIShared, Blizzard_RecentAllies, Blizzard_UnitPopupShared, Blizzard_UnitPopup"
        ),
        "Blizzard_FriendsFrame declares the current optional social UI dependencies, including \
         Blizzard_SocialUIShared and both UnitPopup addons."
    );
    assert!(
        toc_text.contains("## AllowLoad: both"),
        "Blizzard_FriendsFrame declares `## AllowLoad: both` (lowercase) — the friends \
         list is reachable from glue screens (Login/CharacterSelect) too via the BattleNet \
         friend list overlay, not just in-world"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: mainline"),
        "Blizzard_FriendsFrame declares `## AllowLoadGameType: mainline` — the file is \
         the retail-only variant; classic flavors ship a separate FriendsFrame addon"
    );
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_FriendsFrame declares `## DefaultState: enabled` — the social panel \
         is on by default (users can disable via the AddOn list, but the panel is \
         considered core UI)"
    );
}

#[test]
fn blizzard_friends_frame_allows_all_screens_via_allow_load_both() {
    let toc = TocFile::from_file(&friends_frame_toc()).expect("Blizzard_FriendsFrame TOC parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: both` must allow the Game screen (src/toc.rs:307)"
    );
    assert!(
        toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: both` must allow the Login screen — BattleNet friend list is \
         accessible from the login glue UI"
    );
    assert!(
        toc.allows_screen(ScreenKind::CharacterSelect),
        "`## AllowLoad: both` must allow CharacterSelect — friend list is reachable \
         from the realm-list glue screen"
    );
}

#[test]
fn blizzard_friends_frame_auto_loads_on_game_screen() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FriendsFrame");
    assert!(
        in_game,
        "Blizzard_FriendsFrame has no `## LoadOnDemand` line and `## AllowLoad: both`, \
         so it MUST appear in Game-screen auto-discovery"
    );
}

prefork_full_ui_case! {
fn blizzard_friends_frame_loads_via_full_game_ui_without_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("FriendsFrame")
                || message.contains("AddFriendFrame")
                || message.contains("FriendsFriends")
                || message.contains("BattleTagInvite")
                || message.contains("WhoFrame")
                || message.contains("FriendsList")
                || message.contains("RecentAllies")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_FriendsFrame emitted Lua errors during the full Game-screen load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_is_addon_loaded_returns_true_after_full_game_ui_load(env: &WowLuaEnv) {

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_FriendsFrame') and true or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        post_load,
        "After full Game-screen load, IsAddOnLoaded('Blizzard_FriendsFrame') must \
         return true — auto-discovery picks up the addon (no LoadOnDemand) and \
         `mark_addon_loaded` registers it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_three_top_level_frames(env: &WowLuaEnv) {

    let frames: (bool, bool, bool) = env
        .eval(
            "return type(FriendsFrame) == 'table', \
                    type(AddFriendFrame) == 'table', \
                    type(FriendsFriendsFrame) == 'table'",
        )
        .expect("Top-level frame probe should succeed");
    assert_eq!(
        frames,
        (true, true, true),
        "Current FriendsFrame XML declares FriendsFrame (the main social panel), \
         AddFriendFrame (the dialog-strata add-friend panel), and FriendsFriendsFrame \
         (the mutual-friends popup). BattleNet invite UI now belongs to Blizzard_AddFriend, \
         not Blizzard_FriendsFrame."
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_internal_subframes(env: &WowLuaEnv) {

    let subframes: (bool, bool, bool, bool) = env
        .eval(
            "return type(FriendsListFrame) == 'table', \
                    type(WhoFrame) == 'table', \
                    type(RecentAlliesFrame) == 'table', \
                    type(FriendsTabHeader) == 'table'",
        )
        .expect("Subframe probe should succeed");
    assert_eq!(
        subframes,
        (true, true, true, true),
        "FriendsFrame.xml declares four named sub-frames inside the main panel that \
         become globals via `setAllPoints=true` containers: FriendsListFrame (line 776, \
         the BNet+WoW friend list scroll view), WhoFrame (line 915, /who results), \
         RecentAlliesFrame (line 869, recent group members tab — also addressable via \
         the optional Blizzard_RecentAllies addon), FriendsTabHeader (line 517, \
         TabSystemOwnerTemplate hosting the four social-panel tabs)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_friend_list_button_mixins(env: &WowLuaEnv) {

    let list_mixins: (bool, bool, bool, bool) = env
        .eval(
            "return type(FriendsListButtonMixin) == 'table', \
                    type(FriendsFriendsButtonMixin) == 'table', \
                    type(IgnoreListButtonMixin) == 'table', \
                    type(WhoListButtonMixin) == 'table'",
        )
        .expect("List button mixin probe should succeed");
    assert_eq!(
        list_mixins,
        (true, true, true, true),
        "Each scroll-list row type has its own mixin published from FriendsFrame.lua: \
         FriendsListButtonMixin (line 2222 — main BNet+WoW friend row layout), \
         FriendsFriendsButtonMixin (line 2186 — mutual-friends lookup row), \
         IgnoreListButtonMixin (line 2193 — ignore-list row), WhoListButtonMixin (line \
         2200 — /who results row). All four are consumed by ScrollBoxList view-builders \
         in FriendsFrame.xml"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_tab_and_dialog_mixins(env: &WowLuaEnv) {

    let tab_mixins: (bool, bool, bool, bool) = env
        .eval(
            "return type(FriendsTabHeaderMixin) == 'table', \
                    type(FriendsTabMixin) == 'table', \
                    type(FriendsFrameTabMixin) == 'table', \
                    type(FriendsFrameInviteTemplateMixin) == 'table'",
        )
        .expect("Tab/dialog mixin probe should succeed");
    assert_eq!(
        tab_mixins,
        (true, true, true, true),
        "FriendsFrame.lua publishes the tab-bar mixin family: FriendsTabHeaderMixin \
         (line 520 — the TabSystemOwner host for the 4-tab strip), FriendsTabMixin \
         (line 662, = CreateFromMixins(TabSystemButtonMixin) — individual tab button \
         visual state), FriendsFrameTabMixin (line 676 — legacy bottom-edge tab \
         strip), FriendsFrameInviteTemplateMixin (line 706 — pending-invite row in the \
         friends list)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_addfriend_and_summon_mixins(env: &WowLuaEnv) {

    let dialog_mixins: (bool, bool, bool, bool) = env
        .eval(
            "return type(AddFriendFrameMixin) == 'table', \
                    type(AddFriendIconHolderMixin) == 'table', \
                    type(AddFriendEntryFrameInfoButtonMixin) == 'table', \
                    type(AddFriendCloseButtonMixin) == 'table'",
        )
        .expect("AddFriend mixin probe should succeed");
    assert_eq!(
        dialog_mixins,
        (true, true, true, true),
        "AddFriend dialog mixin family from FriendsFrame.lua: AddFriendFrameMixin (line \
         2058 — the BattleTag/character-name entry resize dialog), AddFriendIconHolderMixin \
         (line 2981 — the BattleNet icon swap container), \
         AddFriendEntryFrameInfoButtonMixin (line 2994 — the (?) info tooltip button), \
         AddFriendCloseButtonMixin (line 3023 — the X close button with custom OnHide \
         clearing)"
    );

    let summon_and_who: (bool, bool, bool, bool) = env
        .eval(
            "return type(SummonButtonMixin) == 'table', \
                    type(WhoFrameEditBoxMixin) == 'table', \
                    type(WhoFrameColumnHeaderMixin) == 'table', \
                    type(FriendsBroadcastFrameMixin) == 'table'",
        )
        .expect("Summon/who/broadcast mixin probe should succeed");
    assert_eq!(
        summon_and_who,
        (true, true, true, true),
        "Action / who-search / broadcast mixin family: SummonButtonMixin (line 1128 — \
         the meeting-stone summon arrow on each friend row), WhoFrameEditBoxMixin (line \
         1484 — /who query parser EditBox), WhoFrameColumnHeaderMixin (line 3029 — \
         sortable column header for the /who results table), FriendsBroadcastFrameMixin \
         (line 1992 — the BattleNet broadcast-status edit popup)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_friends_friends_and_contacts_mixins(env: &WowLuaEnv) {

    let extras: (bool, bool, bool) = env
        .eval(
            "return type(FriendsFriendsFrameMixin) == 'table', \
                    type(FriendsIgnoreListMixin) == 'table', \
                    type(ContactsMenuMixin) == 'table'",
        )
        .expect("FriendsFriends/ignore/contacts mixin probe should succeed");
    assert_eq!(
        extras,
        (true, true, true),
        "Three remaining mixins from FriendsFrame.lua: FriendsFriendsFrameMixin (line \
         2449 — the mutual-friends-of-friends popup with potential/mutual/all radio), \
         FriendsIgnoreListMixin (line 3049 — the ignore-list scroll view controller), \
         ContactsMenuMixin (line 3085 — the right-click context menu for friend rows)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_display_count_constants(env: &WowLuaEnv) {

    let counts: (i64, i64, i64, i64, i64) = env
        .eval(
            "return FRIENDS_TO_DISPLAY, \
                    IGNORES_TO_DISPLAY, \
                    PENDING_INVITES_TO_DISPLAY, \
                    FRIENDS_FRIENDS_TO_DISPLAY, \
                    WHOS_TO_DISPLAY",
        )
        .expect("Display-count constant probe should succeed");
    assert_eq!(
        counts,
        (10, 19, 4, 11, 17),
        "FriendsFrame.lua lines 1-10 declare the visible-row counts that drive each \
         scroll list's initial size: FRIENDS_TO_DISPLAY=10 (friends list), \
         IGNORES_TO_DISPLAY=19 (ignore list — denser rows so fits more), \
         PENDING_INVITES_TO_DISPLAY=4 (pending invite header), \
         FRIENDS_FRIENDS_TO_DISPLAY=11 (mutual-friends popup), WHOS_TO_DISPLAY=17 \
         (/who results)"
    );

    let server_cap: i64 = env
        .eval("return MAX_WHOS_FROM_SERVER")
        .expect("MAX_WHOS_FROM_SERVER probe should succeed");
    assert_eq!(
        server_cap, 50,
        "MAX_WHOS_FROM_SERVER=50 (line 11) is the server-side cap on /who response \
         rows — the UI shows WHOS_TO_DISPLAY=17 of these at a time and scrolls through \
         the remaining"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_row_height_constants(env: &WowLuaEnv) {

    let heights: (i64, i64, i64, i64) = env
        .eval(
            "return FRIENDS_FRAME_FRIEND_HEIGHT, \
                    FRIENDS_FRAME_IGNORE_HEIGHT, \
                    FRIENDS_FRAME_FRIENDS_FRIENDS_HEIGHT, \
                    FRIENDS_FRAME_WHO_HEIGHT",
        )
        .expect("Row-height constant probe should succeed");
    assert_eq!(
        heights,
        (34, 16, 16, 16),
        "FriendsFrame.lua lines 2-10 publish per-list row heights consumed by the \
         ScrollBoxList view-builders: FRIEND_HEIGHT=34 (taller — fits the BNet status \
         portrait + class icon + name + game label), IGNORE/FRIENDS_FRIENDS/WHO=16 \
         (single-line text rows)"
    );

    let scroll_h: i64 = env
        .eval("return FRIENDS_SCROLLFRAME_HEIGHT")
        .expect("FRIENDS_SCROLLFRAME_HEIGHT probe should succeed");
    assert_eq!(
        scroll_h, 307,
        "FRIENDS_SCROLLFRAME_HEIGHT=307 (line 13) is the fixed pixel height of the main \
         scroll viewport — fits exactly FRIENDS_TO_DISPLAY=10 rows of \
         FRIENDS_FRAME_FRIEND_HEIGHT=34 with header padding"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_friend_button_type_enum(env: &WowLuaEnv) {

    let button_types: (i64, i64, i64, i64, i64, i64, i64) = env
        .eval(
            "return FRIENDS_BUTTON_TYPE_DIVIDER, \
                    FRIENDS_BUTTON_TYPE_BNET, \
                    FRIENDS_BUTTON_TYPE_WOW, \
                    FRIENDS_BUTTON_TYPE_INVITE, \
                    FRIENDS_BUTTON_TYPE_INVITE_HEADER, \
                    FRIENDS_BUTTON_TYPE_PARTY_INVITE, \
                    FRIENDS_BUTTON_TYPE_PARTY_INVITE_HEADER",
        )
        .expect("FRIENDS_BUTTON_TYPE_* probe should succeed");
    assert_eq!(
        button_types,
        (1, 2, 3, 4, 5, 6, 7),
        "FriendsFrame.lua lines 14-20 enumerate the seven row-types the friends list \
         scroll view can render. Each value is the discriminator stored in element data \
         and dispatched by FriendsFrame_Update<Type>Button helpers (DIVIDER=1 separator, \
         BNET=2 BattleNet account, WOW=3 in-game character, INVITE=4 friend invite, \
         INVITE_HEADER=5, PARTY_INVITE=6 group invite, PARTY_INVITE_HEADER=7)"
    );
}
}

prefork_full_ui_case! {
fn friends_frame_displays_supplied_battle_tags(env: &WowLuaEnv) {
    let tags: (String, String) = env
        .eval(
            r#"
            A_Admin.SetPlayerBattleNetInfo("Malfurion#1234", nil, nil, nil, nil, nil, nil, true, true)
            FriendsFrame_CheckBattlenetStatus()
            local first = FriendsFrameBattlenetFrame.Tag:GetText()
            A_Admin.SetPlayerBattleNetInfo("Tyrande#5678", nil, nil, nil, nil, nil, nil, true, true)
            FriendsFrame_CheckBattlenetStatus()
            return first, FriendsFrameBattlenetFrame.Tag:GetText()
            "#,
        )
        .expect("FriendsFrame should display supplied Battle.net account tags");
    assert_eq!(
        tags,
        (
            "Malfurion|cff416380#1234|r".into(),
            "Tyrande|cff416380#5678|r".into(),
        )
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_friend_tab_enum(env: &WowLuaEnv) {

    let tabs: (i64, i64, i64, i64, i64) = env
        .eval(
            "return FRIEND_TAB_COUNT, \
                    FRIEND_TAB_FRIENDS, \
                    FRIEND_TAB_WHO, \
                    FRIEND_TAB_RAID, \
                    FRIEND_TAB_QUICK_JOIN",
        )
        .expect("FRIEND_TAB_* probe should succeed");
    assert_eq!(
        tabs,
        (4, 1, 2, 3, 4),
        "FriendsFrame.lua lines 36-40 declare the four-tab social-panel layout: \
         FRIEND_TAB_FRIENDS=1 (friend/ignore list), FRIEND_TAB_WHO=2 (/who results), \
         FRIEND_TAB_RAID=3 (raid composition shortcut), FRIEND_TAB_QUICK_JOIN=4 \
         (quick-join group recommendations). FRIEND_TAB_COUNT=4 lets callers iterate \
         without hard-coding the count"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_squelch_types(env: &WowLuaEnv) {

    let squelch: (i64, i64) = env
        .eval(
            "return SQUELCH_TYPE_IGNORE, \
                    SQUELCH_TYPE_BLOCK_INVITE",
        )
        .expect("SQUELCH_TYPE_* probe should succeed");
    assert_eq!(
        squelch,
        (1, 2),
        "FriendsFrame.lua declares the two-state squelch enum used by the ignore/block-invite UI. \
         FriendsFriendsViewType is deliberately local to FriendsFriendsFrame.lua and is not a \
         public runtime global."
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_status_texture_path_globals(env: &WowLuaEnv) {

    let textures: (String, String, String, String, String) = env
        .eval(
            "return FRIENDS_TEXTURE_ONLINE, \
                    FRIENDS_TEXTURE_AFK, \
                    FRIENDS_TEXTURE_DND, \
                    FRIENDS_TEXTURE_OFFLINE, \
                    FRIENDS_TEXTURE_BROADCAST",
        )
        .expect("FRIENDS_TEXTURE_* probe should succeed");
    assert_eq!(
        textures,
        (
            "Interface\\FriendsFrame\\StatusIcon-Online".to_string(),
            "Interface\\FriendsFrame\\StatusIcon-Away".to_string(),
            "Interface\\FriendsFrame\\StatusIcon-DnD".to_string(),
            "Interface\\FriendsFrame\\StatusIcon-Offline".to_string(),
            "Interface\\FriendsFrame\\BroadcastIcon".to_string(),
        ),
        "FriendsFrame.lua lines 22-26 declare the five status-icon texture paths drawn \
         next to each friend row: Online (green dot), AFK (yellow Z), DnD (red bar), \
         Offline (grey dot), and the megaphone Broadcast indicator for accounts with a \
         BattleNet status message set. These are file paths, not atlas keys — resolved \
         by the texture loader's BLP/webp fallback chain"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_subframes_table(env: &WowLuaEnv) {

    let subframes: (String, String, String, String, String, String, i64) = env
        .eval(
            "return FRIENDSFRAME_SUBFRAMES[1], \
                    FRIENDSFRAME_SUBFRAMES[2], \
                    FRIENDSFRAME_SUBFRAMES[3], \
                    FRIENDSFRAME_SUBFRAMES[4], \
                    FRIENDSFRAME_SUBFRAMES[5], \
                    FRIENDSFRAME_SUBFRAMES[6], \
                    #FRIENDSFRAME_SUBFRAMES",
        )
        .expect("FRIENDSFRAME_SUBFRAMES probe should succeed");
    assert_eq!(
        subframes,
        (
            "FriendsListFrame".to_string(),
            "QuickJoinFrame".to_string(),
            "RecentAlliesFrame".to_string(),
            "WhoFrame".to_string(),
            "RecruitAFriendFrame".to_string(),
            "RaidFrame".to_string(),
            6,
        ),
        "FriendsFrame.lua line 63 declares FRIENDSFRAME_SUBFRAMES — the array of \
         frame-name strings that FriendsFrame_ShowSubFrame iterates to mutually \
         exclude. FriendsListFrame / RecentAlliesFrame / WhoFrame are defined in this \
         addon, while QuickJoinFrame / RecruitAFriendFrame / RaidFrame come from \
         sibling addons (Blizzard_QuickJoin / Blizzard_RecruitAFriend / Blizzard_RaidUI). \
         The list must include all six because show/hide is name-based, not \
         load-state-based"
    );

    let plunderstorm: String = env
        .eval("return FRIENDSFRAME_PLUNDERSTORM_SUBFRAMES[1]")
        .expect("FRIENDSFRAME_PLUNDERSTORM_SUBFRAMES probe should succeed");
    assert_eq!(
        plunderstorm, "FriendsListFrame",
        "FriendsFrame.lua line 64 declares FRIENDSFRAME_PLUNDERSTORM_SUBFRAMES = \
         {{\"FriendsListFrame\"}} — the trimmed sub-frame list for Plunderstorm \
         seasonal mode (no /who, no raid recruit, no quick-join — only the friend list \
         is meaningful in PvP brawl mode)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_show_sub_frame_helper(env: &WowLuaEnv) {

    let helpers: (bool, bool, bool, bool) = env
        .eval(
            "return type(FriendsFrame_ShowSubFrame) == 'function', \
                    type(FriendsFrame_OnLoad) == 'function', \
                    type(FriendsFrame_OnEvent) == 'function', \
                    type(FriendsFrame_Update) == 'function'",
        )
        .expect("FriendsFrame_* helper probe should succeed");
    assert_eq!(
        helpers,
        (true, true, true, true),
        "FriendsFrame.lua publishes the panel-level helper functions: \
         FriendsFrame_ShowSubFrame(name) (line 65 — iterates FRIENDSFRAME_SUBFRAMES, \
         hides all but `name`), FriendsFrame_OnLoad (line 231 — registers ~30 events: \
         FRIENDLIST_UPDATE / BN_FRIEND_INFO_CHANGED / WHO_LIST_UPDATE / \
         IGNORELIST_UPDATE / ...), FriendsFrame_OnEvent (line 1172 — central event \
         dispatcher), FriendsFrame_Update (line 423 — refresh-all entry point)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_friends_frame_publishes_dropdown_helpers(env: &WowLuaEnv) {

    let dropdowns: (bool, bool) = env
        .eval(
            "return type(FriendsFrame_ShowDropdown) == 'function', \
                    type(FriendsFrame_ShowBNDropdown) == 'function'",
        )
        .expect("FriendsFrame dropdown probe should succeed");
    assert_eq!(
        dropdowns,
        (true, true),
        "FriendsFrame.lua publishes two right-click context-menu builders consumed by \
         every chat frame's friend-name link path: FriendsFrame_ShowDropdown (line 183 \
         — WoW character row menu: invite / message / unfriend), \
         FriendsFrame_ShowBNDropdown (line 207 — BattleNet account row menu adds \
         block / report / target-game-account submenu)"
    );
}
}
