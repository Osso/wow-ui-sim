use std::path::{Path, PathBuf};

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

pub fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

pub fn env_with_professions_util() -> WowLuaEnv {
    let env = env();
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_professions_support_addons(&env);
    env
}

fn load_professions_support_addons(env: &WowLuaEnv) {
    for (name, toc_path) in professions_support_addons() {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|err| panic!("Failed to load Blizzard addon {name}: {err}"));
    }
}

fn professions_support_addons() -> Vec<(&'static str, PathBuf)> {
    let ui = blizzard_ui_dir();
    vec![
        ("Blizzard_SharedXMLBase", shared_xml_base_toc(&ui)),
        ("Blizzard_SharedXMLGame", shared_xml_game_toc(&ui)),
        ("Blizzard_Colors", colors_toc(&ui)),
        ("Blizzard_StaticPopup", static_popup_toc(&ui)),
        ("Blizzard_FrameXMLUtil", frame_xml_util_toc(&ui)),
    ]
}

/// ProfessionsUtil.lua:111 calls tAppendAll, owned by SharedXMLBase TableUtil.lua
/// since d509ed6aec retired the bootstrap copy.
fn shared_xml_base_toc(ui: &Path) -> PathBuf {
    find_existing_toc(ui, "Blizzard_SharedXMLBase", &["Blizzard_SharedXMLBase.toc"])
}

fn shared_xml_game_toc(ui: &Path) -> PathBuf {
    find_existing_toc(
        ui,
        "Blizzard_SharedXMLGame",
        &[
            "Blizzard_SharedXMLGame.toc",
            "Blizzard_SharedXMLGame_Mainline.toc",
            "Blizzard_SharedXMLGame_Classic.toc",
            "Blizzard_SharedXMLGame_Vanilla.toc",
        ],
    )
}

fn colors_toc(ui: &Path) -> PathBuf {
    find_existing_toc(
        ui,
        "Blizzard_Colors",
        &["Blizzard_Colors.toc", "Blizzard_Colors_Mainline.toc"],
    )
}

fn static_popup_toc(ui: &Path) -> PathBuf {
    find_existing_toc(ui, "Blizzard_StaticPopup", &["Blizzard_StaticPopup.toc"])
}

fn frame_xml_util_toc(ui: &Path) -> PathBuf {
    find_existing_toc(
        ui,
        "Blizzard_FrameXMLUtil",
        &[
            "Blizzard_FrameXMLUtil.toc",
            "Blizzard_FrameXMLUtil_Mainline.toc",
        ],
    )
}

fn find_existing_toc(ui: &Path, addon: &str, candidates: &[&str]) -> PathBuf {
    candidates
        .iter()
        .map(|toc| ui.join(addon).join(toc))
        .find(|toc_path| toc_path.exists())
        .unwrap_or_else(|| ui.join(addon).join(candidates[0]))
}

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(Path::new(env!("CARGO_MANIFEST_DIR")))
}
