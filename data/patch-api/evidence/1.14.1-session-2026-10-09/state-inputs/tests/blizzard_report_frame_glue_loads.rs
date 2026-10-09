use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn report_frame_glue_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ReportFrameGlue")
}

fn report_frame_glue_toc() -> PathBuf {
    report_frame_glue_dir().join("Blizzard_ReportFrameGlue.toc")
}

const TOC_FILE_LIST: &[&str] = &["ReportFrame.xml"];

const HARD_DEPENDENCIES: &[&str] = &["Blizzard_ReportFrameShared"];

const OVERRIDE_METHODS: &[&str] = &[
    "CanDisplayMinorCategory",
    "ShouldDisplayTooltip",
    "ManageButton",
];

const INHERITED_BASE_METHODS: &[&str] = &[
    "OnLoad",
    "OnHide",
    "OnEvent",
    "Reset",
    "InitiateReport",
    "InitiateReportInternal",
    "ReportByType",
    "MajorTypeSelected",
    "SetMajorType",
    "SendReport",
    "SetupDropdownByReportType",
    "UpdateThankYouMessage",
];

fn load_character_select_screen() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::CharacterSelect);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::CharacterSelect);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    env
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&report_frame_glue_dir())
        .expect("Blizzard_ReportFrameGlue TOC should resolve");
    assert_eq!(
        resolved,
        report_frame_glue_toc(),
        "Blizzard_ReportFrameGlue ships exactly one TOC at the bare \
         `Blizzard_ReportFrameGlue.toc` path — no `_Mainline` flavor split. find_toc_file at \
         src/loader/mod.rs:65 tries `_Mainline.toc` first (miss), then bare (hit) and returns \
         it. Distinct from sibling glue addons like Blizzard_DeclensionFrameGlue which DO split \
         into `_Mainline.toc` because their classic flavors ship locale-specific declension \
         logic; the report dialog's glue surface is uniform across flavors so a single TOC \
         suffices"
    );
}

#[test]
fn toc_declares_eager_glue_only_with_one_hard_dep() {
    let toc = TocFile::from_file(&report_frame_glue_toc())
        .expect("Blizzard_ReportFrameGlue TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_ReportFrameGlue must NOT declare `## LoadOnDemand: 1` — the glue-screen \
         player-report dialog has to be live before the player ever right-clicks a name in \
         CharacterSelect / friends-tag / battle.net friends list. The Glue counterpart is \
         eager-loaded for the same reason as the Game counterpart"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_ReportFrameGlue declares no `## AllowLoadGameType` directive — \
         is_game_type_restricted at src/toc.rs:294-302 returns FALSE when the metadata is \
         absent. The glue-screen report dialog applies to every flavor without a server-side \
         gate"
    );
    assert_eq!(
        toc.metadata.get("DefaultState").map(String::as_str),
        Some("enabled"),
        "DefaultState=enabled preserved verbatim — without it the glue-screen eager-loader \
         would skip the addon"
    );
    let deps = toc.dependencies();
    assert_eq!(
        deps.iter().map(String::as_str).collect::<Vec<_>>(),
        HARD_DEPENDENCIES,
        "Blizzard_ReportFrameGlue declares exactly one hard dep `## Dependencies: \
         Blizzard_ReportFrameShared` — the shared addon supplies SharedReportFrameTemplate (the \
         virtual <Frame> that GlueParent's ReportFrame inherits) and SharedReportFrameMixin \
         (the base table that ReportFrameMixin = CreateFromMixins(SharedReportFrameMixin) \
         extends). Identical hard-dep shape to Blizzard_ReportFrame because they both build on \
         the same shared report dialog template"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_ReportFrameGlue declares zero `## OptionalDeps:` — every collaborator (the \
         shared template, the report-system C API) is either a hard dep or part of the \
         always-loaded glue core surface"
    );
    assert!(
        toc.saved_variables().is_empty() && toc.saved_variables_per_character().is_empty(),
        "Blizzard_ReportFrameGlue declares zero saved variables — the glue-screen state cannot \
         persist across logout because SavedVariables/SavedVariablesPerCharacter only flush \
         after a character is selected and the realm enters the Game screen"
    );
}

#[test]
fn toc_lists_only_xml_with_lua_loaded_via_script_include() {
    let toc = TocFile::from_file(&report_frame_glue_toc())
        .expect("Blizzard_ReportFrameGlue TOC should parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        listed, TOC_FILE_LIST,
        "TOC body must list ONLY ReportFrame.xml — distinct from sibling Blizzard_ReportFrame \
         which lists both lua and xml. The glue addon uses the legacy embedded-script-load \
         pattern: `<Script file=\"ReportFrame.lua\"/>` inside ReportFrame.xml pulls the Lua \
         chunk via process_include in src/loader/xml_file.rs:207-213, treating .lua files as \
         load_lua_file calls during XML parsing. The order matters because the <Script> \
         element must precede the <Frame name=\"ReportFrame\"> declaration so \
         ReportFrameMixin = CreateFromMixins(SharedReportFrameMixin) publishes before the \
         frame's `mixin=\"ReportFrameMixin\"` attribute resolves it"
    );
}

