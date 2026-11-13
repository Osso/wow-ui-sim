#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn help_plate_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HelpPlate")
}

fn help_plate_toc() -> PathBuf {
    help_plate_dir().join("Blizzard_HelpPlate.toc")
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

#[test]
fn blizzard_help_plate_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&help_plate_dir()).expect("Blizzard_HelpPlate TOC should resolve");
    assert_eq!(
        resolved,
        help_plate_toc(),
        "Blizzard_HelpPlate ships exactly one bare TOC (`Blizzard_HelpPlate.toc`) — no flavor \
         variants. The cross-screen tutorial / help-overlay surface is shared across mainline, \
         classic, and glue screens, so a single TOC suffices and `find_toc_file` \
         (src/loader/mod.rs:65) falls through to the bare `.toc` suffix"
    );
}

#[test]
fn blizzard_help_plate_toc_declares_non_lod_with_no_dependencies() {
    let toc = TocFile::from_file(&help_plate_toc()).expect("HelpPlate TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_HelpPlate omits `## LoadOnDemand` — the cross-screen tutorial overlay must be \
         available throughout every screen session and auto-loads on every discovery pass that \
         qualifies via `## AllowLoad: Both`"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_HelpPlate does not declare `## LoadFirst: 1` — it has no dependencies and \
         imposes no ordering constraint"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_HelpPlate does not declare `## UseSecureEnvironment` — runs in the standard \
         Lua environment (HelpPlate.Show is called from insecure tutorial-driver code paths and \
         must not require a secure environment)"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_HelpPlate declares NO `## Dependencies` — the addon defines its own \
         HelpPlateCanvas + HelpPlateTooltip frames, references only intrinsics (CreateFramePool, \
         GetAppropriateTopLevelParent, Kiosk.IsEnabled, PlaySound, SetClampedTextureRotation, \
         SOUNDKIT.IG_MAINMENU_OPTION_CHECKBOX_ON, MAIN_HELP_BUTTON_TOOLTIP locale string) and \
         the global StaticPopup_Show is not consumed — no addon is required to load first"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_HelpPlate declares NO `## SavedVariables*` — tutorial-tracking flags are owned \
         by the calling subsystem (e.g. SpellBook, CollectionsJournal), HelpPlate is purely a \
         transient overlay that holds no persistent state"
    );
}

#[test]
fn blizzard_help_plate_toc_declares_allow_load_both_no_game_type_restriction() {
    let toc = TocFile::from_file(&help_plate_toc()).expect("HelpPlate TOC should parse");
    let toc_text = std::fs::read_to_string(help_plate_toc()).expect("HelpPlate TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: Both"),
        "Blizzard_HelpPlate declares `## AllowLoad: Both` (capital B — `allows_screen()` \
         (src/toc.rs:305) lowercases before matching). The `Both` value makes the addon qualify \
         for ALL 4 screen kinds (Game + Login + CharacterSelect + CharacterCreate) — this is \
         the differentiating flag that separates HelpPlate from the dozens of `## AllowLoad: \
         Game` and `## AllowLoad: Glue` addons in the BlizzardUI tree"
    );
    assert!(
        !toc_text.contains("## AllowLoadGameType:"),
        "Blizzard_HelpPlate omits `## AllowLoadGameType` — the help-overlay primitive is shared \
         across mainline (retail) and classic flavors and is not game-type-restricted"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_HelpPlate must NOT be game-type restricted — absent AllowLoadGameType means \
         all flavors qualify"
    );
    assert!(
        !toc_text.contains("## DefaultState:"),
        "Blizzard_HelpPlate omits `## DefaultState` — relies on the loader's implicit-enabled \
         default for Blizzard prefix addons rather than declaring an explicit `enabled` value"
    );
}

