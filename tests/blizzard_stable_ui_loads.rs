use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn stable_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_StableUI")
}

fn stable_ui_toc() -> PathBuf {
    stable_ui_dir().join("Blizzard_StableUI.toc")
}

const PUBLISHED_MIXINS: &[&str] = &[
    "StableFrameMixin",
    "StableTogglePetButtonMixin",
    "StableReleasePetButtonMixin",
    "StablePetFavoriteButtonMixin",
    "StableActivePetListMixin",
    "StablePetNameBoxMixin",
    "StablePetNameEditButtonMixin",
    "StableStabledPetButtonTemplateMixin",
    "StableSearchBoxMixin",
    "StableActivePetButtonTemplateMixin",
    "StableBeastMasterSecondaryPetButtonMixin",
    "StablePetInfoMixin",
    "StablePetTypeStringMixin",
    "StabledPetListCategoryMixin",
    "StableStabledPetListMixin",
    "StableTutorialButtonMixin",
    "StablePetModelSceneMixin",
    "StablePetAbilityMixin",
    "StablePetAbilitiesListMixin",
    "StablePetSpecializationMixin",
];

const STABLE_FRAME_METHODS: &[&str] = &[
    "OnLoad",
    "InitFilterDropdown",
    "OnPetSelected",
    "OnPetSwapRequested",
    "OnShow",
    "OnHide",
    "OnEvent",
    "RefreshSelectedPetData",
    "Refresh",
    "SetupPetCounter",
    "ToggleHelpPlates",
];

const STABLED_PET_LIST_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "SetSortMode",
    "ToggleShowExoticOnly",
    "SetSearchString",
    "Refresh",
    "BuildListCategories",
    "UpdateDisplayedPets",
    "PetPassesSearch",
];

const ACTIVE_PET_BUTTON_METHODS: &[&str] = &[
    "SetLocked",
    "SetDesaturated",
    "SetPet",
    "Reset",
    "OnLoad",
    "OnPetSelected",
    "OnHide",
    "OnClick",
    "RefreshTooltip",
    "OnEnter",
    "OnLeave",
    "OnDragStart",
    "OnReceiveDrag",
    "TryAcceptPetSwap",
];

const STABLED_PET_BUTTON_METHODS: &[&str] = &[
    "OnLoad",
    "OnPetSelected",
    "OnFavoritesUpdated",
    "RefreshFavoriteIcon",
    "OnDragStart",
    "OnReceiveDrag",
    "StablePet",
    "SetPet",
    "OnClick",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "StableActivePetButtonTemplate",
    "StableStabledPetButtonTemplate",
    "StableStabledPetListCategoryButtonTemplate",
    "StablePetAbilityTemplate",
];