#[test]
fn xml_embeds_script_file_directive_for_lua() {
    let xml_text = std::fs::read_to_string(report_frame_glue_dir().join("ReportFrame.xml"))
        .expect("ReportFrame.xml should read");
    assert!(
        xml_text.contains("<Script file=\"ReportFrame.lua\"/>"),
        "ReportFrameGlue's XML must include `<Script file=\"ReportFrame.lua\"/>` — this is the \
         actual mechanism that loads ReportFrame.lua. Without it the TOC's xml-only file list \
         would never trigger Lua execution and ReportFrameMixin would stay nil, breaking the \
         CreateFromMixins chain at frame instantiation"
    );
}

#[test]
fn allows_screen_returns_true_only_for_glue_screens() {
    let toc = TocFile::from_file(&report_frame_glue_toc())
        .expect("Blizzard_ReportFrameGlue TOC should parse");
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "Blizzard_ReportFrameGlue declares `## AllowLoad: Glue` — must allow ALL three glue \
             screens. ScreenKind::is_glue() at src/screen.rs:28-33 enumerates exactly \
             Login/CharacterSelect/CharacterCreate, and src/toc.rs:309 maps `glue` → \
             screen.is_glue(). (Screen tested: {screen:?})"
        );
    }
    assert!(
        !toc.allows_screen(ScreenKind::Game),
        "Blizzard_ReportFrameGlue must NOT allow the Game screen — that responsibility belongs \
         to the sibling Blizzard_ReportFrame addon (with `## AllowLoad: Game`). Allowing both \
         to load on the Game screen would cause a double-instantiation of `ReportFrame` at \
         _G with conflicting parents (UIParent vs GlueParent)"
    );
}

#[test]
fn included_in_eager_glue_screen_discovery() {
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_ReportFrameGlue");
        assert!(
            found,
            "Blizzard_ReportFrameGlue must appear in eager auto-discovery on {screen:?} — no \
             LoadOnDemand gate, no AllowLoadGameType restriction, AllowLoad=Glue passes the \
             is_glue() branch at src/toc.rs:309. Glue-screen friend / battle.net contexts \
             reference `ReportFrame:InitiateReport(...)` directly with no LoadAddOn guard"
        );
    }
}

#[test]
fn excluded_from_game_screen_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_ReportFrameGlue");
    assert!(
        !found,
        "Blizzard_ReportFrameGlue must be filtered out of Game-screen auto-discovery — \
         `## AllowLoad: Glue` rejects ScreenKind::Game at src/toc.rs:309 because is_glue() \
         returns false for Game. The Game screen gets Blizzard_ReportFrame instead, ensuring \
         only one of the two ever owns the `ReportFrame` global per session"
    );
}

#[test]
fn root_directory_holds_xml_and_lua_next_to_toc() {
    let mut entries: Vec<String> = std::fs::read_dir(report_frame_glue_dir())
        .expect("Blizzard_ReportFrameGlue directory should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name != "Blizzard_ReportFrameGlue.toc")
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec!["ReportFrame.lua".to_string(), "ReportFrame.xml".to_string()],
        "Blizzard_ReportFrameGlue/ root must hold both the lua and xml file even though only \
         the xml is listed in the TOC body — the lua is pulled in by ReportFrame.xml's \
         <Script file=\"ReportFrame.lua\"/> directive at parse time"
    );
}

#[test]
fn loads_without_lua_errors_on_character_select() {
    let env = load_character_select_screen();

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("ReportFrame") || message.contains("ReportFrameMixin"))
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ReportFrameGlue emitted Lua errors during eager CharacterSelect-screen load. \
         The addon is structurally trivial: 1 XML chunk that pulls in 1 Lua chunk via \
         <Script file=...>, then declares one named frame inheriting SharedReportFrameTemplate. \
         Any error is a real load failure:\n  {}",
        load_errors.join("\n  ")
    );
}

#[test]
fn is_addon_loaded_after_eager_glue_sweep() {
    let env = load_character_select_screen();

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ReportFrameGlue')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_ReportFrameGlue') must return true after the eager \
         CharacterSelect sweep — confirms the loader registers the non-LOD glue-only addon \
         with the loaded-set without an explicit LoadAddOn call"
    );

    let dep_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ReportFrameShared')")
        .expect("IsAddOnLoaded(Blizzard_ReportFrameShared) probe should succeed");
    assert!(
        dep_loaded,
        "Blizzard_ReportFrameShared must also be loaded — declared as the sole hard \
         `## Dependencies:` entry, eager-loaded itself via `## AllowLoad: Both` so it pre-loads \
         on glue screens too"
    );
}

