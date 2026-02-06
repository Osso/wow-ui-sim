//! Tooltip frame creation (GameTooltip, ItemRefTooltip, ShoppingTooltip, etc.)

use crate::lua_api::frame::FrameHandle;
use crate::lua_api::tooltip::TooltipData;
use crate::lua_api::SimState;
use crate::widget::{Frame, FrameStrata, WidgetType};
use mlua::{Lua, Result};
use std::cell::RefCell;
use std::rc::Rc;

/// Create a tooltip frame, register it, and store in Lua globals.
fn create_tooltip_frame(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    name: &str,
) -> Result<u64> {
    let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
    let mut frame = Frame::new(
        WidgetType::GameTooltip,
        Some(name.to_string()),
        ui_parent_id,
    );
    frame.visible = false;
    frame.width = 200.0;
    frame.height = 100.0;
    frame.frame_strata = FrameStrata::Tooltip;
    frame.has_fixed_frame_strata = true;
    let frame_id = frame.id;

    {
        let mut s = state.borrow_mut();
        s.widgets.register(frame);
        s.tooltips.insert(frame_id, TooltipData::default());
    }

    let handle = FrameHandle {
        id: frame_id,
        state: Rc::clone(state),
    };
    let ud = lua.create_userdata(handle)?;

    let globals = lua.globals();
    globals.set(name, ud.clone())?;
    globals.set(format!("__frame_{}", frame_id).as_str(), ud)?;

    Ok(frame_id)
}

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
    register_game_tooltips(lua, &state)?;
    register_friends_list_frame(lua, &state)?;
    Ok(())
}

/// Register GameTooltip (with NineSlice child) and all other tooltip frames.
fn register_game_tooltips(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    // GameTooltip - needs a NineSlice child frame for SharedTooltipTemplates
    let gt_id = create_tooltip_frame(lua, state, "GameTooltip")?;
    {
        let nine_slice = Frame::new(WidgetType::Frame, None, Some(gt_id));
        let nine_slice_id = nine_slice.id;
        let mut s = state.borrow_mut();
        s.widgets.register(nine_slice);
        s.widgets.add_child(gt_id, nine_slice_id);
        if let Some(f) = s.widgets.get_mut(gt_id) {
            f.children_keys
                .insert("NineSlice".to_string(), nine_slice_id);
        }
    }

    create_tooltip_frame(lua, state, "ItemRefTooltip")?;
    for i in 1..=2 {
        create_tooltip_frame(lua, state, &format!("ItemRefShoppingTooltip{}", i))?;
    }
    for i in 1..=2 {
        create_tooltip_frame(lua, state, &format!("ShoppingTooltip{}", i))?;
    }
    create_tooltip_frame(lua, state, "FriendsTooltip")?;
    Ok(())
}

/// Register FriendsListFrame with a ScrollBox child.
fn register_friends_list_frame(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
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

    let scrollbox = Frame::new(WidgetType::Frame, None, Some(friends_id));
    let scrollbox_id = scrollbox.id;
    {
        let mut s = state.borrow_mut();
        s.widgets.register(scrollbox);
        s.widgets.add_child(friends_id, scrollbox_id);
        if let Some(f) = s.widgets.get_mut(friends_id) {
            f.children_keys
                .insert("ScrollBox".to_string(), scrollbox_id);
        }
    }

    let handle = FrameHandle {
        id: friends_id,
        state: Rc::clone(state),
    };
    let ud = lua.create_userdata(handle)?;
    globals.set("FriendsListFrame", ud.clone())?;
    globals.set(format!("__frame_{}", friends_id).as_str(), ud)?;
    Ok(())
}
