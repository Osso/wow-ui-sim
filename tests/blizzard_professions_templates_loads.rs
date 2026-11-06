use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file, load_addon};
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

fn templates_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ProfessionsTemplates")
}

fn templates_toc() -> PathBuf {
    templates_dir().join("Blizzard_ProfessionsTemplates.toc")
}

const TEMPLATES_TOC_FILES: &[&str] = &[
    "Blizzard_Professions.lua",
    "Blizzard_ProfessionsRecipeLoader.lua",
    "Blizzard_ProfessionsTransaction.lua",
    "Blizzard_ProfessionsTemplates.xml",
    "Blizzard_ProfessionsRecipeList.xml",
    "Blizzard_ProfessionsRecipeCrafterDetails.xml",
    "Blizzard_ProfessionsRecipeSlotBase.lua",
    "Blizzard_ProfessionsRecipeReagentSlotBase.xml",
    "Blizzard_ProfessionsRecipeReagentSlot.xml",
    "Blizzard_ProfessionsRecipeSalvageSlot.xml",
    "Blizzard_ProfessionsRecipeEnchantSlot.xml",
    "Blizzard_ProfessionsRecipeRecraftSlot.xml",
    "Blizzard_ProfessionsRecipeFlyout.xml",
    "Blizzard_ProfessionsRecipeFlyoutInstance.lua",
    "Blizzard_ProfessionsQualityDialog.xml",
    "Blizzard_ProfessionsRecipeSchematicForm.xml",
];

// The unified PTR TOC lists every Lua sibling explicitly, splits the schematic
// form XML into shared/family bodies, and moves the RankBar XML last.
const PTR_TEMPLATES_TOC_FILES: &[&str] = &[
    "Blizzard_Professions.lua",
    "Blizzard_ProfessionsRecipeLoader.lua",
    "Blizzard_ProfessionsTransaction.lua",
    "Blizzard_ProfessionsTemplates.lua",
    "Blizzard_ProfessionsTemplates.xml",
    "Blizzard_ProfessionsRecipeList.lua",
    "Blizzard_ProfessionsRecipeList.xml",
    "Blizzard_ProfessionsRecipeCrafterDetails.xml",
    "Blizzard_ProfessionsRecipeSlotBase.lua",
    "Blizzard_ProfessionsRecipeReagentSlotBase.xml",
    "Blizzard_ProfessionsRecipeReagentSlot.xml",
    "Blizzard_ProfessionsRecipeSalvageSlot.xml",
    "Blizzard_ProfessionsRecipeEnchantSlot.xml",
    "Blizzard_ProfessionsRecipeRecraftSlot.xml",
    "Blizzard_ProfessionsRecipeFlyout.xml",
    "Blizzard_ProfessionsRecipeFlyoutInstance.lua",
    "Blizzard_ProfessionsQualityDialog.xml",
    "Blizzard_ProfessionsRecipeSchematicForm.lua",
    "Mainline/Blizzard_ProfessionsRecipeSchematicForm.xml",
    "Blizzard_ProfessionsRecipeSchematicForm.xml",
    "Blizzard_ProfessionsRankBar.xml",
];

// The pinned Forever TOC resolves `[Game]` to Camelot, so it lists the Camelot
// Profession and schematic-form overrides.
const FOREVER_TEMPLATES_TOC_FILES: &[&str] = &[
    "Blizzard_Professions.lua",
    "Camelot/Blizzard_Professions.lua",
    "Blizzard_ProfessionsRecipeLoader.lua",
    "Blizzard_ProfessionsTransaction.lua",
    "Blizzard_ProfessionsTemplates.lua",
    "Blizzard_ProfessionsTemplates.xml",
    "Blizzard_ProfessionsRecipeList.lua",
    "Blizzard_ProfessionsRecipeList.xml",
    "Blizzard_ProfessionsRecipeCrafterDetails.xml",
    "Blizzard_ProfessionsRecipeSlotBase.lua",
    "Blizzard_ProfessionsRecipeReagentSlotBase.xml",
    "Blizzard_ProfessionsRecipeReagentSlot.xml",
    "Blizzard_ProfessionsRecipeSalvageSlot.xml",
    "Blizzard_ProfessionsRecipeEnchantSlot.xml",
    "Blizzard_ProfessionsRecipeRecraftSlot.xml",
    "Blizzard_ProfessionsRecipeFlyout.xml",
    "Blizzard_ProfessionsRecipeFlyoutInstance.lua",
    "Blizzard_ProfessionsQualityDialog.xml",
    "Blizzard_ProfessionsRecipeSchematicForm.lua",
    "Camelot/Blizzard_ProfessionsRecipeSchematicFormOverride.lua",
    "Camelot/Blizzard_ProfessionsRecipeSchematicForm.xml",
    "Blizzard_ProfessionsRecipeSchematicForm.xml",
    "Blizzard_ProfessionsRankBar.xml",
];

