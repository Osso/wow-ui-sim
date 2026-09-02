use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn selector_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SelectorUI")
}

fn selector_ui_toc() -> PathBuf {
    selector_ui_dir().join("Blizzard_SelectorUI.toc")
}

fn shared_xml_selector_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SharedXML/Shared/Selector")
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&selector_ui_dir()).expect("Blizzard_SelectorUI TOC resolves");
    assert_eq!(
        resolved,
        selector_ui_toc(),
        "Blizzard_SelectorUI ships a single bare `Blizzard_SelectorUI.toc` with no \
         flavor-specific variants. The actual selector widget code (SelectorMixin, \
         GridSelectorMixin, ScrollBoxSelector) lives inside Blizzard_SharedXML's \
         body — this folder is a vestigial empty shell that exists only so other \
         code can `IsAddOnLoaded('Blizzard_SelectorUI')` against a registered name"
    );
}

#[test]
fn toc_declares_only_title_and_load_on_demand() {
    let toc = TocFile::from_file(&selector_ui_toc()).expect("Blizzard_SelectorUI TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 1` — the empty stub addon must NOT \
         appear in any eager-discovery sweep. The actual selector widget templates \
         (SelectorMixin / GridSelectorMixin / ScrollBoxSelectorMixin / \
         SelectableButtonTemplate / GridSelectableButtonTemplate) are bundled into \
         Blizzard_SharedXML's body, which loads them via the shared eager path. \
         The Blizzard_SelectorUI folder itself is a placeholder — it has no Lua, \
         no XML, no body files at all"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_glue_only());

    assert!(
        toc.dependencies().is_empty(),
        "TOC must declare zero Dependencies / RequiredDep / RequiredDeps — there \
         is no body file to depend on anything"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());

    assert!(
        toc.files.is_empty(),
        "TOC body must list zero files — the addon ships only a 2-line metadata \
         header (Title + LoadOnDemand) with no Lua or XML body. Got: {:?}",
        toc.files
    );
}

#[test]
fn toc_lacks_allow_load_so_falls_through_to_game_only() {
    let toc = TocFile::from_file(&selector_ui_toc()).expect("Blizzard_SelectorUI TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without `## AllowLoad`, src/toc.rs:311 None arm restricts the addon to \
         the Game screen — but combined with LoadOnDemand=1 the addon never enters \
         eager discovery, so this restriction is moot at runtime"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must NOT be allowed — no AllowLoad declaration \
             means screen-restriction defaults to Game-only"
        );
    }
}

#[test]
fn toc_raw_bytes_contain_only_title_and_load_on_demand_lines() {
    let raw =
        std::fs::read_to_string(selector_ui_toc()).expect("Blizzard_SelectorUI TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard Selector UI"));
    assert!(raw.contains("## LoadOnDemand: 1"));
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare Dependencies — empty stub has no body to depend on"
    );
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## AllowLoad"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## DefaultState"));

    let body_lines: Vec<&str> = raw
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();
    assert!(
        body_lines.is_empty(),
        "TOC must have zero non-comment body lines — empty stub has no body files. \
         Got: {body_lines:?}"
    );
}

#[test]
fn lod_addon_excluded_from_eager_discovery_on_every_screen() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_SelectorUI");
        assert!(
            !found,
            "Blizzard_SelectorUI must be excluded from eager discovery on \
             {screen:?} — `## LoadOnDemand: 1` puts it in the LoD pool, and no \
             other Blizzard addon declares it as a hard Dependency. The selector \
             widget code consumed at runtime lives inside Blizzard_SharedXML's \
             body and ships through that addon's eager load instead"
        );
    }
}

#[test]
fn root_directory_holds_only_the_toc_with_no_body_files() {
    let dir = selector_ui_dir();
    assert!(dir.join("Blizzard_SelectorUI.toc").is_file());

    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("read addon dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "Blizzard_SelectorUI directory must contain exactly 1 entry (the TOC) — \
         the addon shell is a placeholder. Actual selector code lives in \
         Blizzard_SharedXML/Shared/Selector/. Got: {entries:?}"
    );
}

