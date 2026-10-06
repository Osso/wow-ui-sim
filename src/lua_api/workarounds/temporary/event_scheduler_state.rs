//! Unsupported scheduler continent-name query; no continent/event model exists.
const LUA: &str = r#"
if rawget(C_EventScheduler, "GetActiveContinentName") == nil then
    function C_EventScheduler.GetActiveContinentName()
        return nil
    end
end
"#;
pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(LUA)?;
    Ok(())
}
