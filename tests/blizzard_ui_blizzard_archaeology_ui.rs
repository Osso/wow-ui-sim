//! Wrapper binary for Blizzard_ArchaeologyUI tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_archaeology_ui/` are re-exported here.

use crate::common;

#[path = "blizzard_ui/blizzard_archaeology_ui/load.rs"]
mod load;

// Wrath manifest1-382/cache has no ArchaeologyFrame or its Lua UI handlers.
// Keep load.rs's mixed TOC/dependency/runtime assertion on every profile.
post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_archaeology_ui/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_archaeology_ui/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_archaeology_ui/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_show_opens_the_panel.rs"]
mod behavior_show_opens_the_panel;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_onload_seeds_title_and_pages.rs"]
mod behavior_onload_seeds_title_and_pages;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_summary_page_lists_races.rs"]
mod behavior_summary_page_lists_races;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_artifact_page_renders_selected.rs"]
mod behavior_artifact_page_renders_selected;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_solve_button_clicks_solve.rs"]
mod behavior_solve_button_clicks_solve;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_keystone_click_toggles_socket.rs"]
mod behavior_keystone_click_toggles_socket;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_completed_page_paginates.rs"]
mod behavior_completed_page_paginates;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_race_filter_dropdown_filters_listing.rs"]
mod behavior_race_filter_dropdown_filters_listing;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_show_failed_calls_close_research.rs"]
mod behavior_show_failed_calls_close_research;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_tab_switches_swap_currentframe.rs"]
mod behavior_tab_switches_swap_currentframe;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_research_artifact_complete_event_refreshes_pages.rs"]
mod behavior_research_artifact_complete_event_refreshes_pages;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_bag_update_delayed_refreshes_keystones.rs"]
mod behavior_bag_update_delayed_refreshes_keystones;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_summary_page_mouse_wheel_paginates.rs"]
mod behavior_summary_page_mouse_wheel_paginates;

#[path = "blizzard_ui/blizzard_archaeology_ui/behavior_faction_icon_uses_alliance_or_horde_texcoords.rs"]
mod behavior_faction_icon_uses_alliance_or_horde_texcoords;
}
