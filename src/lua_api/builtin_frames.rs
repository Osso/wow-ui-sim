//! Built-in WoW frames created at startup.

use crate::widget::{Frame, WidgetRegistry, WidgetType};

/// Create all built-in WoW frames (UIParent, WorldFrame, unit frames, etc.).
pub fn create_builtin_frames(widgets: &mut WidgetRegistry) {
    // Create UIParent (the root frame) - must have screen dimensions for layout
    let mut ui_parent = Frame::new(WidgetType::Frame, Some("UIParent".to_string()), None);
    // Set UIParent to screen size (reference coordinate system)
    ui_parent.width = 500.0;
    ui_parent.height = 375.0;
    let ui_parent_id = ui_parent.id;
    widgets.register(ui_parent);

    // Create Minimap (built-in UI element)
    let minimap = Frame::new(
        WidgetType::Frame,
        Some("Minimap".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(minimap);

    // Create WorldFrame (3D world rendering area - used by HUD elements)
    let mut world_frame = Frame::new(
        WidgetType::Frame,
        Some("WorldFrame".to_string()),
        None, // No parent - it's at the same level as UIParent
    );
    world_frame.width = 500.0;
    world_frame.height = 375.0;
    widgets.register(world_frame);

    // Create DEFAULT_CHAT_FRAME (the main chat window)
    let mut chat_frame = Frame::new(
        WidgetType::MessageFrame,
        Some("DEFAULT_CHAT_FRAME".to_string()),
        Some(ui_parent_id),
    );
    chat_frame.width = 430.0;
    chat_frame.height = 120.0;
    widgets.register(chat_frame);

    // Create ChatFrame1 (same as DEFAULT_CHAT_FRAME in WoW)
    let mut chat_frame1 = Frame::new(
        WidgetType::MessageFrame,
        Some("ChatFrame1".to_string()),
        Some(ui_parent_id),
    );
    chat_frame1.width = 430.0;
    chat_frame1.height = 120.0;
    widgets.register(chat_frame1);

    // Create EventToastManagerFrame (UI for event toasts/notifications)
    let mut event_toast_frame = Frame::new(
        WidgetType::Frame,
        Some("EventToastManagerFrame".to_string()),
        Some(ui_parent_id),
    );
    event_toast_frame.width = 300.0;
    event_toast_frame.height = 100.0;
    widgets.register(event_toast_frame);

    // Create EditModeManagerFrame (Edit Mode UI manager)
    let mut edit_mode_frame = Frame::new(
        WidgetType::Frame,
        Some("EditModeManagerFrame".to_string()),
        Some(ui_parent_id),
    );
    edit_mode_frame.width = 400.0;
    edit_mode_frame.height = 300.0;
    widgets.register(edit_mode_frame);

    // Create RolePollPopup (role selection popup for groups)
    let mut role_poll_popup = Frame::new(
        WidgetType::Frame,
        Some("RolePollPopup".to_string()),
        Some(ui_parent_id),
    );
    role_poll_popup.width = 200.0;
    role_poll_popup.height = 150.0;
    widgets.register(role_poll_popup);

    // Create TimerTracker (displays dungeon/raid instance timers)
    let mut timer_tracker = Frame::new(
        WidgetType::Frame,
        Some("TimerTracker".to_string()),
        Some(ui_parent_id),
    );
    timer_tracker.width = 200.0;
    timer_tracker.height = 50.0;
    widgets.register(timer_tracker);

    create_world_map_frame(widgets, ui_parent_id);
    create_player_frame(widgets, ui_parent_id);
    create_target_frame(widgets, ui_parent_id);
    create_focus_frame(widgets, ui_parent_id);
    create_buff_frame(widgets, ui_parent_id);
    create_pet_frame(widgets, ui_parent_id);
    create_misc_frames(widgets, ui_parent_id);
}

fn create_world_map_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Create WorldMapFrame (world map display frame)
    let mut world_map_frame = Frame::new(
        WidgetType::Frame,
        Some("WorldMapFrame".to_string()),
        Some(ui_parent_id),
    );
    world_map_frame.width = 1024.0;
    world_map_frame.height = 768.0;
    world_map_frame.visible = false; // Hidden by default
    let world_map_frame_id = widgets.register(world_map_frame);

    // Create WorldMapFrame.BorderFrame
    let mut world_map_border_frame = Frame::new(
        WidgetType::Frame,
        Some("WorldMapBorderFrame".to_string()),
        Some(world_map_frame_id),
    );
    world_map_border_frame.width = 1024.0;
    world_map_border_frame.height = 768.0;
    let world_map_border_frame_id = widgets.register(world_map_border_frame);

    // Create WorldMapFrame.BorderFrame.MaximizeMinimizeFrame
    let mut max_min_frame = Frame::new(
        WidgetType::Frame,
        Some("WorldMapMaximizeMinimizeFrame".to_string()),
        Some(world_map_border_frame_id),
    );
    max_min_frame.width = 32.0;
    max_min_frame.height = 32.0;
    let max_min_frame_id = widgets.register(max_min_frame);

    // Create WorldMapFrame.BorderFrame.MaximizeMinimizeFrame.MaximizeButton
    let max_button = Frame::new(
        WidgetType::Button,
        Some("WorldMapMaximizeButton".to_string()),
        Some(max_min_frame_id),
    );
    let max_button_id = widgets.register(max_button);

    // Create WorldMapFrame.BorderFrame.MaximizeMinimizeFrame.MinimizeButton
    let min_button = Frame::new(
        WidgetType::Button,
        Some("WorldMapMinimizeButton".to_string()),
        Some(max_min_frame_id),
    );
    let min_button_id = widgets.register(min_button);

    // Create WorldMapFrame.ScrollContainer
    let scroll_container = Frame::new(
        WidgetType::ScrollFrame,
        Some("WorldMapScrollContainer".to_string()),
        Some(world_map_frame_id),
    );
    let scroll_container_id = widgets.register(scroll_container);

    // Set up children_keys for WorldMapFrame hierarchy
    if let Some(wm_frame) = widgets.get_mut(world_map_frame_id) {
        wm_frame
            .children_keys
            .insert("BorderFrame".to_string(), world_map_border_frame_id);
        wm_frame
            .children_keys
            .insert("ScrollContainer".to_string(), scroll_container_id);
    }
    if let Some(border_frame) = widgets.get_mut(world_map_border_frame_id) {
        border_frame
            .children_keys
            .insert("MaximizeMinimizeFrame".to_string(), max_min_frame_id);
    }
    if let Some(mm_frame) = widgets.get_mut(max_min_frame_id) {
        mm_frame
            .children_keys
            .insert("MaximizeButton".to_string(), max_button_id);
        mm_frame
            .children_keys
            .insert("MinimizeButton".to_string(), min_button_id);
    }
}

fn create_player_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Create PlayerFrame (player unit frame)
    let mut player_frame = Frame::new(
        WidgetType::Frame,
        Some("PlayerFrame".to_string()),
        Some(ui_parent_id),
    );
    player_frame.width = 175.0;
    player_frame.height = 76.0;
    let player_frame_id = widgets.register(player_frame);

    // Create PlayerFrame.PlayerFrameContent
    let mut player_frame_content = Frame::new(
        WidgetType::Frame,
        Some("PlayerFrameContent".to_string()),
        Some(player_frame_id),
    );
    player_frame_content.width = 175.0;
    player_frame_content.height = 76.0;
    let player_frame_content_id = widgets.register(player_frame_content);

    // Create PlayerFrame.PlayerFrameContent.PlayerFrameContentMain
    let mut player_frame_content_main = Frame::new(
        WidgetType::Frame,
        Some("PlayerFrameContentMain".to_string()),
        Some(player_frame_content_id),
    );
    player_frame_content_main.width = 175.0;
    player_frame_content_main.height = 76.0;
    let player_frame_content_main_id = widgets.register(player_frame_content_main);

    // Create HealthBarsContainer
    let mut health_bars_container = Frame::new(
        WidgetType::Frame,
        Some("PlayerFrameHealthBarsContainer".to_string()),
        Some(player_frame_content_main_id),
    );
    health_bars_container.width = 120.0;
    health_bars_container.height = 20.0;
    let health_bars_container_id = widgets.register(health_bars_container);

    // Create HealthBar
    let mut health_bar = Frame::new(
        WidgetType::StatusBar,
        Some("PlayerFrameHealthBar".to_string()),
        Some(health_bars_container_id),
    );
    health_bar.width = 120.0;
    health_bar.height = 20.0;
    let health_bar_id = widgets.register(health_bar);

    // Create ManaBarArea
    let mut mana_bar_area = Frame::new(
        WidgetType::Frame,
        Some("PlayerFrameManaBarArea".to_string()),
        Some(player_frame_content_main_id),
    );
    mana_bar_area.width = 120.0;
    mana_bar_area.height = 12.0;
    let mana_bar_area_id = widgets.register(mana_bar_area);

    // Create ManaBar
    let mut mana_bar = Frame::new(
        WidgetType::StatusBar,
        Some("PlayerFrameManaBar".to_string()),
        Some(mana_bar_area_id),
    );
    mana_bar.width = 120.0;
    mana_bar.height = 12.0;
    let mana_bar_id = widgets.register(mana_bar);

    // Create text children for health bar and mana bar
    let health_left_text = Frame::new(WidgetType::FontString, None, Some(health_bar_id));
    let health_left_text_id = widgets.register(health_left_text);

    let health_right_text = Frame::new(WidgetType::FontString, None, Some(health_bar_id));
    let health_right_text_id = widgets.register(health_right_text);

    let health_text_string = Frame::new(WidgetType::FontString, None, Some(health_bar_id));
    let health_text_string_id = widgets.register(health_text_string);

    let mana_left_text = Frame::new(WidgetType::FontString, None, Some(mana_bar_id));
    let mana_left_text_id = widgets.register(mana_left_text);

    let mana_right_text = Frame::new(WidgetType::FontString, None, Some(mana_bar_id));
    let mana_right_text_id = widgets.register(mana_right_text);

    let mana_bar_text = Frame::new(WidgetType::FontString, None, Some(mana_bar_id));
    let mana_bar_text_id = widgets.register(mana_bar_text);

    // Set up children_keys for the hierarchy
    if let Some(pf) = widgets.get_mut(player_frame_id) {
        pf.children_keys
            .insert("PlayerFrameContent".to_string(), player_frame_content_id);
    }
    if let Some(pfc) = widgets.get_mut(player_frame_content_id) {
        pfc.children_keys.insert(
            "PlayerFrameContentMain".to_string(),
            player_frame_content_main_id,
        );
    }
    if let Some(pfcm) = widgets.get_mut(player_frame_content_main_id) {
        pfcm.children_keys
            .insert("HealthBarsContainer".to_string(), health_bars_container_id);
        pfcm.children_keys
            .insert("ManaBarArea".to_string(), mana_bar_area_id);
    }
    if let Some(hbc) = widgets.get_mut(health_bars_container_id) {
        hbc.children_keys
            .insert("HealthBar".to_string(), health_bar_id);
    }
    if let Some(mba) = widgets.get_mut(mana_bar_area_id) {
        mba.children_keys.insert("ManaBar".to_string(), mana_bar_id);
    }
    // Add text children to health bar
    if let Some(hb) = widgets.get_mut(health_bar_id) {
        hb.children_keys
            .insert("LeftText".to_string(), health_left_text_id);
        hb.children_keys
            .insert("RightText".to_string(), health_right_text_id);
        hb.children_keys
            .insert("TextString".to_string(), health_text_string_id);
    }
    // Add text children to mana bar
    if let Some(mb) = widgets.get_mut(mana_bar_id) {
        mb.children_keys
            .insert("LeftText".to_string(), mana_left_text_id);
        mb.children_keys
            .insert("RightText".to_string(), mana_right_text_id);
        mb.children_keys
            .insert("ManaBarText".to_string(), mana_bar_text_id);
    }
}