fn templates_toc_files() -> &'static [&'static str] {
    if toc_layout::forever() {
        FOREVER_TEMPLATES_TOC_FILES
    } else if toc_layout::unified_toc_tree() {
        PTR_TEMPLATES_TOC_FILES
    } else {
        TEMPLATES_TOC_FILES
    }
}

const REQUIRED_DEPS: &[&str] = &["Blizzard_Colors"];

const LOAD_WITH_TRIGGERS: &[&str] = &[
    "Blizzard_ProfessionsCrafting",
    "Blizzard_ProfessionsCustomerOrders",
];

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "ProfessionsButtonMixin",
    "ProfessionsConcentrateToggleButtonMixin",
    "ProfessionsCrafterDetailsStatLineMixin",
    "ProfessionsCrafterTableCellCommissionMixin",
    "ProfessionsCrafterTableCellCustomerNameMixin",
    "ProfessionsCrafterTableCellExpirationMixin",
    "ProfessionsCrafterTableCellItemNameMixin",
    "ProfessionsCrafterTableCellNameMixin",
    "ProfessionsCrafterTableCellNumAvailableMixin",
    "ProfessionsCrafterTableCellQualityMixin",
    "ProfessionsCrafterTableCellReagentsMixin",
    "ProfessionsCrafterTableCellTipMixin",
    "ProfessionsCrafterTableHeaderStringMixin",
    "ProfessionsCurrencyMixin",
    "ProfessionsCurrencyWithLabelMixin",
    "ProfessionsCustomerTableCellExpirationMixin",
    "ProfessionsCustomerTableCellIlvlMixin",
    "ProfessionsCustomerTableCellItemNameMixin",
    "ProfessionsCustomerTableCellLevelMixin",
    "ProfessionsCustomerTableCellSkillMixin",
    "ProfessionsCustomerTableCellSlotsMixin",
    "ProfessionsCustomerTableCellStatusMixin",
    "ProfessionsCustomerTableCellTypeMixin",
    "ProfessionsEnchantSlotMixin",
    "ProfessionsFavoriteButtonMixin",
    "ProfessionsFlyoutCurrencyButtonMixin",
    "ProfessionsFlyoutItemButtonMixin",
    "ProfessionsFlyoutMixin",
    "ProfessionsQualityDialogMixin",
    "ProfessionsQualityMeterMixin",
    "ProfessionsReagentContainerMixin",
    "ProfessionsReagentSlotButtonMixin",
    "ProfessionsReagentSlotMixin",
    "ProfessionsRecipeCrafterDetailsMixin",
    "ProfessionsRecipeListCategoryMixin",
    "ProfessionsRecipeListMixin",
    "ProfessionsRecipeListPanelMixin",
    "ProfessionsRecipeListRecipeMixin",
    "ProfessionsRecipeSchematicFormMixin",
    "ProfessionsRecipeSlotBaseMixin",
    "ProfessionsRecipeTransactionMixin",
    "ProfessionsRecraftInputSlotMixin",
    "ProfessionsRecraftOutputSlotMixin",
    "ProfessionsRecraftSlotMixin",
    "ProfessionsSalvageSlotMixin",
    "ProfessionsTableBuilderMixin",
    "ProfessionsTableCellTextMixin",
];

