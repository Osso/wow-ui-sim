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
    let globals = lua.globals();

    // Create MainMenuMicroButton - main menu micro button
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut micro_btn = Frame::new(
            WidgetType::Button,
            Some("MainMenuMicroButton".to_string()),
            ui_parent_id,
        );
        micro_btn.visible = true;
        micro_btn.width = 28.0;
        micro_btn.height = 36.0;
        let micro_id = micro_btn.id;
        state.borrow_mut().widgets.register(micro_btn);

        let handle = FrameHandle {
            id: micro_id,
            state: Rc::clone(&state),
        };
        let micro_ud = lua.create_userdata(handle)?;

        globals.set("MainMenuMicroButton", micro_ud.clone())?;
        let frame_key = format!("__frame_{}", micro_id);
        globals.set(frame_key.as_str(), micro_ud)?;
    }

    // Create other MicroButtons - UI micro menu buttons (used by EditModeExpanded and others)
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
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
            let mut btn = Frame::new(
                WidgetType::Button,
                Some(name.to_string()),
                ui_parent_id,
            );
            btn.visible = true;
            btn.width = 28.0;
            btn.height = 36.0;
            let btn_id = btn.id;
            state.borrow_mut().widgets.register(btn);

            let handle = FrameHandle {
                id: btn_id,
                state: Rc::clone(&state),
            };
            let btn_ud = lua.create_userdata(handle)?;
            globals.set(name, btn_ud.clone())?;
            let frame_key = format!("__frame_{}", btn_id);
            globals.set(frame_key.as_str(), btn_ud)?;
        }
    }

    // Create MerchantFrame and MerchantItem1-12 - merchant UI frames
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut merchant_frame = Frame::new(
            WidgetType::Frame,
            Some("MerchantFrame".to_string()),
            ui_parent_id,
        );
        merchant_frame.visible = false;
        merchant_frame.width = 336.0;
        merchant_frame.height = 447.0;
        let merchant_id = merchant_frame.id;
        state.borrow_mut().widgets.register(merchant_frame);

        let handle = FrameHandle {
            id: merchant_id,
            state: Rc::clone(&state),
        };
        let merchant_ud = lua.create_userdata(handle)?;
        globals.set("MerchantFrame", merchant_ud.clone())?;
        let frame_key = format!("__frame_{}", merchant_id);
        globals.set(frame_key.as_str(), merchant_ud)?;

        // Create MerchantItem1-12 button frames
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
                state: Rc::clone(&state),
            };
            let item_ud = lua.create_userdata(handle)?;
            globals.set(item_name.as_str(), item_ud.clone())?;
            let frame_key = format!("__frame_{}", item_id);
            globals.set(frame_key.as_str(), item_ud)?;
        }
    }

    // Create action bar frames - used by EditModeExpanded and other addons
    {
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
            let mut bar = Frame::new(
                WidgetType::Frame,
                Some(name.to_string()),
                ui_parent_id,
            );
            bar.visible = true;
            bar.width = 500.0;
            bar.height = 40.0;
            let bar_id = bar.id;
            state.borrow_mut().widgets.register(bar);

            let handle = FrameHandle {
                id: bar_id,
                state: Rc::clone(&state),
            };
            let bar_ud = lua.create_userdata(handle)?;
            globals.set(name, bar_ud.clone())?;
            let frame_key = format!("__frame_{}", bar_id);
            globals.set(frame_key.as_str(), bar_ud)?;
        }
    }

    // Create StatusTrackingBarManager - manages XP/rep/honor bars (used by DynamicCam)
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut bar_mgr = Frame::new(
            WidgetType::Frame,
            Some("StatusTrackingBarManager".to_string()),
            ui_parent_id,
        );
        bar_mgr.visible = true;
        bar_mgr.width = 800.0;
        bar_mgr.height = 14.0;
        let bar_mgr_id = bar_mgr.id;
        state.borrow_mut().widgets.register(bar_mgr);

        let handle = FrameHandle {
            id: bar_mgr_id,
            state: Rc::clone(&state),
        };
        let bar_mgr_ud = lua.create_userdata(handle)?;
        globals.set("StatusTrackingBarManager", bar_mgr_ud.clone())?;

        // Add bars table for DynamicCam - set via Lua so it's accessible
        let bars_table = lua.create_table()?;
        lua.load(r#"
            StatusTrackingBarManager.bars = ...
        "#).call::<()>(bars_table)?;
    }

    // Create MainStatusTrackingBarContainer and SecondaryStatusTrackingBarContainer (used by DynamicCam)
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let containers = [
            "MainStatusTrackingBarContainer",
            "SecondaryStatusTrackingBarContainer",
        ];

        for name in containers {
            let mut container = Frame::new(
                WidgetType::Frame,
                Some(name.to_string()),
                ui_parent_id,
            );
            container.visible = true;
            container.width = 800.0;
            container.height = 14.0;
            let container_id = container.id;
            state.borrow_mut().widgets.register(container);

            let handle = FrameHandle {
                id: container_id,
                state: Rc::clone(&state),
            };
            let container_ud = lua.create_userdata(handle)?;
            globals.set(name, container_ud.clone())?;

            // Add bars table for DynamicCam
            let bars_table = lua.create_table()?;
            lua.load(format!(r#"
                {}.bars = ...
            "#, name).as_str()).call::<()>(bars_table)?;
        }
    }

    // Create CompactRaidFrameContainer (used by DynamicCam for raid frame hiding)
    {
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
            state: Rc::clone(&state),
        };
        let container_ud = lua.create_userdata(handle)?;
        globals.set("CompactRaidFrameContainer", container_ud)?;
    }
    eprintln!("DEBUG: after CompactRaidFrameContainer");

    Ok(())
}