#[test]
fn blizzard_help_plate_toc_lists_lua_then_xml() {
    let toc = TocFile::from_file(&help_plate_toc()).expect("HelpPlate TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_HelpPlate.lua".to_string(),
            "Blizzard_HelpPlate.xml".to_string(),
        ],
        "Blizzard_HelpPlate TOC body lists exactly 2 files in Lua-then-XML order: \
         Blizzard_HelpPlate.lua first (publishes `HelpPlate = {{}}` global table + 5 mixins + \
         module-local CreateFramePool with HelpPlateTile virtual template + ResetHelpPlateTile \
         pool reset callback + FinalizeHide local that depends on HelpPlateCanvas / \
         HelpPlateTooltip globals), then Blizzard_HelpPlate.xml (publishes the named non-virtual \
         frames the Lua references)"
    );
}

#[test]
fn blizzard_help_plate_directory_ships_three_entries() {
    let dir = help_plate_dir();
    let mut entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_HelpPlate directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "Blizzard_HelpPlate.lua".to_string(),
            "Blizzard_HelpPlate.toc".to_string(),
            "Blizzard_HelpPlate.xml".to_string(),
        ],
        "Blizzard_HelpPlate directory ships exactly 3 entries (TOC + Lua + XML), no flavor \
         subdirectory and no Localization.lua — the only locale string consumed \
         (MAIN_HELP_BUTTON_TOOLTIP) is published by the global locale table maintained by \
         Blizzard_Localizable / Blizzard_FrameXML"
    );
}

#[test]
fn blizzard_help_plate_appears_on_all_four_screens() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let count = addons
            .iter()
            .filter(|(name, _)| name == "Blizzard_HelpPlate")
            .count();
        assert_eq!(
            count, 1,
            "Blizzard_HelpPlate must auto-discover EXACTLY ONCE on the {screen:?} screen — \
             non-LoD + `## AllowLoad: Both` qualify it for ALL 4 screen kinds, so the discovery \
             pass for Game / Login / CharacterSelect / CharacterCreate must each surface it once"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_help_plate_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_HelpPlate/")
                || e.contains("Blizzard_HelpPlate\\")
                || e.contains("Blizzard_HelpPlate.lua")
                || e.contains("Blizzard_HelpPlate.xml")
                || e.contains("HelpPlateTooltipMixin")
                || e.contains("HelpPlateCanvas")
                || e.contains("HelpPlateTile")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_HelpPlate emitted addon-specific Lua errors during Game-screen auto-load:\n  \
         {}",
        related
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_help_plate_is_addon_loaded_returns_true_after_game_screen_load(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HelpPlate')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "After Game-screen auto-discovery, \
         `C_AddOns.IsAddOnLoaded('Blizzard_HelpPlate')` should return true — `## AllowLoad: \
         Both` qualifies HelpPlate for the Game-screen auto-load pass alongside Login / \
         CharacterSelect / CharacterCreate"
    );
}
}

