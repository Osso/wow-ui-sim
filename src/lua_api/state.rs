//! Shared state types for the WoW Lua API.

use crate::cvars::CVarStorage;
use crate::event::{EventQueue, ScriptRegistry};
use crate::lua_api::animation::AnimGroupState;
use crate::lua_api::message_frame::MessageFrameData;
use crate::lua_api::simple_html::SimpleHtmlData;
use crate::lua_api::tooltip::TooltipData;
use crate::sound::SoundManager;
use crate::widget::WidgetRegistry;
use mlua::RegistryKey;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::time::Instant;

// Re-export game data types so existing `crate::lua_api::state::X` imports keep working.
pub use super::game_data::{
    AuraInfo, CastingState, PartyMember, TargetInfo,
    CLASS_LABELS, RACE_DATA, ROT_DAMAGE_LEVELS, XP_LEVELS,
    build_target_info, tick_party_health,
};
pub use super::game_data::SpellCooldownState;

/// What is currently held on the cursor (drag-and-drop state).
#[derive(Debug, Clone)]
pub enum CursorInfo {
    /// An action bar spell: PickupAction(slot) removes it from the bar.
    Action { slot: u32, spell_id: u32 },
    /// A spell from the spellbook (doesn't remove from spellbook).
    Spell { spell_id: u32 },
}
use super::game_data::{
    default_action_bars, default_party, default_player_buffs, random_player_name,
};

/// A pending timer callback.
pub struct PendingTimer {
    /// Unique timer ID.
    pub id: u64,
    /// When this timer should fire.
    pub fire_at: Instant,
    /// Lua function to call (stored in registry).
    pub callback_key: RegistryKey,
    /// For tickers: interval between firings.
    pub interval: Option<std::time::Duration>,
    /// For tickers with limited iterations: remaining count.
    pub remaining: Option<i32>,
    /// Whether this timer has been cancelled.
    pub cancelled: bool,
    /// The timer/ticker handle table (stored in registry) to pass to callback.
    pub handle_key: Option<RegistryKey>,
    /// Addon that created this timer (for profiler attribution).
    pub owner_addon: Option<u16>,
}

/// Per-addon runtime profiler metrics, updated each frame.
#[derive(Debug, Clone)]
pub struct AddonRuntimeMetrics {
    /// Time spent in this addon's handlers during the current frame (accumulator).
    pub current_frame_ms: f64,
    /// Rolling window of per-frame times (last 60 frames) for RecentAverageTime.
    pub recent_frames: VecDeque<f64>,
    /// Peak time ever recorded in a single frame.
    pub peak_ms: f64,
    /// Session total time (ms) across all frames.
    pub session_total_ms: f64,
    /// Number of frames where this addon had handlers fire.
    pub session_frame_count: u64,
    /// Threshold counters: frames where addon time exceeded N ms.
    pub count_over_1ms: u32,
    pub count_over_5ms: u32,
    pub count_over_10ms: u32,
    pub count_over_50ms: u32,
    pub count_over_100ms: u32,
    pub count_over_500ms: u32,
    pub count_over_1000ms: u32,
}

impl Default for AddonRuntimeMetrics {
    fn default() -> Self {
        Self {
            current_frame_ms: 0.0,
            recent_frames: VecDeque::with_capacity(60),
            peak_ms: 0.0,
            session_total_ms: 0.0,
            session_frame_count: 0,
            count_over_1ms: 0,
            count_over_5ms: 0,
            count_over_10ms: 0,
            count_over_50ms: 0,
            count_over_100ms: 0,
            count_over_500ms: 0,
            count_over_1000ms: 0,
        }
    }
}

/// Application-level frame timing for profiler (total frame time, not just addon time).
#[derive(Debug, Clone, Default)]
pub struct AppFrameMetrics {
    /// Rolling window of total frame times in ms (last 60 frames).
    pub recent_frame_ms: VecDeque<f64>,
    /// Peak frame time ever recorded.
    pub peak_ms: f64,
    /// Session total frame time in ms.
    pub session_total_ms: f64,
    /// Number of frames recorded.
    pub session_frame_count: u64,
}

