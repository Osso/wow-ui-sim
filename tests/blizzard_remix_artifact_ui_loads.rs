use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn remix_artifact_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_RemixArtifactUI")
}

fn remix_artifact_toc() -> PathBuf {
    remix_artifact_dir().join("Blizzard_RemixArtifactUI.toc")
}

fn artifact_ui_toc() -> PathBuf {
    blizzard_ui_dir()
        .join("Blizzard_ArtifactUI")
        .join("Blizzard_ArtifactUI.toc")
}

const REMIX_FILES: &[&str] = &[
    "Blizzard_RemixArtifactUI_Bootstrap.lua",
    "Blizzard_RemixArtifactUI.lua",
    "Blizzard_RemixArtifactUI.xml",
];

const HARD_DEPENDENCIES: &[&str] = &["Blizzard_ArtifactUI", "Blizzard_SharedTalentUI"];

const MIXIN_TABLES: &[&str] = &[
    "RemixArtifactFrameMixin",
    "RemixArtifactCurrencyFrameMixin",
    "RemixArtifactModelMixin",
    "RemixArtifactUtil",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "RemixArtifactsModelTemplate",
    "TalentButtonLegionChoiceTemplate",
    "RemixArtifactButtonsParentOverlayTemplate",
    "BronzeIncreasedNodeAnim",
    "BronzeInfiniteIncreasedNodeAnim",
];

fn load_remix_artifact_ui_with_dependency(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &artifact_ui_toc())
        .expect("Blizzard_ArtifactUI dependency should load via explicit Rust loader call");

    load_addon(&env.loader_env(), &remix_artifact_toc())
        .expect("Blizzard_RemixArtifactUI should load via explicit Rust loader call");
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved =
        find_toc_file(&remix_artifact_dir()).expect("Blizzard_RemixArtifactUI TOC should resolve");
    assert_eq!(
        resolved,
        remix_artifact_toc(),
        "Blizzard_RemixArtifactUI ships exactly one TOC at the bare \
         `Blizzard_RemixArtifactUI.toc` path — no `_Mainline` flavor split, no per-flavor \
         subdirectory like Blizzard_ReforgingUI's Classic\\ tree. find_toc_file at \
         src/loader/mod.rs:65 tries `_Mainline.toc` first (miss), then bare (hit) and returns it"
    );
}

#[test]
fn toc_declares_load_on_demand_standard_game_only_with_two_hard_deps() {
    let toc = TocFile::from_file(&remix_artifact_toc())
        .expect("Blizzard_RemixArtifactUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_RemixArtifactUI declares `## LoadOnDemand: 1` — the trait-tree panel for the \
         currently-equipped Legion Remix artifact is opened only when the player triggers it via \
         CharacterMicroButton or the equipped-artifact slot, never auto-loaded with the rest of \
         the game UI"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_RemixArtifactUI declares `## AllowLoadGameType: standard` — `standard` matches \
         the cross-flavor allowlist alongside `mainline` at src/toc.rs:294-302, so \
         is_game_type_restricted returns FALSE. The C_RemixArtifactUI and C_Traits backends are \
         retail-only but the directive is a server-side gate, not a client-side filter"
    );
    assert_eq!(
        toc.metadata.get("DefaultState").map(String::as_str),
        Some("enabled"),
        "DefaultState=enabled preserved verbatim — declares the LOD addon is on the \
         enabled-by-default whitelist so C_AddOns.LoadAddOn proceeds without an explicit user \
         opt-in. Required because the timerunning tutorial controller \
         (Blizzard_RemixArtifactTutorialUI) issues C_AddOns.LoadAddOn calls when prompting the \
         player to open the trait panel"
    );
    let deps = toc.dependencies();
    assert_eq!(
        deps.iter().map(String::as_str).collect::<Vec<_>>(),
        HARD_DEPENDENCIES,
        "Blizzard_RemixArtifactUI declares `## Dependencies: Blizzard_ArtifactUI, \
         Blizzard_SharedTalentUI` — exactly two hard deps in this order. Blizzard_ArtifactUI \
         supplies the artifact item-info/appearance backend that RefreshBackgroundModel and \
         RefreshTitle / RefreshBackground draw from; Blizzard_SharedTalentUI supplies \
         TalentFrameBaseTemplate / TalentFrameBaseMixin (inherited by RemixArtifactFrame) plus \
         TalentButtonChoiceTemplate (inherited by TalentButtonLegionChoiceTemplate)"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_RemixArtifactUI declares zero `## OptionalDeps:` — every collaborator is a \
         hard dep, no optional reverse-edges"
    );
    assert!(
        toc.saved_variables().is_empty() && toc.saved_variables_per_character().is_empty(),
        "Blizzard_RemixArtifactUI declares zero saved variables — trait selections persist via \
         the C_Traits server-backed configID/treeID state, not per-addon SavedVariables"
    );
}

