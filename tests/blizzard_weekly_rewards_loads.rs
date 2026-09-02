use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn weekly_rewards_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_WeeklyRewards")
}

fn weekly_rewards_toc() -> PathBuf {
    weekly_rewards_dir().join("Blizzard_WeeklyRewards.toc")
}

const MIXIN_GLOBALS: &[&str] = &[
    "WeeklyRewardsMixin",
    "WeeklyRewardOverlayMixin",
    "WeeklyRewardsActivityMixin",
    "WeeklyRewardActivityItemMixin",
    "WeeklyRewardsConcessionMixin",
    "WeeklyRewardConfirmSelectionMixin",
    "GreatVaultRetirementWarningFrameMixin",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "WeeklyRewardActivityItemFrameTemplate",
    "WeeklyRewardActivityTypeTemplate",
    "WeeklyRewardActivityTemplate",
    "WeeklyRewardAlsoItemTemplate",
    "WeeklyRewardConfirmSelectionTemplate",
    "WeeklyRewardsNineSliceTemplate",
    "WeeklyRewardOverlayTemplate",
];

const NAMED_NON_VIRTUAL_FRAMES: &[&str] =
    &["WeeklyRewardsFrame", "WeeklyRewardExpirationWarningDialog"];

fn load_weekly_rewards(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &weekly_rewards_toc())
        .expect("Blizzard_WeeklyRewards should load via explicit Rust loader call");
}

#[test]
fn find_toc_file_resolves_bare_variant() {
    let resolved =
        find_toc_file(&weekly_rewards_dir()).expect("Blizzard_WeeklyRewards TOC should resolve");
    assert_eq!(
        resolved,
        weekly_rewards_toc(),
        "Blizzard_WeeklyRewards ships exactly one bare TOC — find_toc_file probes the \
         `_Mainline.toc` variant first (miss) and falls through to the bare TOC name (hit). \
         The classic-flavor Great Vault feature did not ship until much later, so no \
         `_Classic.toc` / `_Mists.toc` companion exists"
    );
}

#[test]
fn toc_declares_lod_with_zero_dependencies() {
    let toc =
        TocFile::from_file(&weekly_rewards_toc()).expect("Blizzard_WeeklyRewards TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_WeeklyRewards declares `## LoadOnDemand: 1` — pulled in by \
         `WeeklyRewards_LoadUI()` in Blizzard_UIParent/Mainline/UIParent.lua:519 via \
         `UIParentLoadAddOn(\"Blizzard_WeeklyRewards\")` when the player interacts with the \
         Great Vault NPC. The wrapper `WeeklyRewards_ShowUI()` at line 527 guards on \
         `not WeeklyRewardsFrame` to call the loader before issuing ShowUIPanel — so the \
         addon is brought in lazily on first vault-open and stays loaded for the session"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_WeeklyRewards declares no dependencies (no `## Dependencies` / `## RequiredDep` \
         / `## RequiredDeps` keys) — the addon consumes only the C_WeeklyRewards C API + global \
         locale strings + StaticPopupDialogs (registered into the existing global at file scope) \
         + ItemMixin/NineSliceUtil/HelpTip/RegisterUIPanel/ShowUIPanel helpers. All of those are \
         provided by the eagerly-loaded core addons and the simulator's runtime surface — none \
         of them require declaring an explicit dep"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_WeeklyRewards declares no `## OptionalDeps` either"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_WeeklyRewards declares zero saved variables — every Great Vault open \
         re-fetches reward data via C_WeeklyRewards.GetActivities + GetActivityRewards on each \
         WEEKLY_REWARDS_UPDATE; the chest UI is purely a server-driven view, no client \
         persistence"
    );
}

#[test]
fn toc_omits_allow_load_directives() {
    let toc =
        TocFile::from_file(&weekly_rewards_toc()).expect("Blizzard_WeeklyRewards TOC should parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without `## AllowLoad`, allows_screen falls through to the default Game-only branch \
         at src/toc.rs:311 (None → Game)"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Without `## AllowLoad`, allows_screen rejects every glue screen ({screen:?}) — \
             the Great Vault is in-world UI only"
        );
    }

    assert!(
        !toc.is_game_type_restricted(),
        "Without `## AllowLoadGameType`, is_game_type_restricted (src/toc.rs:294-302) returns \
         false via the `unwrap_or(false)` branch — the addon is treated as gametype-unrestricted \
         (the Great Vault ships on every retail flavor since Shadowlands)"
    );
}