const EXPECTED_BODY: &[&str] = &["Blizzard_StableUI.lua", "Blizzard_StableUI.xml"];
const FOREVER_EXPECTED_BODY: &[&str] = &[
    "Shared/Blizzard_StableUI_Shared.lua",
    "Camelot/Blizzard_StableUI.lua",
    "Camelot/Blizzard_StableUI.xml",
];

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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
retail_ptr_forever_toc_fixture! {
#[test]
fn find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&stable_ui_dir()).expect("Blizzard_StableUI TOC should resolve");
    assert_eq!(
        resolved,
        stable_ui_toc(),
        "Blizzard_StableUI ships exactly one bare TOC — the hunter pet \
         stable panel is mainline-retail-only (no flavor variants), gated \
         via `## AllowLoadGameType: standard` rather than per-flavor TOC \
         files because the C_StableInfo namespace and PET_STABLE_* event \
         family are exclusive to standard retail builds"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_declares_three_directives_with_help_plate_dependency() {
    let toc = TocFile::from_file(&stable_ui_toc()).expect("Blizzard_StableUI TOC should parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_StableUI omits `## LoadOnDemand:` — the addon eager-loads \
         on the Game screen so the PET_STABLE_SHOW event has a registered \
         listener BEFORE the player first interacts with a stable master \
         NPC. ShowUIPanel(self) is the response, and that hook must exist \
         at NPC-interaction time, not be queued for later"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_HelpPlate".to_string()],
        "Blizzard_StableUI declares ONE dependency via plural \
         `## Dependencies:` — Blizzard_HelpPlate provides the global \
         HelpPlate.Show / HelpPlate.Hide / HelpPlate.IsShowingHelpInfo API \
         that StableFrameMixin:ToggleHelpPlates and OnHide call directly. \
         Without HelpPlate loaded first, the StableFrame_HelpPlate ToolTip \
         data table (defined at lines 1043-1049) would have nothing to \
         feed into. Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_StableUI declares zero saved variables — pet sort mode \
         is persisted via the petStableSort CVar (GetCVar/SetCVar), and \
         exotic-only filter via petStableShowExoticOnly CVar. Favorites \
         are server-stored via C_StableInfo.SetPetFavorite, and \
         category-collapse state is per-session-only on the mixin"
    );
    assert!(toc.default_enabled());
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_is_standard_only_via_allow_load_game_type() {
    let toc = TocFile::from_file(&stable_ui_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_StableUI declares `## AllowLoadGameType: standard` — \
         is_game_type_restricted (src/toc.rs:294-302) returns FALSE because \
         `standard` matches the allowed-set {{mainline, standard}}. The \
         addon participates in standard-retail auto-discovery; only \
         non-mainline flavors (plunderstorm, classic, wrath, cata, mists, \
         wowhack) would set the flag to true"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without `## AllowLoad:` directive, the metadata-absent fallback \
         (src/toc.rs:311 None branch) restricts the addon to the Game \
         screen only — pet stable interactions are exclusively in-world, \
         never on Login / CharacterSelect / CharacterCreate glue panels"
    );
    assert!(!toc.allows_screen(ScreenKind::Login));
    assert!(!toc.allows_screen(ScreenKind::CharacterSelect));
    assert!(!toc.allows_screen(ScreenKind::CharacterCreate));
}
}

post_wrath_ui_fixture! {
#[test]
fn raw_bytes_pin_three_metadata_directives_only() {
    let raw = std::fs::read_to_string(stable_ui_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard Stable UI",
        "## AllowLoadGameType: standard",
        "## Dependencies: Blizzard_HelpPlate",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw bytes MUST pin the `{directive}` directive — \
             Blizzard_StableUI's TOC is minimal (3 metadata lines + 2 body \
             entries totaling 139 bytes), so each directive carries its \
             full load-bearing weight. Title gives the in-game UI label; \
             AllowLoadGameType pins the addon to standard retail; \
             Dependencies pulls Blizzard_HelpPlate ahead of this addon's \
             Lua run so HelpPlate.Show is callable from ToggleHelpPlates"
        );
    }

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## SavedVariablesPerCharacter"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## AllowLoad:"));
    assert!(!raw.contains("## OnlyBetaAndPTR"));
}
}

