//! Quest frame creation for WoW UI simulation.
//!
//! Creates QuestFrame and related child frames used by quest-related addons
//! (WorldQuestTracker, etc.).

use super::super::frame::FrameHandle;
use super::super::SimState;
use crate::widget::{Frame, WidgetType};
use mlua::{Lua, Result};
use std::cell::RefCell;
use std::rc::Rc;

/// Register quest-related frames.
pub fn register_quest_frames(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let quest_id = register_quest_frame(lua, &state)?;
    register_quest_panels(lua, &state, quest_id)?;
    register_quest_buttons(lua, &state, quest_id)?;
    Ok(())
}

/// Helper to create a child frame, register it, and set it as a Lua global.
fn create_child_frame(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    name: &str,
    widget_type: WidgetType,
    parent_id: u64,
) -> Result<u64> {
    let mut frame = Frame::new(widget_type, Some(name.to_string()), Some(parent_id));
    frame.visible = false;
    let frame_id = frame.id;
    state.borrow_mut().widgets.register(frame);
    state.borrow_mut().widgets.add_child(parent_id, frame_id);

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

/// Create the main QuestFrame.
fn register_quest_frame(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<u64> {
    let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
    let mut quest_frame = Frame::new(
        WidgetType::Frame,
        Some("QuestFrame".to_string()),
        ui_parent_id,
    );
    quest_frame.visible = false;
    quest_frame.width = 350.0;
    quest_frame.height = 450.0;
    let quest_id = quest_frame.id;
    state.borrow_mut().widgets.register(quest_frame);

    let handle = FrameHandle {
        id: quest_id,
        state: Rc::clone(state),
    };
    let quest_ud = lua.create_userdata(handle)?;
    let globals = lua.globals();
    globals.set("QuestFrame", quest_ud.clone())?;
    globals.set(format!("__frame_{}", quest_id).as_str(), quest_ud)?;

    Ok(quest_id)
}

/// Register quest panel child frames (Reward, Detail, Progress).
fn register_quest_panels(lua: &Lua, state: &Rc<RefCell<SimState>>, quest_id: u64) -> Result<()> {
    create_child_frame(lua, state, "QuestFrameRewardPanel", WidgetType::Frame, quest_id)?;
    create_child_frame(lua, state, "QuestFrameDetailPanel", WidgetType::Frame, quest_id)?;
    create_child_frame(lua, state, "QuestFrameProgressPanel", WidgetType::Frame, quest_id)?;
    Ok(())
}

/// Register quest button child frames (CompleteQuest, Accept, Complete).
fn register_quest_buttons(lua: &Lua, state: &Rc<RefCell<SimState>>, quest_id: u64) -> Result<()> {
    create_child_frame(lua, state, "QuestFrameCompleteQuestButton", WidgetType::Button, quest_id)?;
    create_child_frame(lua, state, "QuestFrameAcceptButton", WidgetType::Button, quest_id)?;
    create_child_frame(lua, state, "QuestFrameCompleteButton", WidgetType::Button, quest_id)?;
    Ok(())
}