/// Information about a loaded addon.
#[derive(Debug, Clone, Default)]
pub struct AddonInfo {
    /// Folder name (used as addon identifier).
    pub folder_name: String,
    /// Display title from TOC metadata.
    pub title: String,
    /// Notes/description from TOC metadata.
    pub notes: String,
    /// Whether the addon is currently enabled.
    pub enabled: bool,
    /// Whether the addon was successfully loaded.
    pub loaded: bool,
    /// Load on demand flag.
    pub load_on_demand: bool,
    /// Total load time in seconds (for profiler metrics).
    pub load_time_secs: f64,
    /// Runtime profiler metrics (updated per frame).
    pub runtime: AddonRuntimeMetrics,
}

/// Shared simulator state accessible from Lua.
pub struct SimState {
    pub widgets: WidgetRegistry,
    pub events: EventQueue,
    pub scripts: ScriptRegistry,
    /// Console output from Lua print() calls.
    pub console_output: Vec<String>,
    /// Pending timer callbacks.
    pub timers: VecDeque<PendingTimer>,
    /// Currently focused frame ID (for keyboard input).
    pub focused_frame_id: Option<u64>,
    /// Registered addons (includes all scanned addons, not just loaded ones).
    pub addons: Vec<AddonInfo>,
    /// Console variables (CVars).
    pub cvars: CVarStorage,
    /// Tooltip state for GameTooltip frames (keyed by frame ID).
    pub tooltips: HashMap<u64, TooltipData>,
    /// SimpleHTML state (keyed by frame ID).
    pub simple_htmls: HashMap<u64, SimpleHtmlData>,
    /// MessageFrame state (keyed by frame ID).
    pub message_frames: HashMap<u64, MessageFrameData>,
    /// Frame IDs with active OnUpdate script handlers.
    pub on_update_frames: HashSet<u64>,
    /// Cached subset of `on_update_frames` whose ancestors are all visible.
    /// Invalidated when `WidgetRegistry::visibility_dirty` is set.
    pub visible_on_update_cache: Option<Vec<u64>>,
    /// Per-strata buckets of visible frame IDs. Index = FrameStrata as usize.
    /// Contains only frames with render_alpha > 0 (visible or button state
    /// textures with visible parent). Built lazily, maintained surgically
    /// by `set_frame_visible`.
    pub strata_buckets: Option<Vec<Vec<u64>>>,
    /// Pending HitGrid updates from `set_frame_visible`. Each entry is the root
    /// frame ID that changed visibility and whether it became visible.
    /// Drained and applied by the App after Lua handlers run.
    pub pending_hit_grid_changes: Vec<(u64, bool)>,
    /// Animation groups keyed by unique group ID.
    pub animation_groups: HashMap<u64, AnimGroupState>,
    /// Counter for generating unique animation group IDs.
    pub next_anim_group_id: u64,
    /// Screen dimensions in UI coordinates.
    pub screen_width: f32,
    pub screen_height: f32,
    /// Action bar slots: slot (1-120) → spell ID.
    pub action_bars: HashMap<u32, u32>,
    /// Addon base paths for runtime on-demand loading (Blizzard UI + AddOns directories).
    pub addon_base_paths: Vec<PathBuf>,
    /// Current mouse position in UI coordinates (for ANCHOR_CURSOR tooltip positioning).
    pub mouse_position: Option<(f32, f32)>,
    /// Currently hovered frame ID (for IsMouseMotionFocus / GetMouseFocus).
    pub hovered_frame: Option<u64>,
    /// Simulated party members (empty = not in group).
    pub party_members: Vec<PartyMember>,
    /// Current target (None = no target).
    pub current_target: Option<TargetInfo>,
    /// Current focus target (None = no focus).
    pub current_focus: Option<TargetInfo>,
    /// Audio playback manager (None when no audio device or WOW_SIM_NO_SOUND=1).
    pub sound_manager: Option<SoundManager>,
    /// Player character name (randomly chosen on startup).
    pub player_name: String,
    /// Player current health.
    pub player_health: i32,
    /// Player maximum health.
    pub player_health_max: i32,
    /// Player class (1-based index matching CLASS_DATA in unit_api).
    pub player_class_index: i32,
    /// Player race (0-based index into RACE_DATA).
    pub player_race_index: usize,
    /// Rot damage intensity (index into ROT_DAMAGE_LEVELS).
    pub rot_damage_level: usize,
    /// Player buffs/debuffs (disabled by WOW_SIM_NO_BUFFS=1).
    pub player_buffs: Vec<AuraInfo>,
    /// Current framerate (FPS), updated by the app's FPS counter.
    pub fps: f32,
    /// Instant at which the UI started (used by GetTime and message timestamps).
    pub start_time: Instant,
    /// Active spell cast (None = not casting).
    pub casting: Option<CastingState>,
    /// Counter for generating unique cast IDs.
    pub next_cast_id: u32,
    /// Global Cooldown: (start_time, duration) in GetTime() seconds.
    pub gcd: Option<(f64, f64)>,
    /// Per-spell cooldowns: spell_id → SpellCooldownState.
    pub spell_cooldowns: HashMap<u32, SpellCooldownState>,
    /// Buttons registered via SetActionUIButton(button, action, cooldownFrame).
    /// Stores (frame_id, action_slot) pairs for engine-pushed state updates.
    pub action_ui_buttons: Vec<(u64, u32)>,
    /// What is currently held on the cursor (drag-and-drop).
    pub cursor_item: Option<CursorInfo>,
    /// Index of the addon currently being loaded (into `addons` vec).
    /// Set by the loader, read by CreateFrame to assign `owner_addon`.
    pub loading_addon_index: Option<u16>,
    /// Application-level frame metrics (total frame time for profiler ratios).
    pub app_frame_metrics: AppFrameMetrics,
    /// Talent tree interactive state (ranks, selections, currency mappings).
    pub talents: super::talent_state::TalentState,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            widgets: WidgetRegistry::default(),
            events: EventQueue::default(),
            scripts: ScriptRegistry::default(),
            console_output: Vec::new(),
            timers: VecDeque::new(),
            focused_frame_id: None,
            addons: Vec::new(),
            cvars: CVarStorage::new(),
            tooltips: HashMap::new(),
            simple_htmls: HashMap::new(),
            message_frames: HashMap::new(),
            on_update_frames: HashSet::new(),
            visible_on_update_cache: None,
            strata_buckets: None,
            pending_hit_grid_changes: Vec::new(),
            animation_groups: HashMap::new(),
            next_anim_group_id: 1,
            screen_width: 1600.0,
            screen_height: 1200.0,
            action_bars: default_action_bars(),
            addon_base_paths: Vec::new(),
            mouse_position: None,
            hovered_frame: None,
            party_members: default_party(),
            current_target: None,
            current_focus: None,
            sound_manager: None,
            player_name: random_player_name(),
            player_health: 100_000,
            player_health_max: 100_000,
            player_class_index: 2,  // Paladin
            player_race_index: 0,   // Human
            rot_damage_level: 0,    // Off
            player_buffs: default_player_buffs(),
            fps: 0.0,
            start_time: Instant::now(),
            casting: None,
            next_cast_id: 1,
            gcd: None,
            spell_cooldowns: HashMap::new(),
            action_ui_buttons: Vec::new(),
            cursor_item: None,
            loading_addon_index: None,
            app_frame_metrics: AppFrameMetrics::default(),
            talents: super::talent_state::TalentState::new(),
        }
    }
}

