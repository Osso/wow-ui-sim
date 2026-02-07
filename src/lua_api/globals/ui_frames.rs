//! UI frame creation for standard WoW UI elements.
//!
//! This module creates standard WoW UI frames that addons expect to exist:
//! - MicroButtons (MainMenuMicroButton, CharacterMicroButton, etc.)
//! - MerchantFrame and MerchantItem buttons
//! - Action bars (MainActionBar, MultiBarBottomLeft, etc.)
//! - Status tracking bars (StatusTrackingBarManager, etc.)
//! - Raid frames (CompactRaidFrameContainer)

use crate::lua_api::frame::FrameHandle;
use crate::lua_api::SimState;
use crate::widget::{Frame, WidgetType};
use mlua::{Lua, Result};
use std::cell::RefCell;
use std::rc::Rc;

/// Register standard UI frames that addons expect to exist.
pub fn register_ui_frames(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_micro_buttons(lua, &state)?;
    register_merchant_frames(lua, &state)?;
    register_action_bars(lua, &state)?;
    register_status_tracking_bars(lua, &state)?;
    register_raid_frames(lua, &state)?;
    eprintln!("DEBUG: after CompactRaidFrameContainer");
    Ok(())
}

/// Helper to create a named frame, register it, and set it as a Lua global.
#[allow(clippy::too_many_arguments)]
fn create_and_register_frame(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    name: &str,
    widget_type: WidgetType,
    parent_id: Option<u64>,
    width: f32,
    height: f32,
    visible: bool,
) -> Result<u64> {
    let mut frame = Frame::new(widget_type, Some(name.to_string()), parent_id);
    frame.visible = visible;
    frame.width = width;
    frame.height = height;
    let frame_id = frame.id;
    state.borrow_mut().widgets.register(frame);

    let handle = FrameHandle {
        id: frame_id,
        state: Rc::clone(state),
    };
    let ud = lua.create_userdata(handle)?;
    let globals = lua.globals();
    globals.set(name, ud.clone())?;
    let frame_key = format!("__frame_{}", frame_id);
    globals.set(frame_key.as_str(), ud)?;
    Ok(frame_id)
}

/// Register MainMenuMicroButton and other micro buttons.
fn register_micro_buttons(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");

    create_and_register_frame(
        lua, state, "MainMenuMicroButton", WidgetType::Button,
        ui_parent_id, 28.0, 36.0, true,
    )?;

    let micro_buttons = [
        "CharacterMicroButton",
        "ProfessionMicroButton",
        "PlayerSpellsMicroButton",
        "AchievementMicroButton",
        "QuestLogMicroButton",
        "GuildMicroButton",
        "LFDMicroButton",
        "CollectionsMicroButton",
        "EJMicroButton",
        "StoreMicroButton",
        "HelpMicroButton",
        "HousingMicroButton",
    ];

    for name in micro_buttons {
        create_and_register_frame(
            lua, state, name, WidgetType::Button,
            ui_parent_id, 28.0, 36.0, true,
        )?;
    }

    Ok(())
}

/// Register MerchantFrame and MerchantItem1-12 buttons.
fn register_merchant_frames(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");

    let merchant_id = create_and_register_frame(
        lua, state, "MerchantFrame", WidgetType::Frame,
        ui_parent_id, 336.0, 447.0, false,
    )?;

    for i in 1..=12 {
        let item_name = format!("MerchantItem{}", i);
        let mut item_frame = Frame::new(
            WidgetType::Button,
            Some(item_name.clone()),
            Some(merchant_id),
        );
        item_frame.visible = true;
        item_frame.width = 160.0;
        item_frame.height = 36.0;
        let item_id = item_frame.id;
        state.borrow_mut().widgets.register(item_frame);
        state.borrow_mut().widgets.add_child(merchant_id, item_id);

        let handle = FrameHandle {
            id: item_id,
            state: Rc::clone(state),
        };
        let item_ud = lua.create_userdata(handle)?;
        let globals = lua.globals();
        globals.set(item_name.as_str(), item_ud.clone())?;
        let frame_key = format!("__frame_{}", item_id);
        globals.set(frame_key.as_str(), item_ud)?;
    }

    Ok(())
}

/// Register action bar frames (MainActionBar, MultiBar*, etc.).
fn register_action_bars(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");

    let action_bars = [
        "MainActionBar",
        "MultiBarBottomLeft",
        "MultiBarBottomRight",
        "MultiBarRight",
        "MultiBarLeft",
        "MultiBar5",
        "MultiBar6",
        "MultiBar7",
        "MultiBar8",
        "StanceBar",
        "PetActionBar",
        "PossessBar",
        "OverrideActionBar",
    ];

    for name in action_bars {
        create_and_register_frame(
            lua, state, name, WidgetType::Frame,
            ui_parent_id, 500.0, 40.0, true,
        )?;
        // ActionBar_OnLoad expects self.actionButtons to be iterable via pairs()
        let code = format!("{}.actionButtons = {{}}", name);
        let _ = lua.load(&code).exec();
    }

    Ok(())
}

/// Register StatusTrackingBarManager and related container frames.
fn register_status_tracking_bars(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");

    // StatusTrackingBarManager with bars table
    create_and_register_frame(
        lua, state, "StatusTrackingBarManager", WidgetType::Frame,
        ui_parent_id, 800.0, 14.0, true,
    )?;
    let bars_table = lua.create_table()?;
    lua.load(r#"
        StatusTrackingBarManager.bars = ...
    "#).call::<()>(bars_table)?;

    // Main/Secondary containers with bars tables
    let containers = [
        "MainStatusTrackingBarContainer",
        "SecondaryStatusTrackingBarContainer",
    ];
    for name in containers {
        create_and_register_frame(
            lua, state, name, WidgetType::Frame,
            ui_parent_id, 800.0, 14.0, true,
        )?;
        let bars_table = lua.create_table()?;
        lua.load(format!(r#"
            {}.bars = ...
        "#, name).as_str()).call::<()>(bars_table)?;
    }

    Ok(())
}

/// Register CompactRaidFrameContainer.
fn register_raid_frames(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");

    let mut container = Frame::new(
        WidgetType::Frame,
        Some("CompactRaidFrameContainer".to_string()),
        ui_parent_id,
    );
    container.visible = true;
    container.width = 200.0;
    container.height = 200.0;
    let container_id = container.id;
    state.borrow_mut().widgets.register(container);

    let handle = FrameHandle {
        id: container_id,
        state: Rc::clone(state),
    };
    let container_ud = lua.create_userdata(handle)?;
    lua.globals().set("CompactRaidFrameContainer", container_ud)?;

    Ok(())
}
