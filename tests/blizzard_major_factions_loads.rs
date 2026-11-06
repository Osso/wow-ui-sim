use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

#[path = "blizzard_major_factions_loads/support.rs"]
mod support;

use support::*;

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

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_major_factions_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&major_factions_dir()).expect("Blizzard_MajorFactions TOC should resolve");
    assert_eq!(
        resolved,
        major_factions_toc(),
        "Blizzard_MajorFactions ships exactly one bare TOC. Major factions are a \
         retail-only Dragonflight+ system (renown tracks, paragon rewards, expansion-gated \
         landing pages — none of these mechanics exist in Classic flavors), so the retail \
         tree carries one Blizzard_MajorFactions.toc with no flavor-suffixed variants — \
         `find_toc_file` resolves to the bare file after the `_Mainline.toc` lookup misses"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_major_factions_toc_declares_eager_load_with_default_allow_load_game() {
    let toc = TocFile::from_file(&major_factions_toc()).expect("Blizzard_MajorFactions TOC parses");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_MajorFactions declares `## LoadOnDemand: 0` — the parser only treats `1` as \
         deferred-load, so `0` keeps it on the eager path. The unlock + renown toast frames \
         must register `MAJOR_FACTION_UNLOCKED` / `MAJOR_FACTION_RENOWN_LEVEL_CHANGED` events \
         at boot so they can trigger TopBannerManager_Show the moment the player crosses an \
         unlock threshold; deferred-load would miss the first event in a session"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        toc.dependencies().is_empty(),
        "Zero `## Dependencies:` — the major-factions toasts depend on TopBannerManager / \
         EventRegistry / C_MajorFactions surfaces, but those live in shared XML / shared C \
         API tiers (no addon dep edge required). The landing-page templates are virtual \
         (`virtual=\"true\"`) and only get instantiated by Blizzard_GenericTraitUI when it \
         calls LandingPageMajorFactionList.Create — that consumer addon owns the dep edge"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — major-faction renown / unlock state is server-authoritative \
         (the player's reputation table is the source of truth, queried via C_MajorFactions \
         + C_Reputation), so no per-character or account-wide persistent state needs to live \
         on the client. The watch-faction CVar + majorFactionRenownMap CVar table cover the \
         small UI-side state"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC omits `## AllowLoadGameType:` — `is_game_type_restricted` returns false (the \
         default at src/toc.rs:294-302). Major factions are a retail-only system but the \
         exclusion is enforced via the absence of Classic-flavor TOC variants, NOT via an \
         explicit game-type marker"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_MajorFactions omits `## AllowLoad:` — the default branch at src/toc.rs:311 \
         routes to Game-screen-only when the directive is missing. Major factions only exist \
         in-world (renown progression, faction rewards, landing-page widgets); glue screens \
         have no concept of the player's reputation table"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_MajorFactions must NOT auto-discover on glue screen {screen:?} — \
             missing `## AllowLoad:` falls through to the default-Game branch at \
             src/toc.rs:311. Glue screens have no MAJOR_FACTION_UNLOCKED event source, so \
             loading the toast frames there would register listeners that can never fire"
        );
    }
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_major_factions_toc_lists_seven_files_in_load_order() {
    let toc = TocFile::from_file(&major_factions_toc()).expect("Blizzard_MajorFactions TOC parses");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        MAJOR_FACTIONS_TOC_FILES,
        "TOC body lists exactly 7 files in load order — landing templates XML first (pulls \
         in the templates .lua via `<Script file=\"...\"/>`), toasts XML second (pulls in \
         the celebration-banner mixin .lua), then unlock-toast .lua + .xml pair, then \
         renown-toast .lua + .xml pair, then Localization.lua last. The .lua-then-.xml \
         sibling-pair pattern for toasts ensures the mixin is published BEFORE the concrete \
         frame's `mixin=\"...\"` attribute resolves at XML parse time. The two leading \
         template XMLs use the cross-XML `<Script file=\"...\"/>` shortcut to keep mixin \
         + virtual-template pairs co-located"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_major_factions_directory_holds_ten_entries() {
    let entries = std::fs::read_dir(major_factions_dir())
        .expect("Blizzard_MajorFactions directory reads")
        .count();
    assert_eq!(
        entries, 10,
        "Directory holds exactly 10 entries — Blizzard_MajorFactions.toc + 7 TOC-listed \
         source files + 2 cross-XML-loaded .lua siblings (Blizzard_MajorFactionsLandingTemplates.lua \
         + Blizzard_MajorFactionToasts.lua). The two cross-XML .lua files are NOT listed in \
         the TOC body — they are pulled in by `<Script file=\"...\"/>` directives inside \
         their respective sibling .xml files"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_major_factions_auto_discovered_on_game_screen_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_MajorFactions");
    assert!(
        game_found,
        "Blizzard_MajorFactions must be auto-discovered on the Game screen — eager-load \
         (LoadOnDemand: 0) + default-AllowLoad-Game routes it into the eager `addons` set \
         during the Game-screen discovery sweep, NOT the lod_pool"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_MajorFactions");
        assert!(
            !found,
            "Blizzard_MajorFactions must NOT be auto-discovered on glue screen {screen:?} — \
             missing `## AllowLoad:` falls through to the default-Game branch. The major \
             factions UI has no purpose on character-select / login screens (no in-world \
             reputation context, no MAJOR_FACTION_* event source)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_MajorFactions")
                || message.contains("Blizzard_MajorFactionsLandingTemplates")
                || message.contains("Blizzard_MajorFactionToasts")
                || message.contains("Blizzard_MajorFactionUnlockToast")
                || message.contains("Blizzard_MajorFactionRenownToast")
                || message.contains("MajorFactionListMixin")
                || message.contains("MajorFactionButtonMixin")
                || message.contains("MajorFactionCelebrationBannerMixin")
                || message.contains("MajorFactionUnlockToastMixin")
                || message.contains("MajorFactionsRenownToastMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_MajorFactions emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_is_addon_loaded_after_auto_discovery(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MajorFactions')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_MajorFactions') must return true after the eager \
         auto-discovery sweep — proves the major-factions addon registers with the \
         loaded-set during the standard Game-screen boot pipeline, no explicit load_addon \
         call required"
    );
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_landing_page_helper_table(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(LandingPageMajorFactionList)")
        .expect("LandingPageMajorFactionList type probe succeeds");
    assert_eq!(
        kind, "table",
        "LandingPageMajorFactionList must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionsLandingTemplates.lua:28. Its single .Create(parent) static \
         method (line 30) is the canonical entry-point used by Blizzard_GenericTraitUI / \
         expansion landing pages to spawn a major-faction list scrollbox parented to the \
         supplied frame. The table is loaded via `<Script file=\"...\"/>` at the top of \
         Blizzard_MajorFactionsLandingTemplates.xml"
    );

    let create_kind: String = env
        .eval("return type(LandingPageMajorFactionList.Create)")
        .expect("LandingPageMajorFactionList.Create type probe succeeds");
    assert_eq!(
        create_kind, "function",
        "LandingPageMajorFactionList.Create must be a function — wraps CreateFrame with the \
         LandingPageMajorFactionListTemplate virtual template (declared at \
         Blizzard_MajorFactionsLandingTemplates.xml:5). The frameName parameter is hard-coded \
         to nil, so each instance is anonymous (no global registration)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_major_faction_list_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MajorFactionListMixin)")
        .expect("MajorFactionListMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MajorFactionListMixin must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionsLandingTemplates.lua:37. Consumed by the XML \
         `mixin=\"MajorFactionListMixin\"` attribute on LandingPageMajorFactionListTemplate \
         (templates XML line 5), which copies its 9 methods onto each list frame instance"
    );

    for method in MAJOR_FACTION_LIST_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(MajorFactionListMixin.{method})"))
            .unwrap_or_else(|err| panic!("MajorFactionListMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "MajorFactionListMixin.{method} must be a function — drives the major-faction \
             list scrollbox lifecycle (OnLoad seeds the WowScrollBoxList view, OnShow / \
             OnHide gate MAJOR_FACTION_UNLOCKED registration, Refresh re-pulls \
             C_MajorFactions.GetMajorFactionIDs and re-sorts by uiPriority, \
             OnRenownTrackFactionChanged + SetSelectedFaction track the EventRegistry \
             callback for cross-frame selection sync)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_major_faction_button_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MajorFactionButtonMixin)")
        .expect("MajorFactionButtonMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MajorFactionButtonMixin must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionsLandingTemplates.lua:149. Consumed by \
         `mixin=\"MajorFactionButtonMixin\"` on MajorFactionButtonTemplate (templates XML \
         line 30); each list element gets Init + UpdateState methods that drive the \
         locked / unlocked state swap and the per-expansion atlas dispatch"
    );

    for method in MAJOR_FACTION_BUTTON_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(MajorFactionButtonMixin.{method})"))
            .unwrap_or_else(|err| panic!("MajorFactionButtonMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "MajorFactionButtonMixin.{method} must be a function — Init carries the \
             per-faction data dispatch (atlas resolution from buttonAtlasFormatsByExpansion, \
             icon size lookup from factionIconSize, RenownProgressBar fill setup), \
             UpdateState toggles LockedState / UnlockedState child visibility based on \
             isUnlocked"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_locked_state_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MajorFactionButtonLockedStateMixin)")
        .expect("MajorFactionButtonLockedStateMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MajorFactionButtonLockedStateMixin must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionsLandingTemplates.lua:211. Consumed by `mixin=\"...\"` on the \
         LockedState child Frame (templates XML line 33). OnEnter shows the \
         unlock-description tooltip via GameTooltip_AddErrorLine, Refresh sets the faction \
         name title text"
    );

    for method in MAJOR_FACTION_BUTTON_LOCKED_STATE_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(MajorFactionButtonLockedStateMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("MajorFactionButtonLockedStateMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "MajorFactionButtonLockedStateMixin.{method} must be a function — wired to the \
             LockedState child via XML `<OnEnter method=\"...\"/>` / \
             `<OnLeave method=\"...\"/>` script hooks (templates XML lines 64-65). Refresh \
             is called from MajorFactionButtonMixin:Init (line 199)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_unlocked_state_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MajorFactionButtonUnlockedStateMixin)")
        .expect("MajorFactionButtonUnlockedStateMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MajorFactionButtonUnlockedStateMixin must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionsLandingTemplates.lua:236. Consumed by `mixin=\"...\"` on the \
         UnlockedState child Button (templates XML line 69). The bulkiest mixin in the \
         addon: drives renown progress display, watch-button hover reveal, click-to-toggle \
         renown panel, and dual tooltip dispatch (renown vs paragon)"
    );

    for method in MAJOR_FACTION_BUTTON_UNLOCKED_STATE_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(MajorFactionButtonUnlockedStateMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("MajorFactionButtonUnlockedStateMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "MajorFactionButtonUnlockedStateMixin.{method} must be a function — covers the \
             14-method surface: lifecycle (OnShow / OnHide / OnEvent), interaction (OnEnter \
             / OnLeave / OnClick / OnUpdate), state (SetSelected), tooltip dispatch \
             (RefreshTooltip + paragon vs renown variants), and unlock celebration \
             (PlayUnlockCelebration / StopUnlockCelebration — currently dead code, both \
             call sites are commented out at lines 261 / 270 but the methods stay live for \
             future re-enablement)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_renown_progress_bar_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MajorFactionRenownProgressBarMixin)")
        .expect("MajorFactionRenownProgressBarMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MajorFactionRenownProgressBarMixin must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionsLandingTemplates.lua:390. Consumed by `mixin=\"...\"` on the \
         RenownProgressBar Cooldown widget (templates XML line 139). The Cooldown frame is \
         the swirl-fill renown progress visual"
    );

    let update_kind: String = env
        .eval("return type(MajorFactionRenownProgressBarMixin.UpdateBar)")
        .expect("MajorFactionRenownProgressBarMixin.UpdateBar type probe succeeds");
    assert_eq!(
        update_kind, "function",
        "MajorFactionRenownProgressBarMixin.UpdateBar must be a function — divides current \
         by max and forwards to CooldownFrame_SetDisplayAsPercentage. The single-method \
         surface mirrors the single responsibility: drive the renown swirl fill from \
         (current, max) reputation values"
    );
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_watch_faction_button_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MajorFactionWatchFactionButtonMixin)")
        .expect("MajorFactionWatchFactionButtonMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MajorFactionWatchFactionButtonMixin must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionsLandingTemplates.lua:406. Consumed by `mixin=\"...\"` on the \
         WatchFactionButton CheckButton nested under UnlockedState (templates XML line 113). \
         The check-state drives C_Reputation.SetWatchedFactionByID + \
         StatusTrackingBarManager:UpdateBarsShown so the player's watched-bar reflects the \
         selection"
    );

    for method in MAJOR_FACTION_WATCH_FACTION_BUTTON_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(MajorFactionWatchFactionButtonMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("MajorFactionWatchFactionButtonMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "MajorFactionWatchFactionButtonMixin.{method} must be a function — OnLoad \
             positions the checkbox + label in the parent's top-right corner (with custom \
             totalWidth / padding math), OnShow / OnHide manage UPDATE_FACTION registration, \
             UpdateState reflects the watched-faction CVar, OnClick toggles the watch + \
             plays the OPTION_CHECKBOX sound"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_celebration_banner_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MajorFactionCelebrationBannerMixin)")
        .expect("MajorFactionCelebrationBannerMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MajorFactionCelebrationBannerMixin must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionToasts.lua:2. The base mixin attached to \
         MajorFactionCelebrationBannerTemplate (toasts XML line 5), inherited by both \
         MajorFactionUnlockToast + MajorFactionsRenownToast concrete frames. Loaded via \
         `<Script file=\"...\"/>` at the top of Blizzard_MajorFactionToasts.xml"
    );

    let texture_kit_kind: String = env
        .eval("return type(MajorFactionCelebrationBannerMixin.SetMajorFactionTextureKit)")
        .expect("MajorFactionCelebrationBannerMixin.SetMajorFactionTextureKit type probe succeeds");
    assert_eq!(
        texture_kit_kind, "function",
        "MajorFactionCelebrationBannerMixin.SetMajorFactionTextureKit must be a function — \
         dispatches the per-faction TopIcon atlas (`majorfactions_icons_<kit>512` format) \
         via SetupTextureKitOnFrames. Called by both subclass PlayBanner methods to swap \
         the banner's faction icon when a celebration toast fires"
    );
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_unlock_toast_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MajorFactionUnlockToastMixin)")
        .expect("MajorFactionUnlockToastMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MajorFactionUnlockToastMixin must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionUnlockToast.lua:2. Consumed by `mixin=\"...\"` on the named \
         MajorFactionUnlockToast frame (unlock-toast XML line 3). Drives the \
         MAJOR_FACTION_UNLOCKED event handler — the celebration toast that fires when the \
         player crosses an unlock threshold for a previously locked faction"
    );

    for method in MAJOR_FACTION_UNLOCK_TOAST_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(MajorFactionUnlockToastMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("MajorFactionUnlockToastMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "MajorFactionUnlockToastMixin.{method} must be a function — OnLoad registers \
             MAJOR_FACTION_UNLOCKED + wires ShowAnim:OnFinished, OnEvent re-fires \
             PlayMajorFactionUnlockToast, PlayBanner sets faction text + plays ShowAnim, \
             StopBanner halts the anim + hides the frame, OnHide notifies \
             TopBannerManager_BannerFinished so queued banners can advance"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_publishes_renown_toast_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MajorFactionsRenownToastMixin)")
        .expect("MajorFactionsRenownToastMixin type probe succeeds");
    assert_eq!(
        kind, "table",
        "MajorFactionsRenownToastMixin must publish at `_G` as a table — declared at \
         Blizzard_MajorFactionRenownToast.lua:2. Consumed by `mixin=\"...\"` on the named \
         MajorFactionsRenownToast frame (renown-toast XML line 3). Note the global name is \
         pluralized (`MajorFactionsRenownToastMixin`) while the unlock variant is singular \
         (`MajorFactionUnlockToastMixin`) — a Blizzard naming inconsistency the simulator \
         must mirror exactly for the XML `mixin=\"...\"` attribute to bind"
    );

    for method in MAJOR_FACTIONS_RENOWN_TOAST_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(MajorFactionsRenownToastMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("MajorFactionsRenownToastMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "MajorFactionsRenownToastMixin.{method} must be a function — drives the \
             MAJOR_FACTION_RENOWN_LEVEL_CHANGED event handler (deferred 1s for reward \
             grants), reward visuals dispatch (icon + multi-line description), tooltip \
             refresh on hover (paragon-aware via C_Reputation.IsFactionParagonForCurrentPlayer)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_named_frames_resolve_globally(env: &WowLuaEnv) {

    for name in NAMED_MAJOR_FACTION_FRAMES {
        let exists: bool = env
            .eval(&format!("return _G[{name:?}] ~= nil"))
            .unwrap_or_else(|err| panic!("{name} existence probe failed: {err}"));
        assert!(
            exists,
            "{name} must publish at `_G` after addon load — declared with \
             `name=\"{name}\"` in its respective .xml file. Both frames are toplevel \
             concrete instances of MajorFactionCelebrationBannerTemplate, parented to \
             UIParent, hidden by default — they only show when their respective event \
             (MAJOR_FACTION_UNLOCKED for the unlock toast, MAJOR_FACTION_RENOWN_LEVEL_CHANGED \
             for the renown toast) fires"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_unlock_toast_inherits_celebration_banner(env: &WowLuaEnv) {

    let parent_name: String = env
        .eval("return MajorFactionUnlockToast:GetParent():GetName()")
        .expect("MajorFactionUnlockToast:GetParent():GetName() probe succeeds");
    assert_eq!(
        parent_name, "UIParent",
        "MajorFactionUnlockToast must parent to UIParent — XML declares `parent=\"UIParent\"` \
         at Blizzard_MajorFactionUnlockToast.xml:3. UIParent is the canonical in-game UI \
         root, so the toast scales / hides with the rest of the player UI"
    );

    let is_hidden: bool = env
        .eval("return not MajorFactionUnlockToast:IsShown()")
        .expect("MajorFactionUnlockToast:IsShown() probe succeeds");
    assert!(
        is_hidden,
        "MajorFactionUnlockToast must be hidden by default — XML declares `hidden=\"true\"` \
         at Blizzard_MajorFactionUnlockToast.xml:3. The toast only shows transiently when \
         TopBannerManager_Show fires it from PlayMajorFactionUnlockToast"
    );

    let strata: String = env
        .eval("return MajorFactionUnlockToast:GetFrameStrata()")
        .expect("MajorFactionUnlockToast:GetFrameStrata() probe succeeds");
    assert_eq!(
        strata, "DIALOG",
        "MajorFactionUnlockToast must inherit DIALOG strata from \
         MajorFactionCelebrationBannerTemplate (toasts XML line 5: \
         `frameStrata=\"DIALOG\"`). DIALOG sits above the world / standard UI strata so the \
         celebration toast surfaces over normal in-game windows when it fires"
    );
}
}

prefork_full_ui_case! {
fn blizzard_major_factions_renown_toast_inherits_celebration_banner(env: &WowLuaEnv) {

    let parent_name: String = env
        .eval("return MajorFactionsRenownToast:GetParent():GetName()")
        .expect("MajorFactionsRenownToast:GetParent():GetName() probe succeeds");
    assert_eq!(
        parent_name, "UIParent",
        "MajorFactionsRenownToast must parent to UIParent — XML declares \
         `parent=\"UIParent\"` at Blizzard_MajorFactionRenownToast.xml:3. Mirrors the \
         unlock-toast layout exactly (both are TopBannerManager-driven celebration \
         toasts, just for different events)"
    );

    let is_hidden: bool = env
        .eval("return not MajorFactionsRenownToast:IsShown()")
        .expect("MajorFactionsRenownToast:IsShown() probe succeeds");
    assert!(
        is_hidden,
        "MajorFactionsRenownToast must be hidden by default — XML declares \
         `hidden=\"true\"` at Blizzard_MajorFactionRenownToast.xml:3. Only shows when \
         ShowRenownLevelUpToast fires from the deferred MAJOR_FACTION_RENOWN_LEVEL_CHANGED \
         handler"
    );

    let strata: String = env
        .eval("return MajorFactionsRenownToast:GetFrameStrata()")
        .expect("MajorFactionsRenownToast:GetFrameStrata() probe succeeds");
    assert_eq!(
        strata, "DIALOG",
        "MajorFactionsRenownToast must inherit DIALOG strata from \
         MajorFactionCelebrationBannerTemplate. The shared celebration-banner template is \
         the only shipping use-site of the toast strata-decoration combo (DIALOG + \
         evergreen-toast-celebration-* atlas family)"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_major_factions_localization_module_is_empty_placeholder() {
    let raw = std::fs::read_to_string(major_factions_dir().join("Localization.lua"))
        .expect("Blizzard_MajorFactions Localization.lua reads");
    let trimmed = raw.trim();
    assert!(
        trimmed.starts_with("--"),
        "Localization.lua must start with a comment — the module is an empty placeholder, \
         the entire file is a single comment line. Major factions is one of the few \
         Blizzard addons that ships a Localization.lua slot but never actually populates it \
         (no per-locale string overrides have been needed since launch)"
    );
    assert!(
        !raw.contains("function ") && !raw.contains(" = {"),
        "Localization.lua must NOT define any functions or tables — placeholder-only \
         contract; either appearing here would mean Blizzard added per-locale post-load \
         tweaks (a meaningful upstream change worth flagging)"
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_MajorFactions, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_majorfactions_toc_is_absent_from_classic_caches() {
    let dir = major_factions_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_MajorFactions"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
