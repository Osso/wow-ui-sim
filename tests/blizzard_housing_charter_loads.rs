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

fn housing_charter_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HousingCharter")
}

fn housing_charter_toc() -> PathBuf {
    housing_charter_dir().join("Blizzard_HousingCharter.toc")
}

fn load_housing_charter(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &housing_charter_toc())
        .expect("Blizzard_HousingCharter should load via explicit Rust loader call");
}

#[test]
fn blizzard_housing_charter_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&housing_charter_dir()).expect("Blizzard_HousingCharter TOC should resolve");
    assert_eq!(
        resolved,
        housing_charter_toc(),
        "Blizzard_HousingCharter ships exactly one bare TOC \
         (`Blizzard_HousingCharter.toc`) — no flavor variants. The neighborhood-charter \
         signing dialog only ships on retail (`## AllowLoadGameType: standard`) and uses the \
         bare TOC suffix that `find_toc_file` (src/loader/mod.rs:65) falls through to"
    );
}

#[test]
fn blizzard_housing_charter_toc_declares_lod_with_single_dependency() {
    let toc = TocFile::from_file(&housing_charter_toc()).expect("HousingCharter TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_HousingCharter declares `## LoadOnDemand: 1` — the charter signing UI only \
         loads when the housing-event-handler dispatches it: \
         HousingEventHandlerMixin:OpenCharter (line 328) and \
         HousingEventHandlerMixin:OpenCharterSignatureRequest (line 336) each call \
         `C_AddOns.LoadAddOn(\"Blizzard_HousingCharter\")` after a nil-check on the global frame"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_HousingCharter does not declare `## LoadFirst: 1` — LoadOnDemand precludes \
         any load-order priority"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_HousingCharter does not declare `## UseSecureEnvironment` — runs in the \
         standard Lua environment"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_HousingTemplates".to_string()],
        "Blizzard_HousingCharter declares exactly one `## Dependencies:` entry — \
         Blizzard_HousingTemplates provides the housing-themed atlas references \
         (`housing-wood-frame`, `housing-basic-panel--stone-background`, \
         `housing-basic-panel-gradient-header-bg`, `housing-basic-container`) and the shared \
         C_Housing API surface (CanEditCharter, OnRequestSignatureClicked, \
         OnSignCharterClicked) plus the SOUNDKIT.HOUSING_CHARTER_* constants needed by the \
         charter mixins"
    );
}

#[test]
fn blizzard_housing_charter_toc_is_retail_only_and_omits_allow_load() {
    let toc = TocFile::from_file(&housing_charter_toc()).expect("HousingCharter TOC should parse");
    let toc_text = std::fs::read_to_string(housing_charter_toc()).expect("TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: standard"),
        "Blizzard_HousingCharter declares `## AllowLoadGameType: standard` — the housing \
         charter signing UI is a Midnight expansion feature that only ships on retail. \
         `is_game_type_restricted()` (src/toc.rs:294) treats `standard` and `mainline` as the \
         unrestricted retail flavor, so this addon is NOT considered game-type-restricted"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_HousingCharter must NOT be game-type restricted — `## AllowLoadGameType: \
         standard` matches the retail flavor that the simulator runs as"
    );
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_HousingCharter omits `## AllowLoad:` — LoadOnDemand precludes auto-discovery \
         gating, so the AllowLoad value would be inert. The addon is pulled exclusively via \
         the Lua-side LoadAddOn(\"Blizzard_HousingCharter\") path"
    );
    assert!(
        !toc_text.contains("## DefaultState:"),
        "Blizzard_HousingCharter omits `## DefaultState:` — relies on the loader's \
         implicit-enabled default for Blizzard prefix LoD addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_HousingCharter declares NO `## SavedVariables*` — the charter state is \
         server-driven via OPEN_NEIGHBORHOOD_CHARTER, ADD_NEIGHBORHOOD_CHARTER_SIGNATURE, and \
         REMOVE_NEIGHBORHOOD_CHARTER_SIGNATURE events, so no per-installation persistence is \
         needed"
    );
}

