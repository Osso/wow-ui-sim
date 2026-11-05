//! Wrapper binary for Blizzard_APIDocumentation tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_apidocumentation/` are re-exported here.

use crate::common;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_add_documentation_table_routes_payload_kinds.rs"]
mod behavior_add_documentation_table_routes_payload_kinds;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_find_all_api_matches_returns_nil_when_empty.rs"]
mod behavior_find_all_api_matches_returns_nil_when_empty;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_find_api_by_name_returns_first_match_across_kinds.rs"]
mod behavior_find_api_by_name_returns_first_match_across_kinds;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_get_api_table_by_type_name_routes_kind.rs"]
mod behavior_get_api_table_by_type_name_routes_kind;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_write_line_uses_system_chat_color.rs"]
mod behavior_write_line_uses_system_chat_color;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_get_indent_string_returns_two_space_blocks.rs"]
mod behavior_get_indent_string_returns_two_space_blocks;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_handle_slash_command_help_writes_usage.rs"]
mod behavior_handle_slash_command_help_writes_usage;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_handle_slash_command_stats_writes_counts.rs"]
mod behavior_handle_slash_command_stats_writes_counts;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_handle_slash_command_search_writes_matches.rs"]
mod behavior_handle_slash_command_search_writes_matches;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_handle_slash_command_system_list_writes_all_systems.rs"]
mod behavior_handle_slash_command_system_list_writes_all_systems;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_handle_slash_command_system_search_dispatches_to_system.rs"]
mod behavior_handle_slash_command_system_search_dispatches_to_system;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_handle_api_link_default_writes_detailed_output.rs"]
mod behavior_handle_api_link_default_writes_detailed_output;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_handle_api_link_copyapi_records_clipboard.rs"]
mod behavior_handle_api_link_copyapi_records_clipboard;

#[path = "blizzard_ui/blizzard_apidocumentation/behavior_handle_api_link_opendump_seeds_chat_edit.rs"]
mod behavior_handle_api_link_opendump_seeds_chat_edit;

#[path = "blizzard_ui/blizzard_apidocumentation/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_apidocumentation/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_apidocumentation/surface_mixins.rs"]
mod surface_mixins;
