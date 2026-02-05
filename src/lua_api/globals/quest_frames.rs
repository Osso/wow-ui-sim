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
    let globals = lua.globals();

    // Create QuestFrame - main quest dialog frame
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
        state: Rc::clone(&state),
    };
    let quest_ud = lua.create_userdata(handle)?;

    globals.set("QuestFrame", quest_ud.clone())?;
    let frame_key = format!("__frame_{}", quest_id);
    globals.set(frame_key.as_str(), quest_ud)?;

    // Create QuestFrameRewardPanel (used by WorldQuestTracker)
    let mut reward_panel = Frame::new(
        WidgetType::Frame,
        Some("QuestFrameRewardPanel".to_string()),
        Some(quest_id),
    );
    reward_panel.visible = false;
    let reward_id = reward_panel.id;
    state.borrow_mut().widgets.register(reward_panel);
    state.borrow_mut().widgets.add_child(quest_id, reward_id);

    let reward_handle = FrameHandle {
        id: reward_id,
        state: Rc::clone(&state),
    };
    let reward_ud = lua.create_userdata(reward_handle)?;
    globals.set("QuestFrameRewardPanel", reward_ud.clone())?;
    globals.set(format!("__frame_{}", reward_id).as_str(), reward_ud)?;

    // Create QuestFrameCompleteQuestButton (used by WorldQuestTracker)
    let mut complete_btn = Frame::new(
        WidgetType::Button,
        Some("QuestFrameCompleteQuestButton".to_string()),
        Some(quest_id),
    );
    complete_btn.visible = false;
    let btn_id = complete_btn.id;
    state.borrow_mut().widgets.register(complete_btn);
    state.borrow_mut().widgets.add_child(quest_id, btn_id);

    let btn_handle = FrameHandle {
        id: btn_id,
        state: Rc::clone(&state),
    };
    let btn_ud = lua.create_userdata(btn_handle)?;
    globals.set("QuestFrameCompleteQuestButton", btn_ud.clone())?;
    globals.set(format!("__frame_{}", btn_id).as_str(), btn_ud)?;

    // Create QuestFrameDetailPanel (used by WorldQuestTracker)
    let mut detail_panel = Frame::new(
        WidgetType::Frame,
        Some("QuestFrameDetailPanel".to_string()),
        Some(quest_id),
    );
    detail_panel.visible = false;
    let detail_id = detail_panel.id;
    state.borrow_mut().widgets.register(detail_panel);
    state.borrow_mut().widgets.add_child(quest_id, detail_id);

    let detail_handle = FrameHandle {
        id: detail_id,
        state: Rc::clone(&state),
    };
    let detail_ud = lua.create_userdata(detail_handle)?;
    globals.set("QuestFrameDetailPanel", detail_ud.clone())?;
    globals.set(format!("__frame_{}", detail_id).as_str(), detail_ud)?;

    // Create QuestFrameProgressPanel (used by WorldQuestTracker)
    let mut progress_panel = Frame::new(
        WidgetType::Frame,
        Some("QuestFrameProgressPanel".to_string()),
        Some(quest_id),
    );
    progress_panel.visible = false;
    let progress_id = progress_panel.id;
    state.borrow_mut().widgets.register(progress_panel);
    state.borrow_mut().widgets.add_child(quest_id, progress_id);

    let progress_handle = FrameHandle {
        id: progress_id,
        state: Rc::clone(&state),
    };
    let progress_ud = lua.create_userdata(progress_handle)?;
    globals.set("QuestFrameProgressPanel", progress_ud.clone())?;
    globals.set(format!("__frame_{}", progress_id).as_str(), progress_ud)?;

    // Create QuestFrameAcceptButton (used by WorldQuestTracker)
    let mut accept_btn = Frame::new(
        WidgetType::Button,
        Some("QuestFrameAcceptButton".to_string()),
        Some(quest_id),
    );
    accept_btn.visible = false;
    let accept_id = accept_btn.id;
    state.borrow_mut().widgets.register(accept_btn);
    state.borrow_mut().widgets.add_child(quest_id, accept_id);

    let accept_handle = FrameHandle {
        id: accept_id,
        state: Rc::clone(&state),
    };
    let accept_ud = lua.create_userdata(accept_handle)?;
    globals.set("QuestFrameAcceptButton", accept_ud.clone())?;
    globals.set(format!("__frame_{}", accept_id).as_str(), accept_ud)?;

    // Create QuestFrameCompleteButton (used by WorldQuestTracker)
    let mut quest_complete_btn = Frame::new(
        WidgetType::Button,
        Some("QuestFrameCompleteButton".to_string()),
        Some(quest_id),
    );
    quest_complete_btn.visible = false;
    let quest_complete_id = quest_complete_btn.id;
    state.borrow_mut().widgets.register(quest_complete_btn);
    state.borrow_mut().widgets.add_child(quest_id, quest_complete_id);

    let quest_complete_handle = FrameHandle {
        id: quest_complete_id,
        state: Rc::clone(&state),
    };
    let quest_complete_ud = lua.create_userdata(quest_complete_handle)?;
    globals.set("QuestFrameCompleteButton", quest_complete_ud.clone())?;
    globals.set(
        format!("__frame_{}", quest_complete_id).as_str(),
        quest_complete_ud,
    )?;

    Ok(())
}