#[test]
fn blizzard_housing_charter_toc_lists_three_files() {
    let toc = TocFile::from_file(&housing_charter_toc()).expect("HousingCharter TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_HousingCharter.lua".to_string(),
            "Blizzard_HousingCharter.xml".to_string(),
            "Blizzard_HousingCharterRegistration.lua".to_string(),
        ],
        "Blizzard_HousingCharter TOC body lists exactly 3 source files in this exact order: \
         Blizzard_HousingCharter.lua (publishes HousingCharterMixin + \
         HousingCharterRequestSignatureFrameMixin and the local \
         HousingCharterFrameShowingEvents table), Blizzard_HousingCharter.xml \
         (HousingCharterSignatureTemplate virtual Frame + the named non-virtual \
         HousingCharterRequestSignatureDialog inheriting TranslucentFrameTemplate at DIALOG \
         strata + the named non-virtual HousingCharterFrame), \
         Blizzard_HousingCharterRegistration.lua (calls \
         RegisterUIPanel(HousingCharterFrame, {{area=\"left\", pushable=2}}) — must run AFTER \
         the XML instantiates HousingCharterFrame)"
    );
}

#[test]
fn blizzard_housing_charter_directory_holds_four_entries() {
    let dir = housing_charter_dir();
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_HousingCharter directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        4,
        "Blizzard_HousingCharter directory ships exactly 4 entries: 3 source files referenced \
         by the TOC + 1 TOC file. No flavor subdirectory and no Localization.lua — the \
         strings (HOUSING_CREATENEIGHBORHOOD_CHARTER, HOUSING_CHARTER_PLAYERS, \
         HOUSING_CHARTER_NEIGHBORHOOD_LOCATION, HOUSING_CHARTER_NEIGHBORHOOD_NAME, \
         HOUSING_CHARTER_DESCRIPTION, HOUSING_CHARTER_SETTINGS_BUTTON, \
         HOUSING_CHARTER_REQUEST_BUTTON, HOUSING_CHARTER_CLOSE_BUTTON, \
         HOUSING_CHARTER_UNSIGNED, HOUSING_CHARTER_REQUEST_*) are pulled from the global \
         locale table maintained by the housing dependency chain. Got: {entries:?}"
    );
    assert!(
        entries.contains(&"Blizzard_HousingCharter.toc".to_string()),
        "Blizzard_HousingCharter directory must contain the bare TOC file"
    );
    assert!(
        entries.contains(&"Blizzard_HousingCharterRegistration.lua".to_string()),
        "Blizzard_HousingCharter directory must contain the Registration tail file (8 lines, \
         runs RegisterUIPanel after the XML instantiates HousingCharterFrame)"
    );
}

