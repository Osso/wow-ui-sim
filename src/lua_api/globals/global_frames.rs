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

/// Register all global frame objects.
pub fn register_global_frames(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    // UIParent reference
    let ui_parent_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("UIParent").unwrap()
    };
    let ui_parent = lua.create_userdata(FrameHandle {
        id: ui_parent_id,
        state: Rc::clone(&state),
    })?;
    globals.set("UIParent", ui_parent)?;

    // UIPanelWindows - registry for UI panel positioning/behavior
    globals.set("UIPanelWindows", lua.create_table()?)?;

    // WorldFrame reference (3D world rendering frame)
    let world_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("WorldFrame").unwrap()
    };
    let world_frame = lua.create_userdata(FrameHandle {
        id: world_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("WorldFrame", world_frame)?;

    // Minimap reference (built-in UI element)
    let minimap_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("Minimap").unwrap()
    };
    let minimap = lua.create_userdata(FrameHandle {
        id: minimap_id,
        state: Rc::clone(&state),
    })?;
    globals.set("Minimap", minimap)?;

    // DEFAULT_CHAT_FRAME reference (main chat window)
    let default_chat_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("DEFAULT_CHAT_FRAME").unwrap()
    };
    let default_chat_frame = lua.create_userdata(FrameHandle {
        id: default_chat_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("DEFAULT_CHAT_FRAME", default_chat_frame)?;

    // ChatFrame1 reference (same as DEFAULT_CHAT_FRAME)
    let chat_frame1_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ChatFrame1").unwrap()
    };
    let chat_frame1 = lua.create_userdata(FrameHandle {
        id: chat_frame1_id,
        state: Rc::clone(&state),
    })?;
    globals.set("ChatFrame1", chat_frame1)?;

    // ChatTypeGroup - maps chat type groups to arrays of chat message types
    let chat_type_group = lua.create_table()?;
    // System messages
    let system_group = lua.create_table()?;
    system_group.set(1, "SYSTEM")?;
    system_group.set(2, "ERROR")?;
    system_group.set(3, "IGNORED")?;
    system_group.set(4, "CHANNEL_NOTICE")?;
    system_group.set(5, "CHANNEL_NOTICE_USER")?;
    chat_type_group.set("SYSTEM", system_group)?;
    // Say messages
    let say_group = lua.create_table()?;
    say_group.set(1, "SAY")?;
    chat_type_group.set("SAY", say_group)?;
    // Yell messages
    let yell_group = lua.create_table()?;
    yell_group.set(1, "YELL")?;
    chat_type_group.set("YELL", yell_group)?;
    // Whisper messages
    let whisper_group = lua.create_table()?;
    whisper_group.set(1, "WHISPER")?;
    whisper_group.set(2, "WHISPER_INFORM")?;
    chat_type_group.set("WHISPER", whisper_group)?;
    // Party messages
    let party_group = lua.create_table()?;
    party_group.set(1, "PARTY")?;
    party_group.set(2, "PARTY_LEADER")?;
    chat_type_group.set("PARTY", party_group)?;
    // Raid messages
    let raid_group = lua.create_table()?;
    raid_group.set(1, "RAID")?;
    raid_group.set(2, "RAID_LEADER")?;
    raid_group.set(3, "RAID_WARNING")?;
    chat_type_group.set("RAID", raid_group)?;
    // Guild messages
    let guild_group = lua.create_table()?;
    guild_group.set(1, "GUILD")?;
    guild_group.set(2, "OFFICER")?;
    chat_type_group.set("GUILD", guild_group)?;
    // Emote messages
    let emote_group = lua.create_table()?;
    emote_group.set(1, "EMOTE")?;
    emote_group.set(2, "TEXT_EMOTE")?;
    chat_type_group.set("EMOTE", emote_group)?;
    // Channel messages
    let channel_group = lua.create_table()?;
    channel_group.set(1, "CHANNEL")?;
    chat_type_group.set("CHANNEL", channel_group)?;
    // Instance messages
    let instance_group = lua.create_table()?;
    instance_group.set(1, "INSTANCE_CHAT")?;
    instance_group.set(2, "INSTANCE_CHAT_LEADER")?;
    chat_type_group.set("INSTANCE_CHAT", instance_group)?;
    // BattleNet messages
    let bn_group = lua.create_table()?;
    bn_group.set(1, "BN_WHISPER")?;
    bn_group.set(2, "BN_WHISPER_INFORM")?;
    bn_group.set(3, "BN_CONVERSATION")?;
    chat_type_group.set("BN_WHISPER", bn_group)?;
    globals.set("ChatTypeGroup", chat_type_group)?;
    eprintln!("DEBUG: after ChatTypeGroup");

    // ChatFrameUtil - utility functions for chat frames
    let chat_frame_util = lua.create_table()?;
    chat_frame_util.set(
        "ProcessMessageEventFilters",
        lua.create_function(|_, (_, event, args): (Value, String, mlua::Variadic<Value>)| {
            // Return the event args unchanged - no filtering in sim
            Ok((false, event, args))
        })?,
    )?;
    chat_frame_util.set(
        "GetChatWindowName",
        lua.create_function(|_, frame_id: i32| {
            Ok(format!("Chat Window {}", frame_id))
        })?,
    )?;
    globals.set("ChatFrameUtil", chat_frame_util)?;
    eprintln!("DEBUG: after ChatFrameUtil");

    // EventToastManagerFrame reference (event toast/notification UI)
    let event_toast_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("EventToastManagerFrame").unwrap()
    };
    let event_toast_frame = lua.create_userdata(FrameHandle {
        id: event_toast_id,
        state: Rc::clone(&state),
    })?;
    globals.set("EventToastManagerFrame", event_toast_frame)?;
    eprintln!("DEBUG: after EventToastManagerFrame");

    // EditModeManagerFrame reference (Edit Mode UI manager)
    let edit_mode_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("EditModeManagerFrame").unwrap()
    };
    let edit_mode_frame = lua.create_userdata(FrameHandle {
        id: edit_mode_id,
        state: Rc::clone(&state),
    })?;
    globals.set("EditModeManagerFrame", edit_mode_frame)?;
    eprintln!("DEBUG: after EditModeManagerFrame");

    // RolePollPopup reference (role selection popup for groups)
    let role_poll_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("RolePollPopup").unwrap()
    };
    let role_poll_popup = lua.create_userdata(FrameHandle {
        id: role_poll_id,
        state: Rc::clone(&state),
    })?;
    globals.set("RolePollPopup", role_poll_popup)?;
    eprintln!("DEBUG: after RolePollPopup");

    // TimerTracker reference (displays dungeon/raid instance timers)
    let timer_tracker_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("TimerTracker").unwrap()
    };
    let timer_tracker = lua.create_userdata(FrameHandle {
        id: timer_tracker_id,
        state: Rc::clone(&state),
    })?;
    globals.set("TimerTracker", timer_tracker)?;

    // WorldMapFrame reference (world map display frame)
    let world_map_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("WorldMapFrame").unwrap()
    };
    let world_map_frame = lua.create_userdata(FrameHandle {
        id: world_map_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("WorldMapFrame", world_map_frame)?;
    eprintln!("DEBUG: after WorldMapFrame");

    // Set up WorldMapFrame.pinPools table (used by HereBeDragons map pins)
    let frame_fields: mlua::Table = lua
        .globals()
        .get::<mlua::Table>("__frame_fields")
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.globals().set("__frame_fields", t.clone()).unwrap();
            t
        });
    let wm_fields: mlua::Table = frame_fields
        .get::<mlua::Table>(world_map_frame_id)
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            frame_fields.set(world_map_frame_id, t.clone()).unwrap();
            t
        });
    wm_fields.set("pinPools", lua.create_table()?)?;
    // Add overlayFrames (used by WorldQuestTracker)
    wm_fields.set("overlayFrames", lua.create_table()?)?;
    eprintln!("DEBUG: after WorldMapFrame overlayFrames");

    // PlayerFrame reference (player unit frame)
    let player_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("PlayerFrame").unwrap()
    };
    let player_frame = lua.create_userdata(FrameHandle {
        id: player_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("PlayerFrame", player_frame)?;
    eprintln!("DEBUG: after PlayerFrame");

    // TargetFrame reference (target unit frame)
    let target_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("TargetFrame").unwrap()
    };
    let target_frame = lua.create_userdata(FrameHandle {
        id: target_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("TargetFrame", target_frame)?;

    // FocusFrame reference (focus target unit frame)
    let focus_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("FocusFrame").unwrap()
    };
    let focus_frame = lua.create_userdata(FrameHandle {
        id: focus_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("FocusFrame", focus_frame)?;

    // FocusFrameSpellBar reference (focus cast bar)
    let focus_spell_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("FocusFrameSpellBar").unwrap()
    };
    let focus_spell_bar = lua.create_userdata(FrameHandle {
        id: focus_spell_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("FocusFrameSpellBar", focus_spell_bar)?;

    // BuffFrame reference (player buff display)
    let buff_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("BuffFrame").unwrap()
    };
    let buff_frame = lua.create_userdata(FrameHandle {
        id: buff_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("BuffFrame", buff_frame)?;

    // Set iconScale on BuffFrame.AuraContainer
    {
        let aura_container_id = {
            let state = state.borrow();
            state.widgets.get_id_by_name("BuffFrameAuraContainer").unwrap()
        };
        // Get or create __frame_fields table
        let fields_table: mlua::Table = lua.globals().get::<mlua::Table>("__frame_fields").unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.globals().set("__frame_fields", t.clone()).unwrap();
            t
        });
        // Create field table for AuraContainer
        let aura_fields = lua.create_table()?;
        aura_fields.set("iconScale", 1.0)?;
        fields_table.set(aura_container_id, aura_fields)?;
    }

    // TargetFrameSpellBar reference (target cast bar)
    let target_spell_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("TargetFrameSpellBar").unwrap()
    };
    let target_spell_bar = lua.create_userdata(FrameHandle {
        id: target_spell_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("TargetFrameSpellBar", target_spell_bar)?;

    // Minimap reference (minimap frame)
    let minimap_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("Minimap").unwrap()
    };
    let minimap = lua.create_userdata(FrameHandle {
        id: minimap_id,
        state: Rc::clone(&state),
    })?;
    globals.set("Minimap", minimap)?;

    // MinimapCluster reference (minimap container frame)
    let minimap_cluster_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("MinimapCluster").unwrap()
    };
    let minimap_cluster = lua.create_userdata(FrameHandle {
        id: minimap_cluster_id,
        state: Rc::clone(&state),
    })?;
    globals.set("MinimapCluster", minimap_cluster)?;

    // ObjectiveTrackerFrame reference (quest/objectives tracker)
    let objective_tracker_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ObjectiveTrackerFrame").unwrap()
    };
    let objective_tracker = lua.create_userdata(FrameHandle {
        id: objective_tracker_id,
        state: Rc::clone(&state),
    })?;
    globals.set("ObjectiveTrackerFrame", objective_tracker)?;

    // SettingsPanel reference (game settings UI)
    let settings_panel_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("SettingsPanel").unwrap()
    };
    let settings_panel = lua.create_userdata(FrameHandle {
        id: settings_panel_id,
        state: Rc::clone(&state),
    })?;
    globals.set("SettingsPanel", settings_panel)?;
    eprintln!("DEBUG: after SettingsPanel");

    // Add Container structure to SettingsPanel (used by DynamicCam)
    lua.load(r#"
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
    "#).exec()?;
    eprintln!("DEBUG: after SettingsPanel.Container");

    // Add Header.MinimizeButton structure to ObjectiveTrackerFrame (used by WorldQuestTracker)
    eprintln!("DEBUG: before ObjectiveTrackerFrame.Header lua.load");
    lua.load(r#"
        ObjectiveTrackerFrame.Header = CreateFrame("Frame", nil, ObjectiveTrackerFrame)
        ObjectiveTrackerFrame.Header.MinimizeButton = CreateFrame("Button", nil, ObjectiveTrackerFrame.Header)
    "#).exec()?;
    eprintln!("DEBUG: after ObjectiveTrackerFrame.Header lua.load");

    // ObjectiveTrackerManager - manages objective tracker modules (used by !KalielsTracker)
    lua.load(r#"
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
    "#).exec()?;
    eprintln!("DEBUG: after ObjectiveTrackerManager");

    // PlayerCastingBarFrame reference (player cast bar)
    let player_casting_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("PlayerCastingBarFrame").unwrap()
    };
    let player_casting_bar = lua.create_userdata(FrameHandle {
        id: player_casting_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("PlayerCastingBarFrame", player_casting_bar)?;

    // PartyFrame reference (party member container)
    let party_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("PartyFrame").unwrap()
    };
    let party_frame = lua.create_userdata(FrameHandle {
        id: party_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("PartyFrame", party_frame)?;

    // PetFrame reference (pet unit frame)
    let pet_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("PetFrame").unwrap()
    };
    let pet_frame = lua.create_userdata(FrameHandle {
        id: pet_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("PetFrame", pet_frame)?;

    // AlternatePowerBar reference (alternate power resource bar)
    let alternate_power_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("AlternatePowerBar").unwrap()
    };
    let alternate_power_bar = lua.create_userdata(FrameHandle {
        id: alternate_power_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("AlternatePowerBar", alternate_power_bar)?;

    // MonkStaggerBar reference (monk stagger resource bar)
    let monk_stagger_bar_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("MonkStaggerBar").unwrap()
    };
    let monk_stagger_bar = lua.create_userdata(FrameHandle {
        id: monk_stagger_bar_id,
        state: Rc::clone(&state),
    })?;
    globals.set("MonkStaggerBar", monk_stagger_bar)?;

    // LFGListFrame reference (Looking For Group list frame)
    let lfg_list_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("LFGListFrame").unwrap()
    };
    let lfg_list_frame = lua.create_userdata(FrameHandle {
        id: lfg_list_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("LFGListFrame", lfg_list_frame)?;

    // Add SearchPanel.SearchBox structure to LFGListFrame (used by WorldQuestTracker)
    lua.load(r#"
        LFGListFrame.SearchPanel = CreateFrame("Frame", nil, LFGListFrame)
        LFGListFrame.SearchPanel.SearchBox = CreateFrame("EditBox", nil, LFGListFrame.SearchPanel)
    "#).exec()?;

    // AlertFrame reference (alert/popup management frame)
    let alert_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("AlertFrame").unwrap()
    };
    let alert_frame = lua.create_userdata(FrameHandle {
        id: alert_frame_id,
        state: Rc::clone(&state),
    })?;
    // Add alertFrameSubSystems table for DynamicCam
    let alert_sub_systems = lua.create_table()?;
    alert_frame.set("alertFrameSubSystems", alert_sub_systems)?;
    globals.set("AlertFrame", alert_frame)?;

    // LFGEventFrame reference (LFG event handling frame)
    let lfg_event_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("LFGEventFrame").unwrap()
    };
    let lfg_event_frame = lua.create_userdata(FrameHandle {
        id: lfg_event_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("LFGEventFrame", lfg_event_frame)?;

    // NamePlateDriverFrame reference (nameplate management frame)
    let nameplate_driver_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("NamePlateDriverFrame").unwrap()
    };
    let nameplate_driver_frame = lua.create_userdata(FrameHandle {
        id: nameplate_driver_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("NamePlateDriverFrame", nameplate_driver_frame)?;

    // UIErrorsFrame reference (error message display frame)
    let ui_errors_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("UIErrorsFrame").unwrap()
    };
    let ui_errors_frame = lua.create_userdata(FrameHandle {
        id: ui_errors_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("UIErrorsFrame", ui_errors_frame)?;

    // InterfaceOptionsFrame reference (legacy interface options)
    let interface_options_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("InterfaceOptionsFrame").unwrap()
    };
    let interface_options_frame = lua.create_userdata(FrameHandle {
        id: interface_options_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("InterfaceOptionsFrame", interface_options_frame)?;

    // AuctionHouseFrame reference (auction house UI)
    let auction_house_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("AuctionHouseFrame").unwrap()
    };
    let auction_house_frame = lua.create_userdata(FrameHandle {
        id: auction_house_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("AuctionHouseFrame", auction_house_frame)?;

    // SideDressUpFrame reference (side dressing room)
    let side_dressup_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("SideDressUpFrame").unwrap()
    };
    let side_dressup_frame = lua.create_userdata(FrameHandle {
        id: side_dressup_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("SideDressUpFrame", side_dressup_frame)?;

    // ContainerFrameContainer reference (bag frame container for combined bags)
    let container_frame_container_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ContainerFrameContainer").unwrap()
    };
    let container_frame_container = lua.create_userdata(FrameHandle {
        id: container_frame_container_id,
        state: Rc::clone(&state),
    })?;
    // ContainerFrames is an empty array (individual bag frames would be added here)
    let container_frames = lua.create_table()?;
    container_frame_container.set("ContainerFrames", container_frames)?;
    globals.set("ContainerFrameContainer", container_frame_container)?;

    // ContainerFrameCombinedBags reference (combined bag frame)
    let container_combined_bags_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ContainerFrameCombinedBags").unwrap()
    };
    let container_combined_bags = lua.create_userdata(FrameHandle {
        id: container_combined_bags_id,
        state: Rc::clone(&state),
    })?;
    globals.set("ContainerFrameCombinedBags", container_combined_bags)?;

    // LootFrame reference (loot window)
    let loot_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("LootFrame").unwrap()
    };
    let loot_frame = lua.create_userdata(FrameHandle {
        id: loot_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("LootFrame", loot_frame)?;

    // AddonCompartmentFrame (retail UI element for addon buttons)
    let addon_compartment_id = {
        let mut state = state.borrow_mut();
        let frame = Frame::new(WidgetType::Frame, Some("AddonCompartmentFrame".to_string()), None);
        state.widgets.register(frame)
    };
    let addon_compartment = lua.create_userdata(FrameHandle {
        id: addon_compartment_id,
        state: Rc::clone(&state),
    })?;
    // Add RegisterAddon/UnregisterAddon methods via custom fields
    // These accept variadic args since they're called as methods (self is first arg)
    {
        let fields_table: mlua::Table = lua.globals().get::<mlua::Table>("__frame_fields").unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.globals().set("__frame_fields", t.clone()).unwrap();
            t
        });
        let frame_fields = lua.create_table()?;
        frame_fields.set("RegisterAddon", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
        frame_fields.set("UnregisterAddon", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
        frame_fields.set("registeredAddons", lua.create_table()?)?;
        fields_table.set(addon_compartment_id, frame_fields)?;
    }
    globals.set("AddonCompartmentFrame", addon_compartment)?;
    eprintln!("DEBUG: after AddonCompartmentFrame");

    // ScenarioObjectiveTracker reference (objective tracker for scenarios/M+)
    let scenario_tracker_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("ScenarioObjectiveTracker").unwrap()
    };
    let scenario_tracker = lua.create_userdata(FrameHandle {
        id: scenario_tracker_id,
        state: Rc::clone(&state),
    })?;
    globals.set("ScenarioObjectiveTracker", scenario_tracker)?;

    // RaidWarningFrame reference (raid warning message display)
    let raid_warning_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("RaidWarningFrame").unwrap()
    };
    let raid_warning_frame = lua.create_userdata(FrameHandle {
        id: raid_warning_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("RaidWarningFrame", raid_warning_frame)?;

    // GossipFrame reference (NPC interaction dialog)
    let gossip_frame_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("GossipFrame").unwrap()
    };
    let gossip_frame = lua.create_userdata(FrameHandle {
        id: gossip_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("GossipFrame", gossip_frame)?;

    // FriendsFrame - friends list panel (used by GlobalIgnoreList)
    let friends_frame_id = {
        let mut state_ref = state.borrow_mut();
        let friends_frame = Frame::new(WidgetType::Frame, Some("FriendsFrame".to_string()), None);
        let friends_frame_id = friends_frame.id;
        state_ref.widgets.register(friends_frame);
        friends_frame_id
    };
    let friends_ud = lua.create_userdata(FrameHandle {
        id: friends_frame_id,
        state: Rc::clone(&state),
    })?;
    globals.set("FriendsFrame", friends_ud)?;

    // PartyMemberFramePool - pool of party member frames (used by Clicked)
    let party_frame_pool = lua.create_table()?;
    party_frame_pool.set("EnumerateActive", lua.create_function(|lua, _self: Value| {
        // Return an empty iterator
        let iter_func = lua.create_function(|_, ()| Ok(Value::Nil))?;
        Ok(iter_func)
    })?)?;
    party_frame_pool.set("GetNumActive", lua.create_function(|_, _self: Value| Ok(0i32))?)?;
    globals.set("PartyMemberFramePool", party_frame_pool)?;

    // UISpecialFrames - table of frame names that close on Escape
    let ui_special_frames = lua.create_table()?;
    globals.set("UISpecialFrames", ui_special_frames)?;

    // StaticPopupDialogs - table for popup dialog definitions
    let static_popup_dialogs = lua.create_table()?;
    globals.set("StaticPopupDialogs", static_popup_dialogs)?;

    Ok(())
}
