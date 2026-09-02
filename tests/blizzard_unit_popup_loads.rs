use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn unit_popup_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_UnitPopup")
}

fn unit_popup_toc() -> PathBuf {
    unit_popup_dir().join("Blizzard_UnitPopup_Mainline.toc")
}

fn unit_popup_mists_toc() -> PathBuf {
    unit_popup_dir().join("Blizzard_UnitPopup_Mists.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &["Blizzard_UnitPopupShared", "Blizzard_SocialUIShared"];

const REPRESENTATIVE_BODY_FILES: &[&str] = &[
    "UnitPopupSlider.lua",
    "UnitPopupSlider.xml",
    "UnitPopupCustomControls.lua",
    "UnitPopupCustomControls.xml",
    "Mainline\\UnitPopupUtils.lua",
    "Mainline\\UnitPopup.lua",
];

const FAMILY_PLACEHOLDER_BODY_FILE: &str = "Mainline\\UnitPopup.lua";
const FAMILY_PLACEHOLDER_RESOLVED_RELATIVE: &str = "Mainline/UnitPopup.lua";

const GAME_PLACEHOLDER_BODY_FILES: &[&str] = &[
    "[Game]\\UnitPopupButtons.lua [AllowLoadGameType standard, plunderstorm, wowhack]",
    "[Game]\\UnitPopupMenus.lua [AllowLoadGameType standard, plunderstorm, wowhack]",
];

const GAME_PLACEHOLDER_RESOLVED_RELATIVES: &[&str] = &[
    "Standard/UnitPopupButtons.lua",
    "Standard/UnitPopupMenus.lua",
];

const REPRESENTATIVE_VOICE_MIXINS: &[&str] = &[
    "UnitPopupAttachableFrameMixin",
    "UnitPopupVoiceMemberInfoMixin",
    "UnitPopupVoiceToggleButtonMixin",
    "UnitPopupVoiceLevelsMixin",
    "UnitPopupToggleMuteMixin",
    "UnitPopupToggleDeafenMixin",
    "UnitPopupToggleUserMuteMixin",
    "UnitPopupVoiceMicrophoneVolumeSliderMixin",
    "UnitPopupVoiceSpeakerVolumeSliderMixin",
    "UnitPopupVoiceUserVolumeSliderMixin",
    "UnitPopupSliderMixin",
];

const REPRESENTATIVE_BUTTON_MIXINS: &[&str] = &[
    "UnitPopupBnetAddFavoriteButtonMixin",
    "UnitPopupBnetRemoveFavoriteButtonMixin",
    "UnitPopupDungeonDifficulty3ButtonMixin",
    "UnitPopupRafRemoveRecruitButtonMixin",
    "UnitPopupGuildSettingButtonMixin",
    "UnitPopupGuildRecruitmentSettingButtonMixin",
    "UnitPopupGuildInviteButtonMixin",
    "UnitPopupLootMethodButtonMixin",
    "UnitPopupLootFreeForAllButtonMixin",
    "UnitPopupLootRoundRobinButtonMixin",
    "UnitPopupMasterLooterButtonMixin",
    "UnitPopupGroupLootButtonMixin",
    "UnitPopupNeedBeforeGreedButtonMixin",
];

const REPRESENTATIVE_TOPLEVEL_MENUS: &[&str] = &[
    "UnitPopupMenuSelf",
    "UnitPopupMenuParty",
    "UnitPopupMenuEnemyPlayer",
    "UnitPopupMenuRaidPlayer",
    "UnitPopupMenuBnFriend",
    "UnitPopupMenuBnFriendOffline",
    "UnitPopupMenuCommunitiesWowMember",
    "UnitPopupMenuCommunitiesGuildMember",
    "UnitPopupRafRecruit",
];

const REPRESENTATIVE_VIRTUAL_TEMPLATES: &[&str] = &[
    "UnitPopupSliderTemplate",
    "UnitPopupVoiceToggleButtonTemplate",
    "UnitPopupVoiceSliderTemplate",
    "UnitPopupVoiceTextTemplate",
    "UnitPopupVoiceLevelsTemplate",
    "UnitPopupVoiceSpeakerVolumeTemplate",
    "UnitPopupVoiceMicrophoneVolumeTemplate",
    "UnitPopupVoiceUserVolumeTemplate",
];

fn fresh_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = fresh_game_env();

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn find_toc_file_resolves_mainline_variant() {
    let resolved = find_toc_file(&unit_popup_dir()).expect("UnitPopup TOC resolves");
    assert_eq!(
        resolved,
        unit_popup_toc(),
        "find_toc_file at src/loader/mod.rs:65-95 prefers \
         `<addon>_Mainline.toc`. Blizzard_UnitPopup ships TWO TOCs \
         (Mainline + Mists, NOT a generic `_Classic.toc` like \
         Blizzard_UnitFrame): the Mainline variant declares one hard \
         dep (Blizzard_UnitPopupShared) and 8 body lines arranged \
         across the addon root + Mainline/Standard subdirs, while the \
         Mists companion adds Blizzard_VoiceToggleButton as a second \
         dep, pulls Wrath/Classic/TBC body files, and carries \
         `## AllowLoadGameType: mists` so it's filtered out on a \
         mainline build"
    );
}

#[test]
fn toc_is_eager_with_one_dependency() {
    let toc = TocFile::from_file(&unit_popup_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` directive → eagerly loaded. The unit \
         popup menus are wired into Blizzard_FrameXML's right-click \
         dispatch path; UnitPopup must be alive at PLAYER_ENTERING_WORLD \
         so right-clicking a player frame, party member, or chat link \
         can show the popup without a load stall"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps, TOC_DEPENDENCIES,
        "TOC must declare exactly Blizzard_UnitPopupShared as a hard \
         dep. Blizzard_UnitPopupShared publishes the cross-flavor base \
         tables (UnitPopupButtonBaseMixin, UnitPopupRadioButtonMixin, \
         UnitPopupSharedUtil util namespace, UnitPopupManager registry) \
         that this addon's local mixins extend via CreateFromMixins(\
         UnitPopupButtonBaseMixin) and that the menu definitions \
         augment in-place. Got: {deps:?}"
    );

    assert!(toc.optional_deps().is_empty());
    assert!(toc.load_with().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_game_restricts_to_in_world() {
    let toc = TocFile::from_file(&unit_popup_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase here, but matched \
         case-insensitively at toc.rs:308 via eq_ignore_ascii_case) \
         hits → Game-only. Right-click unit popups are unit-targeted; \
         glue screens (Login, CharacterSelect, CharacterCreate) have \
         no UnitId concept and no chat/party plumbing"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — `AllowLoad: game` \
             matches only the Game variant via toc.rs:308"
        );
    }
}

#[test]
fn no_allow_load_game_type_means_unrestricted() {
    let toc = TocFile::from_file(&unit_popup_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "Mainline TOC has no `## AllowLoadGameType` directive — \
         is_game_type_restricted() at toc.rs:294-302 returns false \
         when the metadata key is absent (unwrap_or(false) branch). \
         The mainline build loads it without flavor filtering"
    );
}

#[test]
fn mists_companion_is_restricted_on_mainline_build() {
    let mists = TocFile::from_file(&unit_popup_mists_toc()).expect("Mists TOC parses");

    assert!(
        mists.is_game_type_restricted(),
        "Companion `_Mists.toc` carries `## AllowLoadGameType: mists` \
         which hits toc.rs:294-302 as a non-{{standard,mainline}} value \
         → restricted on a mainline build. The Mists body files \
         (Wrath\\UnitPopupButtons.lua, Wrath\\UnitPopupMenus.lua, \
         TBC\\UnitPopup.lua, Classic\\UnitPopupUtils.lua) only matter \
         on Classic-flavor Mists builds; the Mainline build silently \
         skips this TOC at discovery time"
    );

    assert_eq!(
        mists.dependencies(),
        vec![
            "Blizzard_UnitPopupShared".to_string(),
            "Blizzard_VoiceToggleButton".to_string(),
        ],
        "Mists companion adds Blizzard_VoiceToggleButton as a second \
         hard dep — the Mainline build folds the voice toggle into \
         Blizzard_UnitPopupShared's voice plumbing, but Classic Mists \
         keeps it as a separately-loaded addon"
    );
}

#[test]
fn toc_raw_bytes_pin_directives_and_representative_body_files() {
    let raw = std::fs::read_to_string(unit_popup_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_UnitPopup",
        "## Author: Blizzard Entertainment",
        "## DefaultState: enabled",
        "## Dependencies: Blizzard_UnitPopupShared",
        "## AllowLoad: game",
    ];

    for line in expected_directives {
        assert!(raw.contains(line), "Raw TOC must pin directive `{line}`");
    }

    for path in REPRESENTATIVE_BODY_FILES {
        assert!(
            raw.contains(path),
            "Raw TOC must pin body path `{path}`. Body order layers \
             root-level shared XML/Lua (UnitPopupSlider, \
             UnitPopupCustomControls — used cross-flavor) BEFORE \
             family-specific Mainline\\ files. Mainline\\UnitPopupUtils \
             augments UnitPopupSharedUtil with mainline-only BNet \
             helpers; Mainline\\UnitPopup defines the RAID_TOGGLE_MAP \
             table and SetRaidDifficulties() that the dungeon-difficulty \
             button mixins call"
        );
    }

    for path in GAME_PLACEHOLDER_BODY_FILES {
        assert!(
            raw.contains(path),
            "Raw TOC must pin `[Game]` body line `{path}`. The \
             `[Game]` placeholder is substituted at toc.rs:146 to the \
             build's game directory (`Standard` for the Mainline \
             build). The inline `[AllowLoadGameType standard, \
             plunderstorm, wowhack]` annotation is then evaluated at \
             toc.rs:141 via is_allowed_game_type — `standard` is in \
             the allowed-set at toc.rs:56 ({{mainline, standard}}), so \
             the line passes the filter and the resolved \
             Standard\\UnitPopupButtons.lua / UnitPopupMenus.lua get \
             loaded"
        );
    }

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## LoadWith"));
    assert!(!raw.contains("## OptionalDeps"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(
        !raw.contains("## AllowLoadGameType"),
        "Mainline TOC must not carry an AllowLoadGameType directive — \
         the absence is what makes is_game_type_restricted() return \
         false; presence with anything other than mainline/standard \
         would skip the TOC at discovery"
    );
}

#[test]
fn family_placeholder_resolves_to_mainline_directory() {
    let resolved = unit_popup_dir().join(FAMILY_PLACEHOLDER_RESOLVED_RELATIVE);
    assert!(
        resolved.is_file(),
        "`{FAMILY_PLACEHOLDER_BODY_FILE}` resolves to \
         `{resolved:?}` on disk. Mainline\\UnitPopup.lua holds \
         RAID_TOGGLE_MAP plus SetRaidDifficulties() — the Mainline \
         build's body file must exist at the resolved path or \
         load_addon will fail with a missing-file error"
    );
}

#[test]
fn game_placeholder_resolves_to_standard_directory() {
    for relative in GAME_PLACEHOLDER_RESOLVED_RELATIVES {
        let resolved = unit_popup_dir().join(relative);
        assert!(
            resolved.is_file(),
            "`[Game]` placeholder must resolve to existing Standard\\ \
             body file at {resolved:?}. toc.rs:146 substitutes \
             `[Game]` → `Standard` literally; the Standard subdir holds \
             the cross-build button/menu definitions used by retail \
             mainline AND plunderstorm/wowhack flavors. The \
             WoWLabs/WoWHack subdirs each carry their own copies but \
             those `[AllowLoadGameType ...]` filters are written so \
             only Standard variants load on a mainline build"
        );
    }
}

#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons.iter().any(|(name, _)| name == "Blizzard_UnitPopup");
    assert!(
        found,
        "Blizzard_UnitPopup must appear in Game eager discovery — the \
         right-click unit popup is wired into PlayerFrame, TargetFrame, \
         CompactRaidFrame, ChatFrame chat-link clicks, and \
         CommunitiesFrame member rows; it must be alive at \
         PLAYER_ENTERING_WORLD"
    );
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_UnitPopup");
        assert!(
            !found,
            "Blizzard_UnitPopup must NOT appear on {screen:?} — \
             AllowLoad:game restricts to in-world via toc.rs:308, \
             checked at loader/mod.rs:527 BEFORE pool partitioning"
        );
    }
}

#[test]
fn dep_directories_exist_on_disk() {
    for dep in TOC_DEPENDENCIES {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Hard-dep directory `{dep}` must exist on disk"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "{dep} must have a discoverable TOC"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_publishes_voice_mixins(env: &WowLuaEnv) {

    for mixin in REPRESENTATIVE_VOICE_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. The voice \
             mixin family powers the in-popup voice-volume sliders + \
             mute/deafen toggles: UnitPopupAttachableFrameMixin is the \
             contextData-aware base; UnitPopupVoiceLevels = \
             CreateFromMixins(UnitPopupAttachableFrame) hosts the \
             slider+toggle pair and registers \
             VOICE_CHAT_MUTED_CHANGED / VOICE_CHAT_DEAFENED_CHANGED / \
             VOICE_CHAT_CHANNEL_MEMBER_* events on show; the three \
             VolumeSlider mixins (Microphone/Speaker/User) bind \
             accessor+mutator to C_VoiceChat.{{Get,Set}}{{Input,Output, \
             Member}}Volume; the three Toggle mixins \
             (Mute/Deafen/UserMute) wire RegisterStateUpdateEvent to \
             keep the UI synced with backend voice state changes"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_button_mixins(env: &WowLuaEnv) {

    for mixin in REPRESENTATIVE_BUTTON_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. \
             Standard\\UnitPopupButtons.lua publishes ~33 \
             menu-entry mixins: BnetAddFavorite/BnetRemoveFavorite \
             (with mutually-exclusive CanShow checks against \
             UnitPopupSharedUtil.IsPlayerFavorite); \
             DungeonDifficulty3 = CreateFromMixins(DungeonDifficulty1) \
             with overridden GetDifficultyID returning 23; \
             RafRemoveRecruit driving the CONFIRM_RAF_REMOVE_RECRUIT \
             popup; GuildSetting/GuildInvite/GuildRecruitmentSetting \
             gated on IsGuildLeader / CanGuildInvite / \
             C_ClubFinder.IsEnabled; the LootMethod hierarchy \
             (LootMethod parent → LootFreeForAll = \
             CreateFromMixins(UnitPopupRadioButtonMixin) → \
             LootRoundRobin/MasterLooter/GroupLoot/NeedBeforeGreed all \
             extending LootFreeForAll). The radio-button hierarchy is \
             the canonical CreateFromMixins chain pattern"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_top_level_menus(env: &WowLuaEnv) {

    for menu in REPRESENTATIVE_TOPLEVEL_MENUS {
        let kind: String = env
            .eval(&format!("return type({menu})"))
            .unwrap_or_else(|err| panic!("{menu} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{menu} must be a global table after load. \
             Standard\\UnitPopupMenus.lua augments 8 menu tables that \
             Blizzard_UnitPopupShared registers via \
             UnitPopupManager:RegisterMenu (UnitPopupMenuSelf, \
             UnitPopupMenuParty, UnitPopupMenuEnemyPlayer, \
             UnitPopupMenuRaidPlayer, UnitPopupMenuBnFriend, \
             UnitPopupMenuBnFriendOffline, \
             UnitPopupMenuCommunitiesWowMember, \
             UnitPopupMenuCommunitiesGuildMember) by overriding \
             GetEntries() to return the per-context menu-entry list. \
             UnitPopupRafRecruit is uniquely defined inline at \
             UnitPopupMenus.lua:169 via \
             `CreateFromMixins(UnitPopupTopLevelMenuMixin)` + \
             `UnitPopupManager:RegisterMenu(\"RAF_RECRUIT\", ...)` — \
             this is the only menu table this addon creates from \
             scratch rather than augmenting"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_registers_representative_virtual_templates(env: &WowLuaEnv) {
    let _env = env;

    for template in REPRESENTATIVE_VIRTUAL_TEMPLATES {
        let resolved = wow_ui_sim::xml::get_template(template);
        assert!(
            resolved.is_some(),
            "Virtual template `{template}` must be registered. \
             Blizzard_UnitPopup ships exactly 8 virtual templates \
             (every XML element in the addon is virtual=\"true\" — \
             zero non-virtual frames): UnitPopupSliderTemplate \
             (PropertySliderTemplate base, mixin=UnitPopupSliderMixin) \
             and the seven voice-control templates that compose into \
             the right-click voice-volume row. \
             UnitPopupVoiceLevelsTemplate is the parent shell; the \
             three concrete UnitPopupVoiceSpeakerVolume/\
             MicrophoneVolume/UserVolumeTemplate variants inherit it \
             and set per-instance Toggle/Slider mixins (Mute vs \
             Deafen vs UserMute, MicrophoneVolume vs SpeakerVolume vs \
             UserVolume slider). UnitPopupVoiceUserVolumeTemplate's \
             slider stacks two mixins \
             (UnitPopupVoiceUserVolumeSliderMixin + \
             UnitPopupVoiceMemberInfoMixin) — the only multi-mixin \
             slider in the addon"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_UnitPopup/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load must emit zero Blizzard_UnitPopup body \
         errors. The 8 active body files (~1306 lua+xml lines, modest \
         compared to UnitFrame's ~16868) span the slider/custom-control \
         XML pair, the voice mixins (CustomControls.lua), the \
         per-flavor Mainline overlay (UnitPopup.lua + UnitPopupUtils.lua), \
         and the Standard-flavor button/menu trees — any failure \
         breaks the right-click context menu on every frame in-world. \
         Found: {addon_specific:?}"
    );
}
}
