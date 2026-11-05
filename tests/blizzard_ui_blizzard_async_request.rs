//! Wrapper binary for Blizzard_AsyncRequest tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_async_request/` are re-exported here.

use crate::common;

#[path = "blizzard_ui/blizzard_async_request/load.rs"]
mod load;

// No CreateAsyncRequest/AsyncRequestMixin in Wrath manifest1-382/cache.
// Keep load.rs visible: its mixed TOC/runtime cases are not snapshot gates.
post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_async_request/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_async_request/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_async_request/behavior_create_requires_callbacks.rs"]
mod behavior_create_requires_callbacks;

#[path = "blizzard_ui/blizzard_async_request/behavior_create_timeout_pair_required.rs"]
mod behavior_create_timeout_pair_required;

#[path = "blizzard_ui/blizzard_async_request/behavior_start_calls_request_function.rs"]
mod behavior_start_calls_request_function;

#[path = "blizzard_ui/blizzard_async_request/behavior_start_registers_response_event.rs"]
mod behavior_start_registers_response_event;

#[path = "blizzard_ui/blizzard_async_request/behavior_start_is_idempotent_when_running.rs"]
mod behavior_start_is_idempotent_when_running;

#[path = "blizzard_ui/blizzard_async_request/behavior_response_event_fires_callback.rs"]
mod behavior_response_event_fires_callback;

#[path = "blizzard_ui/blizzard_async_request/behavior_response_event_other_name_ignored.rs"]
mod behavior_response_event_other_name_ignored;

#[path = "blizzard_ui/blizzard_async_request/behavior_timeout_fires_callback.rs"]
mod behavior_timeout_fires_callback;

#[path = "blizzard_ui/blizzard_async_request/behavior_response_before_timeout_cancels_timer.rs"]
mod behavior_response_before_timeout_cancels_timer;

#[path = "blizzard_ui/blizzard_async_request/behavior_stop_without_start_is_safe.rs"]
mod behavior_stop_without_start_is_safe;

#[path = "blizzard_ui/blizzard_async_request/behavior_no_timeout_path_skips_timer.rs"]
mod behavior_no_timeout_path_skips_timer;

#[path = "blizzard_ui/blizzard_async_request/behavior_two_concurrent_requests_isolated.rs"]
mod behavior_two_concurrent_requests_isolated;

#[path = "blizzard_ui/blizzard_async_request/behavior_response_callback_can_restart.rs"]
mod behavior_response_callback_can_restart;
}
