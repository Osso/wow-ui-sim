use super::{load_full_game_ui_with_all_lod, missing_runtime_functions};

const PTR_GLOBALS: &[&str] = &[
    "GuildControlUI_Discord_HideAll",
    "GuildControlUI_Discord_Update",
    "GuildControlUI_DiscordFrame_OnLoad",
    "GuildControlUI_LoadUI",
    "GuildControlUI_OnShow",
    "GuildControlUI_SetupDiscord",
    "GuildControlUI_SetupSelected",
    "GuildControlUI_Setup",
    "GuildControlUI_Show",
    "GuildControlUI_UnlinkDiscord",
];

/// Proves the pinned PTR GuildControlUI globals are published after their LoD owner loads.
#[test]
fn ptr_guild_control_globals_are_published_by_the_loaded_owner() {
    let env = load_full_game_ui_with_all_lod();
    let missing = missing_runtime_functions(&env, PTR_GLOBALS);
    assert_eq!(missing, "", "pinned PTR globals must remain published");
}