#[test]
fn publishes_named_top_level_frame_under_glue_parent() {
    let env = load_character_select_screen();

    let frame_kind: String = env
        .eval("return type(ReportFrame)")
        .expect("ReportFrame probe should succeed");
    assert_eq!(
        frame_kind, "table",
        "ReportFrame must publish at `_G` as a table — declared at \
         Blizzard_ReportFrameGlue/ReportFrame.xml:4 with `mixin=\"ReportFrameMixin\"` \
         `inherits=\"SharedReportFrameTemplate\"` `parent=\"GlueParent\"` `toplevel=\"true\"` \
         `enableMouse=\"true\"` `hidden=\"true\"` `frameStrata=\"DIALOG\"`. Same global name as \
         the Game variant — never collides because Game and Glue are mutually-exclusive screen \
         contexts"
    );

    let parented_to_glue_parent: bool = env
        .eval("return ReportFrame:GetParent() == GlueParent")
        .expect("ReportFrame parent probe should succeed");
    assert!(
        parented_to_glue_parent,
        "ReportFrame must be parented to GlueParent (NOT UIParent) — `parent=\"GlueParent\"` \
         XML attribute at line 4. This is the structural difference from the Game variant: \
         GlueParent is the root frame for glue-screen UI created by Blizzard_GlueXML, while \
         UIParent is the in-game equivalent created by Blizzard_UIParent. The two never \
         coexist within a single screen"
    );

    let strata: String = env
        .eval("return ReportFrame:GetFrameStrata()")
        .expect("ReportFrame strata probe should succeed");
    assert_eq!(
        strata, "DIALOG",
        "ReportFrame's strata must be DIALOG so the glue-screen report dialog stacks above \
         CharacterSelect/CharacterCreate UI but below TOOLTIP. Same DIALOG strata as the Game \
         variant — preserves consistent visual hierarchy between the two screen contexts"
    );
}

#[test]
fn report_frame_mixin_extends_shared_via_create_from_mixins() {
    let env = load_character_select_screen();

    let mixin_kind: String = env
        .eval("return type(ReportFrameMixin)")
        .expect("ReportFrameMixin probe should succeed");
    assert_eq!(
        mixin_kind, "table",
        "ReportFrameMixin must publish at `_G` as a table — declared at \
         Blizzard_ReportFrameGlue/ReportFrame.lua:1 via \
         `ReportFrameMixin = CreateFromMixins(SharedReportFrameMixin)`. Same shallow-copy \
         composition as the Game variant; the only difference is which addon's overrides win \
         the screen-gated load race (Game vs Glue)"
    );

    let shared_kind: String = env
        .eval("return type(SharedReportFrameMixin)")
        .expect("SharedReportFrameMixin probe should succeed");
    assert_eq!(
        shared_kind, "table",
        "SharedReportFrameMixin must exist as a table provided by Blizzard_ReportFrameShared — \
         CreateFromMixins's parent argument. Without it ReportFrame.lua line 1 would silently \
         produce an empty table and every method dispatch on the live frame would nil-error"
    );
}