impl SimState {
    /// Return the per-strata buckets, building lazily if needed.
    pub fn get_strata_buckets(&mut self) -> Option<&Vec<Vec<u64>>> {
        if self.strata_buckets.is_none() {
            self.strata_buckets = Some(self.build_strata_buckets());
        }
        self.strata_buckets.as_ref()
    }

    /// Build per-strata ID buckets for visible frames only, sorted by render order.
    ///
    /// A frame is included if its "render alpha" > 0: either its own
    /// `effective_alpha > 0`, or (for button state textures with `visible=false`)
    /// its parent's `effective_alpha > 0`.
    fn build_strata_buckets(&mut self) -> Vec<Vec<u64>> {
        // Ensure effective_alpha is correct for all frames (handles direct
        // .visible = false assignments during initialization that bypass
        // set_frame_visible propagation).
        self.widgets.propagate_all_effective_alpha();
        self.widgets.propagate_all_effective_scale();
        use crate::iced_app::frame_collect::intra_strata_sort_key;
        use crate::widget::WidgetType;
        let mut buckets = vec![Vec::new(); crate::widget::FrameStrata::COUNT];
        for id in self.widgets.iter_ids() {
            let Some(f) = self.widgets.get(id) else { continue };
            // Visibility filter: skip frames with no render alpha.
            let render_alpha = if f.effective_alpha > 0.0 {
                f.effective_alpha
            } else {
                f.parent_id
                    .and_then(|pid| self.widgets.get(pid))
                    .map(|p| p.effective_alpha)
                    .unwrap_or(0.0)
            };
            if render_alpha <= 0.0 {
                continue;
            }
            let strata = if matches!(f.widget_type, WidgetType::Texture | WidgetType::FontString | WidgetType::Line) {
                f.parent_id
                    .and_then(|pid| self.widgets.get(pid))
                    .map(|p| p.frame_strata)
                    .unwrap_or(f.frame_strata)
            } else {
                f.frame_strata
            };
            buckets[strata.as_index()].push(id);
        }
        for bucket in &mut buckets {
            bucket.sort_by(|&a, &b| {
                match (self.widgets.get(a), self.widgets.get(b)) {
                    (Some(fa), Some(fb)) => intra_strata_sort_key(fa, a, &self.widgets).cmp(&intra_strata_sort_key(fb, b, &self.widgets)),
                    _ => a.cmp(&b),
                }
            });
        }
        buckets
    }

