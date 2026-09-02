#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn kiosk_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Kiosk")
}

fn kiosk_toc() -> PathBuf {
    kiosk_dir().join("Blizzard_Kiosk.toc")
}

const KIOSK_TOC_FILES: &[&str] = &[
    "Housing/Config.lua",
    "Blizzard_Kiosk_Bootstrap.lua",
    "Kiosk.lua",
    "Kiosk.xml",
    "Housing/Glue.lua",
    "Housing/Glue.xml",
    "Housing/Game.lua",
    "Housing/Game.xml",
];

const KIOSK_FRAME_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "HasAllowedMaps",
    "GetAllowedMapIDs",
    "SetMode",
    "GetMode",
    "GetModeData",
    "SetAutoEnterWorld",
    "GetAutoEnterWorld",
    "GetRaceList",
    "GetIDForSelection",
];

const GLUE_KIOSK_FRAME_MIXIN_METHODS: &[&str] = &[
    "OnEvent",
    "HandleCharacterCreateOnShow",
    "NavBack",
    "HandleCreateCharacter",
    "HandleCheckEnterWorld",
    "HandleCharacterListUpdate",
    "HandleReturnToCharacterSelect",
    "HandleCharacterSelectShown",
    "HandleAutoLoginToRealm",
];

const GAME_KIOSK_FRAME_MIXIN_OWN_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "DisplayExpireState",
    "DisplayLobbyState",
    "HandlePlayerEnteringWorld",
];

const GAME_KIOSK_MODE_SPLASH_MIXIN_METHODS: &[&str] = &[
    "ShowSpinnerTooltip",
    "OnResetFailed",
    "OnLoad",
    "OnShow",
    "SetButtonEnabled",
];

const GAME_NAMED_FRAMES: &[&str] = &[
    "GameKioskSessionStartedDialog",
    "KioskModeSplash",
    "KioskModeSplashEnd",
    "KioskFrame",
];

fn load_kiosk(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &kiosk_toc())
        .expect("Blizzard_Kiosk should load via explicit Rust loader call");
}

#[test]
fn blizzard_kiosk_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&kiosk_dir()).expect("Blizzard_Kiosk TOC should resolve");
    assert_eq!(
        resolved,
        kiosk_toc(),
        "Blizzard_Kiosk ships exactly one bare TOC — the LoadOnDemand kiosk-mode bootstrap \
         resolves via `find_toc_file` fallthrough after the `_Mainline.toc` lookup misses. \
         Kiosk mode is a cross-flavor concept (the AllowLoad: both contract makes it usable \
         on glue + game screens), so it does NOT ship flavor-suffixed TOCs"
    );
}

#[test]
fn blizzard_kiosk_toc_declares_load_on_demand_with_no_dependencies() {
    let toc = TocFile::from_file(&kiosk_toc()).expect("Blizzard_Kiosk TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_Kiosk declares `## LoadOnDemand: 1` — the kiosk-mode bootstrap stays unloaded \
         until the host engine flips kiosk mode on. Mainline_GlueXML probes `Kiosk.IsEnabled()` \
         and dispatches `C_AddOns.LoadAddOn(\"Blizzard_Kiosk\")` only when the demo session \
         actually runs (the public retail client never triggers this — kiosk mode is for \
         convention-floor demo machines and the housing demo)"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_Kiosk declares ZERO `## Dependencies:` — the kiosk bootstrap relies only on \
         the global runtime surface (StaticPopupDialogs, EditModeManagerFrame, \
         GlueParent_SetScreen, ChatFrameUtil, UIErrorsFrame, SettingsPanel, ConsoleExec, \
         C_HouseEditor / C_Housing / C_CharacterCreation namespaces, the SOUNDKIT alias \
         registry) and the pre-stubbed `Kiosk` namespace seeded by runtime_surface_bootstrap.lua \
         line 2257 (which carries the IsEnabled/IsCompetitiveModeEnabled stubs the addon never \
         re-defines). No explicit Dependencies line"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_Kiosk declares zero `## OptionalDeps` — the housing demo plumbing references \
         EditModeManagerFrame and ObjectiveTrackerManager unconditionally, but those exist in \
         the foundational addon set (Blizzard_EditMode / Blizzard_ObjectiveTracker) which auto- \
         load before any LoD addon"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_Kiosk declares zero saved variables — kiosk session state is fully volatile \
         (a demo machine reset wipes it) and Kiosk.AllowedMapIDs / Kiosk.CharacterData seed at \
         load time from in-source data tables, not persistent storage"
    );
}

