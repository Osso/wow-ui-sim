//! Wrapper binary that pulls every per-aspect Blizzard_AccountSaveUI
//! test file under `tests/blizzard_ui/blizzard_accountsaveui/` into a
//! single `cargo test --test ...` target. Cargo only auto-discovers
//! `tests/*.rs`, so the nested `load.rs` / `surface_*.rs` / `behavior_*.rs`
//! files declared by the per-addon plan template need a flat re-export here
//! to be reachable.

use crate::common;

// Wrath manifest1-382/cache has no AccountSaveFrame or AccountSaveFrameMixin;
// six other manifests ship Blizzard_AccountSaveUI's Glue UI publishers.
post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_accountsaveui/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_accountsaveui/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_accountsaveui/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_accountsaveui/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_accountsaveui/surface_mixins.rs"]
mod surface_mixins;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_account_state_disabled.rs"]
mod behavior_account_state_disabled;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_account_state_locked.rs"]
mod behavior_account_state_locked;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_account_state_unlocked.rs"]
mod behavior_account_state_unlocked;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_save_in_progress.rs"]
mod behavior_save_in_progress;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_lock_string_match.rs"]
mod behavior_lock_string_match;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_save_button_click.rs"]
mod behavior_save_button_click;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_save_result_event.rs"]
mod behavior_save_result_event;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_save_result_errors.rs"]
mod behavior_save_result_errors;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_success_popup_launch_url.rs"]
mod behavior_success_popup_launch_url;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_update_sizing.rs"]
mod behavior_update_sizing;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_on_size_changed.rs"]
mod behavior_on_size_changed;

#[path = "blizzard_ui/blizzard_accountsaveui/behavior_editbox_keys.rs"]
mod behavior_editbox_keys;
}
