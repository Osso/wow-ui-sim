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
/// Look up or create a frame by name and expose it as a Lua global.
/// If the frame already exists in the widget registry (e.g. from builtin_frames),
/// reuse it. Otherwise create a new one on demand. This avoids hardcoding frames
/// in builtin_frames.rs just so global_frames.rs can find them.
fn register_frame_global(lua: &Lua, state: &Rc<RefCell<SimState>>, name: &str) -> Result<u64> {
    register_frame_global_with_visibility(lua, state, name, true)
}

/// Register a frame global that starts hidden.
fn register_hidden_frame_global(lua: &Lua, state: &Rc<RefCell<SimState>>, name: &str) -> Result<u64> {
    register_frame_global_with_visibility(lua, state, name, false)
}

fn register_typed_frame_global(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    name: &str,
    widget_type: WidgetType,
) -> Result<u64> {
    register_frame_global_impl(lua, state, name, true, widget_type)
}

fn register_frame_global_with_visibility(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    name: &str,
    visible: bool,
) -> Result<u64> {
    register_frame_global_impl(lua, state, name, visible, WidgetType::Frame)
}

fn register_frame_global_impl(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    name: &str,
    visible: bool,
    widget_type: WidgetType,
) -> Result<u64> {
    let id = {
        let mut st = state.borrow_mut();
        if let Some(existing) = st.widgets.get_id_by_name(name) {
            if !visible
                && let Some(frame) = st.widgets.get_mut(existing) {
                    frame.visible = false;
                }
            existing
        } else {
            let mut frame = Frame::new(widget_type, Some(name.to_string()), None);
            frame.visible = visible;
            st.widgets.register(frame)
        }
    };
    let ud = lua.create_userdata(FrameHandle {
        id,
        state: Rc::clone(state),
    })?;
    lua.globals().set(name, ud.clone())?;
    // Store internal reference used by event dispatch to pass `self` to handlers.
    let frame_key = format!("__frame_{}", id);
    lua.globals().set(frame_key.as_str(), ud)?;
    Ok(id)
}

/// Get or create the `__frame_fields` table for a given frame ID.
fn get_or_create_frame_fields(lua: &Lua, frame_id: u64) -> Result<mlua::Table> {
    Ok(crate::lua_api::script_helpers::get_or_create_frame_fields(lua, frame_id))
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
    register_frame_global(lua, state, "EditModeManagerFrame")?;
    setup_edit_mode_manager(lua, state)?;
    register_frame_global(lua, state, "RolePollPopup")?;
    register_frame_global(lua, state, "TimerTracker")?;
    register_hidden_frame_global(lua, state, "StoreFrame")?;
    Ok(())
}

/// Register chat-related globals: DEFAULT_CHAT_FRAME, ChatTypeGroup, ChatFrameUtil.
fn register_chat_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    register_typed_frame_global(lua, state, "DEFAULT_CHAT_FRAME", WidgetType::MessageFrame)?;
    register_typed_frame_global(lua, state, "ChatFrame1", WidgetType::MessageFrame)?;
    register_chat_type_group(lua)?;
    register_chat_frame_util(lua)?;
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
    chat_frame_util.set("RegisterForStickyFocus", lua.create_function(|_, _frame: Value| Ok(()))?)?;
    chat_frame_util.set("UnregisterForStickyFocus", lua.create_function(|_, _frame: Value| Ok(()))?)?;
    lua.globals().set("ChatFrameUtil", chat_frame_util)?;
    Ok(())
}

/// Register UI panel globals: SettingsPanel, ObjectiveTracker, etc.
/// Note: WorldMapFrame is now loaded from Blizzard_WorldMap addon XML.
fn register_ui_panel_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    register_frame_global(lua, state, "SettingsPanel")?;
    setup_settings_panel(lua)?;

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

