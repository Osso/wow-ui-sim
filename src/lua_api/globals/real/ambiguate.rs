//! Deterministic public-name shortening with the 12.0.5 secret-context boundary.

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    let source = format!(
        r#"
local rejectSecretContext = {reject_secret_context}
local isSecretContext = issecretvalue
function Ambiguate(fullName, context)
    if rejectSecretContext and isSecretContext(context) then
        error("Ambiguate argument #2 must not be secret", 2)
    end
    if context == "none" then
        return fullName
    end
    return string.match(fullName, "^(.-)%-.+$") or fullName
end
"#,
        reject_secret_context = cfg!(feature = "retail-12-0-5"),
    );
    lua.exec(&source)?;
    Ok(())
}
