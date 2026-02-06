//! Global frame object registrations.
//!
//! This module registers all global frame objects that are expected to exist
//! in the WoW UI environment, such as UIParent, WorldFrame, PlayerFrame, etc.

use crate::lua_api::frame::FrameHandle;
use crate::lua_api::SimState;
use crate::widget::{Frame, WidgetType};
use mlua::{Lua, ObjectLike, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Look up a named widget and register it as a Lua global.
fn register_frame_global(lua: &Lua, state: &Rc<RefCell<SimState>>, name: &str) -> Result<u64> {
    let id = {
        let st = state.borrow();
        st.widgets.get_id_by_name(name).unwrap()
    };
    let ud = lua.create_userdata(FrameHandle {
        id,
        state: Rc::clone(state),
    })?;
    lua.globals().set(name, ud)?;
    Ok(id)
}

/// Create a new frame widget and register it as a Lua global.
fn create_and_register_frame_global(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    name: &str,
) -> Result<u64> {
    let id = {
        let mut st = state.borrow_mut();
        let frame = Frame::new(WidgetType::Frame, Some(name.to_string()), None);
        st.widgets.register(frame)
    };
    let ud = lua.create_userdata(FrameHandle {
        id,
        state: Rc::clone(state),
    })?;
    lua.globals().set(name, ud)?;
    Ok(id)
}

/// Get or create the `__frame_fields` table for a given frame ID.
fn get_or_create_frame_fields(lua: &Lua, frame_id: u64) -> Result<mlua::Table> {
    let fields_table: mlua::Table = lua
        .globals()
        .get::<mlua::Table>("__frame_fields")
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.globals().set("__frame_fields", t.clone()).unwrap();
            t
        });
    let frame_fields = fields_table
        .get::<mlua::Table>(frame_id)
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            fields_table.set(frame_id, t.clone()).unwrap();
            t
        });
    Ok(frame_fields)
}

/// Register all global frame objects.
pub fn register_global_frames(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_core_frame_globals(lua, &state)?;
    register_chat_globals(lua, &state)?;
    register_ui_panel_globals(lua, &state)?;
    register_unit_frame_globals(lua, &state)?;
    register_misc_frame_globals(lua, &state)?;
    register_table_globals(lua)?;
    Ok(())
}

/// Register core frames: UIParent, WorldFrame, Minimap, etc.
fn register_core_frame_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    register_frame_global(lua, state, "UIParent")?;
    lua.globals()
        .set("UIPanelWindows", lua.create_table()?)?;
    register_frame_global(lua, state, "WorldFrame")?;
    register_frame_global(lua, state, "Minimap")?;
    register_frame_global(lua, state, "EventToastManagerFrame")?;
    eprintln!("DEBUG: after EventToastManagerFrame");
    register_frame_global(lua, state, "EditModeManagerFrame")?;
    eprintln!("DEBUG: after EditModeManagerFrame");
    register_frame_global(lua, state, "RolePollPopup")?;
    eprintln!("DEBUG: after RolePollPopup");
    register_frame_global(lua, state, "TimerTracker")?;
    Ok(())
}

/// Register chat-related globals: DEFAULT_CHAT_FRAME, ChatTypeGroup, ChatFrameUtil.
fn register_chat_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    register_frame_global(lua, state, "DEFAULT_CHAT_FRAME")?;
    register_frame_global(lua, state, "ChatFrame1")?;
    register_chat_type_group(lua)?;
    eprintln!("DEBUG: after ChatTypeGroup");
    register_chat_frame_util(lua)?;
    eprintln!("DEBUG: after ChatFrameUtil");
    Ok(())
}

/// Register ChatTypeGroup table mapping chat type groups to arrays of message types.
fn register_chat_type_group(lua: &Lua) -> Result<()> {
    let groups: &[(&str, &[&str])] = &[
        ("SYSTEM", &["SYSTEM", "ERROR", "IGNORED", "CHANNEL_NOTICE", "CHANNEL_NOTICE_USER"]),
        ("SAY", &["SAY"]),
        ("YELL", &["YELL"]),
        ("WHISPER", &["WHISPER", "WHISPER_INFORM"]),
        ("PARTY", &["PARTY", "PARTY_LEADER"]),
        ("RAID", &["RAID", "RAID_LEADER", "RAID_WARNING"]),
        ("GUILD", &["GUILD", "OFFICER"]),
        ("EMOTE", &["EMOTE", "TEXT_EMOTE"]),
        ("CHANNEL", &["CHANNEL"]),
        ("INSTANCE_CHAT", &["INSTANCE_CHAT", "INSTANCE_CHAT_LEADER"]),
        ("BN_WHISPER", &["BN_WHISPER", "BN_WHISPER_INFORM", "BN_CONVERSATION"]),
    ];

    let chat_type_group = lua.create_table()?;
    for (group_name, members) in groups {
        let group_table = lua.create_table()?;
        for (i, member) in members.iter().enumerate() {
            group_table.set(i + 1, *member)?;
        }
        chat_type_group.set(*group_name, group_table)?;
    }
    lua.globals().set("ChatTypeGroup", chat_type_group)?;
    Ok(())
}