fn create_target_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Create TargetFrame (target unit frame)
    let mut target_frame = Frame::new(
        WidgetType::Frame,
        Some("TargetFrame".to_string()),
        Some(ui_parent_id),
    );
    target_frame.width = 175.0;
    target_frame.height = 76.0;
    let target_frame_id = widgets.register(target_frame);

    // Create TargetFrame.TargetFrameContent hierarchy (similar to PlayerFrame)
    let mut target_frame_content = Frame::new(
        WidgetType::Frame,
        Some("TargetFrameContent".to_string()),
        Some(target_frame_id),
    );
    target_frame_content.width = 175.0;
    target_frame_content.height = 76.0;
    let target_frame_content_id = widgets.register(target_frame_content);

    let mut target_frame_content_main = Frame::new(
        WidgetType::Frame,
        Some("TargetFrameContentMain".to_string()),
        Some(target_frame_content_id),
    );
    target_frame_content_main.width = 175.0;
    target_frame_content_main.height = 76.0;
    let target_frame_content_main_id = widgets.register(target_frame_content_main);

    let mut target_health_bars_container = Frame::new(
        WidgetType::Frame,
        Some("TargetFrameHealthBarsContainer".to_string()),
        Some(target_frame_content_main_id),
    );
    target_health_bars_container.width = 120.0;
    target_health_bars_container.height = 20.0;
    let target_health_bars_container_id = widgets.register(target_health_bars_container);

    let mut target_health_bar = Frame::new(
        WidgetType::StatusBar,
        Some("TargetFrameHealthBar".to_string()),
        Some(target_health_bars_container_id),
    );
    target_health_bar.width = 120.0;
    target_health_bar.height = 20.0;
    let target_health_bar_id = widgets.register(target_health_bar);

    let mut target_mana_bar_area = Frame::new(
        WidgetType::Frame,
        Some("TargetFrameManaBarArea".to_string()),
        Some(target_frame_content_main_id),
    );
    target_mana_bar_area.width = 120.0;
    target_mana_bar_area.height = 12.0;
    let target_mana_bar_area_id = widgets.register(target_mana_bar_area);

    let mut target_mana_bar = Frame::new(
        WidgetType::StatusBar,
        Some("TargetFrameManaBar".to_string()),
        Some(target_mana_bar_area_id),
    );
    target_mana_bar.width = 120.0;
    target_mana_bar.height = 12.0;
    let target_mana_bar_id = widgets.register(target_mana_bar);

    // Create totFrame (target-of-target frame) for TargetFrame
    let mut target_tot_frame = Frame::new(
        WidgetType::Frame,
        Some("TargetFrameToTFrame".to_string()),
        Some(target_frame_id),
    );
    target_tot_frame.width = 80.0;
    target_tot_frame.height = 30.0;
    let target_tot_frame_id = widgets.register(target_tot_frame);

    let target_tot_health_bar = Frame::new(
        WidgetType::StatusBar,
        Some("TargetFrameToTHealthBar".to_string()),
        Some(target_tot_frame_id),
    );
    let target_tot_health_bar_id = widgets.register(target_tot_health_bar);

    // Create TargetFrameSpellBar (target cast bar)
    let mut target_spell_bar = Frame::new(
        WidgetType::StatusBar,
        Some("TargetFrameSpellBar".to_string()),
        Some(target_frame_id),
    );
    target_spell_bar.width = 150.0;
    target_spell_bar.height = 16.0;
    widgets.register(target_spell_bar);

    // Set up children_keys for TargetFrame hierarchy
    if let Some(tf) = widgets.get_mut(target_frame_id) {
        tf.children_keys
            .insert("TargetFrameContent".to_string(), target_frame_content_id);
        tf.children_keys
            .insert("totFrame".to_string(), target_tot_frame_id);
    }
    if let Some(tfc) = widgets.get_mut(target_frame_content_id) {
        tfc.children_keys.insert(
            "TargetFrameContentMain".to_string(),
            target_frame_content_main_id,
        );
    }
    if let Some(tfcm) = widgets.get_mut(target_frame_content_main_id) {
        tfcm.children_keys.insert(
            "HealthBarsContainer".to_string(),
            target_health_bars_container_id,
        );
        tfcm.children_keys
            .insert("ManaBarArea".to_string(), target_mana_bar_area_id);
        // Also add ManaBar directly on ContentMain (some addons access it this way)
        tfcm.children_keys
            .insert("ManaBar".to_string(), target_mana_bar_id);
    }
    if let Some(hbc) = widgets.get_mut(target_health_bars_container_id) {
        hbc.children_keys
            .insert("HealthBar".to_string(), target_health_bar_id);
    }
    if let Some(mba) = widgets.get_mut(target_mana_bar_area_id) {
        mba.children_keys
            .insert("ManaBar".to_string(), target_mana_bar_id);
    }
    if let Some(tot) = widgets.get_mut(target_tot_frame_id) {
        tot.children_keys
            .insert("HealthBar".to_string(), target_tot_health_bar_id);
    }
}

