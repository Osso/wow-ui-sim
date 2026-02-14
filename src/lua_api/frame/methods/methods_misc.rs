//! Miscellaneous frame-type-specific method stubs (Minimap, ScrollingMessage, Alerts, etc.).

use crate::lua_api::frame::handle::{frame_lud, lud_to_id};
use mlua::{LightUserData, Lua, Value};

/// Add all miscellaneous frame-type-specific methods.
pub fn add_misc_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_minimap_methods(lua, methods)?;
    add_scrolling_message_methods(lua, methods)?;
    add_alert_and_data_provider_methods(lua, methods)?;
    // DropdownButtonMixin stub
    methods.set("IsMenuOpen", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;
    // StaticPopupElementMixin stub (dialog ownership tracking)
    methods.set("SetOwningDialog", lua.create_function(|_, (_ud, _dialog): (LightUserData, Value)| Ok(()))?)?;
    // GuildRenameFrameMixin / layout tracking methods (no-op in simulator)
    methods.set("RegisterFontStrings", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    methods.set("RegisterFrames", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    methods.set("RegisterBackgroundTexture", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    Ok(())
}

/// Minimap and WorldMap stubs.
fn add_minimap_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_minimap_core_methods(lua, methods)?;
    add_minimap_texture_setters(lua, methods)?;
    add_minimap_blob_setters(lua, methods)?;
    // GetCanvas() - for WorldMapFrame (returns self as the canvas)
    methods.set("GetCanvas", lua.create_function(|_, ud: LightUserData| {
        let id = lud_to_id(ud);
        Ok(frame_lud(id))
    })?)?;
    Ok(())
}

/// Minimap core: zoom, ping, blips.
fn add_minimap_core_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetZoom", lua.create_function(|_, _ud: LightUserData| Ok(0))?)?;
    methods.set("SetZoom", lua.create_function(|_, (_ud, _zoom): (LightUserData, i32)| Ok(()))?)?;
    methods.set("GetZoomLevels", lua.create_function(|_, _ud: LightUserData| Ok(5))?)?;
    methods.set("GetPingPosition", lua.create_function(|_, _ud: LightUserData| Ok((0.0f64, 0.0f64)))?)?;
    methods.set("PingLocation", lua.create_function(|_, (_ud, _x, _y): (LightUserData, f64, f64)| Ok(()))?)?;
    methods.set("UpdateBlips", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    Ok(())
}

/// Minimap texture setters (no-op stubs).
fn add_minimap_texture_setters(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetBlipTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetMaskTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetIconTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetPOIArrowTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetCorpsePOIArrowTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetStaticPOIArrowTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    Ok(())
}

/// Minimap quest/task/arch blob setters (no-op stubs).
fn add_minimap_blob_setters(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetQuestBlobInsideTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetQuestBlobInsideAlpha", lua.create_function(|_, (_ud, _alpha): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetQuestBlobOutsideTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetQuestBlobOutsideAlpha", lua.create_function(|_, (_ud, _alpha): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetQuestBlobRingTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetQuestBlobRingScalar", lua.create_function(|_, (_ud, _scalar): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetQuestBlobRingAlpha", lua.create_function(|_, (_ud, _alpha): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetTaskBlobInsideTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetTaskBlobInsideAlpha", lua.create_function(|_, (_ud, _alpha): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetTaskBlobOutsideTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetTaskBlobOutsideAlpha", lua.create_function(|_, (_ud, _alpha): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetTaskBlobRingTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetTaskBlobRingScalar", lua.create_function(|_, (_ud, _scalar): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetTaskBlobRingAlpha", lua.create_function(|_, (_ud, _alpha): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetArchBlobInsideTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetArchBlobInsideAlpha", lua.create_function(|_, (_ud, _alpha): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetArchBlobOutsideTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetArchBlobOutsideAlpha", lua.create_function(|_, (_ud, _alpha): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetArchBlobRingTexture", lua.create_function(|_, (_ud, _asset): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetArchBlobRingScalar", lua.create_function(|_, (_ud, _scalar): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetArchBlobRingAlpha", lua.create_function(|_, (_ud, _alpha): (LightUserData, f32)| Ok(()))?)?;
    Ok(())
}

/// ScrollingMessageFrame and EditBox stubs.
fn add_scrolling_message_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetTextCopyable", lua.create_function(|_, (_ud, _copyable): (LightUserData, bool)| Ok(()))?)?;
    methods.set("SetInsertMode", lua.create_function(|_, (_ud, _mode): (LightUserData, String)| Ok(()))?)?;
    methods.set("SetFading", lua.create_function(|_, (_ud, _fading): (LightUserData, bool)| Ok(()))?)?;
    methods.set("SetFadeDuration", lua.create_function(|_, (_ud, _duration): (LightUserData, f32)| Ok(()))?)?;
    methods.set("SetTimeVisible", lua.create_function(|_, (_ud, _time): (LightUserData, f32)| Ok(()))?)?;
    Ok(())
}

/// Alert subsystem, data provider, and EditMode stubs.
fn add_alert_and_data_provider_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // AddQueuedAlertFrameSubSystem(system) - for AlertFrame
    methods.set("AddQueuedAlertFrameSubSystem", lua.create_function(|lua, (_ud, _args): (LightUserData, mlua::MultiValue)| {
        let subsystem = lua.create_table()?;
        subsystem.set(
            "SetCanShowMoreConditionFunc",
            lua.create_function(|_, (_self, _func): (Value, Value)| Ok(()))?,
        )?;
        subsystem.set(
            "AddAlert",
            lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
        )?;
        subsystem.set(
            "RemoveAlert",
            lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?,
        )?;
        subsystem.set(
            "ClearAllAlerts",
            lua.create_function(|_, _self: Value| Ok(()))?,
        )?;
        Ok(Value::Table(subsystem))
    })?)?;

    // AddDataProvider(provider) - for WorldMapFrame (used by HereBeDragons)
    methods.set("AddDataProvider", lua.create_function(|_, (_ud, _provider): (LightUserData, Value)| Ok(()))?)?;

    // RemoveDataProvider(provider) - for WorldMapFrame
    methods.set("RemoveDataProvider", lua.create_function(|_, (_ud, _provider): (LightUserData, Value)| Ok(()))?)?;

    // UseRaidStylePartyFrames() -> bool (for EditModeManagerFrame)
    methods.set("UseRaidStylePartyFrames", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;

    // EditModeSystemMixin stubs - delegate to mixin if present, otherwise
    // return safe defaults (in default position, not initialized).
    methods.set("IsInDefaultPosition", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        if let Some((func, ud)) = super::methods_helpers::get_mixin_override(lua, id, "IsInDefaultPosition") {
            return func.call::<bool>(ud);
        }
        Ok(true)
    })?)?;
    methods.set("IsInitialized", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        if let Some((func, ud)) = super::methods_helpers::get_mixin_override(lua, id, "IsInitialized") {
            return func.call::<bool>(ud);
        }
        Ok(false)
    })?)?;
    Ok(())
}