prefork_full_ui_case! {
fn blizzard_help_plate_publishes_help_plate_global_table_with_seven_functions(env: &WowLuaEnv) {

    let table_exists: bool = env
        .eval("return type(HelpPlate) == 'table'")
        .expect("HelpPlate global lookup should succeed");
    assert!(
        table_exists,
        "Blizzard_HelpPlate.lua line 1 declares `HelpPlate = {{}}` — after load, the global \
         must be a plain table (NOT a frame), distinct from the named non-virtual frame \
         `HelpPlateCanvas` that the table's methods drive"
    );

    for func in [
        "Show",
        "Hide",
        "GetEffectiveScale",
        "HideTooltip",
        "IsShowingHelpInfo",
        "IsShowingTutorialTooltip",
        "ShowTutorialTooltip",
    ] {
        let exists: bool = env
            .eval(&format!("return type(HelpPlate['{func}']) == 'function'"))
            .expect("HelpPlate function existence query should succeed");
        assert!(
            exists,
            "HelpPlate must expose `.{func}()` — Blizzard_HelpPlate.lua publishes 7 dot-call \
             functions (NOT colon-method mixins) that drive the overlay lifecycle: Show \
             (acquires tiles from tilePool, parents HelpPlateCanvas to \
             GetAppropriateTopLevelParent, anchors tiles + buttons by HighLightBox / ButtonPos), \
             Hide (animates out via per-tile AnimateOut callbacks or releases all on \
             non-user-input hide), GetEffectiveScale (re-parents canvas before scale query to \
             fix first-show sizing), HideTooltip / IsShowingHelpInfo / IsShowingTutorialTooltip \
             (overlay-state predicates), ShowTutorialTooltip (Kiosk.IsEnabled() guard, plays \
             LingerAndFade animation)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_plate_publishes_five_mixins(env: &WowLuaEnv) {

    for mixin in [
        "MainHelpPlateButtonMixin",
        "HelpPlateButtonMixin",
        "HelpPlateBoxMixin",
        "HelpPlateTileMixin",
        "HelpPlateTooltipMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("HelpPlate mixin existence query should succeed");
        assert!(
            exists,
            "Blizzard_HelpPlate.lua must publish `_G['{mixin}']` — 5 mixin tables drive distinct \
             frame behaviors: MainHelpPlateButtonMixin (the toggle button mounted on calling \
             frames, OnEnter/OnLeave/OnMouseDown/OnMouseUp/OnHide/ShowTooltip), \
             HelpPlateButtonMixin (per-tile help-i button with slideAnimGroup translation+alpha \
             animations + tutorial pulse, OnLoad/OnShow/OnHide/OnEnter/HideTutorial/\
             ConfigureForTutorial/AnimateOut/Reset), HelpPlateBoxMixin (gold-tinted \
             ThinBorder2 box, OnLoad sets vertex color 1.0/0.82/0 on Textures parentArray), \
             HelpPlateTileMixin (tile container with Box highlight swap, \
             OnEnter/OnLeave/Reset), HelpPlateTooltipMixin (the 4-direction arrow tooltip, \
             OnLoad/OnHide/HideTextures/Init/InitFromMainHelpPlateButton)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_plate_main_help_plate_button_mixin_publishes_six_methods(env: &WowLuaEnv) {

    for method in [
        "OnEnter",
        "OnLeave",
        "OnMouseDown",
        "OnMouseUp",
        "OnHide",
        "ShowTooltip",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(MainHelpPlateButtonMixin['{method}']) == 'function'"
            ))
            .expect("MainHelpPlateButtonMixin method existence query should succeed");
        assert!(
            exists,
            "MainHelpPlateButtonMixin must expose `:{method}()` — the floating help-toggle \
             button's behavior is fully captured by these 6 methods: OnEnter calls \
             ShowTooltip (which stops LingerAndFade and re-inits the tooltip), OnLeave hides \
             HelpPlateTooltip, OnMouseDown/OnMouseUp shift the I texture by (1,-1) for press \
             feedback and play SOUNDKIT.IG_MAINMENU_OPTION_CHECKBOX_ON on release, OnHide \
             cleans up the tooltip, ShowTooltip is the public entry point that \
             InitFromMainHelpPlateButton routes through"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_plate_button_mixin_publishes_eight_methods(env: &WowLuaEnv) {

    for method in [
        "OnLoad",
        "OnShow",
        "OnHide",
        "OnEnter",
        "HideTutorial",
        "ConfigureForTutorial",
        "AnimateOut",
        "Reset",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HelpPlateButtonMixin['{method}']) == 'function'"
            ))
            .expect("HelpPlateButtonMixin method existence query should succeed");
        assert!(
            exists,
            "HelpPlateButtonMixin must expose `:{method}()` — the per-tile help-i button's \
             slide-in / pulse / animate-out lifecycle: OnLoad creates slideAnimGroup with a \
             Translation animation + Alpha 1→0 animation (SetSmoothing IN), OnShow reads the \
             button's current SetPoint offset and sets the translation to (-x,-y) over 0.5s \
             then plays in reverse (slide-in), OnHide stops the Pulse animation if forTutorial, \
             OnEnter delegates to HideTutorial, ConfigureForTutorial shows HelpIGlow + BgGlow \
             and plays Pulse, AnimateOut plays the same slideAnimGroup forward over 0.3s and \
             invokes onFinishedCallback, Reset clears the OnFinished script and stops the group"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_plate_tooltip_mixin_publishes_five_methods(env: &WowLuaEnv) {

    for method in [
        "OnLoad",
        "OnHide",
        "HideTextures",
        "Init",
        "InitFromMainHelpPlateButton",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HelpPlateTooltipMixin['{method}']) == 'function'"
            ))
            .expect("HelpPlateTooltipMixin method existence query should succeed");
        assert!(
            exists,
            "HelpPlateTooltipMixin must expose `:{method}()` — the GlowBoxTemplate-inheriting \
             tooltip's lifecycle: OnLoad sets Text spacing 4 and rotates ArrowLeft 270° / \
             ArrowRight 90° / ArrowGlowLeft 270° / ArrowGlowRight 90° via \
             SetClampedTextureRotation (the ArrowDown texture is reused for all 4 directions \
             via rotation), OnHide clears tutorialHelpInfo, HideTextures hides all 8 \
             arrow+arrowglow textures, Init re-parents to GetAppropriateTopLevelParent / sets \
             FULLSCREEN_DIALOG strata + frameLevel 2 / shows + anchors the appropriate \
             arrow+arrowglow pair based on tooltipDir (UP/DOWN/LEFT/RIGHT), \
             InitFromMainHelpPlateButton calls Init with `RIGHT` direction"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_plate_tile_and_box_mixin_methods_publish(env: &WowLuaEnv) {

    for method in ["OnEnter", "OnLeave", "Reset"] {
        let exists: bool = env
            .eval(&format!(
                "return type(HelpPlateTileMixin['{method}']) == 'function'"
            ))
            .expect("HelpPlateTileMixin method existence query should succeed");
        assert!(
            exists,
            "HelpPlateTileMixin must expose `:{method}()` — the 3-method tile container mixin: \
             OnEnter swaps the Box.BG (visible) for the BoxHighlight (hidden) and calls \
             Button:HideTutorial, OnLeave reverts both, Reset calls Button:Reset + \
             ClearAllPoints + Hide and is the pool-release callback signature target"
        );
    }

    let on_load: bool = env
        .eval("return type(HelpPlateBoxMixin['OnLoad']) == 'function'")
        .expect("HelpPlateBoxMixin:OnLoad query should succeed");
    assert!(
        on_load,
        "HelpPlateBoxMixin must expose `:OnLoad()` — the single-method mixin tints all entries \
         in the Textures parentArray (TopLeft/TopRight/BottomLeft/BottomRight/Top/Bottom/Left/\
         Right ThinBorder2 sprites) with vertex color (1.0, 0.82, 0.0) which is the Blizzard \
         gold quest highlight"
    );
}
}

prefork_full_ui_case! {
fn blizzard_help_plate_publishes_named_non_virtual_frames_as_globals(env: &WowLuaEnv) {

    for frame_name in ["HelpPlateTooltip", "HelpPlateCanvas"] {
        let exists: bool = env
            .eval(&format!(
                "local f = _G['{frame_name}']; return type(f) == 'table' and type(f.GetName) == 'function'"
            ))
            .expect("global frame lookup should succeed");
        assert!(
            exists,
            "After Game-screen auto-load, `{frame_name}` should publish as a global frame \
             instance — Blizzard_HelpPlate.xml declares EXACTLY 2 named non-virtual frames at \
             file scope: HelpPlateTooltip (Frame inheriting GlowBoxTemplate, mixin \
             HelpPlateTooltipMixin, hidden — the 4-direction arrow tooltip referenced by \
             HelpPlate.HideTooltip and HelpPlateButtonMixin:OnLeave) and HelpPlateCanvas \
             (Button toplevel, enableMouse + enableKeyboard, frameStrata=DIALOG, hidden — the \
             root overlay parented to GetAppropriateTopLevelParent and resized to FrameSize on \
             every HelpPlate.Show call). The 5 virtual templates (GlowBoxArrowTemplate, \
             GlowBoxTemplate, MainHelpPlateButton, HelpPlateTile, RinglessHelpPlateButtonTemplate) \
             register in the XML template registry but do NOT publish as `_G` globals"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_help_plate_virtual_templates_do_not_publish_as_globals(env: &WowLuaEnv) {

    for template_name in [
        "GlowBoxArrowTemplate",
        "GlowBoxTemplate",
        "MainHelpPlateButton",
        "HelpPlateTile",
        "RinglessHelpPlateButtonTemplate",
    ] {
        let nil_at_global: bool = env
            .eval(&format!("return _G['{template_name}'] == nil"))
            .expect("virtual template global lookup should succeed");
        assert!(
            nil_at_global,
            "`{template_name}` must NOT publish as a `_G` global — virtual templates \
             (`virtual=\"true\"`) register in the XML template registry for inheritance but are \
             never instantiated at file scope. Confirms the loader honors `virtual=\"true\"` \
             and only HelpPlateTooltip / HelpPlateCanvas (the 2 named non-virtual frames) \
             materialize as globals"
        );
    }
}
}

#[cfg(feature = "optional-tests")]
#[test]
fn help_plate_tooltip_rotates_real_arrow_texture_geometry() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let consumer_source = std::fs::read_to_string(help_plate_dir().join("Blizzard_HelpPlate.lua"))
        .expect("Retail Blizzard_HelpPlate.lua should be present");
    env.exec(&consumer_source)
        .expect("original Blizzard HelpPlate consumer should load");

    let failures: String = env
        .eval(
            r#"
            local textureParent = CreateFrame("Frame")
            local tooltip = {
                Text = textureParent:CreateFontString(),
                LingerAndFade = textureParent:CreateAnimationGroup(),
            }

            local function createArrow()
                local arrow = textureParent:CreateTexture()
                arrow:SetSize(20, 10)
                arrow:SetTexCoord(0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8)
                return arrow
            end

            tooltip.ArrowLeft = createArrow()
            tooltip.ArrowRight = createArrow()
            tooltip.ArrowGlowLeft = createArrow()
            tooltip.ArrowGlowRight = createArrow()
            HelpPlateTooltipMixin:OnLoad(tooltip)

            local failures = {}
            local function expectRotation(name, texture, expected)
                if texture:GetWidth() ~= 10 or texture:GetHeight() ~= 20 then
                    table.insert(failures, name .. " dimensions were not clamped to 10x20")
                end
                local actual = { texture:GetTexCoord() }
                for index, value in ipairs(expected) do
                    if math.abs(actual[index] - value) > 0.0001 then
                        table.insert(failures, name .. " texcoords did not match the arrow orientation")
                        break
                    end
                end
            end

            expectRotation("left", tooltip.ArrowLeft, { 0.5, 0.6, 0.1, 0.2, 0.7, 0.8, 0.3, 0.4 })
            expectRotation("right", tooltip.ArrowRight, { 0.3, 0.4, 0.7, 0.8, 0.1, 0.2, 0.5, 0.6 })
            expectRotation("glow left", tooltip.ArrowGlowLeft, { 0.5, 0.6, 0.1, 0.2, 0.7, 0.8, 0.3, 0.4 })
            expectRotation("glow right", tooltip.ArrowGlowRight, { 0.3, 0.4, 0.7, 0.8, 0.1, 0.2, 0.5, 0.6 })
            return table.concat(failures, "\n")
            "#,
        )
        .expect("original HelpPlate tooltip OnLoad geometry probe should run");

    assert!(
        failures.is_empty(),
        "HelpPlate arrow geometry mismatches:\n{failures}"
    );
}