    /// Eagerly recompute layout rect for a frame and all its descendants.
    /// Called when layout-affecting properties change (anchors, size, scale, parent).
    /// Stores the computed rect on each Frame so the renderer can use it directly.
    pub fn invalidate_layout(&mut self, id: u64) {
        let sw = self.screen_width;
        let sh = self.screen_height;
        let mut cache = crate::iced_app::layout::LayoutCache::default();
        Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, &mut cache);
    }

    /// Like `invalidate_layout` but also recomputes sibling frames anchored to
    /// `id`. Uses the reverse anchor index for O(k) lookup where k = number of
    /// dependents. Called by SetWidth/SetHeight/SetSize/SetScale/SetAtlas so
    /// that cross-frame-anchored siblings (e.g. three-slice Center) update.
    pub fn invalidate_layout_with_dependents(&mut self, id: u64) {
        let sw = self.screen_width;
        let sh = self.screen_height;
        let mut cache = crate::iced_app::layout::LayoutCache::default();
        Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, &mut cache);
        Self::recompute_anchor_dependents(&mut self.widgets, id, sw, sh, &mut cache, 0);
    }

    fn recompute_layout_subtree(
        widgets: &mut crate::widget::WidgetRegistry,
        id: u64,
        screen_width: f32,
        screen_height: f32,
        cache: &mut crate::iced_app::layout::LayoutCache,
    ) {
        // Remove stale entry so compute_frame_rect_cached recomputes.
        cache.remove(&id);
        let rect = crate::iced_app::compute_frame_rect_cached(
            widgets, id, screen_width, screen_height, cache,
        ).rect;
        let children: Vec<u64> = widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        if let Some(f) = widgets.get_mut(id) {
            f.layout_rect = Some(rect);
        }
        widgets.mark_layout_resolved(id);
        for child_id in children {
            Self::recompute_layout_subtree(widgets, child_id, screen_width, screen_height, cache);
        }
    }

    /// Recompute frames anchored to `target_id` using the reverse index.
    ///
    /// O(k) where k = number of frames anchored to target_id.
    fn recompute_anchor_dependents(
        widgets: &mut crate::widget::WidgetRegistry,
        target_id: u64,
        sw: f32, sh: f32,
        cache: &mut crate::iced_app::layout::LayoutCache,
        _depth: u32,
    ) {
        let deps: Vec<u64> = widgets.get_anchor_dependents(target_id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        for dep_id in deps {
            Self::recompute_layout_subtree(widgets, dep_id, sw, sh, cache);
        }
    }

}

