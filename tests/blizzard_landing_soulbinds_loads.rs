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

fn landing_soulbinds_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_LandingSoulbinds")
}

fn landing_soulbinds_toc() -> PathBuf {
    landing_soulbinds_dir().join("Blizzard_LandingSoulbinds.toc")
}

const LANDING_SOULBINDS_TOC_FILES: &[&str] = &[
    "Blizzard_LandingSoulbinds_Bootstrap.lua",
    "Blizzard_LandingSoulbindButton.xml",
    "Blizzard_LandingRenownButton.xml",
    "Blizzard_LandingSoulbindPanel.xml",
];

const LANDING_PAGE_SOULBIND_PANEL_MIXIN_METHODS: &[&str] =
    &["Update", "UpdateRenown", "UpdateSoulbind"];

const LANDING_PAGE_SOULBIND_BUTTON_MIXIN_METHODS: &[&str] = &[
    "OnEvent",
    "OnShow",
    "ShowHelpTip",
    "OnHide",
    "OnEnter",
    "OnLeave",
    "OnMouseDown",
    "OnMouseUp",
    "OnClick",
    "SetSoulbind",
];

const LANDING_PAGE_RENOWN_BUTTON_MIXIN_METHODS: &[&str] = &[
    "OnEvent",
    "OnShow",
    "OnHide",
    "OnClick",
    "OnCurrencyUpdate",
    "UpdateRenownLevel",
    "UpdateButtonTextures",
    "OnMouseDown",
    "OnMouseUp",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "LandingPageSoulbindPanelTemplate",
    "LandingPageSoulbindButtonTemplate",
    "LandingPageRenownButtonTemplate",
];

fn load_landing_soulbinds(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &landing_soulbinds_toc())
        .expect("Blizzard_LandingSoulbinds should load via explicit Rust loader call");
}

#[test]
fn blizzard_landing_soulbinds_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&landing_soulbinds_dir()).expect("Blizzard_LandingSoulbinds TOC resolves");
    assert_eq!(
        resolved,
        landing_soulbinds_toc(),
        "Blizzard_LandingSoulbinds ships a single bare TOC. The Shadowlands-era covenant + \
         soulbind landing-page panel is retail-only content but the TOC declares no \
         `## AllowLoadGameType:` filter, so `find_toc_file` resolves it via the bare-name \
         fallthrough after the `_Mainline.toc` lookup misses"
    );
}

#[test]
fn blizzard_landing_soulbinds_toc_declares_load_on_demand_with_no_dependencies() {
    let toc =
        TocFile::from_file(&landing_soulbinds_toc()).expect("Blizzard_LandingSoulbinds TOC parses");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_LandingSoulbinds declares `## LoadOnDemand: 1`. The covenant landing-page \
         panel is fetched on demand by Blizzard_GarrisonLandingPage's selector at \
         line ~181 (`UIParentLoadAddOn(\"Blizzard_LandingSoulbinds\")`) only when the \
         player picks the Shadowlands expansion-mission landing page. Outside that flow \
         the addon stays unloaded — keeps the renown / soulbind UI cost off the boot path \
         for non-Shadowlands characters"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_LandingSoulbinds declares ZERO `## Dependencies:`. The covenant + \
         soulbind landing panel relies only on the global runtime surface — \
         C_Covenants.GetActiveCovenantID / GetCovenantData (small_namespaces.rs:171-172), \
         C_Soulbinds.GetActiveSoulbindID / GetSoulbindData (small_namespaces.rs:180-181), \
         C_CovenantSanctumUI.GetRenownLevel (NOT pre-stubbed — only invoked from OnShow \
         via UpdateRenownLevel, which is gated behind GetActiveCovenantID() ~= 0 so the \
         absence does not error at addon-load time), and the FrameUtil / HelpTip / \
         GetCVarBool / SetCVar / UIParentLoadAddOn / ToggleCovenantRenown surface from \
         shared_bootstrap.lua. The XML template inheritance (ResizeLayoutFrame) provides \
         the layout-driven sizing"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps`. The OnClick handler on LandingPageSoulbindButtonTemplate \
         calls `UIParentLoadAddOn(\"Blizzard_Soulbinds\")` to lazy-load the conduit-tree \
         viewer, but Blizzard_Soulbinds is itself a separate LoadOnDemand addon — there \
         is no static dependency between them, just a runtime UIParentLoadAddOn handshake"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables. Soulbind / renown state lives in C_Soulbinds + \
         C_CovenantSanctumUI (the host engine's covenant tracker), and the help-tip \
         cvar `soulbindsLandingPageTutorial` (set via SetCVar in ShowHelpTip) persists \
         through the engine's cvar store, not addon SVs"
    );
}

