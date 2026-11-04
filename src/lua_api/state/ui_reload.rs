//! Retire UI-owned state without replacing the modeled game session.

use super::SimState;

impl SimState {
    pub(crate) fn in_combat_lockdown(&self) -> bool {
        let reload_window = crate::client_profile::ACTIVE
            == crate::client_profile::ClientProfile::Wrath
            && self.reload_allows_protected_actions;
        self.player.in_combat && !reload_window
    }

    pub(crate) fn discard_ui_environment(&mut self) {
        self.discard_ui_registries();
        self.discard_ui_callbacks();
        self.discard_addon_runtime();
        self.discard_ui_input();
        self.crafting.recipe_favorites = Default::default();
        self.ui_reload_requested = false;
        self.ui_reloaded = true;
        self.is_logged_in = false;
        self.account_cvar_callback_pending = false;
        self.post_event_workarounds_applied = false;
        self.screen_first_displayed = false;
        self.texture_manager
            .borrow_mut()
            .invalidate_source_generation();
    }

    fn discard_ui_registries(&mut self) {
        self.widgets = crate::widget::WidgetRegistry::new();
        self.tooltips.clear();
        self.quest_blobs.clear();
        self.fog_of_war_frames.clear();
        self.unit_position_frames.clear();
        self.message_frames.clear();
        self.private_aura_anchors = Default::default();
        #[cfg(feature = "retail-12-0-5")]
        {
            self.private_aura_sound_registrations = Default::default();
        }
        self.quest_portrait_state = None;
        self.visible_on_update_cache = None;
        self.strata_buckets = None;
        self.active_toplevel_show_orders.clear();
        self.next_toplevel_show_order = 0;
        self.pending_hit_grid_changes.clear();
        self.pending_texture_preloads.clear();
        self.action_ui_buttons.clear();
        self.open_panels.clear();
        self.menu_open = false;
        self.store_frame_shown = false;
    }

    fn discard_ui_callbacks(&mut self) {
        self.events = Default::default();
        self.scripts = Default::default();
        self.timers.clear();
        self.rilua_timers.clear();
        self.timer_callback_sources.clear();
        #[cfg(feature = "timed-signal-maps")]
        self.timed_signal_maps.clear();
        self.on_update_frames.clear();
        self.pending_scroll_range_updates.clear();
        self.animation_groups.clear();
        self.active_animation_groups.clear();
        self.next_anim_group_id = 1;
        self.anim_frame_to_group.clear();
        self.anim_frame_to_anim.clear();
        self.anim_sync_times.clear();
        self.secure_attribute_drivers.clear();
        self.secure_state_dispatch_ready = false;
        self.global_show_hide_depth = 0;
    }

    fn discard_addon_runtime(&mut self) {
        self.addons.clear();
        self.addon_initialization_failures.clear();
        self.loading_addon_index = None;
        self.loading_addon_stack.clear();
        self.executing_addon_index = None;
        self.loading_nil_symbol_environment = None;
        self.loading_scoped_script_env = None;
        self.loading_forbidden = false;
        self.loading_add_to_secure_env = false;
        self.loading_hide_from_global_env = false;
        self.loading_use_forbidden_object_table = false;
        self.loading_allow_untainted_creation = false;
        self.create_frame_initial_hidden = None;
        self.suppress_runtime_on_load_depth = 0;
        self.xml_load_addon_depth = 0;
        self.addon_performance_messages_shown.clear();
        self.nil_symbol_accesses.clear();
        self.global_publications.clear();
        self.secure_global_publications.clear();
        self.pending_nested_addon_diagnostics.clear();
        self.console_output.clear();
    }

    fn discard_ui_input(&mut self) {
        self.focused_frame_id = None;
        self.hovered_frame = None;
        self.active_drag_frame = None;
        self.active_slider_thumb_drag_frame = None;
        self.chat_edit_open_state = None;
        #[cfg(feature = "retail-12-0-7")]
        self.simulated_mouse_inputs.clear();
    }
}