fn create_focus_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Create FocusFrame (focus target unit frame)
    let mut focus_frame = Frame::new(
        WidgetType::Frame,
        Some("FocusFrame".to_string()),
        Some(ui_parent_id),
    );
    focus_frame.width = 175.0;
    focus_frame.height = 76.0;
    let focus_frame_id = widgets.register(focus_frame);

    // Create FocusFrame.TargetFrameContent hierarchy (yes, it's confusingly named TargetFrameContent)
    let mut focus_frame_content = Frame::new(
        WidgetType::Frame,
        Some("FocusFrameContent".to_string()),
        Some(focus_frame_id),
    );
    focus_frame_content.width = 175.0;
    focus_frame_content.height = 76.0;
    let focus_frame_content_id = widgets.register(focus_frame_content);

    let mut focus_frame_content_main = Frame::new(
        WidgetType::Frame,
        Some("FocusFrameContentMain".to_string()),
        Some(focus_frame_content_id),
    );
    focus_frame_content_main.width = 175.0;
    focus_frame_content_main.height = 76.0;
    let focus_frame_content_main_id = widgets.register(focus_frame_content_main);

    let mut focus_health_bars_container = Frame::new(
        WidgetType::Frame,
        Some("FocusFrameHealthBarsContainer".to_string()),
        Some(focus_frame_content_main_id),
    );
    focus_health_bars_container.width = 120.0;
    focus_health_bars_container.height = 20.0;
    let focus_health_bars_container_id = widgets.register(focus_health_bars_container);

    let mut focus_health_bar = Frame::new(
        WidgetType::StatusBar,
        Some("FocusFrameHealthBar".to_string()),
        Some(focus_health_bars_container_id),
    );
    focus_health_bar.width = 120.0;
    focus_health_bar.height = 20.0;
    let focus_health_bar_id = widgets.register(focus_health_bar);

    let mut focus_mana_bar = Frame::new(
        WidgetType::StatusBar,
        Some("FocusFrameManaBar".to_string()),
        Some(focus_frame_content_main_id),
    );
    focus_mana_bar.width = 120.0;
    focus_mana_bar.height = 12.0;
    let focus_mana_bar_id = widgets.register(focus_mana_bar);

    // Create totFrame (focus-target-of-target frame) for FocusFrame
    let mut focus_tot_frame = Frame::new(
        WidgetType::Frame,
        Some("FocusFrameToTFrame".to_string()),
        Some(focus_frame_id),
    );
    focus_tot_frame.width = 80.0;
    focus_tot_frame.height = 30.0;
    let focus_tot_frame_id = widgets.register(focus_tot_frame);

    let focus_tot_health_bar = Frame::new(
        WidgetType::StatusBar,
        Some("FocusFrameToTHealthBar".to_string()),
        Some(focus_tot_frame_id),
    );
    let focus_tot_health_bar_id = widgets.register(focus_tot_health_bar);

    // Create FocusFrameSpellBar (focus cast bar)
    let mut focus_spell_bar = Frame::new(
        WidgetType::StatusBar,
        Some("FocusFrameSpellBar".to_string()),
        Some(focus_frame_id),
    );
    focus_spell_bar.width = 150.0;
    focus_spell_bar.height = 16.0;
    widgets.register(focus_spell_bar);

    // Set up children_keys for FocusFrame hierarchy (uses TargetFrameContent name)
    if let Some(ff) = widgets.get_mut(focus_frame_id) {
        ff.children_keys
            .insert("TargetFrameContent".to_string(), focus_frame_content_id);
        ff.children_keys
            .insert("totFrame".to_string(), focus_tot_frame_id);
    }
    if let Some(ffc) = widgets.get_mut(focus_frame_content_id) {
        ffc.children_keys.insert(
            "TargetFrameContentMain".to_string(),
            focus_frame_content_main_id,
        );
    }
    if let Some(ffcm) = widgets.get_mut(focus_frame_content_main_id) {
        ffcm.children_keys.insert(
            "HealthBarsContainer".to_string(),
            focus_health_bars_container_id,
        );
        ffcm.children_keys
            .insert("ManaBar".to_string(), focus_mana_bar_id);
    }
    if let Some(hbc) = widgets.get_mut(focus_health_bars_container_id) {
        hbc.children_keys
            .insert("HealthBar".to_string(), focus_health_bar_id);
    }
    if let Some(tot) = widgets.get_mut(focus_tot_frame_id) {
        tot.children_keys
            .insert("HealthBar".to_string(), focus_tot_health_bar_id);
    }
}

