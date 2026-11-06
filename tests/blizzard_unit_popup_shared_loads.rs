use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn unit_popup_shared_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_UnitPopupShared")
}

fn unit_popup_shared_toc() -> PathBuf {
    find_toc_file(&unit_popup_shared_dir()).expect("UnitPopupShared TOC resolves")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &["Blizzard_SharedXML", "Blizzard_Colors"];

// The unified PTR TOC turns the two Retail optional dependencies into hard
// `## Dep` entries, keeping their screen/game-type annotations in place.
const PTR_TOC_DEPENDENCIES: &[&str] = &[
    "Blizzard_SharedXML",
    "Blizzard_Colors",
    "Blizzard_GlueXML",
    "Blizzard_FrameXMLBase",
];

fn toc_dependencies() -> &'static [&'static str] {
    if toc_layout::unified_toc_tree() {
        PTR_TOC_DEPENDENCIES
    } else {
        TOC_DEPENDENCIES
    }
}

const TOC_OPTIONAL_DEPS: &[&str] = &["Blizzard_GlueXML", "Blizzard_FrameXMLBase"];

const REPRESENTATIVE_BODY_FILES: &[&str] = &[
    "UnitPopupSharedUtils.lua",
    "UnitPopupSharedButtonMixins.lua",
    "Mainline\\UnitPopupSharedButtonMixins.lua",
    "UnitPopupShared.lua",
    "UnitPopupSharedMenus.lua",
];

const PTR_REPRESENTATIVE_BODY_FILES: &[&str] = &[
    "UnitPopupSharedUtils.lua",
    "UnitPopupSharedButtonMixins.lua",
    "[Family]\\UnitPopupSharedButtonMixins.lua",
    "UnitPopupShared.lua",
    "UnitPopupSharedMenus.lua",
];

// The classic TOC carries the four root files with no Mainline overlay.
const CLASSIC_REPRESENTATIVE_BODY_FILES: &[&str] = &[
    "UnitPopupSharedUtils.lua",
    "UnitPopupSharedButtonMixins.lua",
    "UnitPopupShared.lua",
    "UnitPopupSharedMenus.lua",
];

fn representative_body_files() -> &'static [&'static str] {
    if toc_layout::classic_cache() {
        return CLASSIC_REPRESENTATIVE_BODY_FILES;
    }
    if toc_layout::unified_toc_tree() {
        PTR_REPRESENTATIVE_BODY_FILES
    } else {
        REPRESENTATIVE_BODY_FILES
    }
}

const REPRESENTATIVE_BUTTON_BASE_METHODS: &[&str] = &[
    "GetEntries",
    "IsDisabledInKioskMode",
    "IsEnabled",
    "CanShow",
    "IsChecked",
    "GetText",
    "OnClick",
    "IsTitle",
    "IsDivider",
    "GetTooltipText",
    "ShouldPollTooltip",
    "IsInlineMenu",
    "CreateMenuDescription",
];

const REPRESENTATIVE_BUTTON_MIXINS: &[&str] = &[
    "UnitPopupAttachFrameMixin",
    "UnitPopupCheckboxButtonMixin",
    "UnitPopupRadioButtonMixin",
    "UnitPopupTradeButtonMixin",
    "UnitPopupWhisperButtonMixin",
    "UnitPopupInviteButtonMixin",
    "UnitPopupUninviteButtonMixin",
    "UnitPopupFriendsButtonMixin",
    "UnitPopupRemoveFriendButtonMixin",
    "UnitPopupSetNoteButtonMixin",
    "UnitPopupReportButtonMixin",
    "UnitPopupReportFriendButtonMixin",
    "UnitPopupRaidTargetBaseMixin",
    "UnitPopupRaidTarget1ButtonMixin",
    "UnitPopupSelfHighlightCommonMixin",
    "UnitPopupChatPromoteButtonMixin",
    "UnitPopupGarrisonVisitButtonMixin",
    "UnitPopupVoiceChatButtonMixin",
    "UnitPopupEnterEditModeMixin",
    "UnitPopupSubsectionTitleMixin",
    "UnitPopupSubsectionSeperatorMixin",
];

const REPRESENTATIVE_GLUE_MIXINS: &[&str] = &[
    "UnitPopupGlueInviteButtonMixin",
    "UnitPopupGlueLeavePartyButton",
    "UnitPopupGlueRemovePartyButton",
];