#[test]
fn blizzard_kiosk_toc_declares_allow_load_both_with_no_game_type_restriction() {
    let toc = TocFile::from_file(&kiosk_toc()).expect("Blizzard_Kiosk TOC should parse");
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_Kiosk omits `## AllowLoadGameType:` — `is_game_type_restricted` \
         (src/toc.rs:294-302) returns false when the metadata key is missing, so retail / mists \
         / wowhack clients all keep the addon eligible. Kiosk mode itself is a runtime flag, \
         not a build-time game-type"
    );

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "Blizzard_Kiosk declares `## AllowLoad: both` — `allows_screen` (src/toc.rs:307) \
             returns true for every ScreenKind when `both` is set. The kiosk mode bootstrap \
             must be eligible from the glue login + character-select screens (so the kiosk \
             splash dialog can replace the character list) AND from the in-game world (so \
             KIOSK_SESSION_* events route through the Game-side mixins). Tested screen: \
             {screen:?}"
        );
    }

    let raw = std::fs::read_to_string(kiosk_toc()).expect("Blizzard_Kiosk TOC should read");
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — the on-demand bootstrap stays out of \
         the Game-screen auto-discovery sweep until kiosk mode flips on"
    );
    assert!(
        raw.contains("## AllowLoad: both"),
        "TOC must declare `## AllowLoad: both` exactly — the lowercase `both` variant is what \
         ships on the source tree (allows_screen normalizes case via eq_ignore_ascii_case). \
         The both contract is what makes the addon visible from glue + game screens"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare `## Dependencies:` — the kiosk bootstrap is a self-contained \
         data-and-mixin module"
    );
}

#[test]
fn blizzard_kiosk_toc_lists_eight_files_with_per_file_allow_load_brackets_stripped() {
    let toc = TocFile::from_file(&kiosk_toc()).expect("Blizzard_Kiosk TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        KIOSK_TOC_FILES,
        "TOC body must list exactly 8 files in this order — Housing/Config.lua, \
         Blizzard_Kiosk_Bootstrap.lua, Kiosk.lua, Kiosk.xml, Housing/Glue.lua, \
         Housing/Glue.xml, Housing/Game.lua, Housing/Game.xml. The bracketed `[AllowLoad Glue]` and `[AllowLoad Game]` per-file annotations are \
         STRIPPED by strip_annotations (src/toc.rs:29-41) but the file paths still land in \
         the files vec. The TOC parser only honors `[AllowLoadGameType ...]` and \
         `[AllowLoadTextLocale ...]` per-file gates (src/toc.rs:138-143); per-file \
         `[AllowLoad ...]` annotations are NOT filtered, so the simulator loads all 8 files \
         on every screen mode the addon is invoked on. The Glue.lua/.xml + Game.lua/.xml \
         pair both define a frame named `KioskFrame` — the second `KioskFrame` (from Game.xml \
         since it is last in the body) re-creates the first via `register_new_frame` + \
         `migrate_children_to_new_frame` (create_frame.rs), and the final mixin reflects the \
         Game-side variant"
    );
}

#[test]
fn blizzard_kiosk_directory_holds_five_entries() {
    let entries = std::fs::read_dir(kiosk_dir())
        .expect("Blizzard_Kiosk directory should read")
        .count();
    assert_eq!(
        entries, 5,
        "Directory must hold exactly 5 entries: the TOC, bootstrap, Kiosk.lua, Kiosk.xml, and \
         Housing/ source directory"
    );
}

