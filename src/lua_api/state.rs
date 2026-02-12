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
    CLASS_LABELS, RACE_DATA, ROT_DAMAGE_LEVELS,
    build_target_info, tick_party_health,
};
pub use super::game_data::SpellCooldownState;
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
    /// Cached ancestor-visible IDs with effective alpha. Built lazily on first
    /// `draw()`, then updated eagerly by `set_frame_visible`.
    pub ancestor_visible_cache: Option<HashMap<u64, f32>>,
    /// Per-strata buckets of visible frame IDs. Index = FrameStrata as usize.
    /// Built alongside ancestor_visible_cache, updated eagerly by `set_frame_visible`.
    pub strata_buckets: Option<Vec<Vec<u64>>>,
    /// Persistent layout rect cache. Built lazily on first `draw()`, entries
    /// invalidated eagerly when layout-affecting properties change (anchors,
    /// size, scale, parent). Frames not in cache are recomputed on next rebuild.
    pub layout_rect_cache: Option<crate::iced_app::layout::LayoutCache>,
    /// Cached render and hit-test lists from `collect_sorted_frames`.
    /// Skips the per-frame collection pass when only content (not layout/visibility) changes.
    pub cached_render_list: Option<crate::iced_app::frame_collect::CollectedFrames>,
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
    /// Index of the addon currently being loaded (into `addons` vec).
    /// Set by the loader, read by CreateFrame to assign `owner_addon`.
    pub loading_addon_index: Option<u16>,
    /// Application-level frame metrics (total frame time for profiler ratios).
    pub app_frame_metrics: AppFrameMetrics,
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
            ancestor_visible_cache: None,
            strata_buckets: None,
            layout_rect_cache: None,
            cached_render_list: None,
            animation_groups: HashMap::new(),
            next_anim_group_id: 1,
            screen_width: 1024.0,
            screen_height: 768.0,
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
            rot_damage_level: 0,    // Light
            player_buffs: default_player_buffs(),
            fps: 0.0,
            start_time: Instant::now(),
            casting: None,
            next_cast_id: 1,
            gcd: None,
            spell_cooldowns: HashMap::new(),
            action_ui_buttons: Vec::new(),
            loading_addon_index: None,
            app_frame_metrics: AppFrameMetrics::default(),
        }
    }
}

impl SimState {
    /// Return the cached ancestor-visible map, building it on first call.
    /// Also builds strata_buckets as a side effect.
    pub fn get_ancestor_visible(&mut self) -> &HashMap<u64, f32> {
        if self.ancestor_visible_cache.is_none() {
            let visible = crate::iced_app::frame_collect::collect_ancestor_visible_ids(&self.widgets);
            let buckets = self.build_strata_buckets(&visible);
            self.strata_buckets = Some(buckets);
            self.ancestor_visible_cache = Some(visible);
            self.cached_render_list = None;
        }
        self.ancestor_visible_cache.as_ref().unwrap()
    }

    /// Return the per-strata buckets (requires ancestor_visible to have been built).
    pub fn get_strata_buckets(&self) -> Option<&Vec<Vec<u64>>> {
        self.strata_buckets.as_ref()
    }

    /// Build per-strata ID buckets from the ancestor-visible map, sorted by render order.
    fn build_strata_buckets(&self, visible: &HashMap<u64, f32>) -> Vec<Vec<u64>> {
        use crate::iced_app::frame_collect::intra_strata_sort_key;
        let mut buckets = vec![Vec::new(); crate::widget::FrameStrata::COUNT];
        for &id in visible.keys() {
            if let Some(f) = self.widgets.get(id) {
                buckets[f.frame_strata.as_index()].push(id);
            }
        }
        for bucket in &mut buckets {
            bucket.sort_by(|&a, &b| {
                match (self.widgets.get(a), self.widgets.get(b)) {
                    (Some(fa), Some(fb)) => intra_strata_sort_key(fa, a).cmp(&intra_strata_sort_key(fb, b)),
                    _ => a.cmp(&b),
                }
            });
        }
        buckets
    }

    /// Take the persistent layout cache for use during quad rebuild.
    /// Returns the existing cache (or empty) for the caller to populate.
    /// Caller must return it via `set_layout_cache` after the rebuild.
    pub fn take_layout_cache(&mut self) -> crate::iced_app::layout::LayoutCache {
        self.layout_rect_cache.take().unwrap_or_default()
    }