#[test]
fn actual_selector_code_lives_in_shared_xml_body_not_this_folder() {
    let shared_dir = shared_xml_selector_dir();
    assert!(
        shared_dir.is_dir(),
        "Blizzard_SharedXML/Shared/Selector/ must exist — Blizzard_SelectorUI is \
         an empty shell, the implementing files live here. SelectorMixin and \
         related widget templates are loaded via Blizzard_SharedXML's TOC body \
         entries `Shared\\Selector\\Blizzard_SelectorUI.lua` / .xml and the \
         GridSelectorUI / ScrollBoxSelector siblings"
    );

    for filename in [
        "Blizzard_SelectorUI.lua",
        "Blizzard_SelectorUI.xml",
        "Blizzard_GridSelectorUI.lua",
        "Blizzard_GridSelectorUI.xml",
        "Blizzard_ScrollBoxSelector.lua",
    ] {
        assert!(
            shared_dir.join(filename).is_file(),
            "Blizzard_SharedXML/Shared/Selector/{filename} must exist — the \
             selector widget family ships through SharedXML, not through the \
             empty Blizzard_SelectorUI shell"
        );
    }

    assert!(
        blizzard_ui_dir()
            .join("Blizzard_SharedXML/Mainline/Selector/Blizzard_ScrollBoxSelector.xml")
            .is_file(),
        "The Mainline selector XML companion must exist outside Shared/Selector"
    );
}

#[test]
fn shared_xml_mainline_toc_lists_selector_files_in_its_body() {
    let shared_xml_toc =
        blizzard_ui_dir().join("Blizzard_SharedXML/Blizzard_SharedXML_Mainline.toc");
    let raw = std::fs::read_to_string(&shared_xml_toc)
        .expect("Blizzard_SharedXML_Mainline.toc reads utf-8");

    for line in [
        "Shared\\Selector\\Blizzard_SelectorUI.lua",
        "Shared\\Selector\\Blizzard_SelectorUI.xml",
        "Shared\\Selector\\Blizzard_GridSelectorUI.lua",
        "Shared\\Selector\\Blizzard_GridSelectorUI.xml",
        "Shared\\Selector\\Blizzard_ScrollBoxSelector.lua",
        "[Family]\\Selector\\Blizzard_ScrollBoxSelector.xml",
    ] {
        assert!(
            raw.contains(line),
            "Blizzard_SharedXML_Mainline.toc must list `{line}` in its body — \
             this is how the selector widget code actually loads at runtime, \
             bypassing the empty Blizzard_SelectorUI shell entirely"
        );
    }
}

#[test]
fn explicit_load_emits_no_lua_errors_despite_empty_body() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();

    load_addon(&env.loader_env(), &selector_ui_toc())
        .expect("explicit load_addon on the empty stub must succeed");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_SelectorUI explicit load must not emit Lua errors — body is \
         empty so the loader has zero Lua chunks to execute, but the loader's \
         post-load workarounds and addon-registration paths still run. Got:\n  {}",
        load_errors.join("\n  ")
    );
}

#[test]
fn is_addon_loaded_transitions_false_to_true_after_explicit_load() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();

    let before: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SelectorUI')")
        .expect("pre-load IsAddOnLoaded probe succeeds");
    assert!(
        !before,
        "C_AddOns.IsAddOnLoaded('Blizzard_SelectorUI') must be false before \
         explicit load — LoadOnDemand=1 means it never appears in eager discovery"
    );

    load_addon(&env.loader_env(), &selector_ui_toc())
        .expect("explicit load_addon on the empty stub must succeed");

    let after: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SelectorUI')")
        .expect("post-load IsAddOnLoaded probe succeeds");
    assert!(
        after,
        "C_AddOns.IsAddOnLoaded('Blizzard_SelectorUI') must be true after \
         explicit load — even an empty-body addon registers as 'loaded' in the \
         AddOn state machine; downstream callers can use this to gate behavior \
         on the shell's nominal presence"
    );
}
