use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_addon_closure_for_screen,
    discover_blizzard_addons_for_screen, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[path = "patch_12_1/aura_container.rs"]
mod aura_container;
#[path = "patch_12_1/aura_tooltip.rs"]
mod aura_tooltip;
#[path = "patch_12_1/combat_audio.rs"]
mod combat_audio;
#[path = "patch_12_1/friends_list.rs"]
mod friends_list;
#[path = "patch_12_1/guild_control.rs"]
mod guild_control;
#[path = "patch_12_1/input_util.rs"]
mod input_util;
#[path = "patch_12_1/interface_util.rs"]
mod interface_util;
#[path = "patch_12_1/math_extensions.rs"]
mod math_extensions;
#[path = "patch_12_1/narration.rs"]
mod narration;
#[path = "patch_12_1/player_choice.rs"]
mod player_choice;
#[path = "patch_12_1/ptr_feedback.rs"]
mod ptr_feedback;
#[path = "patch_12_1/remaining_observations.rs"]
mod remaining_observations;
#[path = "patch_12_1/shake.rs"]
mod shake;
#[path = "patch_12_1/social_ui.rs"]
mod social_ui;
#[path = "patch_12_1/source_absent.rs"]
mod source_absent;
#[path = "patch_12_1/table_extensions.rs"]
mod table_extensions;
#[path = "patch_12_1/ui_geometry.rs"]
mod ui_geometry;
#[path = "patch_12_1/utility_namespaces.rs"]
mod utility_namespaces;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn ptr_source_files() -> &'static Vec<(PathBuf, String)> {
    static FILES: OnceLock<Vec<(PathBuf, String)>> = OnceLock::new();
    FILES.get_or_init(|| {
        fn collect(path: &Path, files: &mut Vec<(PathBuf, String)>) {
            for entry in fs::read_dir(path).expect("PTR AddOns directory should be readable") {
                let entry = entry.expect("PTR source entry should be readable");
                let path = entry.path();
                if path.is_dir() {
                    collect(&path, files);
                    continue;
                }

                let extension = path.extension().and_then(|value| value.to_str());
                if matches!(extension, Some("lua" | "xml" | "toc")) {
                    let source =
                        fs::read_to_string(&path).expect("PTR source file should be UTF-8 text");
                    files.push((path, source));
                }
            }
        }

        let mut files = Vec::new();
        collect(&blizzard_ui_dir(), &mut files);
        files
    })
}

fn assert_ptr_source_omits_symbols(symbols: &[&str]) {
    for (path, source) in ptr_source_files() {
        for symbol in symbols {
            assert!(
                !source.contains(symbol),
                "snapshot-only symbol {symbol} unexpectedly appears in {}",
                path.display(),
            );
        }
    }
}

fn assert_ptr_source_omits_tokens(symbols: &[&str]) {
    let tokens = symbols
        .iter()
        .map(|symbol| symbol.split_once('.').map_or(*symbol, |(_, method)| method))
        .collect::<Vec<_>>();
    assert_ptr_source_omits_symbols(&tokens);
}

fn assert_ptr_source_omits_qualified_symbols(symbols: &[&str]) {
    for symbol in symbols {
        let (namespace, method) = symbol
            .split_once('.')
            .expect("qualified patch symbol should contain a dot");
        assert_ptr_source_omits_qualified_methods(namespace, &[method]);
    }
}

fn assert_ptr_source_omits_qualified_methods(namespace: &str, methods: &[&str]) {
    let publications = methods
        .iter()
        .flat_map(|method| {
            [
                format!("{namespace}.{method}"),
                format!("{namespace}:{method}"),
                format!("{namespace}[\"{method}\"]"),
                format!("{namespace}['{method}']"),
                format!("rawset({namespace}, \"{method}\""),
                format!("rawset({namespace}, '{method}'"),
            ]
        })
        .collect::<Vec<_>>();
    let publications = publications.iter().map(String::as_str).collect::<Vec<_>>();
    assert_ptr_source_omits_symbols(&publications);
}

fn assert_ptr_source_contains(symbol: &str) {
    assert!(
        ptr_source_files()
            .iter()
            .any(|(_, source)| source.contains(symbol)),
        "expected PTR source publication {symbol} was not found",
    );
}

fn player_choice_toc() -> PathBuf {
    blizzard_ui_dir()
        .join("Blizzard_PlayerChoice")
        .join("Blizzard_PlayerChoice.toc")
}

fn new_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_game_ui_with_all_lod() -> WowLuaEnv {
    let env = new_game_env();
    let roots = discover_all_blizzard_addons(&blizzard_ui_dir())
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    let root_refs = roots.iter().map(String::as_str).collect::<Vec<_>>();

    for (name, toc_path) in
        discover_blizzard_addon_closure_for_screen(&blizzard_ui_dir(), ScreenKind::Game, &root_refs)
    {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("[load all {name}] FAILED: {error}"));
    }

    env
}

fn load_game_ui_without_player_choice() -> WowLuaEnv {
    let env = new_game_env();

    for (name, toc_path) in
        discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game)
    {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("[load {name}] FAILED: {error}"));
    }

    env
}
