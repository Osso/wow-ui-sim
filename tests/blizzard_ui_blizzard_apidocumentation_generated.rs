//! Wrapper binary for Blizzard_APIDocumentationGenerated tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_apidocumentation_generated/` are re-exported
//! here.

use crate::common;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/load.rs"]
mod load;

// Wrath manifest1-382/cache has neither the documentation UI nor its corpus.
// Keep load.rs's mixed literal-TOC/runtime dependency case on all profiles.
post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_apidocumentation_generated/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/behavior_find_all_api_matches_returns_corpus_results.rs"]
mod behavior_find_all_api_matches_returns_corpus_results;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/behavior_handle_slash_command_stats_writes_real_counts.rs"]
mod behavior_handle_slash_command_stats_writes_real_counts;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/behavior_handle_slash_command_system_list_lists_corpus_systems.rs"]
mod behavior_handle_slash_command_system_list_lists_corpus_systems;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/behavior_handle_slash_command_system_search_dispatches_to_corpus_system.rs"]
mod behavior_handle_slash_command_system_search_dispatches_to_corpus_system;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/behavior_handle_api_link_default_writes_real_detailed_output.rs"]
mod behavior_handle_api_link_default_writes_real_detailed_output;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/behavior_handle_api_link_copyapi_records_real_clipboard.rs"]
mod behavior_handle_api_link_copyapi_records_real_clipboard;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/behavior_handle_api_link_opendump_seeds_real_chat_edit.rs"]
mod behavior_handle_api_link_opendump_seeds_real_chat_edit;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/behavior_data_files_self_register_idempotently.rs"]
mod behavior_data_files_self_register_idempotently;

#[path = "blizzard_ui/blizzard_apidocumentation_generated/behavior_dependency_loads_first.rs"]
mod behavior_dependency_loads_first;
}