const REPRESENTATIVE_REGISTERED_MENU_NAMES: &[&str] = &[
    "SELF",
    "PARTY",
    "PLAYER",
    "ENEMY_PLAYER",
    "RAID_PLAYER",
    "RAID",
    "FRIEND",
    "BN_FRIEND",
    "GLUE_FRIEND",
    "GUILD",
    "TARGET",
    "FOCUS",
    "COMMUNITIES_WOW_MEMBER",
    "COMMUNITIES_GUILD_MEMBER",
    "RAID_TARGET_ICON",
];

const INLINE_SUBMENU_TABLES: &[&str] = &[
    "UnitPopupMenuFriendlyPlayer",
    "UnitPopupMenuFriendlyPlayerInteract",
    "UnitPopupMenuFriendlyPlayerInviteOptions",
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

// Literal packaged TOC/dependency-directory/discovery snapshots only:
// Wrath manifest1-382/cache lacks these inputs; native popup behavior stays.
post_wrath_ui_fixture! {
#[test]
fn find_toc_file_resolves_mainline_variant() {
    let resolved = find_toc_file(&unit_popup_shared_dir()).expect("UnitPopupShared TOC resolves");
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some(toc_layout::expected_toc_name("Blizzard_UnitPopupShared").as_str()),
        "find_toc_file resolves the active profile's UnitPopupShared TOC: the Retail cache \
         ships only `_Mainline.toc`, while the pinned PTR cache ships the bare unified TOC \
         because the shared base is mainline-only"
    );
}

#[test]
fn toc_is_eager_with_two_dependencies_and_two_optional_deps() {
    let toc = TocFile::from_file(&unit_popup_shared_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` directive → eagerly loaded. The shared \
         base must be alive before Blizzard_UnitPopup loads — \
         UnitPopup's TOC declares `## Dependencies: \
         Blizzard_UnitPopupShared`, which forces this addon up the \
         load order regardless. Eager loading also matters for the \
         glue screens: `AllowLoad: both` makes this addon present on \
         Login/CharacterSelect/CharacterCreate, where it backs the \
         friend-list right-click menus that don't have a per-flavor \
         child addon"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        toc_dependencies(),
        "TOC must declare Blizzard_SharedXML and \
         Blizzard_Colors as hard deps; the unified PTR TOC also makes \
         Blizzard_GlueXML and Blizzard_FrameXMLBase hard dependencies. \
         Blizzard_SharedXML provides Mixin/CreateFromMixins, \
         MenuUtil.CreateContextMenu (the backing primitive for \
         UnitPopupManager:OpenMenu), and \
         PropertyButtonMixin/PropertySliderMixin (extended by the \
         voice and difficulty mixins); Blizzard_Colors publishes the \
         named CreateColor/CreateColorFromHexString helpers and the \
         RAID_CLASS_COLORS table consumed by OpenMenu's \
         class-colored title. Got: {deps:?}"
    );

    let opt_deps = toc.optional_deps();
    let expected_optional: &[&str] =
        if toc_layout::classic_cache() || toc_layout::unified_toc_tree() {
            &[]
        } else {
            TOC_OPTIONAL_DEPS
        };
    assert_eq!(
        opt_deps, expected_optional,
        "The Retail TOC declares Blizzard_GlueXML and \
         Blizzard_FrameXMLBase as optional deps; the unified PTR TOC \
         declares them as hard deps. The glue dep is consulted only on \
         glue screens; the FrameXMLBase dep gates enum/global names \
         that exist only when the mainline FrameXML core is present. \
         Got: {opt_deps:?}"
    );

    assert!(toc.load_with().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_both_includes_game_and_all_glue_screens() {
    let toc = TocFile::from_file(&unit_popup_shared_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: both` matches at toc.rs:307 → in-world Game \
         screen passes. The shared base hosts the unit-popup \
         primitives (UnitPopupButtonBaseMixin, RegisterMenu registry, \
         UnitPopupSharedUtil) consumed by the Game-only Blizzard_UnitPopup"
    );
    for screen in GLUE_SCREENS {
        assert!(
            toc.allows_screen(*screen),
            "`AllowLoad: both` must also allow glue screen {screen:?} \
             — toc.rs:307 returns true for all screens when the value \
             matches `both` case-insensitively. This is the FIRST \
             addon analyzed that exercises the both-screens branch: \
             the glue side uses the GLUE_FRIEND / GLUE_FRIEND_OFFLINE \
             / GLUE_PARTY_MEMBER menu trees registered in \
             UnitPopupSharedMenus.lua and the three glue-only mixins \
             in Mainline\\UnitPopupSharedButtonMixins.lua"
        );
    }
}

#[test]
fn allow_load_game_type_mainline_is_not_restricted() {
    let toc = TocFile::from_file(&unit_popup_shared_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: mainline` is in the accept-set at \
         toc.rs:299 ({{mainline, standard}}) → not restricted. The \
         shared base is mainline-only by design (classic-flavor \
         Blizzard_UnitPopup_Mists ships its own self-contained body \
         instead of depending on this addon), but on a mainline build \
         the directive's presence is harmless because mainline is \
         exactly what's running"
    );
}

#[test]
fn toc_raw_bytes_pin_directives_and_representative_body_files() {
    let raw = std::fs::read_to_string(unit_popup_shared_toc()).expect("TOC reads utf-8");

    let expected_directives: &[&str] = if toc_layout::classic_cache() {
        &[
            "## Title: Blizzard_UnitPopupShared",
            "## Author: Blizzard Entertainment",
            "## DefaultState: enabled",
            "## Dependencies: Blizzard_SharedXML, Blizzard_Colors",
            "## AllowLoad: Game",
            "## AllowLoadGameType: classic",
        ]
    } else if toc_layout::unified_toc_tree() {
        &[
            "## Title: Blizzard_UnitPopupShared",
            "## Dep: Blizzard_SharedXML",
            "## Dep: Blizzard_Colors",
            "## Dep: Blizzard_GlueXML [AllowLoad glue]",
            "## Dep: Blizzard_FrameXMLBase [AllowLoad game] [AllowLoadGameType mainline]",
            "## AllowLoad: both [AllowLoadGameType mainline]",
        ]
    } else {
        &[
            "## Title: Blizzard_UnitPopupShared",
            "## Author: Blizzard Entertainment",
            "## DefaultState: enabled",
            "## Dependencies: Blizzard_SharedXML, Blizzard_Colors",
            "## OptionalDeps: Blizzard_GlueXML, Blizzard_FrameXMLBase",
            "## AllowLoad: both",
            "## AllowLoadGameType: mainline",
        ]
    };

    for line in expected_directives {
        assert!(raw.contains(line), "Raw TOC must pin directive `{line}`");
    }

    for path in representative_body_files() {
        assert!(
            raw.contains(path),
            "Raw TOC must pin body path `{path}`. Body order is \
             load-critical: UnitPopupSharedUtils declares \
             UnitPopupSharedUtil first (the util namespace consumed \
             by every later button mixin's CanShow/IsEnabled); \
             UnitPopupSharedButtonMixins declares \
             UnitPopupButtonBaseMixin (chassis with default \
             implementations of GetEntries/IsTitle/CanShow/etc.) \
             then ~155 mixins extending it; \
             Mainline\\UnitPopupSharedButtonMixins overlays three \
             glue-screen mixins driven by C_WoWLabsMatchmaking; \
             UnitPopupShared declares UnitPopupManager + \
             UnitPopupMenus dispatch table + UnitPopup_OpenMenu free \
             function; UnitPopupSharedMenus calls \
             UnitPopupManager:RegisterMenu 36 times to seed the menu \
             registry that Blizzard_UnitPopup later augments"
        );
    }

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## LoadWith"));
    assert!(!raw.contains("## SavedVariables"));
}

#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_UnitPopupShared");
    assert!(
        found,
        "Blizzard_UnitPopupShared must appear in Game eager discovery \
         — the shared base must precede Blizzard_UnitPopup at \
         PLAYER_ENTERING_WORLD because UnitPopup's TOC declares it as \
         a hard dep"
    );
}

#[test]
fn appears_in_all_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_UnitPopupShared");
        assert!(
            found,
            "Blizzard_UnitPopupShared must appear on glue screen \
             {screen:?} — `AllowLoad: both` lets it through the \
             allows_screen filter at loader/mod.rs:527 for both Game \
             AND glue screens. This is the single addon in the unit \
             popup family that bridges glue and game; \
             Blizzard_UnitPopup is Game-only, but the friend-list \
             right-click menus on the character-select screen need \
             this shared base to provide the menu tables and button \
             mixins"
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

#[test]
fn optional_dep_directories_exist_on_disk() {
    for dep in TOC_OPTIONAL_DEPS {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Optional-dep directory `{dep}` must exist on disk — \
             both Blizzard_GlueXML and Blizzard_FrameXMLBase ship \
             with the mainline build, so the optional declaration is \
             a soft hint about load ordering rather than a true \
             may-be-absent guard"
        );
    }
}

}

prefork_full_ui_case! {
fn full_game_load_publishes_unit_popup_manager(env: &WowLuaEnv) {

    let manager_kind: String = env
        .eval("return type(UnitPopupManager)")
        .unwrap_or_else(|err| panic!("UnitPopupManager probe failed: {err}"));
    assert_eq!(
        manager_kind, "table",
        "UnitPopupManager must be a global table after load. \
         UnitPopupShared.lua declares `UnitPopupManager = {{ }}` and \
         attaches three methods: OpenMenu(which, contextData) which \
         delegates to MenuUtil.CreateContextMenu with a class-colored \
         title; GetMenu(which) which returns the registered menu \
         table; RegisterMenu(name, table) which stores the menu in \
         UnitPopupMenus[name]. This manager is the dispatch root for \
         every right-click unit menu in the UI"
    );

    let menus_kind: String = env
        .eval("return type(UnitPopupMenus)")
        .unwrap_or_else(|err| panic!("UnitPopupMenus probe failed: {err}"));
    assert_eq!(
        menus_kind, "table",
        "UnitPopupMenus must be a global table after load. It is the \
         backing dispatch table populated by RegisterMenu calls \
         during this addon's UnitPopupSharedMenus.lua AND later \
         augmented by Blizzard_UnitPopup's Standard\\UnitPopupMenus.lua"
    );

    let util_kind: String = env
        .eval("return type(UnitPopupSharedUtil)")
        .unwrap_or_else(|err| panic!("UnitPopupSharedUtil probe failed: {err}"));
    assert_eq!(
        util_kind, "table",
        "UnitPopupSharedUtil must be a global table after load. \
         UnitPopupSharedUtils.lua declares the namespace and seeds it \
         with two kinds of methods: stub functions returning nil that \
         Blizzard_UnitPopup overrides (GetBNetIDAccount, \
         GetBNetAccountInfo, TryCreatePlayerLocation, GetFullPlayerName) \
         and PROJECT_IMPL_REQUIRED-erroring placeholders that the \
         augmenting addon must replace (IsBNetFriend, CanAddBNetFriend, \
         HasLFGRestrictions). The namespace also publishes the \
         shared-impl utilities (GetGUID, IsSameServer, CanCooperate, \
         IsPlayerOffline, IsPlayerFavorite, etc.) used by every \
         button mixin's CanShow / IsEnabled implementation"
    );

    let toplevel_mixin_kind: String = env
        .eval("return type(UnitPopupTopLevelMenuMixin)")
        .unwrap_or_else(|err| panic!("UnitPopupTopLevelMenuMixin probe failed: {err}"));
    assert_eq!(
        toplevel_mixin_kind, "table",
        "UnitPopupTopLevelMenuMixin must be a global table after \
         load. UnitPopupSharedMenus.lua declares it with \
         IsInlineMenu()=true and AssembleMenuEntries() that flattens \
         inline submenu tables (UnitPopupMenuFriendlyPlayer etc.) \
         into the parent menu's entry list. Every \
         RegisterMenu-registered table is created via \
         CreateFromMixins(UnitPopupTopLevelMenuMixin) so the \
         flattening behavior is uniform across the 36 root menus"
    );
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_button_base_mixin_with_default_methods(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(UnitPopupButtonBaseMixin)")
        .unwrap_or_else(|err| panic!("UnitPopupButtonBaseMixin probe failed: {err}"));
    assert_eq!(
        kind, "table",
        "UnitPopupButtonBaseMixin must be a global table after load. \
         UnitPopupSharedButtonMixins.lua line 94 declares the chassis \
         that ~155 sibling mixins extend via CreateFromMixins. It \
         carries default implementations for every method the menu \
         dispatcher might call so subclasses only override what they \
         change"
    );

    for method in REPRESENTATIVE_BUTTON_BASE_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(UnitPopupButtonBaseMixin.{method})"))
            .unwrap_or_else(|err| panic!("UnitPopupButtonBaseMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "UnitPopupButtonBaseMixin.{method} must be a function. \
             The base mixin's default impl is what makes \
             CreateFromMixins(UnitPopupButtonBaseMixin) sufficient \
             for trivial menu entries — leaf mixins only override \
             the handful of methods that change behavior, relying \
             on the chassis for the rest"
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
             UnitPopupSharedButtonMixins.lua publishes 155 mixins \
             total. Direct UnitPopupButtonBaseMixin extensions cover \
             the bread-and-butter entries (Trade/Whisper/Invite/\
             Uninvite/Friends/Report etc.) plus the structural \
             primitives (AttachFrame for context-aware sub-frames, \
             Checkbox+Radio as toggle parents, SubsectionTitle and \
             SubsectionSeperator for menu-section dividers). \
             Multi-step inheritance chains include \
             RemoveFriend=CreateFromMixins(Friends), \
             SetNote=CreateFromMixins(Friends), then \
             RemoveBnetFriend=CreateFromMixins(RemoveFriend) and \
             SetBNetNote=CreateFromMixins(SetNote); also the radio \
             family RaidTarget1=CreateFromMixins(RaidTargetBase) → \
             RaidTarget2..8=CreateFromMixins(RaidTarget1) and the \
             SelfHighlightCommon=CreateFromMixins(Checkbox) → \
             SelfHighlightCircle/Icon=CreateFromMixins(SelfHighlightCommon) \
             pairs"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_glue_overlay_mixins(env: &WowLuaEnv) {

    for mixin in REPRESENTATIVE_GLUE_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. \
             Mainline\\UnitPopupSharedButtonMixins.lua loads AFTER \
             the cross-flavor button mixins file (per TOC body order) \
             and adds three glue-only entries that drive WoW Labs \
             party flow: UnitPopupGlueInviteButtonMixin uses \
             C_WoWLabsMatchmaking.SendPartyInvite + IsPartyFull; \
             UnitPopupGlueLeavePartyButton uses LeaveParty + \
             IsAloneInWoWLabsParty; UnitPopupGlueRemovePartyButton \
             uses RemovePlayerFromParty + IsPartyLeader. All three \
             extend UnitPopupButtonBaseMixin directly. Even on a Game \
             load these mixins exist (the Mainline overlay always \
             runs on mainline builds) — they just have no live menu \
             host until the player switches to the WoW Labs glue \
             flow"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_registers_root_menus_via_unit_popup_manager(env: &WowLuaEnv) {

    for menu in REPRESENTATIVE_REGISTERED_MENU_NAMES {
        let kind: String = env
            .eval(&format!("return type(UnitPopupMenus[\"{menu}\"])"))
            .unwrap_or_else(|err| panic!("UnitPopupMenus[{menu}] probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "UnitPopupMenus[\"{menu}\"] must be a registered table \
             after load. UnitPopupSharedMenus.lua calls \
             UnitPopupManager:RegisterMenu(name, table) 36 times to \
             seed the dispatch registry. Some entries (PARTY, \
             ENEMY_PLAYER, RAID_PLAYER, COMMUNITIES_WOW_MEMBER, \
             COMMUNITIES_GUILD_MEMBER) error with PROJECT_IMPL_REQUIRED \
             until Blizzard_UnitPopup overrides their GetEntries; \
             others (BN_FRIEND, BN_FRIEND_OFFLINE) return nil from \
             the shared base and rely on the augmenting addon to \
             provide real entries; the rest (SELF, PLAYER, RAID, \
             FRIEND, GUILD, TARGET, FOCUS, RAID_TARGET_ICON, \
             GLUE_FRIEND etc.) are fully implemented here. Either \
             way, the table must exist in the registry so \
             UnitPopupManager:GetMenu can find it"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_inline_submenus(env: &WowLuaEnv) {

    for submenu in INLINE_SUBMENU_TABLES {
        let kind: String = env
            .eval(&format!("return type({submenu})"))
            .unwrap_or_else(|err| panic!("{submenu} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{submenu} must be a global table after load. The three \
             FriendlyPlayer inline submenus are NOT registered with \
             UnitPopupManager:RegisterMenu — they are nested entries \
             inside other menus' GetEntries lists (e.g. \
             UnitPopupMenuParty's entries include \
             UnitPopupMenuFriendlyPlayer as a submenu). \
             UnitPopupTopLevelMenuMixin.AssembleMenuEntries() detects \
             these inline submenus during menu assembly and flattens \
             their entries into the parent. They must exist as \
             globals because the parent menus reference them by name"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_UnitPopupShared/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load must emit zero \
         Blizzard_UnitPopupShared body errors. The 5 active body \
         files (~4620 lua lines, dominated by \
         UnitPopupSharedButtonMixins.lua at ~3673 lines / 155 \
         mixins) span the util namespace, the chassis + 155 \
         button-mixin family, the three glue-overlay mixins, the \
         UnitPopupManager + UnitPopup_OpenMenu wiring, and 36 \
         RegisterMenu calls. Any failure breaks every right-click \
         unit popup AND the friend-list right-click menus on glue \
         screens. Found: {addon_specific:?}"
    );
}
}