fn create_buff_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Create BuffFrame (player buff display)
    let mut buff_frame = Frame::new(
        WidgetType::Frame,
        Some("BuffFrame".to_string()),
        Some(ui_parent_id),
    );
    buff_frame.width = 300.0;
    buff_frame.height = 100.0;
    let buff_frame_id = widgets.register(buff_frame);

    // Create BuffFrame.AuraContainer (child container for buff icons)
    let mut aura_container = Frame::new(
        WidgetType::Frame,
        Some("BuffFrameAuraContainer".to_string()),
        Some(buff_frame_id),
    );
    aura_container.width = 300.0;
    aura_container.height = 100.0;
    let aura_container_id = widgets.register(aura_container);

    // Link AuraContainer to BuffFrame's children_keys
    if let Some(bf) = widgets.get_mut(buff_frame_id) {
        bf.children_keys
            .insert("AuraContainer".to_string(), aura_container_id);
    }
}

fn create_pet_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Create PetFrame (pet unit frame)
    let mut pet_frame = Frame::new(
        WidgetType::Frame,
        Some("PetFrame".to_string()),
        Some(ui_parent_id),
    );
    pet_frame.width = 128.0;
    pet_frame.height = 53.0;
    let pet_frame_id = widgets.register(pet_frame);

    // Create PetFrame.healthbar
    let mut pet_healthbar = Frame::new(
        WidgetType::StatusBar,
        Some("PetFrameHealthBar".to_string()),
        Some(pet_frame_id),
    );
    pet_healthbar.width = 90.0;
    pet_healthbar.height = 12.0;
    let pet_healthbar_id = widgets.register(pet_healthbar);

    // Create text children on healthbar
    let mut pet_left_text = Frame::new(WidgetType::FontString, None, Some(pet_healthbar_id));
    pet_left_text.width = 40.0;
    pet_left_text.height = 12.0;
    let pet_left_text_id = widgets.register(pet_left_text);

    let mut pet_right_text = Frame::new(WidgetType::FontString, None, Some(pet_healthbar_id));
    pet_right_text.width = 40.0;
    pet_right_text.height = 12.0;
    let pet_right_text_id = widgets.register(pet_right_text);

    let mut pet_text_string = Frame::new(WidgetType::FontString, None, Some(pet_healthbar_id));
    pet_text_string.width = 80.0;
    pet_text_string.height = 12.0;
    let pet_text_string_id = widgets.register(pet_text_string);

    // Create PetFrame.manabar
    let pet_manabar = Frame::new(
        WidgetType::StatusBar,
        Some("PetFrameManaBar".to_string()),
        Some(pet_frame_id),
    );
    let pet_manabar_id = widgets.register(pet_manabar);

    // Create text children for pet mana bar
    let pet_mana_left_text = Frame::new(WidgetType::FontString, None, Some(pet_manabar_id));
    let pet_mana_left_text_id = widgets.register(pet_mana_left_text);

    let pet_mana_right_text = Frame::new(WidgetType::FontString, None, Some(pet_manabar_id));
    let pet_mana_right_text_id = widgets.register(pet_mana_right_text);

    let pet_mana_text_string = Frame::new(WidgetType::FontString, None, Some(pet_manabar_id));
    let pet_mana_text_string_id = widgets.register(pet_mana_text_string);

    // Set up children_keys for PetFrame
    if let Some(pf) = widgets.get_mut(pet_frame_id) {
        pf.children_keys
            .insert("healthbar".to_string(), pet_healthbar_id);
        pf.children_keys
            .insert("manabar".to_string(), pet_manabar_id);
    }
    if let Some(hb) = widgets.get_mut(pet_healthbar_id) {
        hb.children_keys
            .insert("LeftText".to_string(), pet_left_text_id);
        hb.children_keys
            .insert("RightText".to_string(), pet_right_text_id);
        hb.children_keys
            .insert("TextString".to_string(), pet_text_string_id);
    }
    if let Some(mb) = widgets.get_mut(pet_manabar_id) {
        mb.children_keys
            .insert("LeftText".to_string(), pet_mana_left_text_id);
        mb.children_keys
            .insert("RightText".to_string(), pet_mana_right_text_id);
        mb.children_keys
            .insert("TextString".to_string(), pet_mana_text_string_id);
    }
}

