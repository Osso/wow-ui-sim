//! TOC parser facade; active-profile selection and host discovery remain here.
use crate::screen::ScreenKind;
use std::path::{Path, PathBuf};

pub use sim_toc::{collect_metadata_lists, selected_file_path};
pub type TocFile = sim_toc::TocFile<ActiveSelection>;
pub const RETAIL_INTERFACE_VERSION: u32 = crate::client_profile::RETAIL_API_INTERFACE_VERSION;
pub const ACTIVE_INTERFACE_VERSION: u32 = crate::client_profile::ACTIVE_INTERFACE_VERSION;

#[derive(Debug, Clone)]
pub struct ActiveSelection;
impl sim_toc::Selection for ActiveSelection {
    type Screen = ScreenKind;
    fn game_types() -> &'static [&'static str] {
        active_game_types()
    }
    fn recognizes_camelot() -> bool {
        matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Ptr
                | crate::client_profile::ClientProfile::WowForever
        )
    }
    fn family_subdir() -> &'static str {
        family_subdir()
    }
    fn game_subdir() -> &'static str {
        game_subdir()
    }
    fn is_variable_metadata(line: &str) -> bool {
        is_variable_metadata(line)
    }
    fn is_mists_game_menu_shared_file(addon_dir: &Path, line: &str) -> bool {
        is_mists_game_menu_shared_file(addon_dir, line)
    }
    fn expands_regional_text_locale() -> bool {
        matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Ptr
                | crate::client_profile::ClientProfile::WowForever
        )
    }
    fn permits_ptr_only() -> bool {
        Self::expands_regional_text_locale()
    }
    fn is_mists() -> bool {
        crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Mists
    }
    fn is_game(screen: ScreenKind) -> bool {
        screen == ScreenKind::Game
    }
    fn is_glue(screen: ScreenKind) -> bool {
        screen.is_glue()
    }
    fn find_case_insensitive(dir: &Path, name: &str) -> Option<PathBuf> {
        crate::paths::find_case_insensitive(dir, name)
    }
    fn is_builtin_addon_folder(name: &str) -> bool {
        crate::blizzard_ui_sync::is_builtin_addon_folder(name)
    }
}

pub(crate) fn addon_loads_as_blizzard_code(
    folder_name: Option<&str>,
    allow_load: bool,
    use_secure_env: bool,
) -> bool {
    sim_toc::addon_loads_as_blizzard_code::<ActiveSelection>(
        folder_name,
        allow_load,
        use_secure_env,
    )
}
fn active_game_types() -> &'static [&'static str] {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::Ptr => &["mainline", "standard"],
        crate::client_profile::ClientProfile::Wrath => &["wrath", "wrath_classic", "classic"],
        crate::client_profile::ClientProfile::Mists => &["mists", "mists_classic", "classic"],
        crate::client_profile::ClientProfile::Era => &["vanilla", "classic_era", "classic"],
        crate::client_profile::ClientProfile::Anniversary => {
            &["vanilla", "classic_anniversary", "classic"]
        }
        crate::client_profile::ClientProfile::WowForever => &["camelot", "mainline"],
    }
}

fn family_subdir() -> &'static str {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::Ptr
        | crate::client_profile::ClientProfile::WowForever => "Mainline",
        crate::client_profile::ClientProfile::Wrath
        | crate::client_profile::ClientProfile::Mists
        | crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => "Classic",
    }
}

fn game_subdir() -> &'static str {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::Ptr => "Standard",
        crate::client_profile::ClientProfile::Wrath => "Wrath",
        crate::client_profile::ClientProfile::Mists => "Mists",
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => "Vanilla",
        crate::client_profile::ClientProfile::WowForever => "Camelot",
    }
}

fn is_variable_metadata(line: &str) -> bool {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Ptr => {
            ["[Family]", "[Game]", "[TextLocale]", "[RegionalTextLocale]"]
                .iter()
                .any(|variable| line.contains(variable))
        }
        _ => false,
    }
}

fn is_mists_game_menu_shared_file(addon_dir: &Path, line: &str) -> bool {
    crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Mists
        && addon_dir.file_name().and_then(|name| name.to_str()) == Some("Blizzard_GameMenu")
        && (line.contains("Shared\\GameMenuFrame.") || line.contains("Shared/GameMenuFrame."))
        && line.contains("standard")
}

#[cfg(test)]
fn matches_active_game_type(types: &str) -> bool {
    sim_toc::matches_active_game_type::<ActiveSelection>(types)
}

#[cfg(test)]
fn is_allowed_game_type(line: &str) -> bool {
    sim_toc::is_allowed_game_type::<ActiveSelection>(line)
}

#[cfg(test)]
fn is_all_template_versions(value: &str) -> bool {
    sim_toc::is_all_template_versions(value)
}

#[cfg(test)]
#[path = "../../crates/sim-toc/src/selected_profile_tests.rs"]
mod tests;