#[test]
fn blizzard_landing_soulbinds_toc_does_not_declare_allow_load_so_defaults_to_game_only() {
    let toc =
        TocFile::from_file(&landing_soulbinds_toc()).expect("Blizzard_LandingSoulbinds TOC parses");
    assert!(
        !toc.is_game_type_restricted(),
        "TOC omits `## AllowLoadGameType:` — `is_game_type_restricted` (src/toc.rs:294-302) \
         returns false when the metadata key is missing, leaving the addon eligible \
         across retail / mists / wowhack flavors. Covenants are retail-only content but \
         the gate is enforced at the data layer (GetActiveCovenantID returns 0 outside \
         retail), not at the TOC level"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_LandingSoulbinds omits `## AllowLoad:` — `allows_screen` (src/toc.rs:307-311) \
         returns true for Game when the metadata key is missing. The renown / soulbind \
         landing panel is in-game UI only (it embeds inside the GarrisonLandingPage flow)"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_LandingSoulbinds must NOT publish on glue screens. With no `## AllowLoad:` \
             declared, the default Game-only behavior keeps the renown panel out of the \
             login / character-select / character-create flow. (Screen tested: {screen:?})"
        );
    }

    let raw =
        std::fs::read_to_string(landing_soulbinds_toc()).expect("Blizzard_LandingSoulbinds reads");
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — keeps the covenant landing panel \
         out of the Game-screen auto-discovery sweep until GarrisonLandingPage's selector \
         flips the Shadowlands branch"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare `## Dependencies:` — the covenant landing panel is \
         self-contained mixin definitions plus three virtual templates"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` — Game-screen-only is the implicit default \
         and the covenant landing panel embeds inside in-game UI (GarrisonLandingPage)"
    );
}

#[test]
fn blizzard_landing_soulbinds_toc_lists_bootstrap_then_three_xml_files_in_dependency_order() {
    let toc =
        TocFile::from_file(&landing_soulbinds_toc()).expect("Blizzard_LandingSoulbinds TOC parses");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        LANDING_SOULBINDS_TOC_FILES,
        "Current retail TOC lists the bootstrap before the Button, RenownButton, and Panel XML \
         files; each XML loads its paired lua through a Script directive"
    );
}

#[test]
fn blizzard_landing_soulbinds_directory_holds_eight_entries() {
    let entries = std::fs::read_dir(landing_soulbinds_dir())
        .expect("Blizzard_LandingSoulbinds directory reads")
        .count();
    assert_eq!(
        entries, 8,
        "Directory must hold exactly 8 entries: the TOC, bootstrap, and three lua/XML pairs"
    );
}