#[test]
fn toc_lists_bootstrap_and_two_files_in_root_directory() {
    let toc = TocFile::from_file(&remix_artifact_toc())
        .expect("Blizzard_RemixArtifactUI TOC should parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        listed, REMIX_FILES,
        "Retail 12.1.0.69497 lists the RemixArtifact bootstrap before its Lua and XML files"
    );
}

#[test]
fn allows_screen_returns_true_only_for_game() {
    let toc = TocFile::from_file(&remix_artifact_toc())
        .expect("Blizzard_RemixArtifactUI TOC should parse");
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_RemixArtifactUI declares `## AllowLoad: Game` — must allow the in-game screen. \
         The trait-tree panel is meaningless from glue screens because it requires an equipped \
         artifact item that only exists in-world"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_RemixArtifactUI must NOT allow non-Game screens — `## AllowLoad: Game` \
             matches the Game branch only at src/toc.rs:308. (Screen tested: {screen:?})"
        );
    }
}

#[test]
fn excluded_from_eager_discovery_via_load_on_demand() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_RemixArtifactUI");
    assert!(
        !found,
        "Blizzard_RemixArtifactUI must be filtered out of eager auto-discovery on the Game \
         screen via the LoadOnDemand=1 gate at src/loader/mod.rs:530. \
         pull_required_lod_addons never imports it because no eager non-LOD addon names it as a \
         hard dep — only the LOD Blizzard_RemixArtifactTutorialUI references it indirectly via \
         the runtime `RemixArtifactFrame` global lookup, never via `## Dependencies:`. The \
         AllowLoadGameType=standard / AllowLoad=Game directives both pass; LOD is the SOLE gate"
    );
}

#[test]
fn root_directory_holds_bootstrap_and_two_files() {
    let mut entries: Vec<String> = std::fs::read_dir(remix_artifact_dir())
        .expect("Blizzard_RemixArtifactUI directory should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name != "Blizzard_RemixArtifactUI.toc")
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_RemixArtifactUI_Bootstrap.lua".to_string(),
            "Blizzard_RemixArtifactUI.lua".to_string(),
            "Blizzard_RemixArtifactUI.xml".to_string(),
        ],
        "Retail 12.1.0.69497 ships the RemixArtifact bootstrap beside its Lua and XML files"
    );
}

