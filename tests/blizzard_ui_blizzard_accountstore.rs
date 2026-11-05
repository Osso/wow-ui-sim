//! Wrapper binary that pulls every per-aspect Blizzard_AccountStore
//! test file under `tests/blizzard_ui/blizzard_accountstore/` into a
//! single `cargo test --test ...` target. Cargo only auto-discovers
//! `tests/*.rs`, so the nested `load.rs` / `surface_*.rs` / `behavior_*.rs`
//! files declared by the per-addon plan template need a flat re-export here
//! to be reachable.

use crate::common;

#[path = "blizzard_ui/blizzard_accountstore/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_accountstore/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_accountstore/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_accountstore/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_accountstore/surface_mixins/mod.rs"]
mod surface_mixins;

#[path = "blizzard_ui/blizzard_accountstore/behavior_toggle.rs"]
mod behavior_toggle;

#[path = "blizzard_ui/blizzard_accountstore/behavior_set_storefront.rs"]
mod behavior_set_storefront;

#[path = "blizzard_ui/blizzard_accountstore/behavior_on_show_sound.rs"]
mod behavior_on_show_sound;

#[path = "blizzard_ui/blizzard_accountstore/behavior_on_hide_sound.rs"]
mod behavior_on_hide_sound;

#[path = "blizzard_ui/blizzard_accountstore/behavior_category_selected.rs"]
mod behavior_category_selected;

#[path = "blizzard_ui/blizzard_accountstore/behavior_card_select_purchase.rs"]
mod behavior_card_select_purchase;

#[path = "blizzard_ui/blizzard_accountstore/behavior_card_select_refund.rs"]
mod behavior_card_select_refund;

#[path = "blizzard_ui/blizzard_accountstore/behavior_currency_format.rs"]
mod behavior_currency_format;

#[path = "blizzard_ui/blizzard_accountstore/behavior_currency_warning.rs"]
mod behavior_currency_warning;

#[path = "blizzard_ui/blizzard_accountstore/behavior_storefront_state_event.rs"]
mod behavior_storefront_state_event;

#[path = "blizzard_ui/blizzard_accountstore/behavior_transaction_error.rs"]
mod behavior_transaction_error;

#[path = "blizzard_ui/blizzard_accountstore/behavior_item_info_updated.rs"]
mod behavior_item_info_updated;

#[path = "blizzard_ui/blizzard_accountstore/behavior_item_rack_paging.rs"]
mod behavior_item_rack_paging;

#[path = "blizzard_ui/blizzard_accountstore/behavior_fullscreen_escape.rs"]
mod behavior_fullscreen_escape;

#[path = "blizzard_ui/blizzard_accountstore/behavior_refund_timer.rs"]
mod behavior_refund_timer;

#[path = "blizzard_ui/blizzard_accountstore/behavior_currency_for_store.rs"]
mod behavior_currency_for_store;

#[path = "blizzard_ui/blizzard_accountstore/behavior_category_list_refresh.rs"]
mod behavior_category_list_refresh;

#[path = "blizzard_ui/blizzard_accountstore/behavior_card_creature_display.rs"]
mod behavior_card_creature_display;