impl SimState {
    /// Ensure every frame has a layout_rect and clear rect_dirty flags.
    /// Computes missing rects using the same eager path as invalidate_layout.
    /// Called before quad rebuilds (acts as the "next frame" layout resolution).
    pub fn ensure_layout_rects(&mut self) {
        let pending = self.widgets.drain_pending_layout();
        if !pending.is_empty() {
            let sw = self.screen_width;
            let sh = self.screen_height;
            let mut cache = crate::iced_app::layout::LayoutCache::default();
            for id in pending {
                if self.widgets.get(id).is_some_and(|f| f.layout_rect.is_none()) {
                    Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, &mut cache);
                }
            }
        }
        // Clear rect_dirty flags using the tracked set.
        self.widgets.drain_rect_dirty();
    }

    /// Force layout resolution for a single frame, clearing its rect_dirty flag.
    /// Called by GetSize/GetWidth/GetHeight/IsRectValid to match WoW behavior
    /// where those methods force immediate rect resolution within the same frame.
    pub fn resolve_rect_if_dirty(&mut self, id: u64) {
        if !self.widgets.is_rect_dirty(id) {
            return;
        }
        self.invalidate_layout(id);
        // invalidate_layout → recompute_layout_subtree already clears rect_dirty
        // on the frame and all descendants via clear_rect_dirty.
    }

    /// Set a frame's visibility and eagerly propagate effective_alpha.
    /// Surgically updates strata_buckets: inserts on show, removes on hide.
    pub fn set_frame_visible(&mut self, id: u64, visible: bool) {
        let was_visible = self.widgets.get(id).map(|f| f.visible).unwrap_or(false);
        self.widgets.set_visible(id, visible);
        if was_visible == visible {
            return;
        }
        // Toplevel frames are raised above siblings when shown (WoW behavior).
        if visible {
            let is_toplevel = self.widgets.get(id).map(|f| f.toplevel).unwrap_or(false);
            if is_toplevel {
                self.raise_frame(id);
            }
        }
        self.update_on_update_cache(id, visible);
        // Propagate effective_alpha: look up parent's effective_alpha.
        let parent_eff = self.widgets.get(id)
            .and_then(|f| f.parent_id)
            .and_then(|pid| self.widgets.get(pid))
            .map(|p| p.effective_alpha)
            .unwrap_or(1.0);
        if !visible {
            // Hide: remove subtree from buckets BEFORE propagating alpha to 0.
            self.remove_subtree_from_buckets(id);
        }
        self.widgets.propagate_effective_alpha(id, parent_eff);
        if visible {
            // Show: insert newly-visible frames AFTER propagating alpha.
            self.insert_subtree_into_buckets(id);
        }
        // Record for incremental HitGrid update (applied by App after Lua runs).
        self.pending_hit_grid_changes.push((id, visible));
    }

    /// Remove a frame and all its descendants from strata_buckets.
    fn remove_subtree_from_buckets(&mut self, root_id: u64) {
        let Some(buckets) = self.strata_buckets.as_mut() else { return };
        // Collect all IDs in the subtree.
        let mut subtree = std::collections::HashSet::new();
        let mut queue = vec![root_id];
        while let Some(fid) = queue.pop() {
            subtree.insert(fid);
            if let Some(f) = self.widgets.get(fid) {
                queue.extend(f.children.iter().copied());
            }
        }
        for bucket in buckets.iter_mut() {
            bucket.retain(|id| !subtree.contains(id));
        }
    }

    /// Insert newly-visible frames from a subtree into strata_buckets.
    ///
    /// Walks all descendants and inserts those with render_alpha > 0
    /// (own effective_alpha, or parent's for button state textures).
    fn insert_subtree_into_buckets(&mut self, root_id: u64) {
        let Some(buckets) = self.strata_buckets.as_mut() else { return };
        use crate::iced_app::frame_collect::intra_strata_sort_key;
        use crate::widget::WidgetType;
        // Walk all descendants.
        let mut queue = vec![root_id];
        while let Some(fid) = queue.pop() {
            let Some(f) = self.widgets.get(fid) else { continue };
            queue.extend(f.children.iter().copied());
            let render_alpha = if f.effective_alpha > 0.0 {
                f.effective_alpha
            } else {
                f.parent_id
                    .and_then(|pid| self.widgets.get(pid))
                    .map(|p| p.effective_alpha)
                    .unwrap_or(0.0)
            };
            if render_alpha <= 0.0 {
                continue;
            }
            let strata = if matches!(f.widget_type, WidgetType::Texture | WidgetType::FontString | WidgetType::Line) {
                f.parent_id
                    .and_then(|pid| self.widgets.get(pid))
                    .map(|p| p.frame_strata)
                    .unwrap_or(f.frame_strata)
            } else {
                f.frame_strata
            };
            let key = intra_strata_sort_key(f, fid, &self.widgets);
            let bucket = &mut buckets[strata.as_index()];
            let pos = bucket.partition_point(|&existing_id| {
                self.widgets.get(existing_id)
                    .map(|ef| intra_strata_sort_key(ef, existing_id, &self.widgets))
                    .unwrap_or_default()
                    < key
            });
            bucket.insert(pos, fid);
        }
    }

    /// Raise a frame above all siblings in the same strata.
    ///
    /// Finds the maximum frame_level among sibling frames (same parent, same
    /// strata) and sets this frame's level to max + 1. Propagates the new level
    /// to all descendants.
    pub fn raise_frame(&mut self, id: u64) {
        let (parent_id, strata) = match self.widgets.get(id) {
            Some(f) => (f.parent_id, f.frame_strata),
            None => return,
        };
        // Find max level among siblings in the same strata.
        let max_sibling_level = self.max_sibling_level(id, parent_id, strata);
        let current_level = self.widgets.get(id).map(|f| f.frame_level).unwrap_or(0);
        if current_level > max_sibling_level {
            return; // Already on top
        }
        let new_level = max_sibling_level + 1;
        if let Some(f) = self.widgets.get_mut_visual(id) {
            f.frame_level = new_level;
        }
        crate::lua_api::frame::propagate_strata_level_pub(
            &mut self.widgets, id,
        );
        // Invalidate strata buckets since level changed (affects sort order).
        self.strata_buckets = None;
    }

    /// Find the maximum frame_level among siblings of `id` in the given strata.
    fn max_sibling_level(&self, id: u64, parent_id: Option<u64>, strata: crate::widget::FrameStrata) -> i32 {
        let sibling_ids: Vec<u64> = if let Some(pid) = parent_id {
            self.widgets.get(pid)
                .map(|p| p.children.clone())
                .unwrap_or_default()
        } else {
            // Root frames: all frames with no parent
            self.widgets.iter_ids()
                .filter(|&fid| self.widgets.get(fid).map(|f| f.parent_id.is_none()).unwrap_or(false))
                .collect()
        };
        sibling_ids.iter()
            .filter(|&&sid| sid != id)
            .filter_map(|&sid| self.widgets.get(sid))
            .filter(|f| f.frame_strata == strata)
            .map(|f| f.frame_level)
            .max()
            .unwrap_or(0)
    }

    fn update_on_update_cache(&mut self, id: u64, visible: bool) {
        let Some(mut cache) = self.visible_on_update_cache.take() else {
            return;
        };
        if visible {
            self.add_on_update_descendants(id, &mut cache);
        } else {
            self.remove_on_update_descendants(id, &mut cache);
        }
        self.visible_on_update_cache = Some(cache);
    }

    /// Add `id` and its descendants to cache if they have OnUpdate and are ancestor-visible.
    fn add_on_update_descendants(&self, id: u64, cache: &mut Vec<u64>) {
        if self.on_update_frames.contains(&id) && self.widgets.is_ancestor_visible(id) {
            if !cache.contains(&id) {
                cache.push(id);
            }
        }
        let children: Vec<u64> = self.widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        for child_id in children {
            if self.widgets.get(child_id).is_some_and(|f| f.visible) {
                self.add_on_update_descendants(child_id, cache);
            }
        }
    }

    /// Remove `id` and all its descendants from cache (hidden ancestor = all hidden).
    fn remove_on_update_descendants(&self, id: u64, cache: &mut Vec<u64>) {
        cache.retain(|&cached_id| cached_id != id);
        let children: Vec<u64> = self.widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        for child_id in children {
            self.remove_on_update_descendants(child_id, cache);
        }
    }

}