const VIRTUAL_TEMPLATES_SAMPLE: &[&str] = &[
    "ProfessionsButtonTemplate",
    "ProfessionsCurrencyTemplate",
    "ProfessionsFlyoutTemplate",
    "ProfessionsQualityDialogTemplate",
    "ProfessionsReagentSlotTemplate",
    "ProfessionsReagentSalvageTemplate",
    "ProfessionsReagentEnchantTemplate",
    "ProfessionsRecraftSlotTemplate",
    "ProfessionsRecipeListTemplate",
    "ProfessionsRecipeListRecipeTemplate",
    "ProfessionsRecipeListCategoryTemplate",
    "ProfessionsRecipeSchematicFormTemplate",
    "ProfessionsRecipeCrafterDetailsTemplate",
    "ProfessionsStatusBarArtTemplate",
    "ProfessionsTableCellTextTemplate",
];

fn load_templates_with_deps(env: &WowLuaEnv) {
    let colors_toc = blizzard_ui_dir()
        .join("Blizzard_Colors")
        .join("Blizzard_Colors.toc");
    load_addon(&env.loader_env(), &colors_toc).expect("Blizzard_Colors loads cleanly");
    load_addon(&env.loader_env(), &templates_toc())
        .expect("Blizzard_ProfessionsTemplates loads cleanly");
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

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn profession_recipe_source_button_displays_supplied_tooltip_text() {
    let env = load_full_game_ui();
    // WM92's universal character default is UNKNOWN. Supply a present object;
    // the fixture's empty rule collection leaves CraftFromBankRestricted inactive.
    env.state().borrow_mut().bank_reagent_object = Some(false);
    load_addon(&env.loader_env(), &templates_toc()).expect("load original profession templates");
    let profession_toc = find_toc_file(&blizzard_ui_dir().join("Blizzard_Professions"))
        .expect("find original professions addon");
    load_addon(&env.loader_env(), &profession_toc).expect("load original professions addon");
    env.state()
        .borrow_mut()
        .recipe_source_text
        .insert(100001, "Found in a weathered journal".into());
    env.exec(
        r#"
        local form = CreateFrame("Frame", "RecipeSourceTimingForm", UIParent, "ProfessionsRecipeSchematicFormTemplate")
        local recipe = C_TradeSkillUI.GetRecipeInfo(100001)
        assert(recipe, "fixture recipe must exist")
        recipe.learned = false
        recipe.disabled = true
        recipe.disabledReason = "Requires a trainer"
        form:Init(recipe)
        assert(form.RecipeSourceButton:IsShown(), "recipe source button must be shown")
        form.RecipeSourceButton:GetScript("OnEnter")(form.RecipeSourceButton)
        assert(GameTooltip:IsShown(), "recipe source hover must show tooltip")
        assert(GameTooltip:GetLeftLine(1):GetText() == "Found in a weathered journal")
        "#,
    )
    .unwrap();
}

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn blizzard_professions_templates_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&templates_dir()).expect("Blizzard_ProfessionsTemplates TOC resolves");
    assert_eq!(
        resolved,
        templates_toc(),
        "Blizzard_ProfessionsTemplates ships exactly one bare TOC — no `_Mainline.toc` \
         variant. The shared template surface is reused across mainline + classic flavors"
    );

    let mainline = templates_dir().join("Blizzard_ProfessionsTemplates_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {}",
        mainline.display()
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_professions_templates_toc_declares_lod_with_load_with_triggers() {
    let toc =
        TocFile::from_file(&templates_toc()).expect("Blizzard_ProfessionsTemplates TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 1` — the templates surface is materialized \
         lazily. Despite being LOD, the `## LoadWith:` directive triggers eager load \
         when any sibling profession addon loads"
    );
    assert!(!toc.is_load_first());
    assert!(
        !toc.is_secure_env(),
        "TOC must NOT declare `## UseSecureEnvironment:` — pure template/mixin surface"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC has no `## AllowLoadGameType:` directive — `is_game_type_restricted()` \
         returns false when the metadata key is absent (cross-flavor)"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC has no `## AllowLoad:` directive — `allows_screen` defaults to Game-only \
         when the key is absent"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Default Game-only screen gate must EXCLUDE {screen:?}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_professions_templates_toc_declares_load_with_triggers() {
    let toc =
        TocFile::from_file(&templates_toc()).expect("Blizzard_ProfessionsTemplates TOC parses");

    let triggers = toc.load_with();
    let actual: Vec<&str> = triggers.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        actual, LOAD_WITH_TRIGGERS,
        "`## LoadWith:` must list both sibling addons in declaration order: \
         Blizzard_ProfessionsCrafting (the historical name — note: this addon does \
         NOT exist as a directory in the current source tree, so this trigger is \
         effectively a no-op; it survives in the metadata as a forward-compat hook \
         for a future split) and Blizzard_ProfessionsCustomerOrders (the actual \
         consumer that triggers eager-load of the templates surface). The \
         `load_with()` accessor at src/toc.rs:221 splits the comma-separated value \
         the same way `dependencies()` does, but the loader treats the entries as \
         REVERSE deps (when X loads, also load Y) rather than forward deps (when \
         Y loads, ensure X is loaded first)"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_professions_templates_toc_declares_one_dependency() {
    let toc =
        TocFile::from_file(&templates_toc()).expect("Blizzard_ProfessionsTemplates TOC parses");

    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, REQUIRED_DEPS,
        "TOC must declare exactly 1 dep: Blizzard_Colors (publishes the \
         PROFESSIONS_QUALITY_COLORS table consumed by the quality-meter, the \
         crafting-detail star-tier coloring, the recipe-list quality borders, and \
         the table-cell quality-icon tinting). No dependency on the simulator's \
         core surface — every TableBuilder / CallbackRegistry / FlyoutButtonMixin \
         primitive lives in the always-loaded Blizzard_SharedXML / SharedXMLBase \
         tier"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` declared"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_professions_templates_toc_declares_no_saved_variables() {
    let toc =
        TocFile::from_file(&templates_toc()).expect("Blizzard_ProfessionsTemplates TOC parses");

    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare zero `## SavedVariables:` — pure template/mixin surface; \
         all state lives on the parent profession addon (Blizzard_Professions / \
         Blizzard_ProfessionsCustomerOrders)"
    );
    assert!(
        toc.saved_variables_per_character().is_empty(),
        "TOC must declare zero `## SavedVariablesPerCharacter:`"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_professions_templates_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(templates_toc())
        .expect("Blizzard_ProfessionsTemplates TOC reads utf-8");

    assert!(
        raw.contains("## Title: Blizzard Professions Templates"),
        "TOC must declare `## Title: Blizzard Professions Templates`"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment`"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly"
    );
    assert!(
        raw.contains(
            "## LoadWith: Blizzard_ProfessionsCrafting, Blizzard_ProfessionsCustomerOrders"
        ),
        "TOC must declare the 2-trigger `## LoadWith:` line in the canonical \
         comma-separated form"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_Colors"),
        "TOC must declare `## Dependencies: Blizzard_Colors` as a single-entry line"
    );

    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare `## AllowLoad:` or `## AllowLoadGameType:`"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare `## SavedVariables:`"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare `## OptionalDeps:`"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:`"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:`"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_professions_templates_toc_lists_sixteen_files_in_canonical_order() {
    let toc =
        TocFile::from_file(&templates_toc()).expect("Blizzard_ProfessionsTemplates TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed,
        templates_toc_files(),
        "TOC must list the Lua bootstraps first (Blizzard_Professions.lua \
         publishes ProfessionsTableConstants + the helper globals; \
         ProfessionsRecipeLoader.lua publishes the recipe-data lazy-load \
         shim; ProfessionsTransaction.lua publishes \
         ProfessionsRecipeTransactionMixin which models a pending craft as a \
         transaction object), then the multi-template XML files \
         (Templates.xml ships the table-cell families; RecipeList.xml ships \
         the scrollable list templates; RecipeCrafterDetails.xml ships the \
         crafter detail panel), then the base-slot Lua followed by the \
         slot-variant XML files in dependency order, then Flyout.xml + \
         FlyoutInstance.lua, and finally QualityDialog.xml + the schematic \
         form (the heaviest template). The interleaved Lua/XML ordering is \
         critical — each XML file's `mixin=...` attributes resolve at parse \
         time against globals declared by the preceding Lua files. The unified \
         PTR TOC lists every Lua sibling explicitly, splits the schematic form \
         into shared/family bodies, and moves the RankBar XML last"
    );
}
}

#[test]
fn blizzard_professions_templates_does_not_appear_in_eager_discovery() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_ProfessionsTemplates");
        assert!(
            !found,
            "Blizzard_ProfessionsTemplates must NOT appear in eager discovery for \
             {screen:?} — `## LoadOnDemand: 1` excludes it from the eager pool. \
             Loaded transitively via the consumer addon's dep chain or via the \
             `## LoadWith:` reverse-dep trigger when a sibling profession addon loads"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_professions_templates_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_ProfessionsTemplates");
    assert!(
        found,
        "Blizzard_ProfessionsTemplates must appear in `discover_all_blizzard_addons`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_professions_templates_loads_explicitly_after_dependencies(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_templates_with_deps(&env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_ProfessionsTemplates")
                || message.contains("ProfessionsTableConstants")
                || message.contains("ProfessionsButtonMixin")
                || message.contains("ProfessionsRecipeListMixin")
                || message.contains("ProfessionsTableBuilderMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ProfessionsTemplates emitted addon-specific Lua errors during \
         load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_professions_templates_publishes_forty_seven_mixin_globals(env: &WowLuaEnv) {
    load_templates_with_deps(&env);

    assert_eq!(
        PUBLIC_MIXIN_GLOBALS.len(),
        47,
        "Blizzard_ProfessionsTemplates publishes exactly 47 mixin globals — \
         changes to the mixin set must be reflected in this constant"
    );

    for name in PUBLIC_MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{name})"))
            .unwrap_or_else(|err| panic!("type(_G.{name}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{name} must publish as a table — Blizzard_ProfessionsTemplates is the \
             shared template-and-mixin surface for all profession panels. The 47 \
             mixins span: 12 ProfessionsCrafterTableCell* mixins (commission / \
             customer-name / expiration / item-name / name / num-available / quality \
             / reagents / tip + table-header-string + 2 standalone table-cell-text \
             variants) for the crafter-side order-list rows; 8 ProfessionsCustomerTableCell* \
             mixins (expiration / ilvl / item-name / level / skill / slots / status \
             / type) for the customer-side order-list rows; ProfessionsButtonMixin / \
             ProfessionsConcentrateToggleButtonMixin / ProfessionsFavoriteButtonMixin \
             for the toolbar buttons; ProfessionsCurrencyMixin (+ WithLabel variant) \
             for currency frames; ProfessionsFlyout* / ProfessionsRecipeFlyout* for \
             the reagent picker; ProfessionsRecipeList* (Mixin / Category / Recipe / \
             Panel) for the scrollable recipe list; ProfessionsRecipeSlotBase / \
             ProfessionsReagentSlot* / ProfessionsRecraft* / ProfessionsSalvageSlot \
             / ProfessionsEnchantSlot for the slot-variant family; \
             ProfessionsRecipeCrafterDetailsMixin (+ ProfessionsCrafterDetailsStatLineMixin) \
             for the crafter-detail panel; ProfessionsRecipeSchematicFormMixin for \
             the recipe-detail form; ProfessionsRecipeTransactionMixin for the \
             pending-craft transaction object; ProfessionsTableBuilderMixin / \
             ProfessionsTableCellTextMixin for the shared TableBuilder integration; \
             ProfessionsQualityMeterMixin / ProfessionsQualityDialogMixin for quality \
             tier picking; ProfessionsReagentContainerMixin for reagent-row layout"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_templates_publishes_table_constants_global(env: &WowLuaEnv) {
    load_templates_with_deps(&env);

    let kind: String = env
        .eval("return type(_G.ProfessionsTableConstants)")
        .expect("ProfessionsTableConstants type probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.ProfessionsTableConstants must publish as a table — declared at line 1 \
         of Blizzard_ProfessionsTemplates.lua. Holds the shared TableBuilder column \
         configuration: Width / Padding / FillCoefficient / LeftCellPadding / \
         RightCellPadding for each named column (Name, Tip, NumAvailable, Quality, \
         Reagents, Expiration, ItemName, Ilvl, Slots, Level, Skill, Status, Type, \
         Commission, MaxCommission, ActualCommission, AvgCommission, CustomerName, \
         Reagents.Padding) plus the StandardPadding / NoPadding constants"
    );

    let standard_padding: i64 = env
        .eval("return ProfessionsTableConstants.StandardPadding")
        .expect("StandardPadding probe succeeds");
    let no_padding: i64 = env
        .eval("return ProfessionsTableConstants.NoPadding")
        .expect("NoPadding probe succeeds");
    assert_eq!(
        standard_padding, 10,
        "ProfessionsTableConstants.StandardPadding pins to 10 pixels — every \
         column's Padding field is initialized to one of these two constants"
    );
    assert_eq!(
        no_padding, 0,
        "ProfessionsTableConstants.NoPadding pins to 0 — used for columns that \
         pack flush against the row border"
    );

    let name_width: i64 = env
        .eval("return ProfessionsTableConstants.Name.Width")
        .expect("Name.Width probe succeeds");
    assert_eq!(
        name_width, 100,
        "ProfessionsTableConstants.Name.Width pins to 100 — the canonical column \
         layout that drives every Crafter / Customer table view"
    );
}
}

prefork_full_ui_case! {
fn blizzard_professions_templates_virtual_templates_not_in_global_env(env: &WowLuaEnv) {
    load_templates_with_deps(&env);

    for template in VIRTUAL_TEMPLATES_SAMPLE {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the template \
             registry, NOT in the global environment. Blizzard_ProfessionsTemplates \
             ships 46 virtual templates total; this test pins a representative \
             sample covering the major template families: ProfessionsButtonTemplate \
             (the standard profession action button); ProfessionsCurrencyTemplate \
             (currency display); ProfessionsFlyoutTemplate (reagent flyout); \
             ProfessionsQualityDialogTemplate (the quality-tier picker); the 4 slot \
             variants (ReagentSlot / ReagentSalvage / ReagentEnchant / RecraftSlot); \
             RecipeListTemplate + RecipeListRecipeTemplate + RecipeListCategoryTemplate \
             (the scrollable list family); RecipeSchematicFormTemplate + \
             RecipeCrafterDetailsTemplate (the heaviest two recipe-detail templates); \
             ProfessionsStatusBarArtTemplate (the rank progress bar); \
             ProfessionsTableCellTextTemplate (the shared TableBuilder text-cell)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_templates_publishes_no_named_top_level_frames(env: &WowLuaEnv) {
    load_templates_with_deps(&env);

    let kind: String = env
        .eval(
            "return type(_G.ProfessionsTemplatesFrame) \
             .. ',' .. type(_G.ProfessionsTemplatesPanel) \
             .. ',' .. type(_G.ProfessionsTemplates)",
        )
        .expect("templates-namespace probe succeeds");
    assert_eq!(
        kind, "nil,nil,nil",
        "Blizzard_ProfessionsTemplates publishes ZERO named non-virtual top-level \
         frames — it is a pure template / mixin / constants surface. The named \
         frames live on the consumer addons (ProfessionsFrame from \
         Blizzard_Professions, ProfessionsCustomerOrdersFrame from \
         Blizzard_ProfessionsCustomerOrders). This is the canonical pattern for a \
         shared-templates addon: declare virtual templates + mixins, never \
         materialize a named frame"
    );
}
}
