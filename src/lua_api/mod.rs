//! Lua API bindings implementing WoW's addon API.

mod frame_methods;
mod globals;

use crate::event::{EventQueue, ScriptRegistry};
use crate::widget::WidgetRegistry;
use crate::Result;
use mlua::{Lua, MultiValue, RegistryKey, Value};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static NEXT_TIMER_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a unique timer ID.
fn next_timer_id() -> u64 {
    NEXT_TIMER_ID.fetch_add(1, Ordering::Relaxed)
}

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
}

/// The WoW Lua environment.
pub struct WowLuaEnv {
    lua: Lua,
    state: Rc<RefCell<SimState>>,
}

/// Shared simulator state accessible from Lua.
#[derive(Default)]
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
}

impl WowLuaEnv {
    /// Create a new WoW Lua environment with the API initialized.
    pub fn new() -> Result<Self> {
        // Use unsafe_new to get full standard library including debug
        // This is safe for our simulator since we control the Lua code
        let lua = unsafe { Lua::unsafe_new() };
        let state = Rc::new(RefCell::new(SimState::default()));

        // Create UIParent (the root frame) - must have screen dimensions for layout
        {
            let mut s = state.borrow_mut();
            let mut ui_parent = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("UIParent".to_string()),
                None,
            );
            // Set UIParent to screen size (reference coordinate system)
            ui_parent.width = 500.0;
            ui_parent.height = 375.0;
            let ui_parent_id = ui_parent.id;
            s.widgets.register(ui_parent);

            // Create Minimap (built-in UI element)
            let minimap = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("Minimap".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(minimap);

            // Create WorldFrame (3D world rendering area - used by HUD elements)
            let mut world_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("WorldFrame".to_string()),
                None, // No parent - it's at the same level as UIParent
            );
            world_frame.width = 500.0;
            world_frame.height = 375.0;
            s.widgets.register(world_frame);

            // Create DEFAULT_CHAT_FRAME (the main chat window)
            let mut chat_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::MessageFrame,
                Some("DEFAULT_CHAT_FRAME".to_string()),
                Some(ui_parent_id),
            );
            chat_frame.width = 430.0;
            chat_frame.height = 120.0;
            s.widgets.register(chat_frame);

            // Create ChatFrame1 (same as DEFAULT_CHAT_FRAME in WoW)
            let mut chat_frame1 = crate::widget::Frame::new(
                crate::widget::WidgetType::MessageFrame,
                Some("ChatFrame1".to_string()),
                Some(ui_parent_id),
            );
            chat_frame1.width = 430.0;
            chat_frame1.height = 120.0;
            s.widgets.register(chat_frame1);