#[test]
fn glue_overrides_publish_with_glue_specific_semantics() {
    let env = load_character_select_screen();

    for method in OVERRIDE_METHODS {
        let kind: String = env
            .eval(&format!("return type(ReportFrameMixin.{method})"))
            .unwrap_or_else(|err| panic!("type(ReportFrameMixin.{method}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ReportFrameMixin.{method} must be a function — declared with `--override` comment \
             marker in Blizzard_ReportFrameGlue/ReportFrame.lua. The glue-screen overrides have \
             different semantics than the Game variant: CanDisplayMinorCategory hides BTag (when \
             not isBnetReport), GuildName (always — no guilds in Glue), and CharacterName \
             (always — only battletags in Glue, no character GUIDs); ShouldDisplayTooltip \
             returns FALSE so glue-screen minor-category buttons get NO tooltip text (the Game \
             variant returns true); ManageButton uses Show/Hide instead of SetEnabled — at the \
             glue layer the report-button widget cannot rely on CSS-style disabled visuals \
             because the glue button atlas does not ship a `*-Disabled` state texture"
        );
    }

    let tooltip_disabled: bool = env
        .eval("return ReportFrameMixin:ShouldDisplayTooltip() == false")
        .expect("ShouldDisplayTooltip probe should succeed");
    assert!(
        tooltip_disabled,
        "Blizzard_ReportFrameGlue's ReportFrameMixin:ShouldDisplayTooltip() must return false — \
         this is the glue-specific override. The Game variant returns true to enable the \
         minor-category tooltips because in-game GameTooltip is always available; glue screens \
         lack the in-game tooltip stack so the override suppresses tooltip display entirely"
    );

    let guild_hidden: bool = env
        .eval(
            "return ReportFrameMixin:CanDisplayMinorCategory(Enum.ReportMinorCategory.GuildName) \
                == false",
        )
        .expect("GuildName CanDisplayMinorCategory probe should succeed");
    assert!(
        guild_hidden,
        "Blizzard_ReportFrameGlue's CanDisplayMinorCategory(GuildName) must return false — the \
         Lua chunk's hard `return false; --no guilds in Glue` branch at line 7-8 of \
         ReportFrameGlue/ReportFrame.lua. Glue screens have no concept of guilds (no \
         GetGuildInfo / IsInGuild API) so the minor-category button is unconditionally hidden, \
         distinct from the Game variant which dispatches through C_ClubFinder.GetClubTypeFromFinderGUID"
    );
}

#[test]
fn inherited_base_methods_carry_through_via_create_from_mixins() {
    let env = load_character_select_screen();

    for method in INHERITED_BASE_METHODS {
        let kind: String = env
            .eval(&format!("return type(ReportFrameMixin.{method})"))
            .unwrap_or_else(|err| panic!("type(ReportFrameMixin.{method}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ReportFrameMixin.{method} must be a function inherited from SharedReportFrameMixin \
             via CreateFromMixins — the shallow-copy semantics mean every key on the parent \
             table at line 1's evaluation time is copied into ReportFrameMixin. \
             OnLoad/OnHide/OnEvent are the lifecycle handlers wired through the inherited \
             SharedReportFrameTemplate's <Scripts method=...> entries; InitiateReport is the \
             public entry point called by glue-screen battle.net friends UI when the player \
             elects to report a btag contact"
        );
    }
}

#[test]
fn registers_for_report_lifecycle_events() {
    let env = load_character_select_screen();

    let event_count: f64 = env
        .eval(
            "local n = 0 \
             if ReportFrame:IsEventRegistered('REPORT_PLAYER_RESULT') then n = n + 1 end \
             if ReportFrame:IsEventRegistered('REPORT_SCREENSHOT_READY') then n = n + 1 end \
             return n",
        )
        .expect("Event-registration probe should succeed");
    assert_eq!(
        event_count, 2.0,
        "ReportFrame (glue variant) must register exactly two events at OnLoad time: \
         REPORT_PLAYER_RESULT (the async server ack that flips the panel into the thank-you \
         state) and REPORT_SCREENSHOT_READY (fires when the screenshot subsystem hands back a \
         usable preview). Registration happens inside the inherited SharedReportFrameMixin:OnLoad \
         via self:RegisterEvent at ReportFrameShared.lua:9-10 — confirmation that \
         CreateFromMixins-inherited OnLoad ran on the live derived glue frame, identical to \
         the Game variant's lifecycle"
    );
}

#[test]
fn xml_uses_pure_inheritance_with_script_include_only() {
    let xml_text = std::fs::read_to_string(report_frame_glue_dir().join("ReportFrame.xml"))
        .expect("ReportFrame.xml should read");
    assert!(
        xml_text.contains("inherits=\"SharedReportFrameTemplate\""),
        "ReportFrame.xml must declare `inherits=\"SharedReportFrameTemplate\"` — every visible \
         child (Border NineSlice, ReportButton, ReportingMajorCategoryDropdown, \
         ScreenshotReportingFrame, Comment EditBox, MinorReportDescription/ReportString/\
         ThankYouText/Watermark FontStrings) is supplied by the inherited template, not \
         declared inline"
    );
    assert!(
        xml_text.contains("mixin=\"ReportFrameMixin\""),
        "ReportFrame.xml must wire the mixin via `mixin=\"ReportFrameMixin\"` — required so the \
         inherited <Scripts> handlers in SharedReportFrameTemplate dispatch against the \
         glue-override-aware ReportFrameMixin table rather than falling back to the parent \
         SharedReportFrameMixin"
    );
    assert!(
        xml_text.contains("parent=\"GlueParent\""),
        "ReportFrame.xml must wire `parent=\"GlueParent\"` — this is the structural marker that \
         differentiates the glue variant from the Game variant's `parent=\"UIParent\"`. Both \
         use the same global frame name `ReportFrame` but anchor under different roots"
    );
    assert!(
        !xml_text.contains("<Layers>") && !xml_text.contains("<Frames>"),
        "ReportFrame.xml must NOT declare any <Layers>/<Frames> blocks — the entire visual \
         structure is inherited from SharedReportFrameTemplate. The glue variant matches the \
         Game variant in this respect: pure inheritance, zero inline children"
    );
}