#[test]
fn blizzard_housing_charter_excluded_from_all_screen_auto_discovery_passes() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_HousingCharter");
        assert!(
            !discovered,
            "Blizzard_HousingCharter MUST NOT appear in {screen:?} auto-discovery — \
             `## LoadOnDemand: 1` keeps it out of every screen pass. The only consumers \
             (Blizzard_HousingEventHandler/Blizzard_HousingEventHandler.lua:330 and :338) call \
             `C_AddOns.LoadAddOn(\"Blizzard_HousingCharter\")` from event-handler methods — \
             never via `## RequiredDep:`, so the LoD-pull promotion path in \
             `pull_required_lod_addons` (src/loader/mod.rs:357) does not escalate \
             HousingCharter onto any auto-discovery pass"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_housing_charter_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_housing_charter(env);

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_HousingCharter/")
                || e.contains("Blizzard_HousingCharter\\")
                || e.contains("HousingCharterMixin")
                || e.contains("HousingCharterRequestSignatureFrameMixin")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_HousingCharter emitted addon-specific Lua errors during explicit LoD load:\n  {}",
        related
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_is_addon_loaded_returns_true_after_explicit_lod_load(env: &WowLuaEnv) {
    load_housing_charter(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HousingCharter')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "After explicit `load_addon` of Blizzard_HousingCharter.toc following Game-screen \
         auto-discovery (which loads the single Blizzard_HousingTemplates dep but skips \
         HousingCharter itself due to LoadOnDemand), \
         `C_AddOns.IsAddOnLoaded('Blizzard_HousingCharter')` should return true"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_publishes_housing_charter_frame_global(env: &WowLuaEnv) {
    load_housing_charter(env);

    let exists: bool = env
        .eval(
            "local f = _G['HousingCharterFrame']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("HousingCharterFrame global lookup should succeed");
    assert!(
        exists,
        "After LoD load, `HousingCharterFrame` should publish as a global frame instance — \
         Blizzard_HousingCharter.xml line 70 declares `<Frame name=\"HousingCharterFrame\" \
         mixin=\"HousingCharterMixin\" parent=\"UIParent\" movable=\"true\" \
         enableMouse=\"true\" hidden=\"true\">` and the Registration.lua tail then calls \
         `RegisterUIPanel(HousingCharterFrame, {{area=\"left\", pushable=2}})` to register the \
         frame as a left-area pushable UI panel with pushable rank 2 (vs HouseList's rank 1)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_publishes_request_signature_dialog_global(env: &WowLuaEnv) {
    load_housing_charter(env);

    let exists: bool = env
        .eval(
            "local f = _G['HousingCharterRequestSignatureDialog']; return type(f) == 'table' and type(f.GetName) == 'function'",
        )
        .expect("HousingCharterRequestSignatureDialog global lookup should succeed");
    assert!(
        exists,
        "After LoD load, `HousingCharterRequestSignatureDialog` should publish as a global \
         frame instance — Blizzard_HousingCharter.xml line 18 declares \
         `<Frame name=\"HousingCharterRequestSignatureDialog\" \
         mixin=\"HousingCharterRequestSignatureFrameMixin\" \
         inherits=\"TranslucentFrameTemplate\" toplevel=\"true\" frameStrata=\"DIALOG\" \
         hidden=\"true\" parent=\"UIParent\">`. \
         HousingEventHandlerMixin:OpenCharterSignatureRequest displays it via \
         `StaticPopupSpecial_Show(HousingCharterRequestSignatureDialog)` (line 341 of the \
         event handler addon)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_mixin_publishes_eleven_methods(env: &WowLuaEnv) {
    load_housing_charter(env);

    for method in [
        "OnLoad",
        "OnRequestClicked",
        "OnSettingsClicked",
        "OnCloseClicked",
        "OnEvent",
        "OnShow",
        "OnHide",
        "UpdateRequestButton",
        "AddSignature",
        "RemoveSignature",
        "UpdateSettingsButton",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HousingCharterMixin['{method}']) == 'function'"
            ))
            .expect("HousingCharterMixin method existence query should succeed");
        assert!(
            exists,
            "HousingCharterMixin must expose `:{method}()` — the mixin drives the charter \
             signing dialog: OnLoad creates the FramePool of HousingCharterSignatureTemplate \
             frames into self.SignaturesFrame and wires Request/Settings/Close button \
             OnClicks via GenerateClosure; OnRequestClicked calls \
             C_Housing.OnRequestSignatureClicked + plays HOUSING_CHARTER_BUTTON; \
             OnSettingsClicked LoadAddOn-pulls Blizzard_HousingCreateNeighborhood, sets the \
             charter info on HousingCreateNeighborhoodCharterFrame, ShowUIPanels it; \
             OnCloseClicked HideUIPanels self + plays HOUSING_CHARTER_BUTTON; OnEvent \
             dispatches the four HousingCharterFrameShowingEvents \
             (OPEN_NEIGHBORHOOD_CHARTER, PLAYER_TARGET_CHANGED, \
             ADD_NEIGHBORHOOD_CHARTER_SIGNATURE, REMOVE_NEIGHBORHOOD_CHARTER_SIGNATURE) into \
             SetCharterInfo / UpdateRequestButton / AddSignature / RemoveSignature; OnShow / \
             OnHide register/unregister those events and play HOUSING_CHARTER_OPEN / \
             HOUSING_CHARTER_CLOSE; UpdateRequestButton enables the Request button only when \
             the player has a non-self human target; AddSignature finds the next unsigned \
             slot in the pool and stamps the player name; RemoveSignature finds the matching \
             slot and resets it to HOUSING_CHARTER_UNSIGNED; UpdateSettingsButton enables/\
             disables the Settings button via C_Housing.CanEditCharter()"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_mixin_does_not_publish_set_charter_info_method_alias(env: &WowLuaEnv) {
    load_housing_charter(env);

    let exists: bool = env
        .eval("return type(HousingCharterMixin['SetCharterInfo']) == 'function'")
        .expect("HousingCharterMixin SetCharterInfo lookup should succeed");
    assert!(
        exists,
        "HousingCharterMixin must expose `:SetCharterInfo()` — called by OnEvent in response \
         to OPEN_NEIGHBORHOOD_CHARTER (line 42), it releases the signature pool, stores the \
         neighborhoodInfo on self, populates LocationText and NeighborhoodNameText, then \
         instantiates one signature frame per existing signature plus enough \
         HOUSING_CHARTER_UNSIGNED placeholder frames to reach numSignaturesRequired-1, and \
         applies an AnchorUtil.GridLayout (TopLeftToBottomRight, 2 columns) anchored TOPLEFT \
         to self.SignaturesFrame at offset (25, -15). Also called externally by \
         HousingEventHandlerMixin:OpenCharter (line 332)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_request_signature_frame_mixin_publishes_four_methods(env: &WowLuaEnv) {
    load_housing_charter(env);

    for method in ["OnLoad", "OnShow", "OnHide", "SetNeighborhoodInfo"] {
        let exists: bool = env
            .eval(&format!(
                "return type(HousingCharterRequestSignatureFrameMixin['{method}']) == 'function'"
            ))
            .expect(
                "HousingCharterRequestSignatureFrameMixin method existence query should succeed",
            );
        assert!(
            exists,
            "HousingCharterRequestSignatureFrameMixin must expose `:{method}()` — the dialog \
             mixin drives the popup that asks the local player to sign a neighborhood \
             charter: OnLoad wires ConfirmButton (calls \
             C_Housing.OnSignCharterClicked(ownerGUID), \
             StaticPopupSpecial_Hide(HousingCharterRequestSignatureDialog), plays \
             HOUSING_CHARTER_REQUEST_SIGN) and CancelButton (StaticPopupSpecial_Hide + plays \
             HOUSING_CHARTER_REQUEST_DECLINE); OnShow plays HOUSING_CHARTER_REQUEST_OPEN; \
             OnHide plays HOUSING_CHARTER_REQUEST_CLOSED; SetNeighborhoodInfo stores the \
             info, formats DescriptionText with HOUSING_CHARTER_REQUEST_DESCRIPTION + \
             ownerName, formats LocationText with HOUSING_CHARTER_REQUEST_LOCATION + \
             color-wrapped locationName, formats NeighborhoodNameText with \
             HOUSING_CHARTER_REQUEST_NAME + color-wrapped neighborhoodName"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_does_not_publish_signature_template_global(env: &WowLuaEnv) {
    load_housing_charter(env);

    let template_published: bool = env
        .eval("return _G['HousingCharterSignatureTemplate'] ~= nil")
        .expect("HousingCharterSignatureTemplate global lookup should succeed");
    assert!(
        !template_published,
        "HousingCharterSignatureTemplate is declared `virtual=\"true\"` \
         (Blizzard_HousingCharter.xml line 4) — virtual XML templates are NOT instantiated as \
         global frames at load time. They only materialize when a parent frame inherits them. \
         The HousingCharterFrame's SignaturesFrame instantiates one per signature via the \
         CreateFramePool(\"Frame\", self.SignaturesFrame, \"HousingCharterSignatureTemplate\") \
         call in HousingCharterMixin:OnLoad, but the template name itself stays out of `_G`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_frame_publishes_all_named_children(env: &WowLuaEnv) {
    load_housing_charter(env);

    for parent_key in [
        "Border",
        "Background",
        "Header",
        "Title",
        "PlayersLabel",
        "LocationLabel",
        "LocationText",
        "NeighborhoodNameLabel",
        "NeighborhoodNameText",
        "CharterDescription",
        "SignaturesFrame",
        "SettingsButton",
        "RequestButton",
        "CloseButton",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HousingCharterFrame['{parent_key}']) ~= 'nil'"
            ))
            .expect("HousingCharterFrame parentKey child lookup should succeed");
        assert!(
            exists,
            "HousingCharterFrame.{parent_key} must publish via `parentKey` — the XML wires \
             the named children with `parentKey=` so that the HousingCharterMixin can address \
             them without touching `_G`: Border (housing-wood-frame), Background \
             (housing-basic-panel--stone-background), Header \
             (housing-basic-panel-gradient-header-bg), Title (Game21Font, \
             HOUSING_CREATENEIGHBORHOOD_CHARTER), PlayersLabel (Game15Font_Shadow, gold), \
             LocationLabel/LocationText, NeighborhoodNameLabel/NeighborhoodNameText, \
             CharterDescription (590-wide Game15Font_Shadow), SignaturesFrame \
             (housing-basic-container, hosts the FramePool), SettingsButton/RequestButton/\
             CloseButton (each UIPanelButtonTemplate at 152x28 with GameFontNormalSmall)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_request_signature_dialog_publishes_named_children(env: &WowLuaEnv) {
    load_housing_charter(env);

    for parent_key in [
        "TitleText",
        "DescriptionText",
        "LocationText",
        "NeighborhoodNameText",
        "ConfirmButton",
        "CancelButton",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HousingCharterRequestSignatureDialog['{parent_key}']) ~= 'nil'"
            ))
            .expect("HousingCharterRequestSignatureDialog parentKey child lookup should succeed");
        assert!(
            exists,
            "HousingCharterRequestSignatureDialog.{parent_key} must publish via `parentKey` — \
             the XML wires the named children so that \
             HousingCharterRequestSignatureFrameMixin:SetNeighborhoodInfo can populate them \
             without touching `_G`: TitleText (Game12Font, gold, \
             HOUSING_CREATENEIGHBORHOOD_CHARTER), DescriptionText (Game15Font_Shadow, \
             HOUSING_CHARTER_REQUEST_DESCRIPTION), LocationText (Game15Font_Shadow, gold, \
             right-justified), NeighborhoodNameText (Game15Font_Shadow, gold, \
             right-justified), ConfirmButton/CancelButton (each UIPanelButtonTemplate at \
             152x28, anchored to the bottom-center)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_settings_reflect_supplied_editability(env: &WowLuaEnv) {
    load_housing_charter(env);
    for can_edit in [false, true, false] {
        env.state().borrow_mut().housing.charter_can_edit = can_edit;
        env.exec("HousingCharterFrame:UpdateSettingsButton()")
            .expect("original charter settings update should succeed");
        let enabled: bool = env
            .eval("return HousingCharterFrame.SettingsButton:IsEnabled()")
            .expect("settings button state should be readable");
        assert_eq!(enabled, can_edit);
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_registers_charter_events_on_show(env: &WowLuaEnv) {
    load_housing_charter(env);

    env.eval::<()>("HousingCharterFrame:Show()")
        .expect("HousingCharterFrame:Show should succeed");

    for event in [
        "OPEN_NEIGHBORHOOD_CHARTER",
        "PLAYER_TARGET_CHANGED",
        "ADD_NEIGHBORHOOD_CHARTER_SIGNATURE",
        "REMOVE_NEIGHBORHOOD_CHARTER_SIGNATURE",
    ] {
        let registered: bool = env
            .eval(&format!(
                "return HousingCharterFrame:IsEventRegistered('{event}')"
            ))
            .expect("IsEventRegistered query should succeed");
        assert!(
            registered,
            "After `HousingCharterFrame:Show()`, the `{event}` event must be registered — \
             HousingCharterMixin:OnShow calls `FrameUtil.RegisterFrameForEvents(self, \
             HousingCharterFrameShowingEvents)` where HousingCharterFrameShowingEvents is the \
             local table containing all four charter lifecycle events (defined at lines 3-9 \
             of Blizzard_HousingCharter.lua)"
        );
    }

    env.eval::<()>("HousingCharterFrame:Hide()")
        .expect("HousingCharterFrame:Hide should succeed");

    for event in [
        "OPEN_NEIGHBORHOOD_CHARTER",
        "PLAYER_TARGET_CHANGED",
        "ADD_NEIGHBORHOOD_CHARTER_SIGNATURE",
        "REMOVE_NEIGHBORHOOD_CHARTER_SIGNATURE",
    ] {
        let still_registered: bool = env
            .eval(&format!(
                "return HousingCharterFrame:IsEventRegistered('{event}')"
            ))
            .expect("IsEventRegistered query should succeed");
        assert!(
            !still_registered,
            "After `HousingCharterFrame:Hide()`, the `{event}` event must be unregistered — \
             HousingCharterMixin:OnHide calls `FrameUtil.UnregisterFrameForEvents(self, \
             HousingCharterFrameShowingEvents)`. This show/hide-scoped registration ensures \
             the charter dialog only consumes the events while visible, avoiding stale \
             updates against a hidden frame"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_publishes_no_event_listeners_before_show(env: &WowLuaEnv) {
    load_housing_charter(env);

    let registered_before_show: bool = env
        .eval("return HousingCharterFrame:IsEventRegistered('OPEN_NEIGHBORHOOD_CHARTER')")
        .expect("IsEventRegistered query should succeed");
    assert!(
        !registered_before_show,
        "Before any `HousingCharterFrame:Show()`, the `OPEN_NEIGHBORHOOD_CHARTER` event must \
         NOT be registered — HousingCharterMixin:OnLoad does NOT call RegisterEvent / \
         RegisterFrameForEvents at load time. Event registration is exclusively driven by \
         OnShow/OnHide so that a hidden HousingCharterFrame does not consume \
         neighborhood-charter updates destined for other surfaces (e.g. while another panel \
         in the `area=\"left\"` slot pushes HousingCharterFrame off the visible stack)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_charter_dependency_loads_via_game_screen_pass(env: &WowLuaEnv) {
    load_housing_charter(env);

    let templates_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HousingTemplates')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        templates_loaded,
        "Blizzard_HousingTemplates must be loaded by the Game-screen auto-discovery pass \
         before the explicit HousingCharter LoD load runs. HousingTemplates is the only \
         `## Dependencies` entry on HousingCharter's TOC, and the test harness's full \
         Game-screen pass hits it via the normal discovery flow because HousingTemplates is \
         itself non-LoD with `## AllowLoad: Both` semantics"
    );
}
}