post_wrath_ui_fixture! {
#[test]
fn body_lists_lua_before_xml_for_mixin_resolution() {
    let toc = TocFile::from_file(&stable_ui_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected_body = if cfg!(feature = "client-wowforever") {
        FOREVER_EXPECTED_BODY
    } else {
        EXPECTED_BODY
    };
    assert_eq!(
        body.len(),
        expected_body.len(),
        "selected profile body entries must match its TOC: {body:?}"
    );

    for (i, want) in expected_body.iter().enumerate() {
        assert_eq!(
            &body[i], want,
            "Body entry {i}: expected {want}, got {}",
            body[i]
        );
    }

    let xml_index = body
        .iter()
        .position(|path| path.ends_with(".xml"))
        .expect("selected stable UI TOC lists XML");
    assert!(
        body.iter()
            .enumerate()
            .filter(|(_, path)| path.ends_with(".lua"))
            .all(|(index, _)| index < xml_index),
        "every selected Lua mixin source must load before XML registration: {body:?}"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn auto_discovers_on_game_screen_with_help_plate_dep() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let names: Vec<&str> = addons.iter().map(|(n, _)| n.as_str()).collect();

    assert!(
        names.contains(&"Blizzard_StableUI"),
        "Blizzard_StableUI MUST appear in Game-screen auto-discovery — \
         it lacks ## LoadOnDemand, declares ## AllowLoadGameType: standard \
         (which is_game_type_restricted treats as allowed), and the \
         AllowLoad-absent fallback maps it to the Game screen. Found \
         addons: {names:?}"
    );

    assert!(
        names.contains(&"Blizzard_HelpPlate"),
        "Blizzard_HelpPlate MUST also be in Game-screen discovery — its \
         TOC declares `## AllowLoad: Both` so it's eligible on every \
         screen. StableUI's ## Dependencies on HelpPlate is satisfied at \
         load-time because both addons appear in the same eager-discovery \
         pass; the loader orders deps before dependents internally"
    );
}
}

#[test]
fn excluded_from_glue_screens() {
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_StableUI");
        assert!(
            !found,
            "Blizzard_StableUI MUST NOT auto-discover on glue screen \
             {screen:?} — without ## AllowLoad the toc.rs:311 fallback \
             allows only ScreenKind::Game. Hunter pet stables are an \
             in-world NPC interaction, never reachable before character \
             enters world"
        );
    }
}

prefork_full_ui_case! {
fn eager_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_StableUI")
                || message.contains("StableFrameMixin")
                || message.contains("StableActivePetListMixin")
                || message.contains("StableStabledPetListMixin")
                || message.contains("StablePetModelSceneMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_StableUI emitted addon-specific Lua errors during \
         eager load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_reports_true_after_eager_load(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_StableUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_StableUI') must return true \
         after the eager-discovery sweep — the addon's loaded-set entry \
         is the gate that addon-checked code (other addons asking 'is \
         StableUI ready?') uses to know StableFrame is materialized"
    );
}
}

prefork_full_ui_case! {
fn publishes_twenty_mixin_tables_at_global_scope(env: &WowLuaEnv) {

    for mixin in PUBLISHED_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} type probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must publish at `_G` as a table after \
             Blizzard_StableUI loads. The .lua file declares 20 mixin \
             tables in a flat namespace: 1 panel-frame mixin (StableFrameMixin), \
             3 button-action mixins (Toggle/Release/PetFavorite), 2 list \
             container mixins (Active/Stabled), 4 list-row mixins \
             (Stabled-row/Search/ActivePet/BeastMaster — last one \
             CreateFromMixins-derived from Active), 4 info / label mixins \
             (NameBox/NameEdit/PetInfo/PetTypeString), 2 list-category \
             mixins (StabledPetListCategory + the actual list mixin), 1 \
             tutorial mixin, 1 model-scene mixin (CreateFromMixins-derived \
             from PanningModelSceneMixin), 2 ability-display mixins \
             (Ability/AbilitiesList), 1 pet-spec dropdown mixin"
        );
    }
}
}

prefork_full_ui_case! {
fn stable_frame_mixin_carries_eleven_canonical_methods(env: &WowLuaEnv) {

    for method in STABLE_FRAME_METHODS {
        let kind: String = env
            .eval(&format!("return type(StableFrameMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("StableFrameMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "StableFrameMixin.{method} must publish as a function — the \
             top-level panel mixin owns 11 methods covering panel \
             registration (OnLoad calls RegisterUIPanel with width=1040 \
             height=638 area=left), filter dropdown wiring \
             (InitFilterDropdown installs MENU_STABLE_FILTER with \
             exotic-only checkbox + 4-radio sort submenu), pet-selection \
             event flow (OnPetSelected updates ModelScene + Specialization \
             refresh, OnPetSwapRequested calls C_StableInfo.SetPetSlot \
             with reverseSelectedDisplay flag), event lifecycle \
             (OnShow/OnHide register/unregister STABLE_FRAME_ON_SHOW_EVENTS \
             family, OnEvent dispatches PET_STABLE_SHOW/CLOSED/UPDATE etc.), \
             and refresh helpers (RefreshSelectedPetData walks slot-id \
             after swap, Refresh fans out to both pet lists + counter, \
             SetupPetCounter formats STABLE_PET_COUNTER, ToggleHelpPlates \
             gates HelpPlate.Show/Hide on IsShowingHelpInfo)"
        );
    }
}
}

prefork_full_ui_case! {
fn stabled_pet_list_mixin_carries_nine_canonical_methods(env: &WowLuaEnv) {

    for method in STABLED_PET_LIST_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(StableStabledPetListMixin['{method}'])"
            ))
            .unwrap_or_else(|err| panic!("StableStabledPetListMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "StableStabledPetListMixin.{method} must publish as a function \
             — the stabled-pet ScrollBox driver owns the BuildListCategories \
             → UpdateDisplayedPets → SetSortMode pipeline that converts \
             C_StableInfo.GetStabledPetList output into a tree-structured \
             dataProvider, with category buckets keyed by specialization \
             or familyName depending on the active sortMode CVar. \
             PetPassesSearch handles the FilterBar.SearchBox text filter, \
             scanning name/family/spec/type plus all petAbilities and \
             specAbilities via C_Spell.GetSpellInfo lookups. \
             ToggleShowExoticOnly flips the petStableShowExoticOnly CVar \
             and re-runs UpdateDisplayedPets"
        );
    }
}
}