#[test]
fn blizzard_kiosk_excluded_from_every_screen_auto_discovery() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_Kiosk");
        assert!(
            !found,
            "Blizzard_Kiosk must be filtered out of auto-discovery on every ScreenKind. The \
             TOC declares `## LoadOnDemand: 1`, and discover_blizzard_addons_for_screen routes \
             LoD addons into the lod_pool rather than the eager `addons` set. The AllowLoad: \
             both contract makes it eligible on every screen mode, but LoadOnDemand still \
             keeps it out of every eager sweep — only `C_AddOns.LoadAddOn(\"Blizzard_Kiosk\")` \
             from the glue/game-side bootstrap pulls it in. (Screen tested: {screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_kiosk_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_kiosk(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_Kiosk")
                || message.contains("KioskFrameMixin")
                || message.contains("GlueKioskFrameMixin")
                || message.contains("GameKioskFrameMixin")
                || message.contains("GameKioskModeSplashMixin")
                || message.contains("GameKioskSessionStartedDialogMixin")
                || message.contains("GameKioskModeSplashEndMixin")
                || message.contains("KioskFrame")
                || message.contains("KioskModeSplash")
                || message.contains("GameKioskSessionStartedDialog")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_Kiosk emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_is_addon_loaded_via_explicit_load(env: &WowLuaEnv) {
    load_kiosk(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_Kiosk')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_Kiosk') must return true after the explicit \
         load_addon call — confirms the loader registers the addon with the loaded-set even \
         though the auto-discovery sweep skipped it (LoadOnDemand)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_namespace_extends_pre_stubbed_bootstrap_globals(env: &WowLuaEnv) {
    load_kiosk(env);

    let kind: String = env
        .eval("return type(Kiosk)")
        .expect("Kiosk namespace probe should succeed");
    assert_eq!(
        kind, "table",
        "_G.Kiosk must publish as a table — runtime_surface_bootstrap.lua line 2257 pre-seeds \
         the namespace with IsEnabled / IsCompetitiveModeEnabled stubs via \
         __wow_merge_namespace BEFORE Blizzard_Kiosk loads. The addon then attaches \
         AllowedMapIDs / ExpirationWarningSoundKit / CharacterData to the same table"
    );

    let sound_kit: f64 = env
        .eval("return Kiosk.ExpirationWarningSoundKit")
        .expect("Kiosk.ExpirationWarningSoundKit probe should succeed");
    assert_eq!(
        sound_kit, 15273.0,
        "Kiosk.ExpirationWarningSoundKit must equal 15273 — Kiosk.lua line 1 publishes the \
         sound-kit ID at file scope. KioskFrameMixin:OnEvent passes this to PlaySound when \
         KIOSK_SESSION_EXPIRATION_WARNING fires (the audible 60-second-warning chime)"
    );

    let allowed_maps_kind: String = env
        .eval("return type(Kiosk.AllowedMapIDs)")
        .expect("Kiosk.AllowedMapIDs probe should succeed");
    assert_eq!(
        allowed_maps_kind, "table",
        "Kiosk.AllowedMapIDs must publish as a table — Housing/Config.lua line 1 seeds the \
         empty allow-list (empty = no map restrictions). KioskFrameMixin:HasAllowedMaps probes \
         #Kiosk.AllowedMapIDs > 0 to gate the map-restriction surface; \
         Kiosk.AllowedMapIDs MUST be a table (not nil) for the # operator to work"
    );

    let character_data_high: String = env
        .eval("return type(Kiosk.CharacterData.highlevel)")
        .expect("Kiosk.CharacterData.highlevel probe should succeed");
    assert_eq!(
        character_data_high, "table",
        "Kiosk.CharacterData.highlevel must publish as a table — Kiosk.lua lines 4-92 publish \
         the character-creation allow-list keyed by mode (`highlevel` enables every retail race \
         + class + allied-race + the level-up template; `newcharacter` mirrors the race set but \
         disables the demonhunter / deathknight hero classes and the entire allied-race set). \
         KioskFrameMixin:GetModeData reads Kiosk.CharacterData[self.mode] to feed the race / \
         class dropdowns on the kiosk-locked character-create screen"
    );

    let is_enabled_kind: String = env
        .eval("return type(Kiosk.IsEnabled)")
        .expect("Kiosk.IsEnabled probe should succeed");
    assert_eq!(
        is_enabled_kind, "function",
        "Kiosk.IsEnabled must remain a function after Blizzard_Kiosk loads — \
         runtime_surface_bootstrap.lua line 2258 seeds the stub returning false, and the addon \
         does NOT redefine it. __wow_merge_namespace (line 2071-2083) only fills missing keys \
         via rawget==nil, so the bootstrap function survives the merge"
    );

    let competitive_kind: String = env
        .eval("return type(Kiosk.IsCompetitiveModeEnabled)")
        .expect("Kiosk.IsCompetitiveModeEnabled probe should succeed");
    assert_eq!(
        competitive_kind, "function",
        "Kiosk.IsCompetitiveModeEnabled must remain a function — bootstrap stub at line 2259 \
         seeds the always-false return; the addon does not override it. \
         GlueKioskFrameMixin's HandleCharacterCreateOnShow / NavBack / HandleCheckEnterWorld / \
         HandleCharacterListUpdate / HandleCharacterSelectShown / HandleAutoLoginToRealm all \
         call this via the early-return guards"
    );
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_kiosk_frame_mixin_carries_eleven_methods(env: &WowLuaEnv) {
    load_kiosk(env);

    let kind: String = env
        .eval("return type(KioskFrameMixin)")
        .expect("KioskFrameMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "KioskFrameMixin must publish at `_G` as a table — Kiosk.lua line 100 creates the \
         empty mixin table at file scope before binding 11 methods to it. The mixin is the \
         shared base for both screen-side variants (GlueKioskFrameMixin extends it manually \
         via `KioskFrameMixin.OnEvent(self, event, ...)`; GameKioskFrameMixin extends it via \
         CreateFromMixins)"
    );

    for method in KIOSK_FRAME_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(KioskFrameMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("KioskFrameMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "KioskFrameMixin.{method} must publish as a function — the shared base mixin owns \
             11 methods: lifecycle (OnLoad / OnEvent — OnLoad registers the 8 KIOSK_SESSION_* \
             + TOGGLE_CONSOLE + DEBUG_MENU_TOGGLED events; OnEvent dispatches the warning / \
             changed / shutdown branches), map allow-list (HasAllowedMaps / GetAllowedMapIDs), \
             mode/state plumbing (SetMode / GetMode / GetModeData / SetAutoEnterWorld / \
             GetAutoEnterWorld), and the character-create dropdown helpers (GetRaceList / \
             GetIDForSelection)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_glue_kiosk_frame_mixin_carries_nine_methods(env: &WowLuaEnv) {
    load_kiosk(env);

    let kind: String = env
        .eval("return type(GlueKioskFrameMixin)")
        .expect("GlueKioskFrameMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "GlueKioskFrameMixin must publish at `_G` as a table — Housing/Glue.lua line 2 creates \
         the empty mixin table at file scope before binding 9 methods. Glue.lua is loaded on \
         the Game screen too (the per-file `[AllowLoad Glue]` annotation is stripped but not \
         filtered by the simulator's TOC parser), so the mixin still publishes during a \
         Game-side load"
    );

    for method in GLUE_KIOSK_FRAME_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(GlueKioskFrameMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("GlueKioskFrameMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GlueKioskFrameMixin.{method} must publish as a function — the glue mixin owns 9 \
             methods. OnEvent extends KioskFrameMixin.OnEvent (manual delegation, not \
             CreateFromMixins); the 8 Handle* methods are no-ops because HasGlueFlow returns \
             false (Housing demo has no glue-side surface — kept around because character- \
             select code still calls them unconditionally)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_game_kiosk_frame_mixin_extends_base_via_create_from_mixins(env: &WowLuaEnv) {
    load_kiosk(env);

    let kind: String = env
        .eval("return type(GameKioskFrameMixin)")
        .expect("GameKioskFrameMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "GameKioskFrameMixin must publish at `_G` as a table — Housing/Game.lua line 92 calls \
         `GameKioskFrameMixin = CreateFromMixins(KioskFrameMixin)` to seed the mixin with the \
         11 base methods, then attaches 5 game-specific methods (OnLoad / OnEvent override the \
         base; DisplayExpireState / DisplayLobbyState / HandlePlayerEnteringWorld are new). \
         CreateFromMixins copies the base method table by reference so subsequent base- \
         method additions DO propagate, but the override at line 94 (OnLoad) replaces the \
         function ref on the new table only"
    );

    for method in GAME_KIOSK_FRAME_MIXIN_OWN_METHODS {
        let kind: String = env
            .eval(&format!("return type(GameKioskFrameMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("GameKioskFrameMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameKioskFrameMixin.{method} must publish as a function — the 5 own methods of \
             the game-side variant: OnLoad (calls KioskFrameMixin.OnLoad then registers \
             KIOSK_HOUSING_RESET), OnEvent (extends base via manual delegation, dispatches \
             KIOSK_SESSION_STARTED / EXPIRED / SHUTDOWN / RESTART / KIOSK_HOUSING_RESET), \
             DisplayExpireState (shows KioskModeSplashEnd), DisplayLobbyState (closes all \
             windows + shows KioskModeSplash), HandlePlayerEnteringWorld (the entering-house \
             reset / classic-preset / god-mode / module-suppression hook)"
        );
    }

    let inherits_base: bool = env
        .eval("return type(GameKioskFrameMixin.HasAllowedMaps) == 'function'")
        .expect("GameKioskFrameMixin.HasAllowedMaps inheritance probe should succeed");
    assert!(
        inherits_base,
        "GameKioskFrameMixin must inherit KioskFrameMixin.HasAllowedMaps via CreateFromMixins \
         — the mixin holds 11 base methods + 5 own methods. HasAllowedMaps is one of the base- \
         only methods that should resolve through the CreateFromMixins copy (Game.lua line 92 \
         seeds the mixin via CreateFromMixins before attaching its own methods)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_game_mode_splash_mixin_carries_five_methods(env: &WowLuaEnv) {
    load_kiosk(env);

    let kind: String = env
        .eval("return type(GameKioskModeSplashMixin)")
        .expect("GameKioskModeSplashMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "GameKioskModeSplashMixin must publish at `_G` as a table — Housing/Game.lua line 1 \
         creates the empty mixin table. The mixin drives the KioskModeSplash full-screen \
         landing-page widget (the kiosk-button + tooltip-spinner + welcome-keyart background \
         that the player clicks to start a session)"
    );

    for method in GAME_KIOSK_MODE_SPLASH_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(GameKioskModeSplashMixin['{method}'])"
            ))
            .unwrap_or_else(|err| panic!("GameKioskModeSplashMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameKioskModeSplashMixin.{method} must publish as a function — owns 5 methods. \
             OnLoad seeds the welcome-background atlas / kiosk-button atlas / start-button \
             OnClick (calls Kiosk.StartSession) / spinner OnEnter/OnLeave wiring (shows \
             reset-in-progress tooltip). OnShow probes Kiosk.IsHousingResetPending to gate the \
             button. SetButtonEnabled toggles the spinner. ShowSpinnerTooltip / OnResetFailed \
             handle reset-failed state"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_game_session_started_and_splash_end_mixins_publish_with_one_method_each(env: &WowLuaEnv) {
    load_kiosk(env);

    for mixin in [
        "GameKioskSessionStartedDialogMixin",
        "GameKioskModeSplashEndMixin",
    ] {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must publish at `_G` as a table — Housing/Game.lua creates each empty \
             mixin and attaches a single OnLoad method. GameKioskSessionStartedDialogMixin \
             paints the FULLSCREEN_DIALOG-strata SessionStarted dialog (housing-basic-panel \
             stone-background atlas, housing-wood-frame trim, header/body/continue-button \
             text from KIOSK_HOUSING_START_DLG_*). GameKioskModeSplashEndMixin paints the \
             end-of-session FULLSCREEN splash (end-screen-background-keyart atlas + 3 body / \
             footer text strings)"
        );

        let on_load_kind: String = env
            .eval(&format!("return type({mixin}.OnLoad)"))
            .unwrap_or_else(|err| panic!("{mixin}.OnLoad probe failed: {err}"));
        assert_eq!(
            on_load_kind, "function",
            "{mixin}.OnLoad must publish as a function — each dialog mixin owns exactly one \
             OnLoad method that wires SetParent(GetAppropriateTopLevelParent()) + frame- \
             strata + atlas + text setters. Subsequent state changes are handled by the \
             KIOSK_SESSION_* events through GameKioskFrameMixin:OnEvent rather than per-mixin \
             event handlers"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_game_named_frames_publish_after_explicit_load(env: &WowLuaEnv) {
    load_kiosk(env);

    for frame in GAME_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type({frame})"))
            .unwrap_or_else(|err| panic!("{frame} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{frame} must publish at `_G` as a table — Housing/Game.xml declares 4 named non- \
             virtual top-level frames: GameKioskSessionStartedDialog (FULLSCREEN_DIALOG strata, \
             448x320), KioskModeSplash (toplevel + setAllPoints + enableMouse, the welcome \
             splash full-screen overlay), KioskModeSplashEnd (toplevel + setAllPoints, the \
             end-of-session full-screen overlay), KioskFrame (inherits KioskFrameTemplate with \
             mixin=GameKioskFrameMixin). All start hidden=true and become visible only when \
             the kiosk session lifecycle events fire"
        );

        let name: String = env
            .eval(&format!("return {frame}:GetName()"))
            .unwrap_or_else(|err| panic!("{frame}:GetName() probe failed: {err}"));
        assert_eq!(
            &name, frame,
            "{frame}:GetName() must return the literal frame name — proves the named frame \
             was registered under the expected key in the global lookup"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_kiosk_frame_template_stays_nil_at_global_scope(env: &WowLuaEnv) {
    load_kiosk(env);

    let kind: String = env
        .eval("return type(_G['KioskFrameTemplate'])")
        .expect("KioskFrameTemplate probe should succeed");
    assert_eq!(
        kind, "nil",
        "_G['KioskFrameTemplate'] must be nil — Kiosk.xml line 5 declares \
         `<Frame name=\"KioskFrameTemplate\" mixin=\"KioskFrameMixin\" virtual=\"true\">`. \
         Virtual frames register only in the template registry (consumed via XML \
         `inherits=\"KioskFrameTemplate\"` resolution from Glue.xml line 3 + Game.xml line \
         136), NOT at `_G`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_kiosk_frame_uses_final_game_mixin_definition(env: &WowLuaEnv) {
    load_kiosk(env);

    let exists: bool = env
        .eval("return type(KioskFrame) == 'table'")
        .expect("KioskFrame probe should succeed");
    assert!(
        exists,
        "_G.KioskFrame must publish — both Glue.xml line 3 and Game.xml line 136 declare a \
         non-virtual `<Frame name=\"KioskFrame\" inherits=\"KioskFrameTemplate\" \
         mixin=\"...\">`. Both files load when Blizzard_Kiosk is invoked from Game screen \
         because the simulator's TOC parser does NOT honor per-file `[AllowLoad Glue]` / \
         `[AllowLoad Game]` annotations (only `[AllowLoadGameType]` and \
         `[AllowLoadTextLocale]` are filtered at src/toc.rs:138-143)"
    );

    let display_expire_kind: String = env
        .eval("return type(KioskFrame.DisplayExpireState)")
        .expect("KioskFrame.DisplayExpireState probe should succeed");
    assert_eq!(
        display_expire_kind, "function",
        "KioskFrame.DisplayExpireState must publish as a function — the method lives on \
         GameKioskFrameMixin (Game.lua line 141). Its presence confirms the Game-side mixin \
         was attached when Game.xml's KioskFrame definition processed"
    );

    let display_lobby_kind: String = env
        .eval("return type(KioskFrame.DisplayLobbyState)")
        .expect("KioskFrame.DisplayLobbyState probe should succeed");
    assert_eq!(
        display_lobby_kind, "function",
        "KioskFrame.DisplayLobbyState must publish as a function — Game.lua line 145 owns \
         this method on GameKioskFrameMixin"
    );

    let nav_back_kind: String = env
        .eval("return type(KioskFrame.NavBack)")
        .expect("KioskFrame.NavBack probe should succeed");
    assert_eq!(
        nav_back_kind, "nil",
        "KioskFrame.NavBack must be nil after Game.xml replaces the earlier Glue.xml frame. \
         NavBack exists only on GlueKioskFrameMixin; the final same-named KioskFrame is \
         declared by Game.xml with GameKioskFrameMixin, so duplicate frame registration \
         does not merge unrelated mixin methods"
    );
}
}

prefork_full_ui_case! {
fn blizzard_kiosk_static_popup_dialog_registers_kiosk_enabled_entry(env: &WowLuaEnv) {
    load_kiosk(env);

    let entry_kind: String = env
        .eval("return type(StaticPopupDialogs['KIOSK_ENABLED'])")
        .expect("StaticPopupDialogs.KIOSK_ENABLED probe should succeed");
    assert_eq!(
        entry_kind, "table",
        "StaticPopupDialogs['KIOSK_ENABLED'] must register as a table — Kiosk.lua line 94 \
         publishes the entry with text=KIOSK_ENABLED_DLG_TEXT, button1=OKAY, button2=nil. \
         This is the dialog the engine surfaces via StaticPopup_Show('KIOSK_ENABLED') when \
         the kiosk session toggle flips on (an OK-only acknowledgement that demo mode is now \
         active). The button2=nil ensures it renders as a single-button confirmation, not a \
         two-button accept/cancel"
    );

    let button1: String = env
        .eval("return tostring(StaticPopupDialogs['KIOSK_ENABLED'].button1)")
        .expect("StaticPopupDialogs.KIOSK_ENABLED.button1 probe should succeed");
    assert_eq!(
        button1, "Okay",
        "StaticPopupDialogs['KIOSK_ENABLED'].button1 must equal the locale-resolved OKAY \
         string — the simulator's en-US locale resolves the OKAY global to the title-cased \
         literal 'Okay' (matching the retail Blizzard locale string)"
    );
}
}
