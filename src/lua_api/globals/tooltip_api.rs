//! Tooltip frame creation (GameTooltip, ItemRefTooltip, ShoppingTooltip, etc.)

use crate::lua_api::frame::FrameHandle;
use crate::lua_api::SimState;
use crate::widget::{Frame, WidgetType};
use mlua::{Lua, Result};
use std::cell::RefCell;
use std::rc::Rc;

/// Registers built-in tooltip frames in the global namespace.
///
/// Creates the following frames:
/// - `GameTooltip` - Main tooltip used by most addons
/// - `ItemRefTooltip` - Tooltip for item links clicked in chat
/// - `ItemRefShoppingTooltip1/2` - Comparison tooltips for item links
/// - `ShoppingTooltip1/2` - Comparison tooltips for GameTooltip
/// - `FriendsTooltip` - Tooltip for friends list
/// - `FriendsListFrame` - Friends list UI frame
pub fn register_tooltip_frames(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    // Create GameTooltip - a built-in tooltip frame with special methods
    // GameTooltip is used by almost all addons for displaying tooltips
    {
        // Create the GameTooltip frame in the widget registry
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some("GameTooltip".to_string()),
            ui_parent_id,
        );
        tooltip_frame.visible = false; // Hidden by default
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;

        // Create NineSlice child frame (required by SharedTooltipTemplates)
        let nine_slice = Frame::new(WidgetType::Frame, None, Some(tooltip_id));
        let nine_slice_id = nine_slice.id;
        tooltip_frame.children.push(nine_slice_id);
        tooltip_frame
            .children_keys
            .insert("NineSlice".to_string(), nine_slice_id);

        {
            let mut s = state.borrow_mut();
            s.widgets.register(nine_slice);
            s.widgets.register(tooltip_frame);
        }

        // Create the FrameHandle for GameTooltip
        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        // Store in globals
        globals.set("GameTooltip", tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create ItemRefTooltip - tooltip for item links clicked in chat
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some("ItemRefTooltip".to_string()),
            ui_parent_id,
        );
        tooltip_frame.visible = false;
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;
        state.borrow_mut().widgets.register(tooltip_frame);

        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        globals.set("ItemRefTooltip", tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create ItemRefShoppingTooltip1/2 - comparison tooltips for item links
    for i in 1..=2 {
        let tooltip_name = format!("ItemRefShoppingTooltip{}", i);
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some(tooltip_name.clone()),
            ui_parent_id,
        );
        tooltip_frame.visible = false;
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;
        state.borrow_mut().widgets.register(tooltip_frame);

        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        globals.set(tooltip_name.as_str(), tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create ShoppingTooltip1/2 - comparison tooltips for GameTooltip
    for i in 1..=2 {
        let tooltip_name = format!("ShoppingTooltip{}", i);
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some(tooltip_name.clone()),
            ui_parent_id,
        );
        tooltip_frame.visible = false;
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;
        state.borrow_mut().widgets.register(tooltip_frame);

        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        globals.set(tooltip_name.as_str(), tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create FriendsTooltip - tooltip for friends list
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut tooltip_frame = Frame::new(
            WidgetType::Frame,
            Some("FriendsTooltip".to_string()),
            ui_parent_id,
        );
        tooltip_frame.visible = false;
        tooltip_frame.width = 200.0;
        tooltip_frame.height = 100.0;
        let tooltip_id = tooltip_frame.id;
        state.borrow_mut().widgets.register(tooltip_frame);

        let handle = FrameHandle {
            id: tooltip_id,
            state: Rc::clone(&state),
        };
        let tooltip_ud = lua.create_userdata(handle)?;

        globals.set("FriendsTooltip", tooltip_ud.clone())?;
        let frame_key = format!("__frame_{}", tooltip_id);
        globals.set(frame_key.as_str(), tooltip_ud)?;
    }

    // Create FriendsListFrame - friends list UI frame
    {
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut friends_frame = Frame::new(
            WidgetType::Frame,
            Some("FriendsListFrame".to_string()),
            ui_parent_id,
        );
        friends_frame.visible = false;
        friends_frame.width = 300.0;
        friends_frame.height = 400.0;
        let friends_id = friends_frame.id;
        state.borrow_mut().widgets.register(friends_frame);

        // Create ScrollBox child (used by addons to iterate friend buttons)
        let scrollbox = Frame::new(WidgetType::Frame, None, Some(friends_id));
        let scrollbox_id = scrollbox.id;
        state.borrow_mut().widgets.register(scrollbox);
        state.borrow_mut().widgets.add_child(friends_id, scrollbox_id);
        if let Some(f) = state.borrow_mut().widgets.get_mut(friends_id) {
            f.children_keys.insert("ScrollBox".to_string(), scrollbox_id);
        }

        let handle = FrameHandle {
            id: friends_id,
            state: Rc::clone(&state),
        };
        let friends_ud = lua.create_userdata(handle)?;

        globals.set("FriendsListFrame", friends_ud.clone())?;
        let frame_key = format!("__frame_{}", friends_id);
        globals.set(frame_key.as_str(), friends_ud)?;
    }

    Ok(())
}