/// Register ChatFrameUtil utility functions.
fn register_chat_frame_util(lua: &Lua) -> Result<()> {
    let chat_frame_util = lua.create_table()?;
    chat_frame_util.set(
        "ProcessMessageEventFilters",
        lua.create_function(|_, (_, event, args): (Value, String, mlua::Variadic<Value>)| {
            Ok((false, event, args))
        })?,
    )?;
    chat_frame_util.set(
        "GetChatWindowName",
        lua.create_function(|_, frame_id: i32| Ok(format!("Chat Window {}", frame_id)))?,
    )?;
    lua.globals().set("ChatFrameUtil", chat_frame_util)?;
    Ok(())
}

/// Register UI panel globals: WorldMapFrame, SettingsPanel, ObjectiveTracker, etc.
fn register_ui_panel_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let world_map_id = register_frame_global(lua, state, "WorldMapFrame")?;
    eprintln!("DEBUG: after WorldMapFrame");
    setup_world_map_frame_fields(lua, world_map_id)?;
    eprintln!("DEBUG: after WorldMapFrame overlayFrames");

    register_frame_global(lua, state, "SettingsPanel")?;
    eprintln!("DEBUG: after SettingsPanel");
    setup_settings_panel(lua)?;
    eprintln!("DEBUG: after SettingsPanel.Container");

    register_frame_global(lua, state, "ObjectiveTrackerFrame")?;
    setup_objective_tracker(lua)?;

    register_frame_global(lua, state, "MinimapCluster")?;

    register_frame_global(lua, state, "LFGListFrame")?;
    setup_lfg_list_frame(lua)?;

    register_frame_global(lua, state, "AlertFrame")?;
    setup_alert_frame(lua)?;

    register_frame_global(lua, state, "InterfaceOptionsFrame")?;
    register_frame_global(lua, state, "AuctionHouseFrame")?;
    register_frame_global(lua, state, "SideDressUpFrame")?;
    register_frame_global(lua, state, "GossipFrame")?;
    Ok(())
}

/// Set pinPools and overlayFrames on WorldMapFrame.
fn setup_world_map_frame_fields(lua: &Lua, world_map_frame_id: u64) -> Result<()> {
    let wm_fields = get_or_create_frame_fields(lua, world_map_frame_id)?;
    wm_fields.set("pinPools", lua.create_table()?)?;
    wm_fields.set("overlayFrames", lua.create_table()?)?;
    Ok(())
}

/// Set up SettingsPanel.Container structure (used by DynamicCam).
fn setup_settings_panel(lua: &Lua) -> Result<()> {
    lua.load(
        r#"
        SettingsPanel.Container = {
            SettingsList = {
                ScrollBox = {
                    ScrollTarget = {
                        GetChildren = function() return end
                    }
                },
                Header = {
                    Title = {
                        GetText = function() return "" end
                    }
                }
            }
        }
    "#,
    )
    .exec()
}

/// Set up ObjectiveTrackerFrame.Header and ObjectiveTrackerManager.
fn setup_objective_tracker(lua: &Lua) -> Result<()> {
    eprintln!("DEBUG: before ObjectiveTrackerFrame.Header lua.load");
    lua.load(
        r#"
        ObjectiveTrackerFrame.Header = CreateFrame("Frame", nil, ObjectiveTrackerFrame)
        ObjectiveTrackerFrame.Header.MinimizeButton = CreateFrame("Button", nil, ObjectiveTrackerFrame.Header)
    "#,
    )
    .exec()?;
    eprintln!("DEBUG: after ObjectiveTrackerFrame.Header lua.load");

    lua.load(
        r#"
        ObjectiveTrackerManager = {
            modules = {},
            containers = {},
            AssignModulesOrder = function(self, modules) end,
            AddContainer = function(self, container) end,
            HasAnyModules = function(self) return false end,
            UpdateAll = function(self) end,
            UpdateModule = function(self, module) end,
            GetContainerForModule = function(self, module) return nil end,
            SetModuleContainer = function(self, module, container) end,
            AcquireFrame = function(self, parent, template) return nil end,
            ReleaseFrame = function(self, frame) end,
            SetOpacity = function(self, opacity) end,
            SetTextSize = function(self, textSize) end,
            ShowRewardsToast = function(self, rewards, module, block, headerText, callback) end,
            HideRewardsToast = function(self, rewardsToast) end,
            HasRewardsToastForBlock = function(self, block) return false end,
            UpdatePOIEnabled = function(self, enabled) end,
            OnVariablesLoaded = function(self) end,
            OnCVarChanged = function(self, cvar, value) end,
            CanShowPOIs = function(self, module) return false end,
            EnumerateActiveBlocksByTag = function(self, tag, callback) end,
        }
    "#,
    )
    .exec()?;
    eprintln!("DEBUG: after ObjectiveTrackerManager");
    Ok(())
}