prefork_full_ui_case! {
fn active_pet_button_template_mixin_carries_fourteen_methods(env: &WowLuaEnv) {

    for method in ACTIVE_PET_BUTTON_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(StableActivePetButtonTemplateMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("StableActivePetButtonTemplateMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "StableActivePetButtonTemplateMixin.{method} must publish as a \
             function — the active-pet-slot button mixin manages slot-lock \
             state (SetLocked toggles desaturation + Lock atlas based on \
             whether the player has learned the corresponding Call Pet \
             spell), pet portrait textures (SetPet calls \
             SetPortraitTextureFromCreatureDisplayIDFlipped to mirror the \
             portrait horizontally), drag-and-drop swap routing \
             (TryAcceptPetSwap fires StableFrameMixin.PetSwapRequested via \
             EventRegistry), and tooltip rendering (RefreshTooltip \
             distinguishes locked / empty / filled / secondary-slot states \
             with PET_STABLE_SLOT_LOCKED / STABLE_EMPTY_SLOT_LABEL / \
             STABLE_SECONDARY_PET_LABEL strings)"
        );
    }
}
}

prefork_full_ui_case! {
fn stabled_pet_button_template_mixin_carries_nine_methods(env: &WowLuaEnv) {

    for method in STABLED_PET_BUTTON_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(StableStabledPetButtonTemplateMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("StableStabledPetButtonTemplateMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "StableStabledPetButtonTemplateMixin.{method} must publish as \
             a function — the stabled-pet list-row mixin handles row \
             selection (OnPetSelected swaps pet-list_active-ring vs \
             pet-list_default-ring border atlas), favorite-icon refresh \
             (RefreshFavoriteIcon shows the auctionhouse-favorite atlas \
             when C_StableInfo.IsPetFavorite returns true), and \
             pet-cursor drag handlers (OnDragStart calls \
             C_StableInfo.PickupStablePet, OnReceiveDrag calls \
             StablePet which fires PetSwapRequested with reverseSelectedDisplay=true)"
        );
    }
}
}

prefork_full_ui_case! {
fn beastmaster_secondary_pet_button_inherits_active_pet_button(env: &WowLuaEnv) {

    for inherited in ACTIVE_PET_BUTTON_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(StableBeastMasterSecondaryPetButtonMixin['{inherited}'])"
            ))
            .unwrap_or_else(|err| panic!("BeastMaster.{inherited} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "StableBeastMasterSecondaryPetButtonMixin must inherit \
             '{inherited}' from StableActivePetButtonTemplateMixin via \
             CreateFromMixins(StableActivePetButtonTemplateMixin) at \
             Blizzard_StableUI.lua:731. The bonus-pet slot reuses every \
             behavior of a regular active-pet button (lock/desaturate/portrait \
             texture/click-routing/tooltip) and only OVERRIDES Refresh, \
             OnShow, OnHide, OnEvent to gate availability on \
             C_StableInfo.IsBonusPetSlotAvailable() — the BM Animal \
             Companion talent must be active for the slot to unlock"
        );
    }

    let extra_methods = ["OnShow", "OnHide", "OnEvent", "Refresh"];
    for method in extra_methods {
        let kind: String = env
            .eval(&format!(
                "return type(StableBeastMasterSecondaryPetButtonMixin['{method}'])"
            ))
            .unwrap_or_else(|err| panic!("BeastMaster.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "BeastMaster.{method} must be defined directly on the derived \
             mixin — these four methods are the bonus-pet-slot-specific \
             override layer that toggles ACTIVE_COMBAT_CONFIG_CHANGED \
             registration and re-evaluates IsBonusPetSlotAvailable on \
             talent-config swaps"
        );
    }
}
}