#[test]
fn toc_lists_bootstrap_then_xml_body_file() {
    let toc =
        TocFile::from_file(&weekly_rewards_toc()).expect("Blizzard_WeeklyRewards TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "Blizzard_WeeklyRewards_Bootstrap.lua".to_string(),
            "Blizzard_WeeklyRewards.xml".to_string(),
        ],
        "Retail 12.1.0.69497 TOC lists the bootstrap then XML; the main lua file is loaded transitively via \
         `<Script file=\"Blizzard_WeeklyRewards.lua\"/>` at line 3 of the XML, NOT from the TOC \
         body. This is a deliberate Blizzard pattern: by deferring the lua load to the XML, the \
         mixin tables (WeeklyRewardsMixin / WeeklyRewardActivityItemMixin / etc.) are populated \
         BEFORE the XML's `mixin=\"...\"` attributes try to resolve them via _G during element \
         instantiation, since `<Script file>` runs synchronously at the start of XML parsing"
    );
}

#[test]
fn toc_raw_bytes_pin_minimal_directives() {
    let raw = std::fs::read_to_string(weekly_rewards_toc())
        .expect("Blizzard_WeeklyRewards TOC should read");

    for directive in ["## Title: Blizzard Weekly Rewards", "## LoadOnDemand: 1"] {
        assert!(
            raw.contains(directive),
            "TOC must contain directive line `{directive}`"
        );
    }

    for body in [
        "Blizzard_WeeklyRewards_Bootstrap.lua [Bootstrap]",
        "Blizzard_WeeklyRewards.xml",
    ] {
        assert!(
            raw.contains(body),
            "TOC must contain body file line `{body}`"
        );
    }

    assert!(
        !raw.contains("Blizzard_WeeklyRewards.lua"),
        "TOC must NOT list the lua file — it is loaded via `<Script file>` from inside the XML"
    );

    for absent_directive in [
        "## Author:",
        "## Version:",
        "## Notes:",
        "## Dependencies:",
        "## RequiredDep:",
        "## OptionalDeps:",
        "## SavedVariables:",
        "## AllowLoad:",
        "## AllowLoadGameType:",
        "## DefaultState:",
    ] {
        assert!(
            !raw.contains(absent_directive),
            "TOC must NOT contain `{absent_directive}` — Blizzard_WeeklyRewards is the most \
             minimal addon analyzed in this campaign: Title + LoadOnDemand:1 + bootstrap/XML body lines"
        );
    }
}

#[test]
fn directory_holds_four_entries_with_bootstrap() {
    let entries = std::fs::read_dir(weekly_rewards_dir())
        .expect("Blizzard_WeeklyRewards directory should read")
        .count();
    assert_eq!(
        entries, 4,
        "Retail 12.1.0.69497 directory holds the TOC, bootstrap Lua, main Lua, and XML"
    );
}

#[test]
fn excluded_from_every_screen_auto_discovery() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_WeeklyRewards");
        assert!(
            !found,
            "Blizzard_WeeklyRewards must NOT appear in any ScreenKind auto-discovery sweep — \
             `## LoadOnDemand: 1` excludes it from every eager pass; only the explicit \
             load_addon call (driven by WeeklyRewards_LoadUI in Blizzard_UIParent) pulls it in. \
             (Screen tested: {screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_weekly_rewards(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_WeeklyRewards")
                || message.contains("WeeklyRewardsMixin")
                || message.contains("WeeklyRewardsFrame")
                || message.contains("WeeklyRewardExpirationWarningDialog")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_WeeklyRewards emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_explicit_lod(env: &WowLuaEnv) {
    load_weekly_rewards(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_WeeklyRewards')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_WeeklyRewards') must return true after the explicit \
         load_addon call"
    );
}
}

