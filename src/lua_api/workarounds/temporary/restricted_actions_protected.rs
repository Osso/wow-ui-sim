//! Existing protected-function permission placeholder; protected-call policy is not modeled here.

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(
        r#"
C_RestrictedActions = C_RestrictedActions or __wow_namespace()
if rawget(C_RestrictedActions, "CheckAllowProtectedFunctions") == nil then
    function C_RestrictedActions.CheckAllowProtectedFunctions()
        return true
    end
end
"#,
    )?;
    Ok(())
}