prefork_full_ui_case! {
fn pet_model_scene_inherits_panning_model_scene(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(StablePetModelSceneMixin)")
        .expect("StablePetModelSceneMixin probe");
    assert_eq!(
        kind, "table",
        "StablePetModelSceneMixin must publish as table — \
         CreateFromMixins(PanningModelSceneMixin) at Blizzard_StableUI.lua:1066 \
         constructs a new table whose metatable __index falls through to \
         PanningModelSceneMixin, so any panning-camera method (drag-to-pan, \
         scroll-to-zoom) inherited from the parent stays resolvable without \
         being copied verbatim"
    );

    let own_methods = [
        "OnLoad",
        "OnMouseDown",
        "SetPet",
        "UpdatePetModel",
        "OnModelLoaded",
        "UpdateBackgroundForPet",
    ];
    for method in own_methods {
        let kind: String = env
            .eval(&format!(
                "return type(StablePetModelSceneMixin['{method}'])"
            ))
            .unwrap_or_else(|err| panic!("ModelScene.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "StablePetModelSceneMixin.{method} must publish as a function \
             — the 6 own methods on the model-scene mixin handle pet \
             model swap (UpdatePetModel calls TransitionToModelSceneID + \
             SetModelByCreatureDisplayID via the 'pet'-tagged actor), \
             specialization-keyed background swap (UpdateBackgroundForPet \
             maps STABLE_PET_SPEC_CUNNING/FEROCITY/TENACITY → \
             hunter-stable-bg-art_<spec> atlas), and the right-click \
             cursor-clear bypass for OnMouseDown"
        );
    }
}
}

prefork_full_ui_case! {
fn xml_registers_four_virtual_templates(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES {
        let probe = format!(
            "local f = CreateFrame('Button', nil, UIParent, '{template}') \
             return f ~= nil and type(f) == 'table'"
        );
        let result: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("Template {template} probe failed: {err}"));
        assert!(
            result,
            "Virtual template {template} must be registered after \
             Blizzard_StableUI.xml processes — the .xml declares 4 \
             virtual templates (StableActivePetButtonTemplate, \
             StableStabledPetButtonTemplate, \
             StableStabledPetListCategoryButtonTemplate, \
             StablePetAbilityTemplate). All four are CreateFrame-able \
             after load even though the named StableFrame is the only \
             instance materialized at load-time"
        );
    }
}
}

prefork_full_ui_case! {
fn named_stable_frame_publishes_with_panel_attributes(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(StableFrame)")
        .expect("StableFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "StableFrame (named, NOT virtual) must publish at `_G` as a \
         table after Blizzard_StableUI.xml processes — declared at \
         line 236 with name=\"StableFrame\" parent=\"UIParent\" \
         inherits=\"PortraitFrameTemplate\" toplevel=\"true\" \
         hidden=\"true\" mixin=\"StableFrameMixin\". The frame is the \
         singleton panel that PET_STABLE_SHOW event handler calls \
         ShowUIPanel(self) on"
    );

    let portrait_icon: String = env
        .eval("return tostring(StableFrame.portraitIcon)")
        .expect("portraitIcon KeyValue probe");
    assert_eq!(
        portrait_icon, "Interface\\Icons\\ClassIcon_Hunter",
        "StableFrame.portraitIcon KeyValue must be \
         'Interface\\Icons\\ClassIcon_Hunter' — pinned via <KeyValue \
         key=\"portraitIcon\" value=\"...\" type=\"string\"/> at xml:239 \
         and consumed by SetPortraitToAsset(self.portraitIcon) in \
         StableFrameMixin:OnLoad. Got: {portrait_icon}"
    );
}
}