/// Set up ObjectiveTrackerFrame.Header.
/// ObjectiveTrackerManager is defined by Blizzard_ObjectiveTrackerManager.lua.
fn setup_objective_tracker(lua: &Lua) -> Result<()> {
    lua.load(
        r#"
        ObjectiveTrackerFrame.Header = CreateFrame("Frame", nil, ObjectiveTrackerFrame)
        ObjectiveTrackerFrame.Header.MinimizeButton = CreateFrame("Button", nil, ObjectiveTrackerFrame.Header)
    "#,
    )
    .exec()?;
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

/// Set up EditModeManagerFrame with AccountSettings child frame.
fn setup_edit_mode_manager(_lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    // AccountSettings is a child frame with parentKey="AccountSettings"
    let emm_id = state.borrow().widgets.get_id_by_name("EditModeManagerFrame");
    if let Some(parent_id) = emm_id {
        let mut child = Frame::new(WidgetType::Frame, None, Some(parent_id));
        child.visible = false;
        let child_id = child.id;
        state.borrow_mut().widgets.register(child);
        state.borrow_mut().widgets.add_child(parent_id, child_id);
        {
            let mut st = state.borrow_mut();
            if let Some(parent) = st.widgets.get_mut(parent_id) {
                parent.children_keys.insert("AccountSettings".to_string(), child_id);
            }
        }
    }
    Ok(())
}

/// Register unit frame globals: PlayerFrame, TargetFrame, FocusFrame, etc.
fn register_unit_frame_globals(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    register_frame_global(lua, state, "PlayerFrame")?;

    // Target/Focus frames hidden by default (no target selected)
    register_hidden_frame_global(lua, state, "TargetFrame")?;
    register_hidden_frame_global(lua, state, "FocusFrame")?;
    register_hidden_frame_global(lua, state, "FocusFrameSpellBar")?;
    register_hidden_frame_global(lua, state, "TargetFrameSpellBar")?;

    register_frame_global(lua, state, "BuffFrame")?;
    setup_buff_frame_aura_container(lua, state)?;
    setup_editmode_stub_methods(lua, "BuffFrame")?;
    register_frame_global(lua, state, "DebuffFrame")?;
    setup_editmode_stub_methods(lua, "DebuffFrame")?;

    register_frame_global(lua, state, "PlayerCastingBarFrame")?;

    // Party frame hidden by default (not in group)
    register_hidden_frame_global(lua, state, "PartyFrame")?;

    // Pet frame hidden by default (no pet)
    register_hidden_frame_global(lua, state, "PetFrame")?;

    // Class-specific bars hidden by default
    register_hidden_frame_global(lua, state, "AlternatePowerBar")?;
    register_hidden_frame_global(lua, state, "MonkStaggerBar")?;
    Ok(())
}

/// Add EditModeSystemMixin stub methods to a stub frame global.
///
/// Stub frames (e.g. BuffFrame, DebuffFrame) don't load the XML that applies
/// EditModeSystemMixin, but Blizzard code calls `frame:IsInDefaultPosition()`
/// without nil-guarding. Returns false (not initialized = not in default position).
fn setup_editmode_stub_methods(lua: &Lua, frame_name: &str) -> Result<()> {
    lua.load(format!(
        r#"
        local f = {frame_name}
        if f then
            function f:IsInDefaultPosition() return false end
            function f:IsInitialized() return false end
        end
        "#
    ))
    .exec()
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

    // Bag/loot frames hidden by default
    register_hidden_frame_global(lua, state, "ContainerFrameContainer")?;
    setup_container_frame(lua)?;

    register_hidden_frame_global(lua, state, "ContainerFrameCombinedBags")?;
    register_hidden_frame_global(lua, state, "LootFrame")?;

    let addon_compartment_id = register_frame_global(lua, state, "AddonCompartmentFrame")?;
    setup_addon_compartment(lua, addon_compartment_id)?;

    register_hidden_frame_global(lua, state, "ScenarioObjectiveTracker")?;
    register_hidden_frame_global(lua, state, "RaidWarningFrame")?;

    // FriendsFrame has hidden="true" in its XML but we create it as a stub
    register_hidden_frame_global(lua, state, "FriendsFrame")?;

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

/// Hide frames that WoW's C++ engine hides by default at startup.
///
/// Must be called AFTER addon XML loading, because `CreateFrame` in the XML
/// loader creates new frames (orphaning any pre-registered hidden stubs).
/// These frames have no `hidden="true"` in XML — WoW hides them at runtime
/// based on game state (no target, no group, no encounter, etc.).
pub fn hide_runtime_hidden_frames(lua: &Lua) -> Result<()> {
    let frames_to_hide: &[&str] = &[
        // Unit frames: no target/focus/group/pet by default
        "TargetFrame",
        "FocusFrame",
        "TargetFrameSpellBar",
        "FocusFrameSpellBar",
        "PartyFrame",
        "PetFrame",
        // Boss frames: no encounter active
        "BossTargetFrameContainer",
        // Class-specific power bars
        "AlternatePowerBar",
        "MonkStaggerBar",
        "InsanityBarFrame",
        "EvokerEbonMightBar",
        // Encounter bar: no encounter active
        "EncounterBar",
        // Casting bar: not casting
        "PlayerCastingBarFrame",
        // Bags/loot not open
        "ContainerFrameContainer",
        "ContainerFrameCombinedBags",
        "LootFrame",
        // Misc UI not shown at login
        "RoleChangedFrame",
        "QuestInfoRequiredMoneyFrame",
        "SpellActivationOverlayFrame",
        "ScenarioObjectiveTracker",
        "RaidWarningFrame",
        "FriendsFrame",
        // Combat log quick buttons: combat log tab not active
        "CombatLogQuickButtonFrame_Custom",
    ];

    for name in frames_to_hide {
        let code = format!("if {} then {}:Hide() end", name, name);
        if let Err(e) = lua.load(&code).exec() {
            eprintln!("[hide_runtime] Failed to hide {}: {}", name, e);
        }
    }

    hide_child_overlays(lua)?;
    hide_orphaned_anonymous_frames(lua)?;
    Ok(())
}

/// Lua helper function injected once for hiding child elements.
const HIDE_HELPER: &str = r#"
    if not __hide_child then
        function __hide_child(parent, key)
            if parent and parent[key] then parent[key]:Hide() end
        end
    end
"#;

/// Hide child textures/frames that are only shown contextually in WoW.
fn hide_child_overlays(lua: &Lua) -> Result<()> {
    let _ = lua.load(HIDE_HELPER).exec();
    hide_action_bar_overlays(lua);
    hide_player_frame_overlays(lua);
    hide_micro_menu_flashes(lua);
    hide_xp_bar_effects(lua);
    hide_misc_overlays(lua);
    Ok(())
}

fn hide_action_bar_overlays(lua: &Lua) {
    let _ = lua.load(r#"
        if MainActionBar then
            local h = __hide_child
            h(MainActionBar, "QuickKeybindGlowLarge")
            h(MainActionBar, "QuickKeybindGlowSmall")
            h(MainActionBar, "QuickKeybindBottomShadow")
            h(MainActionBar, "QuickKeybindRightShadow")
        end
    "#).exec();
}

fn hide_player_frame_overlays(lua: &Lua) {
    let _ = lua.load(r#"
        if not PlayerFrame then return end
        local h = __hide_child
        -- Container-level textures (vehicle/alternate overlays)
        local pfc = PlayerFrame.PlayerFrameContainer
        if pfc then
            h(pfc, "VehicleFrameTexture")
            h(pfc, "AlternatePowerFrameTexture")
        end
        -- Content-level: main overlays + contextual icons
        local content = PlayerFrame.PlayerFrameContent
        if content then
            local main = content.PlayerFrameContentMain
            if main then h(main, "StatusTexture") end
            local ctx = content.PlayerFrameContentContextual
            if ctx then
                h(ctx, "LeaderIcon")
                h(ctx, "GuideIcon")
                h(ctx, "RoleIcon")
                h(ctx, "AttackIcon")
                h(ctx, "PlayerPortraitCornerIcon")
                h(ctx, "PrestigePortrait")
                h(ctx, "PrestigeBadge")
            end
        end
        -- Mana bar full-power glow
        if PlayerFrame.manabar then h(PlayerFrame.manabar, "FullPowerFrame") end
    "#).exec();
}

fn hide_micro_menu_flashes(lua: &Lua) {
    let _ = lua.load(r#"
        if MicroMenu then
            for _, child in ipairs({ MicroMenu:GetChildren() }) do
                __hide_child(child, "FlashContent")
            end
        end
    "#).exec();
}

fn hide_xp_bar_effects(lua: &Lua) {
    let _ = lua.load(r#"
        if MainStatusTrackingBarContainer then
            local h = __hide_child
            for _, child in ipairs({ MainStatusTrackingBarContainer:GetChildren() }) do
                if child.StatusBar then
                    h(child.StatusBar, "GainFlareAnimationTexture")
                    h(child.StatusBar, "LevelUpTexture")
                end
            end
        end
    "#).exec();
}

fn hide_misc_overlays(lua: &Lua) {
    let _ = lua.load(r#"
        if GameTimeCalendarEventAlarmTexture then
            GameTimeCalendarEventAlarmTexture:Hide()
        end
        if ItemButton then ItemButton:Hide() end
    "#).exec();
}

/// Hide anonymous frames orphaned to UIParent during addon loading.
///
/// Some Blizzard addons create child frames in OnLoad handlers that reference
/// properties not yet initialized (e.g. `self.ScrollFrame.ScrollChild`).
/// When the parent is nil, `CreateFrame` falls back to UIParent, making these
/// frames visible at the top level. In real WoW they'd be inside hidden panels.
fn hide_orphaned_anonymous_frames(lua: &Lua) -> Result<()> {
    if let Err(e) = lua.load(
        r#"
        for _, child in ipairs({ UIParent:GetChildren() }) do
            if not child:GetName() and child:IsShown() then
                -- MapLegend categories (Quests, Activities, etc.)
                local ok, tt = pcall(function() return child.TitleText end)
                if ok and tt then child:Hide() end
                -- CampaignTooltip (Story Progress panel)
                local ok2, ns = pcall(function() return child.NineSlice end)
                local ok3, it = pcall(function() return child.ItemTooltip end)
                if ok2 and ns and ok3 and it then child:Hide() end
                -- HelpTip frames (OkayButton + Arrow)
                local ok4, ob = pcall(function() return child.OkayButton end)
                local ok5, ar = pcall(function() return child.Arrow end)
                if ok4 and ob and ok5 and ar then child:Hide() end
            end
        end
        "#,
    )
    .exec() {
        eprintln!("[hide_orphaned] Error: {}", e);
    }
    Ok(())
}