prefork_full_ui_case! {
fn loads_with_only_expected_remix_event_warnings(env: &WowLuaEnv) {
    load_remix_artifact_ui_with_dependency(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_RemixArtifactUI")
                || message.contains("RemixArtifact")
                || message.contains("BronzeIncreasedNodeAnim")
                || message.contains("BronzeInfiniteIncreasedNodeAnim")
        })
        .cloned()
        .collect();
    let unexpected_errors: Vec<&String> = load_errors
        .iter()
        .filter(|message| {
            !message.contains("TRAIT_TREE_CURRENCY_INFO_UPDATED")
                && !message.contains("TRY_PURCHASE_TO_NODE_PARTIAL_SUCCESS")
        })
        .collect();
    assert!(
        unexpected_errors.is_empty(),
        "Blizzard_RemixArtifactUI emitted UNEXPECTED Lua errors during explicit load. The \
         retail-only TRAIT_TREE_CURRENCY_INFO_UPDATED / TRY_PURCHASE_TO_NODE_PARTIAL_SUCCESS \
         events registered lazily by FrameUtil.RegisterFrameForEvents in OnShow are not fired \
         during static load (only on panel-open) so any warning about them is expected. \
         Anything else is a real load failure:\n  {}",
        unexpected_errors
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_explicit_load(env: &WowLuaEnv) {
    load_remix_artifact_ui_with_dependency(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_RemixArtifactUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_RemixArtifactUI') must return true after the explicit \
         load_addon call — confirms the loader registers the LoadOnDemand standard-only addon \
         with the loaded-set even though discover_blizzard_addons_for_screen filtered it out"
    );

    let dep_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ArtifactUI')")
        .expect("IsAddOnLoaded(Blizzard_ArtifactUI) probe should succeed");
    assert!(
        dep_loaded,
        "Blizzard_ArtifactUI must also be loaded — declared as a hard `## Dependencies:` entry \
         and explicitly pre-loaded by the test harness because the loader's load_addon does NOT \
         auto-resolve LOD-on-LOD dependencies (Blizzard_ArtifactUI is itself LoadOnDemand)"
    );
}
}

prefork_full_ui_case! {
fn publishes_named_top_level_frame_under_uiparent(env: &WowLuaEnv) {
    load_remix_artifact_ui_with_dependency(env);

    let frame_kind: String = env
        .eval("return type(RemixArtifactFrame)")
        .expect("RemixArtifactFrame probe should succeed");
    assert_eq!(
        frame_kind, "table",
        "RemixArtifactFrame must publish at `_G` as a table — declared at \
         Blizzard_RemixArtifactUI.xml:57 with `mixin=\"RemixArtifactFrameMixin\"` \
         `inherits=\"TalentFrameBaseTemplate\"` `parent=\"UIParent\"` `toplevel=\"true\"` \
         `enableMouse=\"true\"` `hidden=\"true\"`. The 1618×883 frame is the actual trait-tree \
         panel that opens when the player views the equipped Legion Remix artifact"
    );

    let parented_to_uiparent: bool = env
        .eval("return RemixArtifactFrame:GetParent() == UIParent")
        .expect("RemixArtifactFrame parent probe should succeed");
    assert!(
        parented_to_uiparent,
        "RemixArtifactFrame must be parented to UIParent — `parent=\"UIParent\"` XML attribute. \
         Required for the RegisterUIPanel call in OnLoad to dock the frame into the UIParent \
         center-panel slot via `area=\"center\"` attribute"
    );
}
}

prefork_full_ui_case! {
fn publishes_four_mixin_tables(env: &WowLuaEnv) {
    load_remix_artifact_ui_with_dependency(env);

    for mixin in MIXIN_TABLES {
        let kind: String = env
            .eval(&format!("return type(_G['{mixin}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must publish at `_G` as a table — Blizzard_RemixArtifactUI uses the modern \
             Mixin pattern: each mixin is declared as a `*Mixin = {{}}` table at the file's top \
             level, then methods hang off via `function Mixin:Name(...)` colon syntax. \
             RemixArtifactFrameMixin drives the trait-tree panel; RemixArtifactCurrencyFrameMixin \
             drives the currency-display Setup/OnEnter pair; RemixArtifactModelMixin drives the \
             3D PlayerModel camera/light setup; RemixArtifactUtil exposes static template-type \
             dispatch helpers"
        );
    }
}
}

prefork_full_ui_case! {
fn registers_panel_window_metadata_for_remix_artifact_frame(env: &WowLuaEnv) {
    load_remix_artifact_ui_with_dependency(env);

    let area: String = env
        .eval("return UIPanelWindows.RemixArtifactFrame.area")
        .expect("UIPanelWindows.RemixArtifactFrame.area probe should succeed");
    assert_eq!(
        area, "center",
        "UIPanelWindows.RemixArtifactFrame.area must equal \"center\" — \
         Blizzard_RemixArtifactUI.lua:41 calls RegisterUIPanel(RemixArtifactFrame, attributes) \
         with `area=\"center\"` so ShowUIPanel docks the trait-tree to the center panel slot. \
         Distinct from Blizzard_ReforgingUI's `area=\"left\"` left-slot dock"
    );

    let allow_other_panels: f64 = env
        .eval("return UIPanelWindows.RemixArtifactFrame.allowOtherPanels")
        .expect("UIPanelWindows.RemixArtifactFrame.allowOtherPanels probe should succeed");
    assert_eq!(
        allow_other_panels, 1.0,
        "UIPanelWindows.RemixArtifactFrame.allowOtherPanels must equal 1 — registered with \
         allowOtherPanels=1 so other panels (bag, character info) can sit alongside the trait \
         tree without forcing a hide-on-show"
    );
}
}

prefork_full_ui_case! {
fn virtual_templates_stay_off_global_scope(env: &WowLuaEnv) {
    load_remix_artifact_ui_with_dependency(env);

    for template in VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G['{template}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "{template} must NOT publish at `_G` — declared as `virtual=\"true\"` so the loader \
             keeps it in the template registry only, never as a global. \
             RemixArtifactsModelTemplate is the 3D PlayerModel scaffolding shared by Model and \
             AltModel children; TalentButtonLegionChoiceTemplate inherits TalentButtonChoiceTemplate \
             with the LegionChoice artSet KeyValue; RemixArtifactButtonsParentOverlayTemplate \
             carries the 3-vignette overlay layered above the talent buttons; \
             BronzeIncreasedNodeAnim / BronzeInfiniteIncreasedNodeAnim wrap looping-REPEAT \
             rotation+alpha animation groups for the per-node purchase visuals"
        );
    }
}
}

prefork_full_ui_case! {
fn module_constants_stay_file_local(env: &WowLuaEnv) {
    load_remix_artifact_ui_with_dependency(env);

    let no_constant_leak: bool = env
        .eval(
            "return _G['HEADER_WIDTH'] == nil \
                and _G['HEADER_HEIGHT'] == nil \
                and _G['BUTTON_PURCHASE_FXIDS'] == nil \
                and _G['LegionTemplatesByTalentType'] == nil \
                and _G['TemplatesByEdgeVisualStyle'] == nil \
                and _G['RemixArtifactFrameEvents'] == nil",
        )
        .expect("Module-constant probes should succeed");
    assert!(
        no_constant_leak,
        "Blizzard_RemixArtifactUI's file-local helpers must NOT leak into `_G`: HEADER_WIDTH=500 \
         and HEADER_HEIGHT=50 (used by UpdateLayout), BUTTON_PURCHASE_FXIDS={{150,142,143}} \
         (the script-animated FX IDs assigned to self.buttonPurchaseFXIDs), \
         LegionTemplatesByTalentType (Enum.TraitNodeEntryType-keyed talent-button-template \
         dispatch table consumed by RemixArtifactUtil.GetTemplateForTalentType), \
         TemplatesByEdgeVisualStyle (Enum.TraitEdgeVisualStyle-keyed edge-template dispatch \
         consumed by RemixArtifactUtil.GetEdgeTemplateType), and RemixArtifactFrameEvents (the \
         FrameUtil.RegisterFrameForEvents list with TRAIT_TREE_CURRENCY_INFO_UPDATED / \
         TRY_PURCHASE_TO_NODE_PARTIAL_SUCCESS). All declared via `local` so they stay scoped to \
         the chunk"
    );
}
}

prefork_full_ui_case! {
fn frame_mixin_inherits_talent_frame_base_lifecycle(env: &WowLuaEnv) {
    load_remix_artifact_ui_with_dependency(env);

    let lifecycle_ok: bool = env
        .eval(
            "return type(RemixArtifactFrameMixin.OnLoad) == 'function' \
                and type(RemixArtifactFrameMixin.OnShow) == 'function' \
                and type(RemixArtifactFrameMixin.OnHide) == 'function' \
                and type(RemixArtifactFrameMixin.OnEvent) == 'function' \
                and type(RemixArtifactFrameMixin.UpdateTraitTree) == 'function' \
                and type(RemixArtifactFrameMixin.SetSelection) == 'function' \
                and type(RemixArtifactFrameMixin.PurchaseRank) == 'function' \
                and type(RemixArtifactFrameMixin.IsLocked) == 'function' \
                and type(TalentFrameBaseMixin) == 'table'",
        )
        .expect("Lifecycle probe should succeed");
    assert!(
        lifecycle_ok,
        "RemixArtifactFrameMixin must expose OnLoad / OnShow / OnHide / OnEvent / UpdateTraitTree \
         / SetSelection / PurchaseRank / IsLocked as functions, AND TalentFrameBaseMixin must \
         exist as a table provided by Blizzard_SharedTalentUI. The mixin overrides several base \
         methods (CheckAndReportCommitOperation / GetConfigCommitErrorString / IsLocked / \
         AttemptConfigOperation / SetSelection / PurchaseRank) and chains via \
         TalentFrameBaseMixin.OnLoad(self) / OnShow(self) / OnHide(self) / OnEvent(self, ...) \
         calls — the inheritance must work even though it's call-chained rather than \
         CreateFromMixins-composed (RemixArtifactFrameMixin = {{}} is a plain table, not built \
         via CreateFromMixins(TalentFrameBaseMixin))"
    );
}
}

prefork_full_ui_case! {
fn util_dispatch_helpers_resolve_template_names(env: &WowLuaEnv) {
    load_remix_artifact_ui_with_dependency(env);

    let helpers_ok: bool = env
        .eval(
            "return type(RemixArtifactUtil.GetTemplateForTalentType) == 'function' \
                and type(RemixArtifactUtil.GetEdgeTemplateType) == 'function'",
        )
        .expect("RemixArtifactUtil helper probe should succeed");
    assert!(
        helpers_ok,
        "RemixArtifactUtil must expose GetTemplateForTalentType and GetEdgeTemplateType as \
         functions — they are wired into the RemixArtifactFrame XML at xml:59-60 via \
         `<KeyValue key=\"getTemplateType\" value=\"RemixArtifactUtil.GetTemplateForTalentType\" \
         type=\"global\"/>` and similar for getEdgeTemplateType. The XML's `type=\"global\"` \
         attribute means the loader resolves the global path at frame-creation time, so these \
         must exist in `_G` before the XML's <Frame name=\"RemixArtifactFrame\"> instantiates"
    );

    let circle_template: String = env
        .eval(
            "return RemixArtifactUtil.GetTemplateForTalentType(nil, \
                Enum.TraitNodeEntryType.SpendCircle, false)",
        )
        .expect("GetTemplateForTalentType probe should succeed");
    assert_eq!(
        circle_template, "TalentButtonLegionCircleTemplate",
        "RemixArtifactUtil.GetTemplateForTalentType(nil, SpendCircle, false) must return \
         \"TalentButtonLegionCircleTemplate\" — the LegionTemplatesByTalentType dispatch table \
         maps Enum.TraitNodeEntryType.SpendCircle directly to the circle button template. The \
         nil nodeInfo branch falls through to the table lookup since the Selection / \
         SubTreeSelection early-return checks `nodeInfo and (nodeInfo.type == ...)`"
    );
}
}

#[test]
fn xml_uses_method_attribute_dispatch() {
    let xml_text =
        std::fs::read_to_string(remix_artifact_dir().join("Blizzard_RemixArtifactUI.xml"))
            .expect("Blizzard_RemixArtifactUI.xml should read");
    assert!(
        xml_text.contains("<OnLoad method=\"OnLoad\"/>"),
        "Blizzard_RemixArtifactUI.xml must wire OnLoad via `<OnLoad method=\"OnLoad\"/>` — the \
         modern mixin-method dispatch attribute looks the handler up on the frame's mixin table, \
         NOT through `_G`. Used by RemixArtifactsModelTemplate's PlayerModel for \
         RemixArtifactModelMixin.OnLoad / OnEvent / OnModelLoaded"
    );
    assert!(
        xml_text.contains("<OnEnter method=\"OnEnter\"/>"),
        "Blizzard_RemixArtifactUI.xml must wire the Currency button's OnEnter via mixin-method \
         dispatch — RemixArtifactCurrencyFrameMixin.OnEnter shows the currency tooltip via \
         GameTooltip:SetCurrencyByID(self.currencyTypeID)"
    );
    assert!(
        xml_text.contains("<OnLeave function=\"GameTooltip_Hide\"/>"),
        "Blizzard_RemixArtifactUI.xml must wire the Currency button's OnLeave via the legacy \
         function= attribute pointing at the global GameTooltip_Hide — distinct from the \
         method= form, this resolves through `_G` instead of the mixin table because \
         GameTooltip_Hide is a free FrameXML helper, not a mixin method"
    );
}
