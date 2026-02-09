//! Miscellaneous frame-type-specific method stubs (Minimap, ScrollingMessage, Alerts, etc.).

use super::FrameHandle;
use mlua::{UserDataMethods, Value};
use std::rc::Rc;

/// Add all miscellaneous frame-type-specific methods.
pub fn add_misc_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_minimap_methods(methods);
    add_scrolling_message_methods(methods);
    add_alert_and_data_provider_methods(methods);
    // DropdownButtonMixin stub
    methods.add_method("IsMenuOpen", |_, _this, ()| Ok(false));
    // StaticPopupElementMixin stub (dialog ownership tracking)
    methods.add_method("SetOwningDialog", |_, _this, _dialog: Value| Ok(()));
    // GuildRenameFrameMixin / layout tracking methods (no-op in simulator)
    methods.add_method("RegisterFontStrings", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("RegisterFrames", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("RegisterBackgroundTexture", |_, _this, _args: mlua::MultiValue| Ok(()));
}

/// Minimap and WorldMap stubs.
fn add_minimap_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_minimap_core_methods(methods);
    add_minimap_texture_setters(methods);
    add_minimap_blob_setters(methods);
    // GetCanvas() - for WorldMapFrame (returns self as the canvas)
    methods.add_method("GetCanvas", |lua, this, ()| {
        let handle = FrameHandle {
            id: this.id,
            state: Rc::clone(&this.state),
        };
        lua.create_userdata(handle)
    });
}

/// Minimap core: zoom, ping, blips.
fn add_minimap_core_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetZoom", |_, _this, ()| Ok(0));
    methods.add_method("SetZoom", |_, _this, _zoom: i32| Ok(()));
    methods.add_method("GetZoomLevels", |_, _this, ()| Ok(5));
    methods.add_method("GetPingPosition", |_, _this, ()| Ok((0.0f64, 0.0f64)));
    methods.add_method("PingLocation", |_, _this, (_x, _y): (f64, f64)| Ok(()));
    methods.add_method("UpdateBlips", |_, _this, ()| Ok(()));
}

/// Minimap texture setters (no-op stubs).
fn add_minimap_texture_setters<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetBlipTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetMaskTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetIconTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetPOIArrowTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetCorpsePOIArrowTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetStaticPOIArrowTexture", |_, _this, _asset: Value| Ok(()));
}

/// Minimap quest/task/arch blob setters (no-op stubs).
fn add_minimap_blob_setters<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetQuestBlobInsideTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetQuestBlobInsideAlpha", |_, _this, _alpha: f32| Ok(()));
    methods.add_method("SetQuestBlobOutsideTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetQuestBlobOutsideAlpha", |_, _this, _alpha: f32| Ok(()));
    methods.add_method("SetQuestBlobRingTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetQuestBlobRingScalar", |_, _this, _scalar: f32| Ok(()));
    methods.add_method("SetQuestBlobRingAlpha", |_, _this, _alpha: f32| Ok(()));
    methods.add_method("SetTaskBlobInsideTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetTaskBlobInsideAlpha", |_, _this, _alpha: f32| Ok(()));
    methods.add_method("SetTaskBlobOutsideTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetTaskBlobOutsideAlpha", |_, _this, _alpha: f32| Ok(()));
    methods.add_method("SetTaskBlobRingTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetTaskBlobRingScalar", |_, _this, _scalar: f32| Ok(()));
    methods.add_method("SetTaskBlobRingAlpha", |_, _this, _alpha: f32| Ok(()));
    methods.add_method("SetArchBlobInsideTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetArchBlobInsideAlpha", |_, _this, _alpha: f32| Ok(()));
    methods.add_method("SetArchBlobOutsideTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetArchBlobOutsideAlpha", |_, _this, _alpha: f32| Ok(()));
    methods.add_method("SetArchBlobRingTexture", |_, _this, _asset: Value| Ok(()));
    methods.add_method("SetArchBlobRingScalar", |_, _this, _scalar: f32| Ok(()));
    methods.add_method("SetArchBlobRingAlpha", |_, _this, _alpha: f32| Ok(()));
}

/// ScrollingMessageFrame and EditBox stubs.
fn add_scrolling_message_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetTextCopyable", |_, _this, _copyable: bool| Ok(()));
    methods.add_method("SetInsertMode", |_, _this, _mode: String| Ok(()));
    methods.add_method("SetFading", |_, _this, _fading: bool| Ok(()));
    methods.add_method("SetFadeDuration", |_, _this, _duration: f32| Ok(()));
    methods.add_method("SetTimeVisible", |_, _this, _time: f32| Ok(()));
}

/// Alert subsystem, data provider, and EditMode stubs.
fn add_alert_and_data_provider_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // AddQueuedAlertFrameSubSystem(system) - for AlertFrame
    methods.add_method(
        "AddQueuedAlertFrameSubSystem",
        |lua, _this, _args: mlua::MultiValue| {
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
        },
    );

    // AddDataProvider(provider) - for WorldMapFrame (used by HereBeDragons)
    methods.add_method("AddDataProvider", |_, _this, _provider: mlua::Value| Ok(()));

    // RemoveDataProvider(provider) - for WorldMapFrame
    methods.add_method("RemoveDataProvider", |_, _this, _provider: mlua::Value| {
        Ok(())
    });

    // UseRaidStylePartyFrames() -> bool (for EditModeManagerFrame)
    methods.add_method("UseRaidStylePartyFrames", |_, _this, ()| Ok(false));

    // EditModeSystemMixin stubs - delegate to mixin if present, otherwise
    // return safe defaults (in default position, not initialized).
    methods.add_method("IsInDefaultPosition", |lua, this, ()| {
        if let Some((func, ud)) = super::methods_helpers::get_mixin_override(lua, this.id, "IsInDefaultPosition") {
            return func.call::<bool>(ud);
        }
        Ok(true)
    });
    methods.add_method("IsInitialized", |lua, this, ()| {
        if let Some((func, ud)) = super::methods_helpers::get_mixin_override(lua, this.id, "IsInitialized") {
            return func.call::<bool>(ud);
        }
        Ok(false)
    });
}