#[test]
fn blizzard_landing_soulbinds_excluded_from_every_screen_auto_discovery() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_LandingSoulbinds");
        assert!(
            !found,
            "Blizzard_LandingSoulbinds must be filtered out of auto-discovery on every \
             ScreenKind. The `## LoadOnDemand: 1` declaration routes it into the lod_pool \
             rather than the eager `addons` set in discover_blizzard_addons_for_screen — \
             only `UIParentLoadAddOn(\"Blizzard_LandingSoulbinds\")` from \
             Blizzard_GarrisonLandingPage's covenant selector pulls it in. (Screen tested: \
             {screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_landing_soulbinds_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_landing_soulbinds(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_LandingSoulbinds")
                || message.contains("LandingPageSoulbindPanelMixin")
                || message.contains("LandingPageSoulbindButtonMixin")
                || message.contains("LandingPageRenownButtonMixin")
                || message.contains("LandingSoulbind")
                || message.contains("LandingPageSoulbindPanelTemplate")
                || message.contains("LandingPageSoulbindButtonTemplate")
                || message.contains("LandingPageRenownButtonTemplate")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_LandingSoulbinds emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_landing_soulbinds_is_addon_loaded_via_explicit_load(env: &WowLuaEnv) {
    load_landing_soulbinds(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_LandingSoulbinds')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_LandingSoulbinds') must return true after the \
         explicit load_addon call — confirms the loader registers the addon with the \
         loaded-set even though auto-discovery skipped it (LoadOnDemand)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_landing_soulbinds_panel_mixin_publishes_with_three_methods(env: &WowLuaEnv) {
    load_landing_soulbinds(env);

    let kind: String = env
        .eval("return type(LandingPageSoulbindPanelMixin)")
        .expect("LandingPageSoulbindPanelMixin probe succeeds");
    assert_eq!(
        kind, "table",
        "LandingPageSoulbindPanelMixin must publish at `_G` as a table — \
         Blizzard_LandingSoulbindPanel.lua line 1 creates the empty mixin table at file \
         scope before binding three methods. The mixin is bound to \
         LandingPageSoulbindPanelTemplate via `mixin=\"LandingPageSoulbindPanelMixin\"` \
         on the <Frame> tag"
    );

    for method in LANDING_PAGE_SOULBIND_PANEL_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(LandingPageSoulbindPanelMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("LandingPageSoulbindPanelMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "LandingPageSoulbindPanelMixin.{method} must publish as a function. The panel \
             mixin owns three methods: Update (orchestrator — calls UpdateRenown + \
             UpdateSoulbind, sets visibility based on either result, then triggers the \
             ResizeLayoutFrame::Layout pass), UpdateRenown (probes \
             C_Covenants.GetActiveCovenantID() ~= 0 to gate the renown button visibility, \
             then re-anchors RenownButton TOP→Spacer.BOTTOM), UpdateSoulbind (probes \
             C_Soulbinds.GetActiveSoulbindID() > 0 to gate the soulbind button visibility, \
             then re-anchors SoulbindButton TOP→RenownButton.BOTTOM with -5px y-offset)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_landing_soulbinds_button_mixin_publishes_with_ten_methods(env: &WowLuaEnv) {
    load_landing_soulbinds(env);

    let kind: String = env
        .eval("return type(LandingPageSoulbindButtonMixin)")
        .expect("LandingPageSoulbindButtonMixin probe succeeds");
    assert_eq!(
        kind, "table",
        "LandingPageSoulbindButtonMixin must publish at `_G` as a table — \
         Blizzard_LandingSoulbindButton.lua line 1 seeds the mixin. \
         LandingPageSoulbindButtonTemplate (Button widget) attaches the mixin via \
         `mixin=\"LandingPageSoulbindButtonMixin\"` and routes 8 script handlers \
         (OnEvent / OnShow / OnHide / OnClick / OnEnter / OnLeave / OnMouseDown / \
         OnMouseUp) through method= bindings"
    );

    for method in LANDING_PAGE_SOULBIND_BUTTON_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(LandingPageSoulbindButtonMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("LandingPageSoulbindButtonMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "LandingPageSoulbindButtonMixin.{method} must publish as a function. The button \
             mixin owns 10 methods: 8 script handlers (OnEvent dispatches \
             SOULBIND_ACTIVATED → SetSoulbind; OnShow registers \
             SOULBIND_ACTIVATED via FrameUtil.RegisterFrameForEvents and triggers the \
             tutorial HelpTip on first open; OnHide unregisters; OnEnter / OnLeave \
             toggle the highlight overlay; OnMouseDown / OnMouseUp toggle the press \
             overlay; OnClick lazy-loads Blizzard_Soulbinds via UIParentLoadAddOn then \
             opens SoulbindViewer) plus 2 helpers (ShowHelpTip, SetSoulbind which \
             swaps the portrait atlas based on the soulbind textureKit and updates \
             the label text)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_landing_soulbinds_renown_button_mixin_publishes_with_nine_methods(env: &WowLuaEnv) {
    load_landing_soulbinds(env);

    let kind: String = env
        .eval("return type(LandingPageRenownButtonMixin)")
        .expect("LandingPageRenownButtonMixin probe succeeds");
    assert_eq!(
        kind, "table",
        "LandingPageRenownButtonMixin must publish at `_G` as a table — \
         Blizzard_LandingRenownButton.lua line 1 seeds the mixin. \
         LandingPageRenownButtonTemplate (Button widget) attaches the mixin via \
         `mixin=\"LandingPageRenownButtonMixin\"`"
    );

    for method in LANDING_PAGE_RENOWN_BUTTON_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(LandingPageRenownButtonMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("LandingPageRenownButtonMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "LandingPageRenownButtonMixin.{method} must publish as a function. The renown \
             button mixin owns 9 methods: 6 script handlers (OnEvent dispatches \
             CURRENCY_DISPLAY_UPDATE → OnCurrencyUpdate; OnShow registers the currency \
             event and refreshes UpdateRenownLevel + UpdateButtonTextures; OnHide \
             unregisters; OnClick fires ToggleCovenantRenown to expand the renown UI; \
             OnMouseDown / OnMouseUp toggle the PushedImage overlay) plus 3 helpers \
             (OnCurrencyUpdate filters by SOULBINDS_RENOWN_CURRENCY_ID and refreshes \
             the level display; UpdateRenownLevel reads C_CovenantSanctumUI.GetRenownLevel \
             into the FontString; UpdateButtonTextures swaps NormalAtlas / PushedAtlas / \
             PushedImage atlas to the active covenant's textureKit)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_landing_soulbinds_landing_soulbind_namespace_publishes_create_factory(env: &WowLuaEnv) {
    load_landing_soulbinds(env);

    let ns_kind: String = env
        .eval("return type(LandingSoulbind)")
        .expect("LandingSoulbind probe succeeds");
    assert_eq!(
        ns_kind, "table",
        "LandingSoulbind must publish at `_G` as a table — \
         Blizzard_LandingSoulbindPanel.lua line 30 creates the namespace as a thin \
         factory wrapper around CreateFrame. This is the public entry point that \
         GarrisonLandingPage uses to instantiate the soulbind panel inside the renown \
         tab"
    );

    let create_kind: String = env
        .eval("return type(LandingSoulbind.Create)")
        .expect("LandingSoulbind.Create probe succeeds");
    assert_eq!(
        create_kind, "function",
        "LandingSoulbind.Create must publish as a function. The factory takes a parent \
         frame and returns `CreateFrame(\"Frame\", nil, parent, \
         \"LandingPageSoulbindPanelTemplate\")` — anonymous (no global name), the \
         template inheritance carries the layout config (fixedWidth=361, \
         minimumHeight=116, heightPadding=10) plus the SoulbindButton + RenownButton \
         children defined inline in the panel XML"
    );
}
}

prefork_full_ui_case! {
fn blizzard_landing_soulbinds_templates_remain_nil_at_global_scope(env: &WowLuaEnv) {
    load_landing_soulbinds(env);

    for template in VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G['{template}'])"))
            .unwrap_or_else(|err| panic!("_G['{template}'] probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G['{template}'] must remain nil — the XML declares `virtual=\"true\"`, so \
             the template lives in the simulator's template registry but does NOT publish \
             at `_G`. Only direct CreateFrame instantiation via `LandingSoulbind.Create` \
             produces a live frame, and that frame is unnamed (CreateFrame parameter 2 \
             is nil), so even instantiation does not mint a global"
        );
    }
}
}
