//! Wrapper binary that pulls every per-aspect Blizzard_ActionBar
//! test file under `tests/blizzard_ui/blizzard_actionbar/` into a
//! single `cargo test --test ...` target. Cargo only auto-discovers
//! `tests/*.rs`, so the nested `load.rs` / `surface_*.rs` / `behavior_*.rs`
//! files declared by the per-addon plan template need a flat re-export here
//! to be reachable.

use crate::common;

// 12340 has no EditMode attach/detach, HideBarArt setting, or honor XP bar.
post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbar/behavior_attach_detach.rs"]
mod behavior_attach_detach;

#[path = "blizzard_ui/blizzard_actionbar/behavior_battlefield_hides_xp_bar.rs"]
mod behavior_battlefield_hides_xp_bar;

#[path = "blizzard_ui/blizzard_actionbar/behavior_main_bar_endcaps.rs"]
mod behavior_main_bar_endcaps;
}

#[path = "blizzard_ui/blizzard_actionbar/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_actionbar/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_actionbar/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_actionbar/surface_mixins.rs"]
mod surface_mixins;

#[path = "blizzard_ui/blizzard_actionbar/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_actionbar/behavior_main_bar_buttons.rs"]
mod behavior_main_bar_buttons;

#[path = "blizzard_ui/blizzard_actionbar/behavior_page_change.rs"]
mod behavior_page_change;

#[path = "blizzard_ui/blizzard_actionbar/behavior_keybind_dispatch.rs"]
mod behavior_keybind_dispatch;

#[path = "blizzard_ui/blizzard_actionbar/behavior_show_grid.rs"]
mod behavior_show_grid;

#[path = "blizzard_ui/blizzard_actionbar/behavior_stance_select.rs"]
mod behavior_stance_select;

#[path = "blizzard_ui/blizzard_actionbar/behavior_pet_bar_update.rs"]
mod behavior_pet_bar_update;

#[path = "blizzard_ui/blizzard_actionbar/behavior_possess_bar_show.rs"]
mod behavior_possess_bar_show;

#[path = "blizzard_ui/blizzard_actionbar/behavior_extra_action_bar.rs"]
mod behavior_extra_action_bar;

#[path = "blizzard_ui/blizzard_actionbar/behavior_vehicle_leave_button.rs"]
mod behavior_vehicle_leave_button;

#[path = "blizzard_ui/blizzard_actionbar/behavior_xp_bar_update.rs"]
mod behavior_xp_bar_update;

#[path = "blizzard_ui/blizzard_actionbar/behavior_reputation_bar_update.rs"]
mod behavior_reputation_bar_update;

#[path = "blizzard_ui/blizzard_actionbar/behavior_honor_bar_update.rs"]
mod behavior_honor_bar_update;

#[path = "blizzard_ui/blizzard_actionbar/behavior_house_favor_bar.rs"]
mod behavior_house_favor_bar;

#[path = "blizzard_ui/blizzard_actionbar/behavior_artifact_bar.rs"]
mod behavior_artifact_bar;

#[path = "blizzard_ui/blizzard_actionbar/behavior_azerite_bar.rs"]
mod behavior_azerite_bar;

#[path = "blizzard_ui/blizzard_actionbar/behavior_status_bar_swap.rs"]
mod behavior_status_bar_swap;

#[path = "blizzard_ui/blizzard_actionbar/behavior_spell_flyout.rs"]
mod behavior_spell_flyout;

#[path = "blizzard_ui/blizzard_actionbar/behavior_spell_flyout_glyph.rs"]
mod behavior_spell_flyout_glyph;

#[path = "blizzard_ui/blizzard_actionbar/behavior_assisted_combat_highlight.rs"]
mod behavior_assisted_combat_highlight;

#[path = "blizzard_ui/blizzard_actionbar/behavior_action_highlight.rs"]
mod behavior_action_highlight;

#[path = "blizzard_ui/blizzard_actionbar/behavior_locked_action.rs"]
mod behavior_locked_action;

#[path = "blizzard_ui/blizzard_actionbar/behavior_outfit_locked.rs"]
mod behavior_outfit_locked;

#[path = "blizzard_ui/blizzard_actionbar/behavior_quick_keybind_mode.rs"]
mod behavior_quick_keybind_mode;

#[path = "blizzard_ui/blizzard_actionbar/behavior_main_bar_dividers.rs"]
mod behavior_main_bar_dividers;

#[path = "blizzard_ui/blizzard_actionbar/behavior_spell_alert_animation.rs"]
mod behavior_spell_alert_animation;

#[path = "blizzard_ui/blizzard_actionbar/behavior_button_range_check.rs"]
mod behavior_button_range_check;

#[path = "blizzard_ui/blizzard_actionbar/behavior_resting_indicator.rs"]
mod behavior_resting_indicator;

#[path = "blizzard_ui/blizzard_actionbar/behavior_paragon_tooltip.rs"]
mod behavior_paragon_tooltip;

#[path = "blizzard_ui/blizzard_actionbar/behavior_button_tooltip.rs"]
mod behavior_button_tooltip;