            // Create EventToastManagerFrame (UI for event toasts/notifications)
            let mut event_toast_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("EventToastManagerFrame".to_string()),
                Some(ui_parent_id),
            );
            event_toast_frame.width = 300.0;
            event_toast_frame.height = 100.0;
            s.widgets.register(event_toast_frame);

            // Create EditModeManagerFrame (Edit Mode UI manager)
            let mut edit_mode_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("EditModeManagerFrame".to_string()),
                Some(ui_parent_id),
            );
            edit_mode_frame.width = 400.0;
            edit_mode_frame.height = 300.0;
            s.widgets.register(edit_mode_frame);

            // Create RolePollPopup (role selection popup for groups)
            let mut role_poll_popup = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("RolePollPopup".to_string()),
                Some(ui_parent_id),
            );
            role_poll_popup.width = 200.0;
            role_poll_popup.height = 150.0;
            s.widgets.register(role_poll_popup);

            // Create TimerTracker (displays dungeon/raid instance timers)
            let mut timer_tracker = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("TimerTracker".to_string()),
                Some(ui_parent_id),
            );
            timer_tracker.width = 200.0;
            timer_tracker.height = 50.0;
            s.widgets.register(timer_tracker);

            // Create WorldMapFrame (world map display frame)
            let mut world_map_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("WorldMapFrame".to_string()),
                Some(ui_parent_id),
            );
            world_map_frame.width = 1024.0;
            world_map_frame.height = 768.0;
            world_map_frame.visible = false; // Hidden by default
            let world_map_frame_id = s.widgets.register(world_map_frame);

            // Create WorldMapFrame.BorderFrame
            let mut world_map_border_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("WorldMapBorderFrame".to_string()),
                Some(world_map_frame_id),
            );
            world_map_border_frame.width = 1024.0;
            world_map_border_frame.height = 768.0;
            let world_map_border_frame_id = s.widgets.register(world_map_border_frame);

            // Create WorldMapFrame.BorderFrame.MaximizeMinimizeFrame
            let mut max_min_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("WorldMapMaximizeMinimizeFrame".to_string()),
                Some(world_map_border_frame_id),
            );
            max_min_frame.width = 32.0;
            max_min_frame.height = 32.0;
            let max_min_frame_id = s.widgets.register(max_min_frame);

            // Create WorldMapFrame.BorderFrame.MaximizeMinimizeFrame.MaximizeButton
            let max_button = crate::widget::Frame::new(
                crate::widget::WidgetType::Button,
                Some("WorldMapMaximizeButton".to_string()),
                Some(max_min_frame_id),
            );
            let max_button_id = s.widgets.register(max_button);

            // Create WorldMapFrame.BorderFrame.MaximizeMinimizeFrame.MinimizeButton
            let min_button = crate::widget::Frame::new(
                crate::widget::WidgetType::Button,
                Some("WorldMapMinimizeButton".to_string()),
                Some(max_min_frame_id),
            );
            let min_button_id = s.widgets.register(min_button);

            // Create WorldMapFrame.ScrollContainer
            let scroll_container = crate::widget::Frame::new(
                crate::widget::WidgetType::ScrollFrame,
                Some("WorldMapScrollContainer".to_string()),
                Some(world_map_frame_id),
            );
            let scroll_container_id = s.widgets.register(scroll_container);

            // Set up children_keys for WorldMapFrame hierarchy
            if let Some(wm_frame) = s.widgets.get_mut(world_map_frame_id) {
                wm_frame
                    .children_keys
                    .insert("BorderFrame".to_string(), world_map_border_frame_id);
                wm_frame
                    .children_keys
                    .insert("ScrollContainer".to_string(), scroll_container_id);
            }
            if let Some(border_frame) = s.widgets.get_mut(world_map_border_frame_id) {
                border_frame
                    .children_keys
                    .insert("MaximizeMinimizeFrame".to_string(), max_min_frame_id);
            }
            if let Some(mm_frame) = s.widgets.get_mut(max_min_frame_id) {
                mm_frame
                    .children_keys
                    .insert("MaximizeButton".to_string(), max_button_id);
                mm_frame
                    .children_keys
                    .insert("MinimizeButton".to_string(), min_button_id);
            }

            // Create PlayerFrame (player unit frame)
            let mut player_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("PlayerFrame".to_string()),
                Some(ui_parent_id),
            );
            player_frame.width = 175.0;
            player_frame.height = 76.0;
            let player_frame_id = s.widgets.register(player_frame);

            // Create PlayerFrame.PlayerFrameContent
            let mut player_frame_content = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("PlayerFrameContent".to_string()),
                Some(player_frame_id),
            );
            player_frame_content.width = 175.0;
            player_frame_content.height = 76.0;
            let player_frame_content_id = s.widgets.register(player_frame_content);

            // Create PlayerFrame.PlayerFrameContent.PlayerFrameContentMain
            let mut player_frame_content_main = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("PlayerFrameContentMain".to_string()),
                Some(player_frame_content_id),
            );
            player_frame_content_main.width = 175.0;
            player_frame_content_main.height = 76.0;
            let player_frame_content_main_id = s.widgets.register(player_frame_content_main);

            // Create HealthBarsContainer
            let mut health_bars_container = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("PlayerFrameHealthBarsContainer".to_string()),
                Some(player_frame_content_main_id),
            );
            health_bars_container.width = 120.0;
            health_bars_container.height = 20.0;
            let health_bars_container_id = s.widgets.register(health_bars_container);

            // Create HealthBar
            let mut health_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("PlayerFrameHealthBar".to_string()),
                Some(health_bars_container_id),
            );
            health_bar.width = 120.0;
            health_bar.height = 20.0;
            let health_bar_id = s.widgets.register(health_bar);

            // Create ManaBarArea
            let mut mana_bar_area = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("PlayerFrameManaBarArea".to_string()),
                Some(player_frame_content_main_id),
            );
            mana_bar_area.width = 120.0;
            mana_bar_area.height = 12.0;
            let mana_bar_area_id = s.widgets.register(mana_bar_area);

            // Create ManaBar
            let mut mana_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("PlayerFrameManaBar".to_string()),
                Some(mana_bar_area_id),
            );
            mana_bar.width = 120.0;
            mana_bar.height = 12.0;
            let mana_bar_id = s.widgets.register(mana_bar);

            // Create text children for health bar and mana bar
            let health_left_text = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString, None, Some(health_bar_id));
            let health_left_text_id = s.widgets.register(health_left_text);

            let health_right_text = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString, None, Some(health_bar_id));
            let health_right_text_id = s.widgets.register(health_right_text);

            let health_text_string = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString, None, Some(health_bar_id));
            let health_text_string_id = s.widgets.register(health_text_string);

            let mana_left_text = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString, None, Some(mana_bar_id));
            let mana_left_text_id = s.widgets.register(mana_left_text);

            let mana_right_text = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString, None, Some(mana_bar_id));
            let mana_right_text_id = s.widgets.register(mana_right_text);

            let mana_bar_text = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString, None, Some(mana_bar_id));
            let mana_bar_text_id = s.widgets.register(mana_bar_text);

            // Set up children_keys for the hierarchy
            if let Some(pf) = s.widgets.get_mut(player_frame_id) {
                pf.children_keys.insert("PlayerFrameContent".to_string(), player_frame_content_id);
            }
            if let Some(pfc) = s.widgets.get_mut(player_frame_content_id) {
                pfc.children_keys.insert("PlayerFrameContentMain".to_string(), player_frame_content_main_id);
            }
            if let Some(pfcm) = s.widgets.get_mut(player_frame_content_main_id) {
                pfcm.children_keys.insert("HealthBarsContainer".to_string(), health_bars_container_id);
                pfcm.children_keys.insert("ManaBarArea".to_string(), mana_bar_area_id);
            }
            if let Some(hbc) = s.widgets.get_mut(health_bars_container_id) {
                hbc.children_keys.insert("HealthBar".to_string(), health_bar_id);
            }
            if let Some(mba) = s.widgets.get_mut(mana_bar_area_id) {
                mba.children_keys.insert("ManaBar".to_string(), mana_bar_id);
            }
            // Add text children to health bar
            if let Some(hb) = s.widgets.get_mut(health_bar_id) {
                hb.children_keys.insert("LeftText".to_string(), health_left_text_id);
                hb.children_keys.insert("RightText".to_string(), health_right_text_id);
                hb.children_keys.insert("TextString".to_string(), health_text_string_id);
            }
            // Add text children to mana bar
            if let Some(mb) = s.widgets.get_mut(mana_bar_id) {
                mb.children_keys.insert("LeftText".to_string(), mana_left_text_id);
                mb.children_keys.insert("RightText".to_string(), mana_right_text_id);
                mb.children_keys.insert("ManaBarText".to_string(), mana_bar_text_id);
            }

            // Create TargetFrame (target unit frame)
            let mut target_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("TargetFrame".to_string()),
                Some(ui_parent_id),
            );
            target_frame.width = 175.0;
            target_frame.height = 76.0;
            let target_frame_id = s.widgets.register(target_frame);

            // Create TargetFrame.TargetFrameContent hierarchy (similar to PlayerFrame)
            let mut target_frame_content = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("TargetFrameContent".to_string()),
                Some(target_frame_id),
            );
            target_frame_content.width = 175.0;
            target_frame_content.height = 76.0;
            let target_frame_content_id = s.widgets.register(target_frame_content);

            let mut target_frame_content_main = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("TargetFrameContentMain".to_string()),
                Some(target_frame_content_id),
            );
            target_frame_content_main.width = 175.0;
            target_frame_content_main.height = 76.0;
            let target_frame_content_main_id = s.widgets.register(target_frame_content_main);

            let mut target_health_bars_container = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("TargetFrameHealthBarsContainer".to_string()),
                Some(target_frame_content_main_id),
            );
            target_health_bars_container.width = 120.0;
            target_health_bars_container.height = 20.0;
            let target_health_bars_container_id = s.widgets.register(target_health_bars_container);

            let mut target_health_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("TargetFrameHealthBar".to_string()),
                Some(target_health_bars_container_id),
            );
            target_health_bar.width = 120.0;
            target_health_bar.height = 20.0;
            let target_health_bar_id = s.widgets.register(target_health_bar);

            let mut target_mana_bar_area = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("TargetFrameManaBarArea".to_string()),
                Some(target_frame_content_main_id),
            );
            target_mana_bar_area.width = 120.0;
            target_mana_bar_area.height = 12.0;
            let target_mana_bar_area_id = s.widgets.register(target_mana_bar_area);

            let mut target_mana_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("TargetFrameManaBar".to_string()),
                Some(target_mana_bar_area_id),
            );
            target_mana_bar.width = 120.0;
            target_mana_bar.height = 12.0;
            let target_mana_bar_id = s.widgets.register(target_mana_bar);

            // Create totFrame (target-of-target frame) for TargetFrame
            let mut target_tot_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("TargetFrameToTFrame".to_string()),
                Some(target_frame_id),
            );
            target_tot_frame.width = 80.0;
            target_tot_frame.height = 30.0;
            let target_tot_frame_id = s.widgets.register(target_tot_frame);

            let target_tot_health_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("TargetFrameToTHealthBar".to_string()),
                Some(target_tot_frame_id),
            );
            let target_tot_health_bar_id = s.widgets.register(target_tot_health_bar);

            // Set up children_keys for TargetFrame hierarchy
            if let Some(tf) = s.widgets.get_mut(target_frame_id) {
                tf.children_keys.insert("TargetFrameContent".to_string(), target_frame_content_id);
                tf.children_keys.insert("totFrame".to_string(), target_tot_frame_id);
            }
            if let Some(tfc) = s.widgets.get_mut(target_frame_content_id) {
                tfc.children_keys.insert("TargetFrameContentMain".to_string(), target_frame_content_main_id);
            }
            if let Some(tfcm) = s.widgets.get_mut(target_frame_content_main_id) {
                tfcm.children_keys.insert("HealthBarsContainer".to_string(), target_health_bars_container_id);
                tfcm.children_keys.insert("ManaBarArea".to_string(), target_mana_bar_area_id);
                // Also add ManaBar directly on ContentMain (some addons access it this way)
                tfcm.children_keys.insert("ManaBar".to_string(), target_mana_bar_id);
            }
            if let Some(hbc) = s.widgets.get_mut(target_health_bars_container_id) {
                hbc.children_keys.insert("HealthBar".to_string(), target_health_bar_id);
            }
            if let Some(mba) = s.widgets.get_mut(target_mana_bar_area_id) {
                mba.children_keys.insert("ManaBar".to_string(), target_mana_bar_id);
            }
            if let Some(tot) = s.widgets.get_mut(target_tot_frame_id) {
                tot.children_keys.insert("HealthBar".to_string(), target_tot_health_bar_id);
            }

            // Create FocusFrame (focus target unit frame)
            let mut focus_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("FocusFrame".to_string()),
                Some(ui_parent_id),
            );
            focus_frame.width = 175.0;
            focus_frame.height = 76.0;
            let focus_frame_id = s.widgets.register(focus_frame);

            // Create FocusFrame.TargetFrameContent hierarchy (yes, it's confusingly named TargetFrameContent)
            let mut focus_frame_content = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("FocusFrameContent".to_string()),
                Some(focus_frame_id),
            );
            focus_frame_content.width = 175.0;
            focus_frame_content.height = 76.0;
            let focus_frame_content_id = s.widgets.register(focus_frame_content);

            let mut focus_frame_content_main = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("FocusFrameContentMain".to_string()),
                Some(focus_frame_content_id),
            );
            focus_frame_content_main.width = 175.0;
            focus_frame_content_main.height = 76.0;
            let focus_frame_content_main_id = s.widgets.register(focus_frame_content_main);

            let mut focus_health_bars_container = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("FocusFrameHealthBarsContainer".to_string()),
                Some(focus_frame_content_main_id),
            );
            focus_health_bars_container.width = 120.0;
            focus_health_bars_container.height = 20.0;
            let focus_health_bars_container_id = s.widgets.register(focus_health_bars_container);

            let mut focus_health_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("FocusFrameHealthBar".to_string()),
                Some(focus_health_bars_container_id),
            );
            focus_health_bar.width = 120.0;
            focus_health_bar.height = 20.0;
            let focus_health_bar_id = s.widgets.register(focus_health_bar);

            let mut focus_mana_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("FocusFrameManaBar".to_string()),
                Some(focus_frame_content_main_id),
            );
            focus_mana_bar.width = 120.0;
            focus_mana_bar.height = 12.0;
            let focus_mana_bar_id = s.widgets.register(focus_mana_bar);

            // Create totFrame (focus-target-of-target frame) for FocusFrame
            let mut focus_tot_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("FocusFrameToTFrame".to_string()),
                Some(focus_frame_id),
            );
            focus_tot_frame.width = 80.0;
            focus_tot_frame.height = 30.0;
            let focus_tot_frame_id = s.widgets.register(focus_tot_frame);

            let focus_tot_health_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("FocusFrameToTHealthBar".to_string()),
                Some(focus_tot_frame_id),
            );
            let focus_tot_health_bar_id = s.widgets.register(focus_tot_health_bar);

            // Set up children_keys for FocusFrame hierarchy (uses TargetFrameContent name)
            if let Some(ff) = s.widgets.get_mut(focus_frame_id) {
                ff.children_keys.insert("TargetFrameContent".to_string(), focus_frame_content_id);
                ff.children_keys.insert("totFrame".to_string(), focus_tot_frame_id);
            }
            if let Some(ffc) = s.widgets.get_mut(focus_frame_content_id) {
                ffc.children_keys.insert("TargetFrameContentMain".to_string(), focus_frame_content_main_id);
            }
            if let Some(ffcm) = s.widgets.get_mut(focus_frame_content_main_id) {
                ffcm.children_keys.insert("HealthBarsContainer".to_string(), focus_health_bars_container_id);
                ffcm.children_keys.insert("ManaBar".to_string(), focus_mana_bar_id);
            }
            if let Some(hbc) = s.widgets.get_mut(focus_health_bars_container_id) {
                hbc.children_keys.insert("HealthBar".to_string(), focus_health_bar_id);
            }
            if let Some(tot) = s.widgets.get_mut(focus_tot_frame_id) {
                tot.children_keys.insert("HealthBar".to_string(), focus_tot_health_bar_id);
            }

            // Create FocusFrameSpellBar (focus cast bar)
            let mut focus_spell_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("FocusFrameSpellBar".to_string()),
                Some(focus_frame_id),
            );
            focus_spell_bar.width = 150.0;
            focus_spell_bar.height = 16.0;
            s.widgets.register(focus_spell_bar);

            // Create BuffFrame (player buff display)
            let mut buff_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("BuffFrame".to_string()),
                Some(ui_parent_id),
            );
            buff_frame.width = 300.0;
            buff_frame.height = 100.0;
            let buff_frame_id = s.widgets.register(buff_frame);

            // Create BuffFrame.AuraContainer (child container for buff icons)
            let mut aura_container = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("BuffFrameAuraContainer".to_string()),
                Some(buff_frame_id),
            );
            aura_container.width = 300.0;
            aura_container.height = 100.0;
            let aura_container_id = s.widgets.register(aura_container);

            // Link AuraContainer to BuffFrame's children_keys
            if let Some(buff_frame) = s.widgets.get_mut(buff_frame_id) {
                buff_frame.children_keys.insert("AuraContainer".to_string(), aura_container_id);
            }

            // Create TargetFrameSpellBar (target cast bar)
            // target_frame_id is already defined above
            let mut target_spell_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("TargetFrameSpellBar".to_string()),
                Some(target_frame_id),
            );
            target_spell_bar.width = 150.0;
            target_spell_bar.height = 16.0;
            s.widgets.register(target_spell_bar);

            // Create Minimap (minimap frame)
            let mut minimap = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("Minimap".to_string()),
                Some(ui_parent_id),
            );
            minimap.width = 140.0;
            minimap.height = 140.0;
            s.widgets.register(minimap);

            // Create MinimapCluster (minimap container frame)
            let mut minimap_cluster = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("MinimapCluster".to_string()),
                Some(ui_parent_id),
            );
            minimap_cluster.width = 192.0;
            minimap_cluster.height = 192.0;
            s.widgets.register(minimap_cluster);

            // Create ObjectiveTrackerFrame (quest/objectives tracker)
            let mut objective_tracker = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("ObjectiveTrackerFrame".to_string()),
                Some(ui_parent_id),
            );
            objective_tracker.width = 248.0;
            objective_tracker.height = 600.0;
            s.widgets.register(objective_tracker);

            // Create SettingsPanel (game settings UI)
            let mut settings_panel = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("SettingsPanel".to_string()),
                Some(ui_parent_id),
            );
            settings_panel.width = 800.0;
            settings_panel.height = 600.0;
            settings_panel.visible = false; // Hidden by default
            let settings_panel_id = s.widgets.register(settings_panel);

            // Create SettingsPanel.FrameContainer (child container for settings content)
            let mut frame_container = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("SettingsPanelFrameContainer".to_string()),
                Some(settings_panel_id),
            );
            frame_container.width = 780.0;
            frame_container.height = 580.0;
            let frame_container_id = s.widgets.register(frame_container);

            // Link FrameContainer to SettingsPanel's children_keys
            if let Some(sp) = s.widgets.get_mut(settings_panel_id) {
                sp.children_keys.insert("FrameContainer".to_string(), frame_container_id);
            }

            // Create PlayerCastingBarFrame (player cast bar)
            let mut player_casting_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("PlayerCastingBarFrame".to_string()),
                Some(ui_parent_id),
            );
            player_casting_bar.width = 200.0;
            player_casting_bar.height = 20.0;
            s.widgets.register(player_casting_bar);

            // Create PartyFrame (container for party member frames)
            let mut party_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("PartyFrame".to_string()),
                Some(ui_parent_id),
            );
            party_frame.width = 200.0;
            party_frame.height = 400.0;
            s.widgets.register(party_frame);

            // Create PetFrame (pet unit frame)
            let mut pet_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("PetFrame".to_string()),
                Some(ui_parent_id),
            );
            pet_frame.width = 128.0;
            pet_frame.height = 53.0;
            let pet_frame_id = s.widgets.register(pet_frame);

            // Create PetFrame.healthbar
            let mut pet_healthbar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("PetFrameHealthBar".to_string()),
                Some(pet_frame_id),
            );
            pet_healthbar.width = 90.0;
            pet_healthbar.height = 12.0;
            let pet_healthbar_id = s.widgets.register(pet_healthbar);

            // Create text children on healthbar
            let mut pet_left_text = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString,
                None,
                Some(pet_healthbar_id),
            );
            pet_left_text.width = 40.0;
            pet_left_text.height = 12.0;
            let pet_left_text_id = s.widgets.register(pet_left_text);

            let mut pet_right_text = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString,
                None,
                Some(pet_healthbar_id),
            );
            pet_right_text.width = 40.0;
            pet_right_text.height = 12.0;
            let pet_right_text_id = s.widgets.register(pet_right_text);

            let mut pet_text_string = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString,
                None,
                Some(pet_healthbar_id),
            );
            pet_text_string.width = 80.0;
            pet_text_string.height = 12.0;
            let pet_text_string_id = s.widgets.register(pet_text_string);

            // Create PetFrame.manabar
            let pet_manabar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("PetFrameManaBar".to_string()),
                Some(pet_frame_id),
            );
            let pet_manabar_id = s.widgets.register(pet_manabar);

            // Create text children for pet mana bar
            let pet_mana_left_text = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString, None, Some(pet_manabar_id));
            let pet_mana_left_text_id = s.widgets.register(pet_mana_left_text);

            let pet_mana_right_text = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString, None, Some(pet_manabar_id));
            let pet_mana_right_text_id = s.widgets.register(pet_mana_right_text);

            let pet_mana_text_string = crate::widget::Frame::new(
                crate::widget::WidgetType::FontString, None, Some(pet_manabar_id));
            let pet_mana_text_string_id = s.widgets.register(pet_mana_text_string);

            // Set up children_keys for PetFrame
            if let Some(pf) = s.widgets.get_mut(pet_frame_id) {
                pf.children_keys.insert("healthbar".to_string(), pet_healthbar_id);
                pf.children_keys.insert("manabar".to_string(), pet_manabar_id);
            }
            if let Some(hb) = s.widgets.get_mut(pet_healthbar_id) {
                hb.children_keys.insert("LeftText".to_string(), pet_left_text_id);
                hb.children_keys.insert("RightText".to_string(), pet_right_text_id);
                hb.children_keys.insert("TextString".to_string(), pet_text_string_id);
            }
            if let Some(mb) = s.widgets.get_mut(pet_manabar_id) {
                mb.children_keys.insert("LeftText".to_string(), pet_mana_left_text_id);
                mb.children_keys.insert("RightText".to_string(), pet_mana_right_text_id);
                mb.children_keys.insert("TextString".to_string(), pet_mana_text_string_id);
            }

            // Create AlternatePowerBar (alternate power resource bar)
            let alternate_power_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("AlternatePowerBar".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(alternate_power_bar);

            // Create MonkStaggerBar (monk stagger resource bar)
            let monk_stagger_bar = crate::widget::Frame::new(
                crate::widget::WidgetType::StatusBar,
                Some("MonkStaggerBar".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(monk_stagger_bar);

            // Create LFGListFrame (Looking For Group list frame)
            let mut lfg_list_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("LFGListFrame".to_string()),
                Some(ui_parent_id),
            );
            lfg_list_frame.width = 400.0;
            lfg_list_frame.height = 500.0;
            lfg_list_frame.visible = false;
            let lfg_list_frame_id = s.widgets.register(lfg_list_frame);

            // Create LFGListFrame.SearchPanel (the search panel child)
            let mut lfg_search_panel = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("LFGListSearchPanel".to_string()),
                Some(lfg_list_frame_id),
            );
            lfg_search_panel.width = 380.0;
            lfg_search_panel.height = 450.0;
            let lfg_search_panel_id = s.widgets.register(lfg_search_panel);

            // Create ScrollFrame child for SearchPanel
            let mut lfg_scroll_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::ScrollFrame,
                Some("LFGListSearchPanelScrollFrame".to_string()),
                Some(lfg_search_panel_id),
            );
            lfg_scroll_frame.width = 360.0;
            lfg_scroll_frame.height = 400.0;
            let lfg_scroll_frame_id = s.widgets.register(lfg_scroll_frame);

            // Add children_keys for LFGListFrame
            if let Some(lfg) = s.widgets.get_mut(lfg_list_frame_id) {
                lfg.children_keys.insert("SearchPanel".to_string(), lfg_search_panel_id);
            }

            // Add ScrollFrame to SearchPanel's children_keys
            if let Some(sp) = s.widgets.get_mut(lfg_search_panel_id) {
                sp.children_keys.insert("ScrollFrame".to_string(), lfg_scroll_frame_id);
            }

            // Create AlertFrame (alert/popup management frame)
            let alert_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("AlertFrame".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(alert_frame);

            // Create LFGEventFrame (LFG event handling frame)
            let lfg_event_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("LFGEventFrame".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(lfg_event_frame);

            // Create NamePlateDriverFrame (nameplate management frame)
            let nameplate_driver_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("NamePlateDriverFrame".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(nameplate_driver_frame);

            // Create UIErrorsFrame (error message display frame)
            let ui_errors_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::MessageFrame,
                Some("UIErrorsFrame".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(ui_errors_frame);

            // Create InterfaceOptionsFrame (legacy interface options)
            let interface_options_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("InterfaceOptionsFrame".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(interface_options_frame);

            // Create AuctionHouseFrame (auction house UI)
            let auction_house_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("AuctionHouseFrame".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(auction_house_frame);

            // Create SideDressUpFrame (side dressing room)
            let side_dressup_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("SideDressUpFrame".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(side_dressup_frame);

            // Create ContainerFrameContainer (bag frame container for combined bags)
            let container_frame_container = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("ContainerFrameContainer".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(container_frame_container);

            // Create ContainerFrameCombinedBags (combined bag frame)
            let container_combined_bags = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("ContainerFrameCombinedBags".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(container_combined_bags);

            // Create LootFrame (loot window)
            let loot_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("LootFrame".to_string()),
                Some(ui_parent_id),
            );
            let loot_frame_id = loot_frame.id;
            s.widgets.register(loot_frame);

            // Create LootFrame.ScrollBox (scroll container for loot items)
            let loot_scroll_box = crate::widget::Frame::new(
                crate::widget::WidgetType::ScrollFrame,
                Some("LootFrameScrollBox".to_string()),
                Some(loot_frame_id),
            );
            let loot_scroll_box_id = loot_scroll_box.id;
            s.widgets.register(loot_scroll_box);

            // Add children_keys for LootFrame.ScrollBox access
            if let Some(lf) = s.widgets.get_mut(loot_frame_id) {
                lf.children_keys.insert("ScrollBox".to_string(), loot_scroll_box_id);
            }

            // Create ScenarioObjectiveTracker (objective tracker for scenarios/M+)
            let scenario_tracker = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("ScenarioObjectiveTracker".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(scenario_tracker);

            // Create RaidWarningFrame (raid warning message display)
            let raid_warning_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::MessageFrame,
                Some("RaidWarningFrame".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(raid_warning_frame);

            // Create GossipFrame (NPC interaction dialog)
            let mut gossip_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("GossipFrame".to_string()),
                Some(ui_parent_id),
            );
            gossip_frame.width = 338.0;
            gossip_frame.height = 479.0;
            s.widgets.register(gossip_frame);

            // Create QuestFrame (quest interaction dialog)
            let mut quest_frame = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("QuestFrame".to_string()),
                Some(ui_parent_id),
            );
            quest_frame.width = 384.0;
            quest_frame.height = 512.0;
            s.widgets.register(quest_frame);
        }

        // Register global functions
        globals::register_globals(&lua, Rc::clone(&state))?;

        Ok(Self { lua, state })
    }

    /// Execute Lua code.
    pub fn exec(&self, code: &str) -> Result<()> {
        self.lua.load(code).exec()?;
        Ok(())
    }

    /// Execute Lua code with a custom chunk name (for better error messages and debugstack).
    pub fn exec_named(&self, code: &str, name: &str) -> Result<()> {
        self.lua.load(code).set_name(name).exec()?;
        Ok(())
    }

    /// Execute Lua code with varargs (like WoW addon loading).
    /// In WoW, each addon file receives (addonName, addonTable) as varargs.
    pub fn exec_with_varargs(
        &self,
        code: &str,
        name: &str,
        addon_name: &str,
        addon_table: mlua::Table,
    ) -> Result<()> {
        let chunk = self.lua.load(code).set_name(name);
        let func: mlua::Function = chunk.into_function()?;
        func.call::<()>((addon_name.to_string(), addon_table))?;
        Ok(())
    }

    /// Create a new empty table for addon private storage.
    /// Includes a default `unpack` method that returns values at numeric indices.
    pub fn create_addon_table(&self) -> Result<mlua::Table> {
        let table = self.lua.create_table()?;
        // Add default unpack method - returns values at indices 1, 2, 3, 4
        // Addons like OmniCD use this pattern: local E, L, C = select(2, ...):unpack()
        let unpack_fn = self.lua.create_function(|_, this: mlua::Table| {
            let v1: mlua::Value = this.get(1).unwrap_or(mlua::Value::Nil);
            let v2: mlua::Value = this.get(2).unwrap_or(mlua::Value::Nil);
            let v3: mlua::Value = this.get(3).unwrap_or(mlua::Value::Nil);
            let v4: mlua::Value = this.get(4).unwrap_or(mlua::Value::Nil);
            Ok((v1, v2, v3, v4))
        })?;
        table.set("unpack", unpack_fn)?;
        Ok(table)
    }

    /// Execute Lua code and return the result.
    pub fn eval<T: mlua::FromLuaMulti>(&self, code: &str) -> Result<T> {
        let result = self.lua.load(code).eval()?;
        Ok(result)
    }

    /// Fire an event to all registered frames.
    pub fn fire_event(&self, event: &str) -> Result<()> {
        self.fire_event_with_args(event, &[])
    }

    /// Fire an event with arguments to all registered frames.
    pub fn fire_event_with_args(&self, event: &str, args: &[Value]) -> Result<()> {
        let listeners = {
            let state = self.state.borrow();
            state.widgets.get_event_listeners(event)
        };

        for widget_id in listeners {
            // Get the handler function from our scripts table
            let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();

            if let Some(table) = scripts_table {
                let frame_key = format!("{}_OnEvent", widget_id);
                let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

                if let Some(handler) = handler {
                    // Get the frame userdata
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;

                    // Build arguments: (self, event, ...args)
                    let mut call_args = vec![frame, Value::String(self.lua.create_string(event)?)];
                    call_args.extend(args.iter().cloned());

                    handler.call::<()>(MultiValue::from_vec(call_args))?;
                }
            }
        }

        Ok(())
    }

    /// Fire a script handler for a specific widget.
    /// handler_name is like "OnClick", "OnEnter", etc.
    /// extra_args are passed after the frame (self) argument.
    pub fn fire_script_handler(
        &self,
        widget_id: u64,
        handler_name: &str,
        extra_args: Vec<Value>,
    ) -> Result<()> {
        let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();

        if let Some(table) = scripts_table {
            let frame_key = format!("{}_{}", widget_id, handler_name);
            let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

            if let Some(handler) = handler {
                // Get the frame userdata
                let frame_ref_key = format!("__frame_{}", widget_id);
                let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;

                // Build arguments: (self, ...extra_args)
                let mut call_args = vec![frame];
                call_args.extend(extra_args);

                handler.call::<()>(MultiValue::from_vec(call_args))?;
            }
        }

        Ok(())
    }

    /// Dispatch a slash command (e.g., "/wa options").
    /// Returns Ok(true) if a handler was found and called, Ok(false) if no handler matched.
    pub fn dispatch_slash_command(&self, input: &str) -> Result<bool> {
        let input = input.trim();
        if !input.starts_with('/') {
            return Ok(false);
        }

        // Parse command and message: "/wa options" -> cmd="/wa", msg="options"
        let (cmd, msg) = match input.find(' ') {
            Some(pos) => (&input[..pos], input[pos + 1..].trim()),
            None => (input, ""),
        };
        let cmd_lower = cmd.to_lowercase();

        // Scan globals for SLASH_* variables to find a matching command
        let globals = self.lua.globals();
        let slash_cmd_list: mlua::Table = globals.get("SlashCmdList")?;

        // Iterate through all globals looking for SLASH_* patterns
        for pair in globals.pairs::<String, Value>() {
            let (key, value) = pair?;

            // Look for SLASH_NAME1, SLASH_NAME2, etc.
            if !key.starts_with("SLASH_") {
                continue;
            }

            // Extract the command name (e.g., "SLASH_WEAKAURAS1" -> "WEAKAURAS")
            let suffix = &key[6..]; // Skip "SLASH_"
            let name = suffix.trim_end_matches(|c: char| c.is_ascii_digit());
            if name.is_empty() {
                continue;
            }

            // Check if this SLASH_ variable matches our command
            if let Value::String(slash_str) = value {
                if slash_str.to_str()?.to_lowercase() == cmd_lower {
                    // Found a match! Look up the handler in SlashCmdList
                    let handler: Option<mlua::Function> = slash_cmd_list.get(name).ok();
                    if let Some(handler) = handler {
                        let msg_value = self.lua.create_string(msg)?;
                        handler.call::<()>(msg_value)?;
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get access to the Lua state.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Get access to the simulator state.
    pub fn state(&self) -> &Rc<RefCell<SimState>> {
        &self.state
    }

    /// Schedule a timer callback.
    pub fn schedule_timer(
        &self,
        seconds: f64,
        callback: mlua::Function,
        interval: Option<std::time::Duration>,
        iterations: Option<i32>,
    ) -> Result<u64> {
        let id = next_timer_id();
        let callback_key = self.lua.create_registry_value(callback)?;
        let fire_at = Instant::now() + std::time::Duration::from_secs_f64(seconds);

        let timer = PendingTimer {
            id,
            fire_at,
            callback_key,
            interval,
            remaining: iterations,
            cancelled: false,
            handle_key: None,
        };

        self.state.borrow_mut().timers.push_back(timer);
        Ok(id)
    }

    /// Cancel a timer by ID.
    pub fn cancel_timer(&self, timer_id: u64) {
        let mut state = self.state.borrow_mut();
        for timer in state.timers.iter_mut() {
            if timer.id == timer_id {
                timer.cancelled = true;
                break;
            }
        }
    }

    /// Process any timers that are ready to fire.
    /// Returns the number of callbacks invoked.
    pub fn process_timers(&self) -> Result<usize> {
        let now = Instant::now();
        let mut fired = 0;
        let mut to_reschedule = Vec::new();

        // Collect timers that need to fire
        let mut state = self.state.borrow_mut();
        let mut i = 0;
        while i < state.timers.len() {
            if state.timers[i].cancelled {
                // Remove cancelled timers and clean up registry
                let timer = state.timers.remove(i).unwrap();
                self.lua.remove_registry_value(timer.callback_key).ok();
                if let Some(hk) = timer.handle_key {
                    self.lua.remove_registry_value(hk).ok();
                }
                continue;
            }

            if state.timers[i].fire_at <= now {
                let mut timer = state.timers.remove(i).unwrap();

                // Get the callback from registry
                if let Ok(callback) = self.lua.registry_value::<mlua::Function>(&timer.callback_key) {
                    // Get the handle table if present (for NewTimer/NewTicker)
                    let handle: Option<mlua::Table> = timer
                        .handle_key
                        .as_ref()
                        .and_then(|k| self.lua.registry_value(k).ok());

                    // Drop state borrow before calling Lua
                    drop(state);

                    // Call the callback with the handle as argument (if present)
                    let result = if let Some(h) = handle {
                        callback.call::<()>(h)
                    } else {
                        callback.call::<()>(())
                    };
                    if let Err(e) = result {
                        eprintln!("Timer callback error: {}", e);
                    }
                    fired += 1;

                    // Re-borrow state
                    state = self.state.borrow_mut();

                    // Check if this is a ticker that should repeat
                    if let Some(interval) = timer.interval {
                        let should_repeat = match &mut timer.remaining {
                            Some(n) if *n > 1 => {
                                *n -= 1;
                                true
                            }
                            Some(_) => false, // Last iteration
                            None => true,     // Infinite ticker
                        };

                        if should_repeat {
                            timer.fire_at = now + interval;
                            to_reschedule.push(timer);
                        } else {
                            // Clean up registry keys for finished timer
                            self.lua.remove_registry_value(timer.callback_key).ok();
                            if let Some(hk) = timer.handle_key {
                                self.lua.remove_registry_value(hk).ok();
                            }
                        }
                    } else {
                        // One-shot timer, clean up registry keys
                        self.lua.remove_registry_value(timer.callback_key).ok();
                        if let Some(hk) = timer.handle_key {
                            self.lua.remove_registry_value(hk).ok();
                        }
                    }
                } else {
                    // Callback not found, clean up
                    self.lua.remove_registry_value(timer.callback_key).ok();
                    if let Some(hk) = timer.handle_key {
                        self.lua.remove_registry_value(hk).ok();
                    }
                }
                continue;
            }
            i += 1;
        }

        // Re-add tickers that should repeat
        for timer in to_reschedule {
            state.timers.push_back(timer);
        }

        Ok(fired)
    }

    /// Check if there are any pending timers.
    pub fn has_pending_timers(&self) -> bool {
        !self.state.borrow().timers.is_empty()
    }

    /// Get the time until the next timer fires, if any.
    pub fn next_timer_delay(&self) -> Option<std::time::Duration> {
        let state = self.state.borrow();
        let now = Instant::now();
        state
            .timers
            .iter()
            .filter(|t| !t.cancelled)
            .map(|t| {
                if t.fire_at > now {
                    t.fire_at - now
                } else {
                    std::time::Duration::ZERO
                }
            })
            .min()
    }

    /// Dump all frame positions for debugging.
    /// Returns a formatted string similar to iced-debug output.
    pub fn dump_frames(&self) -> String {
        let state = self.state.borrow();
        let screen_width = 500.0_f32;
        let screen_height = 375.0_f32;

        let mut output = String::new();
        output.push_str(&format!(
            "[WoW Frames: {}x{}]\n\n",
            screen_width, screen_height
        ));

        // Collect and sort frames by strata/level
        let mut frames: Vec<_> = state.widgets.all_ids().into_iter().collect();
        frames.sort_by(|&a, &b| {
            let fa = state.widgets.get(a);
            let fb = state.widgets.get(b);
            match (fa, fb) {
                (Some(fa), Some(fb)) => fa
                    .frame_strata
                    .cmp(&fb.frame_strata)
                    .then_with(|| fa.frame_level.cmp(&fb.frame_level)),
                _ => std::cmp::Ordering::Equal,
            }
        });

        for id in frames {
            let frame = match state.widgets.get(id) {
                Some(f) => f,
                None => continue,
            };

            // Compute position
            let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);

            // Format: Name [Type] (x,y w×h) visible/hidden
            let name = frame.name.as_deref().unwrap_or("(anon)");
            let vis = if frame.visible { "" } else { " HIDDEN" };
            let mouse = if frame.mouse_enabled { " mouse" } else { "" };

            // Indentation based on parent depth
            let depth = get_parent_depth(&state.widgets, id);
            let indent = "  ".repeat(depth);

            // Get parent name for context
            let parent_name = frame.parent_id
                .and_then(|pid| state.widgets.get(pid))
                .and_then(|p| p.name.as_deref())
                .unwrap_or("(root)");

            output.push_str(&format!(
                "{}{} [{}] ({:.0},{:.0} {:.0}x{:.0}){}{} parent={}\n",
                indent,
                name,
                frame.widget_type.as_str(),
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                vis,
                mouse,
                parent_name,
            ));

            // Show anchor info
            if !frame.anchors.is_empty() {
                let anchor = &frame.anchors[0];
                output.push_str(&format!(
                    "{}  └─ {:?} -> {:?} offset ({:.0},{:.0})\n",
                    indent, anchor.point, anchor.relative_point, anchor.x_offset, anchor.y_offset
                ));
            } else {
                output.push_str(&format!("{}  └─ (no anchors - centered)\n", indent));
            }
        }

        output
    }
}

/// Get depth in parent hierarchy (for indentation).
fn get_parent_depth(registry: &crate::widget::WidgetRegistry, id: u64) -> usize {
    let mut depth = 0;
    let mut current = id;
    while let Some(frame) = registry.get(current) {
        if let Some(parent_id) = frame.parent_id {
            depth += 1;
            current = parent_id;
        } else {
            break;
        }
    }
    depth
}

/// Compute frame rect for debugging (same algorithm as renderer).
fn compute_frame_rect(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    screen_width: f32,
    screen_height: f32,
) -> LayoutRect {
    let frame = match registry.get(id) {
        Some(f) => f,
        None => return LayoutRect::default(),
    };

    let width = frame.width;
    let height = frame.height;

    // If no anchors, default to center of parent
    if frame.anchors.is_empty() {
        let parent_rect = if let Some(parent_id) = frame.parent_id {
            compute_frame_rect(registry, parent_id, screen_width, screen_height)
        } else {
            LayoutRect {
                x: 0.0,
                y: 0.0,
                width: screen_width,
                height: screen_height,
            }
        };

        return LayoutRect {
            x: parent_rect.x + (parent_rect.width - width) / 2.0,
            y: parent_rect.y + (parent_rect.height - height) / 2.0,
            width,
            height,
        };
    }

    let anchor = &frame.anchors[0];

    let parent_rect = if let Some(parent_id) = frame.parent_id {
        compute_frame_rect(registry, parent_id, screen_width, screen_height)
    } else {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        }
    };

    let (parent_anchor_x, parent_anchor_y) = anchor_position(
        anchor.relative_point,
        parent_rect.x,
        parent_rect.y,
        parent_rect.width,
        parent_rect.height,
    );

    let target_x = parent_anchor_x + anchor.x_offset;
    // WoW uses Y-up coordinate system, screen uses Y-down
    let target_y = parent_anchor_y - anchor.y_offset;

    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect {
        x: frame_x,
        y: frame_y,
        width,
        height,
    }
}

fn anchor_position(
    point: crate::widget::AnchorPoint,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
    match point {
        AnchorPoint::TopLeft => (x, y),
        AnchorPoint::Top => (x + w / 2.0, y),
        AnchorPoint::TopRight => (x + w, y),
        AnchorPoint::Left => (x, y + h / 2.0),
        AnchorPoint::Center => (x + w / 2.0, y + h / 2.0),
        AnchorPoint::Right => (x + w, y + h / 2.0),
        AnchorPoint::BottomLeft => (x, y + h),
        AnchorPoint::Bottom => (x + w / 2.0, y + h),
        AnchorPoint::BottomRight => (x + w, y + h),
    }
}

fn frame_position_from_anchor(
    point: crate::widget::AnchorPoint,
    anchor_x: f32,
    anchor_y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
    match point {
        AnchorPoint::TopLeft => (anchor_x, anchor_y),
        AnchorPoint::Top => (anchor_x - w / 2.0, anchor_y),
        AnchorPoint::TopRight => (anchor_x - w, anchor_y),
        AnchorPoint::Left => (anchor_x, anchor_y - h / 2.0),
        AnchorPoint::Center => (anchor_x - w / 2.0, anchor_y - h / 2.0),
        AnchorPoint::Right => (anchor_x - w, anchor_y - h / 2.0),
        AnchorPoint::BottomLeft => (anchor_x, anchor_y - h),
        AnchorPoint::Bottom => (anchor_x - w / 2.0, anchor_y - h),
        AnchorPoint::BottomRight => (anchor_x - w, anchor_y - h),
    }
}

/// Simple layout rect for frame positioning.
#[derive(Debug, Default, Clone, Copy)]
struct LayoutRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}