fn create_misc_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Create Minimap (minimap frame) - note: duplicate, but matches original
    let mut minimap = Frame::new(
        WidgetType::Frame,
        Some("Minimap".to_string()),
        Some(ui_parent_id),
    );
    minimap.width = 140.0;
    minimap.height = 140.0;
    widgets.register(minimap);

    // Create MinimapCluster (minimap container frame)
    let mut minimap_cluster = Frame::new(
        WidgetType::Frame,
        Some("MinimapCluster".to_string()),
        Some(ui_parent_id),
    );
    minimap_cluster.width = 192.0;
    minimap_cluster.height = 192.0;
    widgets.register(minimap_cluster);

    // Create ObjectiveTrackerFrame (quest/objectives tracker)
    let mut objective_tracker = Frame::new(
        WidgetType::Frame,
        Some("ObjectiveTrackerFrame".to_string()),
        Some(ui_parent_id),
    );
    objective_tracker.width = 248.0;
    objective_tracker.height = 600.0;
    widgets.register(objective_tracker);

    // Create SettingsPanel (game settings UI)
    let mut settings_panel = Frame::new(
        WidgetType::Frame,
        Some("SettingsPanel".to_string()),
        Some(ui_parent_id),
    );
    settings_panel.width = 800.0;
    settings_panel.height = 600.0;
    settings_panel.visible = false; // Hidden by default
    let settings_panel_id = widgets.register(settings_panel);

    // Create SettingsPanel.FrameContainer (child container for settings content)
    let mut frame_container = Frame::new(
        WidgetType::Frame,
        Some("SettingsPanelFrameContainer".to_string()),
        Some(settings_panel_id),
    );
    frame_container.width = 780.0;
    frame_container.height = 580.0;
    let frame_container_id = widgets.register(frame_container);

    // Link FrameContainer to SettingsPanel's children_keys
    if let Some(sp) = widgets.get_mut(settings_panel_id) {
        sp.children_keys
            .insert("FrameContainer".to_string(), frame_container_id);
    }

    // Create PlayerCastingBarFrame (player cast bar)
    let mut player_casting_bar = Frame::new(
        WidgetType::StatusBar,
        Some("PlayerCastingBarFrame".to_string()),
        Some(ui_parent_id),
    );
    player_casting_bar.width = 200.0;
    player_casting_bar.height = 20.0;
    widgets.register(player_casting_bar);

    // Create PartyFrame (container for party member frames)
    let mut party_frame = Frame::new(
        WidgetType::Frame,
        Some("PartyFrame".to_string()),
        Some(ui_parent_id),
    );
    party_frame.width = 200.0;
    party_frame.height = 400.0;
    widgets.register(party_frame);

    // Create AlternatePowerBar (alternate power resource bar)
    let alternate_power_bar = Frame::new(
        WidgetType::StatusBar,
        Some("AlternatePowerBar".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(alternate_power_bar);

    // Create MonkStaggerBar (monk stagger resource bar)
    let monk_stagger_bar = Frame::new(
        WidgetType::StatusBar,
        Some("MonkStaggerBar".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(monk_stagger_bar);

    create_lfg_frames(widgets, ui_parent_id);
    create_utility_frames(widgets, ui_parent_id);
}

fn create_lfg_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Create LFGListFrame (Looking For Group list frame)
    let mut lfg_list_frame = Frame::new(
        WidgetType::Frame,
        Some("LFGListFrame".to_string()),
        Some(ui_parent_id),
    );
    lfg_list_frame.width = 400.0;
    lfg_list_frame.height = 500.0;
    lfg_list_frame.visible = false;
    let lfg_list_frame_id = widgets.register(lfg_list_frame);

    // Create LFGListFrame.SearchPanel (the search panel child)
    let mut lfg_search_panel = Frame::new(
        WidgetType::Frame,
        Some("LFGListSearchPanel".to_string()),
        Some(lfg_list_frame_id),
    );
    lfg_search_panel.width = 380.0;
    lfg_search_panel.height = 450.0;
    let lfg_search_panel_id = widgets.register(lfg_search_panel);

    // Create ScrollFrame child for SearchPanel
    let mut lfg_scroll_frame = Frame::new(
        WidgetType::ScrollFrame,
        Some("LFGListSearchPanelScrollFrame".to_string()),
        Some(lfg_search_panel_id),
    );
    lfg_scroll_frame.width = 360.0;
    lfg_scroll_frame.height = 400.0;
    let lfg_scroll_frame_id = widgets.register(lfg_scroll_frame);

    // Add children_keys for LFGListFrame
    if let Some(lfg) = widgets.get_mut(lfg_list_frame_id) {
        lfg.children_keys
            .insert("SearchPanel".to_string(), lfg_search_panel_id);
    }

    // Add ScrollFrame to SearchPanel's children_keys
    if let Some(sp) = widgets.get_mut(lfg_search_panel_id) {
        sp.children_keys
            .insert("ScrollFrame".to_string(), lfg_scroll_frame_id);
    }
}

fn create_utility_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Create AlertFrame (alert/popup management frame)
    let alert_frame = Frame::new(
        WidgetType::Frame,
        Some("AlertFrame".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(alert_frame);

    // Create LFGEventFrame (LFG event handling frame)
    let lfg_event_frame = Frame::new(
        WidgetType::Frame,
        Some("LFGEventFrame".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(lfg_event_frame);

    // Create NamePlateDriverFrame (nameplate management frame)
    let nameplate_driver_frame = Frame::new(
        WidgetType::Frame,
        Some("NamePlateDriverFrame".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(nameplate_driver_frame);

    // Create UIErrorsFrame (error message display frame)
    let ui_errors_frame = Frame::new(
        WidgetType::MessageFrame,
        Some("UIErrorsFrame".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(ui_errors_frame);

    // Create InterfaceOptionsFrame (legacy interface options)
    let interface_options_frame = Frame::new(
        WidgetType::Frame,
        Some("InterfaceOptionsFrame".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(interface_options_frame);

    // Create AuctionHouseFrame (auction house UI)
    let auction_house_frame = Frame::new(
        WidgetType::Frame,
        Some("AuctionHouseFrame".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(auction_house_frame);

    // Create SideDressUpFrame (side dressing room)
    let side_dressup_frame = Frame::new(
        WidgetType::Frame,
        Some("SideDressUpFrame".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(side_dressup_frame);

    // Create ContainerFrameContainer (bag frame container for combined bags)
    let container_frame_container = Frame::new(
        WidgetType::Frame,
        Some("ContainerFrameContainer".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(container_frame_container);

    // Create ContainerFrameCombinedBags (combined bag frame)
    let container_combined_bags = Frame::new(
        WidgetType::Frame,
        Some("ContainerFrameCombinedBags".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(container_combined_bags);

    // Create LootFrame (loot window)
    let loot_frame = Frame::new(
        WidgetType::Frame,
        Some("LootFrame".to_string()),
        Some(ui_parent_id),
    );
    let loot_frame_id = loot_frame.id;
    widgets.register(loot_frame);

    // Create LootFrame.ScrollBox (scroll container for loot items)
    let loot_scroll_box = Frame::new(
        WidgetType::ScrollFrame,
        Some("LootFrameScrollBox".to_string()),
        Some(loot_frame_id),
    );
    let loot_scroll_box_id = loot_scroll_box.id;
    widgets.register(loot_scroll_box);

    // Add children_keys for LootFrame.ScrollBox access
    if let Some(lf) = widgets.get_mut(loot_frame_id) {
        lf.children_keys
            .insert("ScrollBox".to_string(), loot_scroll_box_id);
    }

    // Create ScenarioObjectiveTracker (objective tracker for scenarios/M+)
    let scenario_tracker = Frame::new(
        WidgetType::Frame,
        Some("ScenarioObjectiveTracker".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(scenario_tracker);

    // Create RaidWarningFrame (raid warning message display)
    let raid_warning_frame = Frame::new(
        WidgetType::MessageFrame,
        Some("RaidWarningFrame".to_string()),
        Some(ui_parent_id),
    );
    widgets.register(raid_warning_frame);

    // Create GossipFrame (NPC interaction dialog)
    let mut gossip_frame = Frame::new(
        WidgetType::Frame,
        Some("GossipFrame".to_string()),
        Some(ui_parent_id),
    );
    gossip_frame.width = 338.0;
    gossip_frame.height = 479.0;
    widgets.register(gossip_frame);

    // Create QuestFrame (quest interaction dialog)
    let mut quest_frame = Frame::new(
        WidgetType::Frame,
        Some("QuestFrame".to_string()),
        Some(ui_parent_id),
    );
    quest_frame.width = 384.0;
    quest_frame.height = 512.0;
    widgets.register(quest_frame);
}