prefork_full_ui_case! {
fn loader_function_publishes_in_uiparent(env: &WowLuaEnv) {
    load_weekly_rewards(env);

    for fn_name in ["WeeklyRewards_LoadUI", "WeeklyRewards_ShowUI"] {
        let kind: String = env
            .eval(&format!("return type({fn_name})"))
            .unwrap_or_else(|err| panic!("{fn_name} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{fn_name} must publish at `_G` as a function — declared in eagerly-loaded \
             Blizzard_UIParent/Mainline/UIParent.lua (lines 519-534) BEFORE the addon itself \
             loads, so the loader hook + show wrapper are both available without bringing in \
             the addon"
        );
    }
}
}

prefork_full_ui_case! {
fn xml_script_directive_loads_the_lua_file(env: &WowLuaEnv) {
    load_weekly_rewards(env);

    let mixin_count: i64 = env
        .eval(
            "local n = 0 \
             for _, name in ipairs({'WeeklyRewardsMixin', 'WeeklyRewardOverlayMixin', \
             'WeeklyRewardsActivityMixin', 'WeeklyRewardActivityItemMixin', \
             'WeeklyRewardsConcessionMixin', 'WeeklyRewardConfirmSelectionMixin', \
             'GreatVaultRetirementWarningFrameMixin'}) do \
                if type(_G[name]) == 'table' then n = n + 1 end \
             end \
             return n",
        )
        .expect("mixin-count probe should succeed");
    assert_eq!(
        mixin_count, 7,
        "All 7 mixins from Blizzard_WeeklyRewards.lua must publish at _G as tables — proves \
         the `<Script file=\"Blizzard_WeeklyRewards.lua\"/>` directive at line 3 of the XML \
         executed during XML parsing (the lua file is NOT in the TOC body, so this is the only \
         load path)"
    );
}
}

prefork_full_ui_case! {
fn all_seven_mixins_publish_with_method_signatures(env: &WowLuaEnv) {
    load_weekly_rewards(env);

    for mixin_name in MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type({mixin_name})"))
            .unwrap_or_else(|err| panic!("{mixin_name} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin_name} must publish at `_G` as a table — declared at file scope in \
             Blizzard_WeeklyRewards.lua and consumed by the corresponding XML \
             `mixin=\"{mixin_name}\"` attribute"
        );
    }

    let chassis_methods: bool = env
        .eval(
            "return type(WeeklyRewardsMixin.OnLoad) == 'function' \
                and type(WeeklyRewardsMixin.OnShow) == 'function' \
                and type(WeeklyRewardsMixin.OnHide) == 'function' \
                and type(WeeklyRewardsMixin.OnEvent) == 'function' \
                and type(WeeklyRewardsMixin.SetUpActivity) == 'function' \
                and type(WeeklyRewardsMixin.SetUpConditionalActivities) == 'function'",
        )
        .expect("WeeklyRewardsMixin chassis probe should succeed");
    assert!(
        chassis_methods,
        "WeeklyRewardsMixin must expose its chassis methods (OnLoad / OnShow / OnHide / OnEvent \
         + SetUpActivity / SetUpConditionalActivities) — OnLoad pre-builds the Raid + Mythic + \
         World rows then RegisterUIPanel(WeeklyRewardsFrame, attributes); OnShow registers \
         WEEKLY_REWARDS_UPDATE / CHALLENGE_MODE_COMPLETED / CHALLENGE_MODE_MAPS_UPDATE then \
         calls C_WeeklyRewards.OnUIInteract; OnHide unregisters and calls \
         C_WeeklyRewards.CloseInteraction; SetUpConditionalActivities walks the activity list \
         to decide whether to show the World row or the PVP row (mutually exclusive)"
    );
}
}

prefork_full_ui_case! {
fn weekly_rewards_frame_publishes_with_register_ui_panel(env: &WowLuaEnv) {
    load_weekly_rewards(env);

    let kind: String = env
        .eval("return type(WeeklyRewardsFrame)")
        .expect("WeeklyRewardsFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "WeeklyRewardsFrame must publish at `_G` as a table — declared at \
         Blizzard_WeeklyRewards.xml:496 with `parent=\"UIParent\"` `mixin=\"WeeklyRewardsMixin\"` \
         `hidden=\"true\"` `enableMouse=\"true\"` `toplevel=\"true\"` `frameLevel=\"100\"`. \
         Confirmed registered with the UI panel system via OnLoad's RegisterUIPanel call \
         (area=\"center\", pushable=0, allowOtherPanels=1, checkFit=1)"
    );

    let name: String = env
        .eval("return WeeklyRewardsFrame:GetName()")
        .expect("WeeklyRewardsFrame:GetName() probe should succeed");
    assert_eq!(name, "WeeklyRewardsFrame");

    let hidden: bool = env
        .eval("return WeeklyRewardsFrame:IsShown()")
        .expect("WeeklyRewardsFrame:IsShown() probe should succeed");
    assert!(
        !hidden,
        "WeeklyRewardsFrame must remain hidden after load — `hidden=\"true\"` in XML and the \
         vault flow only ShowUIPanels it on first interaction via WeeklyRewards_ShowUI"
    );
}
}

prefork_full_ui_case! {
fn expiration_warning_dialog_publishes_with_high_strata(env: &WowLuaEnv) {
    load_weekly_rewards(env);

    let kind: String = env
        .eval("return type(WeeklyRewardExpirationWarningDialog)")
        .expect("WeeklyRewardExpirationWarningDialog probe should succeed");
    assert_eq!(
        kind, "table",
        "WeeklyRewardExpirationWarningDialog must publish at `_G` as a table — declared at \
         Blizzard_WeeklyRewards.xml:742 with `mixin=\"GreatVaultRetirementWarningFrameMixin\"` \
         `frameLevel=\"500\"` `frameStrata=\"HIGH\"` `hidden=\"true\"`. The dialog is shown by \
         WeeklyRewardsMixin:OnShow when C_WeeklyRewards.ShouldShowRetirementMessage or \
         ShouldShowFinalRetirementMessage returns true (end-of-expansion vault sunset warning)"
    );

    let strata: String = env
        .eval("return WeeklyRewardExpirationWarningDialog:GetFrameStrata()")
        .expect("WeeklyRewardExpirationWarningDialog:GetFrameStrata probe should succeed");
    assert_eq!(
        strata, "HIGH",
        "WeeklyRewardExpirationWarningDialog must render in HIGH strata so it floats above the \
         WeeklyRewardsFrame (which uses the default panel strata)"
    );
}
}

prefork_full_ui_case! {
fn xml_templates_are_registered(env: &WowLuaEnv) {
    load_weekly_rewards(env);
    let _ = env;

    for template_name in VIRTUAL_TEMPLATES {
        assert!(
            wow_ui_sim::xml::get_template(template_name).is_some(),
            "{template_name} (`<Frame virtual=\"true\">` from Blizzard_WeeklyRewards.xml) must \
             be registered in the template registry after the addon loads"
        );
    }
}
}

prefork_full_ui_case! {
fn named_non_virtual_frames_publish_at_globals(env: &WowLuaEnv) {
    load_weekly_rewards(env);

    for frame_name in NAMED_NON_VIRTUAL_FRAMES {
        let kind: String = env
            .eval(&format!("return type({frame_name})"))
            .unwrap_or_else(|err| panic!("{frame_name} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{frame_name} must publish at `_G` as a table — non-virtual XML elements with `name=` \
             auto-publish to globals via the loader's name-resolution path"
        );
    }
}
}

prefork_full_ui_case! {
fn confirm_select_static_popup_dialog_registers_at_file_scope(env: &WowLuaEnv) {
    load_weekly_rewards(env);

    let registered: bool = env
        .eval(
            "return type(StaticPopupDialogs) == 'table' \
                and type(StaticPopupDialogs['CONFIRM_SELECT_WEEKLY_REWARD']) == 'table' \
                and type(StaticPopupDialogs['CONFIRM_SELECT_WEEKLY_REWARD'].OnAccept) == 'function' \
                and StaticPopupDialogs['CONFIRM_SELECT_WEEKLY_REWARD'].button1 == YES \
                and StaticPopupDialogs['CONFIRM_SELECT_WEEKLY_REWARD'].button2 == CANCEL \
                and StaticPopupDialogs['CONFIRM_SELECT_WEEKLY_REWARD'].timeout == 0 \
                and StaticPopupDialogs['CONFIRM_SELECT_WEEKLY_REWARD'].hideOnEscape == 1 \
                and StaticPopupDialogs['CONFIRM_SELECT_WEEKLY_REWARD'].showAlert == 1 \
                and StaticPopupDialogs['CONFIRM_SELECT_WEEKLY_REWARD'].acceptDelay == 5",
        )
        .expect("StaticPopupDialogs.CONFIRM_SELECT_WEEKLY_REWARD probe should succeed");
    assert!(
        registered,
        "StaticPopupDialogs[\"CONFIRM_SELECT_WEEKLY_REWARD\"] must register at file scope \
         (lines 8-21) with the YES/CANCEL buttons + 5-second acceptDelay (anti-misclick floor) + \
         OnAccept that plays UI_WEEKLY_REWARD_CONFIRMED_REWARD, calls \
         C_WeeklyRewards.ClaimReward(data), and HideUIPanels the WeeklyRewardsFrame. The \
         registration runs the moment the lua file is loaded via the XML's `<Script file>` \
         directive, NOT lazily on first use"
    );
}
}