/// Set up LFGListFrame.SearchPanel.SearchBox structure.
fn setup_lfg_list_frame(lua: &Lua) -> Result<()> {
    lua.load(
        r#"
        LFGListFrame.SearchPanel = CreateFrame("Frame", nil, LFGListFrame)
        LFGListFrame.SearchPanel.SearchBox = CreateFrame("EditBox", nil, LFGListFrame.SearchPanel)
    "#,
    )
    .exec()
}

/// Set up AlertFrame.alertFrameSubSystems table.
fn setup_alert_frame(lua: &Lua) -> Result<()> {
    let alert_ud: mlua::AnyUserData = lua.globals().get("AlertFrame")?;
    alert_ud.set("alertFrameSubSystems", lua.create_table()?)?;
    Ok(())
}

/// Register unit frame globals: PlayerFrame, TargetFrame, FocusFrame, etc.
fn register_unit_frame_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    register_frame_global(lua, state, "PlayerFrame")?;
    eprintln!("DEBUG: after PlayerFrame");
    register_frame_global(lua, state, "TargetFrame")?;
    register_frame_global(lua, state, "FocusFrame")?;
    register_frame_global(lua, state, "FocusFrameSpellBar")?;

    register_frame_global(lua, state, "BuffFrame")?;
    setup_buff_frame_aura_container(lua, state)?;

    register_frame_global(lua, state, "TargetFrameSpellBar")?;
    register_frame_global(lua, state, "PlayerCastingBarFrame")?;
    register_frame_global(lua, state, "PartyFrame")?;
    register_frame_global(lua, state, "PetFrame")?;
    register_frame_global(lua, state, "AlternatePowerBar")?;
    register_frame_global(lua, state, "MonkStaggerBar")?;
    Ok(())
}

/// Set iconScale on BuffFrame.AuraContainer.
fn setup_buff_frame_aura_container(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let aura_container_id = {
        let st = state.borrow();
        st.widgets
            .get_id_by_name("BuffFrameAuraContainer")
            .unwrap()
    };
    let aura_fields = get_or_create_frame_fields(lua, aura_container_id)?;
    aura_fields.set("iconScale", 1.0)?;
    Ok(())
}

/// Register miscellaneous frame globals.
fn register_misc_frame_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    // Minimap (re-registered, same as in core — keeps existing behavior)
    register_frame_global(lua, state, "Minimap")?;

    register_frame_global(lua, state, "LFGEventFrame")?;
    register_frame_global(lua, state, "NamePlateDriverFrame")?;
    register_frame_global(lua, state, "UIErrorsFrame")?;

    register_frame_global(lua, state, "ContainerFrameContainer")?;
    setup_container_frame(lua)?;

    register_frame_global(lua, state, "ContainerFrameCombinedBags")?;
    register_frame_global(lua, state, "LootFrame")?;

    let addon_compartment_id = create_and_register_frame_global(lua, state, "AddonCompartmentFrame")?;
    setup_addon_compartment(lua, addon_compartment_id)?;
    eprintln!("DEBUG: after AddonCompartmentFrame");

    register_frame_global(lua, state, "ScenarioObjectiveTracker")?;
    register_frame_global(lua, state, "RaidWarningFrame")?;

    create_and_register_frame_global(lua, state, "FriendsFrame")?;

    setup_party_member_frame_pool(lua)?;
    Ok(())
}

/// Set ContainerFrames table on ContainerFrameContainer.
fn setup_container_frame(lua: &Lua) -> Result<()> {
    let ud: mlua::AnyUserData = lua.globals().get("ContainerFrameContainer")?;
    ud.set("ContainerFrames", lua.create_table()?)?;
    Ok(())
}

/// Set up AddonCompartmentFrame with RegisterAddon/UnregisterAddon methods.
fn setup_addon_compartment(lua: &Lua, addon_compartment_id: u64) -> Result<()> {
    let frame_fields = get_or_create_frame_fields(lua, addon_compartment_id)?;
    frame_fields.set(
        "RegisterAddon",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    frame_fields.set(
        "UnregisterAddon",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
    )?;
    frame_fields.set("registeredAddons", lua.create_table()?)?;
    Ok(())
}

/// Register PartyMemberFramePool with empty iterator.
fn setup_party_member_frame_pool(lua: &Lua) -> Result<()> {
    let party_frame_pool = lua.create_table()?;
    party_frame_pool.set(
        "EnumerateActive",
        lua.create_function(|lua, _self: Value| {
            let iter_func = lua.create_function(|_, ()| Ok(Value::Nil))?;
            Ok(iter_func)
        })?,
    )?;
    party_frame_pool.set(
        "GetNumActive",
        lua.create_function(|_, _self: Value| Ok(0i32))?,
    )?;
    lua.globals()
        .set("PartyMemberFramePool", party_frame_pool)?;
    Ok(())
}

/// Register empty table globals.
fn register_table_globals(lua: &Lua) -> Result<()> {
    lua.globals()
        .set("UISpecialFrames", lua.create_table()?)?;
    lua.globals()
        .set("StaticPopupDialogs", lua.create_table()?)?;
    Ok(())
}