prefork_full_ui_case! {
fn named_stable_frame_exposes_active_and_stabled_pet_lists(env: &WowLuaEnv) {

    let probe = "local f = StableFrame \
                 if not f then return 'frame nil' end \
                 local missing = {} \
                 if type(f.PetModelScene) ~= 'table' then table.insert(missing, 'PetModelScene:'..type(f.PetModelScene)) end \
                 if type(f.StabledPetList) ~= 'table' then table.insert(missing, 'StabledPetList:'..type(f.StabledPetList)) end \
                 if type(f.ActivePetList) ~= 'table' then table.insert(missing, 'ActivePetList:'..type(f.ActivePetList)) end \
                 if type(f.StableTogglePetButton) ~= 'table' then table.insert(missing, 'StableTogglePetButton:'..type(f.StableTogglePetButton)) end \
                 if type(f.ReleasePetButton) ~= 'table' then table.insert(missing, 'ReleasePetButton:'..type(f.ReleasePetButton)) end \
                 if type(f.MainHelpButton) ~= 'table' then table.insert(missing, 'MainHelpButton:'..type(f.MainHelpButton)) end \
                 if #missing == 0 then return 'OK' else return table.concat(missing, ',') end";
    let report: String = env
        .eval(probe)
        .expect("StableFrame children probe should succeed");
    assert_eq!(
        report, "OK",
        "StableFrame must materialize with all 6 top-level parentKey \
         children: PetModelScene (708x550 ModelScene anchored \
         BOTTOMRIGHT, hosting the pet 3D actor + ControlFrame + PetInfo), \
         StabledPetList (left-side ScrollBox with FilterBar.SearchBox + \
         FilterBar.FilterDropdown + ListCounter), ActivePetList (bottom \
         strip with 5 PetButtons + Divider + BeastMasterSecondaryPetButton), \
         StableTogglePetButton (155x22 stable-vs-make-active toggle), \
         ReleasePetButton (155x22 release pet popup trigger), \
         MainHelpButton (HelpPlate tutorial trigger anchored TOPLEFT). \
         Report: {report}"
    );
}
}

prefork_full_ui_case! {
fn active_pet_list_materializes_five_pet_buttons_and_secondary(env: &WowLuaEnv) {

    let probe = "local list = StableFrame and StableFrame.ActivePetList \
                 if not list then return 'list nil' end \
                 local missing = {} \
                 for i = 1, 5 do \
                     local btn = list['PetButton'..i] \
                     if type(btn) ~= 'table' then table.insert(missing, 'PetButton'..i..':'..type(btn)) end \
                 end \
                 if type(list.BeastMasterSecondaryPetButton) ~= 'table' then \
                     table.insert(missing, 'BeastMasterSecondaryPetButton:'..type(list.BeastMasterSecondaryPetButton)) \
                 end \
                 if type(list.Divider) ~= 'table' then \
                     table.insert(missing, 'Divider:'..type(list.Divider)) \
                 end \
                 if #missing == 0 then return 'OK' else return table.concat(missing, ',') end";
    let report: String = env
        .eval(probe)
        .expect("ActivePetList children probe should succeed");
    assert_eq!(
        report, "OK",
        "StableFrame.ActivePetList must materialize with all 5 numbered \
         PetButton<N> parentKeys (id=1..5, parentArray=PetButtons, each \
         inheriting StableActivePetButtonTemplate, anchored LEFT-of-previous \
         with x=25 spacing), the gold-divide Divider frame, and \
         BeastMasterSecondaryPetButton (id=6, mixin override = \
         StableBeastMasterSecondaryPetButtonMixin) on the right of the \
         divider. Report: {report}"
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_StableUI, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_stableui_toc_is_absent_from_classic_caches() {
    let dir = stable_ui_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_StableUI"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