    /// Store the layout cache back after a quad rebuild, propagating rects to frames.
    pub fn set_layout_cache(&mut self, cache: crate::iced_app::layout::LayoutCache) {
        for (&id, &cached) in &cache {
            if let Some(f) = self.widgets.get_mut_silent(id) {
                f.layout_rect = Some(cached.rect);
            }
        }
        self.layout_rect_cache = Some(cache);
    }

    /// Eagerly recompute layout rect for a frame and all its descendants.
    /// Called when layout-affecting properties change (anchors, size, scale, parent).
    /// Stores the computed rect on each Frame so the renderer can use it directly.
    pub fn invalidate_layout(&mut self, id: u64) {
        self.cached_render_list = None;
        let sw = self.screen_width;
        let sh = self.screen_height;
        if let Some(cache) = self.layout_rect_cache.as_mut() {
            Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, cache);
        } else {
            let mut cache = crate::iced_app::layout::LayoutCache::default();
            Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, &mut cache);
        }
    }

    /// Like `invalidate_layout` but also recomputes sibling frames anchored to
    /// `id`. Use this at runtime when a frame moves (e.g. cast bar Spark) and
    /// siblings anchored to it need updating. Avoid during bulk loading — it
    /// scans all widgets to find dependents.
    pub fn invalidate_layout_with_dependents(&mut self, id: u64) {
        self.cached_render_list = None;
        let sw = self.screen_width;
        let sh = self.screen_height;
        if let Some(cache) = self.layout_rect_cache.as_mut() {
            Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, cache);
            Self::recompute_anchor_dependents(&mut self.widgets, id, sw, sh, cache, 0);
        } else {
            let mut cache = crate::iced_app::layout::LayoutCache::default();
            Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, &mut cache);
            Self::recompute_anchor_dependents(&mut self.widgets, id, sw, sh, &mut cache, 0);
        }
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
        if let Some(f) = widgets.get_mut_silent(id) {
            f.layout_rect = Some(rect);
        }
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
        let sw = self.screen_width;
        let sh = self.screen_height;
        let mut cache = self.layout_rect_cache.take().unwrap_or_default();
        let missing: Vec<u64> = self.widgets.iter_ids()
            .filter(|&id| self.widgets.get(id).is_some_and(|f| f.layout_rect.is_none()))
            .collect();
        for id in missing {
            Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, &mut cache);
        }
        // Clear rect_dirty on all frames that have anchors (layout resolved).
        let dirty_ids: Vec<u64> = self.widgets.iter_ids()
            .filter(|&id| self.widgets.get(id).is_some_and(|f| f.rect_dirty))
            .collect();
        for id in dirty_ids {
            if let Some(f) = self.widgets.get_mut_silent(id) {
                f.rect_dirty = false;
            }
        }
        self.layout_rect_cache = Some(cache);
    }

    /// Force layout resolution for a single frame, clearing its rect_dirty flag.
    /// Called by GetSize/GetWidth/GetHeight to match WoW behavior where those
    /// methods force immediate rect resolution within the same frame.
    pub fn resolve_rect_if_dirty(&mut self, id: u64) {
        let is_dirty = self.widgets.get(id).is_some_and(|f| f.rect_dirty);
        if !is_dirty {
            return;
        }
        self.invalidate_layout(id);
        // Clear dirty on this frame and descendants.
        Self::clear_rect_dirty_subtree(&mut self.widgets, id);
    }

    fn clear_rect_dirty_subtree(widgets: &mut crate::widget::WidgetRegistry, id: u64) {
        let children: Vec<u64> = widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        if let Some(f) = widgets.get_mut_silent(id) {
            f.rect_dirty = false;
        }
        for child_id in children {
            Self::clear_rect_dirty_subtree(widgets, child_id);
        }
    }

    /// Set a frame's visibility and eagerly update both caches.
    ///
    /// When hiding: remove the frame and all descendants from both caches.
    /// When showing: add the frame and any descendants that are now
    /// ancestor-visible.
    pub fn set_frame_visible(&mut self, id: u64, visible: bool) {
        let was_visible = self.widgets.get(id).map(|f| f.visible).unwrap_or(false);
        self.widgets.set_visible(id, visible);
        if was_visible == visible {
            return;
        }
        self.update_on_update_cache(id, visible);
        self.update_ancestor_visible_cache(id, visible);
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

    fn update_ancestor_visible_cache(&mut self, id: u64, visible: bool) {
        self.cached_render_list = None;
        let Some(mut cache) = self.ancestor_visible_cache.take() else {
            return;
        };
        let mut buckets = self.strata_buckets.take();
        if visible {
            // Only add to cache if parent is ancestor-visible (or frame is a root).
            // This prevents hidden-ancestor frames from leaking into the render cache
            // (e.g. OverrideActionBar children becoming visible while the bar is hidden).
            let parent_visible = self
                .widgets
                .get(id)
                .and_then(|f| f.parent_id)
                .map(|pid| cache.contains_key(&pid))
                .unwrap_or(true); // root frames have no parent → always eligible
            if parent_visible {
                self.add_visible_descendants(id, &mut cache, &mut buckets);
            }
        } else {
            self.remove_visible_descendants(id, &mut cache, &mut buckets);
        }
        self.ancestor_visible_cache = Some(cache);
        self.strata_buckets = buckets;
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

    /// Insert `id` into the correct sorted position in its strata bucket.
    /// Skips insertion if the id is already present (avoids duplicate rendering).
    fn insert_into_strata_bucket(&self, f: &crate::widget::Frame, id: u64, buckets: &mut Option<Vec<Vec<u64>>>) {
        use crate::iced_app::frame_collect::intra_strata_sort_key;
        let Some(b) = buckets.as_mut() else { return };
        let bucket = &mut b[f.frame_strata.as_index()];
        if bucket.contains(&id) {
            return;
        }
        let key = intra_strata_sort_key(f, id);
        let pos = bucket.binary_search_by(|&other_id| {
            self.widgets.get(other_id)
                .map(|o| intra_strata_sort_key(o, other_id).cmp(&key))
                .unwrap_or(std::cmp::Ordering::Less)
        }).unwrap_or_else(|p| p);
        bucket.insert(pos, id);
    }

    /// Add `id` and visible descendants to the ancestor-visible cache with alpha.
    ///
    /// Caller (`update_ancestor_visible_cache`) must ensure the parent is already
    /// in the cache before calling this. The `unwrap_or(1.0)` for parent alpha
    /// handles root frames (no parent) — non-root frames with missing parents
    /// should never reach here.
    fn add_visible_descendants(
        &self, id: u64, cache: &mut HashMap<u64, f32>,
        buckets: &mut Option<Vec<Vec<u64>>>,
    ) {
        let Some(f) = self.widgets.get(id) else { return };
        let parent_alpha = f
            .parent_id
            .and_then(|pid| cache.get(&pid).copied())
            .unwrap_or(1.0);
        if !f.visible {
            if is_button_state_texture(f, id, &self.widgets) {
                cache.insert(id, parent_alpha * f.alpha);
                self.insert_into_strata_bucket(f, id, buckets);
            }
            return;
        }
        let eff = parent_alpha * f.alpha;
        cache.insert(id, eff);
        self.insert_into_strata_bucket(f, id, buckets);
        if f.widget_type == crate::widget::WidgetType::GameTooltip {
            return;
        }
        let children: Vec<u64> = f.children.clone();
        for child_id in children {
            self.add_visible_descendants(child_id, cache, buckets);
        }
    }

    /// Remove `id` and all descendants from ancestor-visible cache.
    fn remove_visible_descendants(
        &self, id: u64, cache: &mut HashMap<u64, f32>,
        buckets: &mut Option<Vec<Vec<u64>>>,
    ) {
        if cache.remove(&id).is_some() {
            if let Some(b) = buckets.as_mut() {
                if let Some(f) = self.widgets.get(id) {
                    let bucket = &mut b[f.frame_strata.as_index()];
                    if let Some(pos) = bucket.iter().position(|&x| x == id) {
                        bucket.remove(pos); // preserve sorted order
                    }
                }
            }
        }
        let children: Vec<u64> = self.widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        for child_id in children {
            self.remove_visible_descendants(child_id, cache, buckets);
        }
    }
}

/// Check if a frame is a button state texture (NormalTexture, PushedTexture, etc.).
fn is_button_state_texture(
    f: &crate::widget::Frame,
    id: u64,
    registry: &crate::widget::WidgetRegistry,
) -> bool {
    use crate::widget::WidgetType;
    if !matches!(f.widget_type, WidgetType::Texture) {
        return false;
    }
    let Some(parent_id) = f.parent_id else { return false };
    let Some(parent) = registry.get(parent_id) else { return false };
    if !matches!(parent.widget_type, WidgetType::Button | WidgetType::CheckButton) {
        return false;
    }
    ["NormalTexture", "PushedTexture", "HighlightTexture", "DisabledTexture"]
        .iter()
        .any(|key| parent.children_keys.get(*key) == Some(&id))
}
