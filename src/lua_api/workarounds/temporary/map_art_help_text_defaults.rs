//! Temporary `C_Map.GetMapArtHelpTextPosition` default.
//!
//! Per-map help-text positions are not modeled: the exported UiMapArt,
//! UiMapArtStyleLayer and UiMapArtTile tables carry no position column. Replace
//! this when a map-art help-position source exists.

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        return Ok(());
    }
    lua.exec(
        r#"
        if rawget(C_Map, "GetMapArtHelpTextPosition") == nil then
            function C_Map.GetMapArtHelpTextPosition(uiMapID)
                if type(uiMapID) ~= "number" then
                    error("bad argument #1 to 'GetMapArtHelpTextPosition' (number expected)", 2)
                end
                return 2 -- Enum.MapCanvasPosition.BottomRight
            end
        end
    "#,
    )?;
    Ok(())
}
