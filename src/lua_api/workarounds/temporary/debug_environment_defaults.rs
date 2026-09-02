//! Temporary debug/environment globals for partial Blizzard loads.
//!
//! These helpers are compatibility defaults for Blizzard Lua that expects the
//! client debug environment and widget metatable helpers to exist. They are not
//! modeled simulator state, so keep them out of the central runtime bootstrap.

const DEBUG_ENVIRONMENT_DEFAULTS_LUA: &str = r#"
if AddSourceLocationExclude == nil then
  function AddSourceLocationExclude()
  end
end

if CreateSecureDelegate == nil then
  function CreateSecureDelegate(fn)
    return fn
  end
end

if GetButtonMetatable == nil then
  function GetButtonMetatable()
    if CreateFrame == nil then
      return nil
    end
    local frame = CreateFrame("Button")
    return frame and getmetatable(frame) or nil
  end
end

if GetEditBoxMetatable == nil then
  function GetEditBoxMetatable()
    if CreateFrame == nil then
      return nil
    end
    local frame = CreateFrame("EditBox")
    return frame and getmetatable(frame) or nil
  end
end

if secretwrap == nil then
  function secretwrap(fn)
    return fn
  end
end

-- secretunwrap(value) hands back the plain value of a secret; the simulator
-- has no secret values, so every value is already plain
-- (Blizzard_AuraContainerGroups.lua:229 keys its frame map with it).
if secretunwrap == nil then
  function secretunwrap(value)
    return value
  end
end

if GetCallstackHeight == nil then
  function GetCallstackHeight()
    return 0
  end
end

if SetErrorCallstackHeight == nil then
  function SetErrorCallstackHeight()
  end
end

if RunScript == nil then
  function RunScript(script)
    if type(script) ~= "string" then
      return nil
    end
    local chunk, err = loadstring(script)
    if not chunk then
      error(err)
    end
    return chunk()
  end
end

if CallErrorHandler == nil then
  function CallErrorHandler(message)
    if message == nil or message == "unknown" then
      return message
    end
    local handler = geterrorhandler and geterrorhandler()
    if type(handler) == "function" then
      return handler(message)
    end
    error(message)
  end
end

if GetErrorCallstackHeight == nil then
  function GetErrorCallstackHeight()
    return 0
  end
end

if debugstack == nil then
  local function debugstack_source(info)
    local source = info and info.source or nil
    if type(source) ~= "string" or source == "" then
      source = info and info.short_src or "?"
    end
    if source:sub(1, 1) == "@" then
      return "[" .. source:sub(2) .. "]"
    end
    return source
  end

  local function debugstack_line(info)
    local source = debugstack_source(info)
    local currentline = tonumber(info and info.currentline) or -1
    if currentline > 0 then
      source = source .. ":" .. currentline
    else
      source = source .. ":"
    end

    if info and type(info.name) == "string" and info.name ~= "" then
      return source .. ": in function '" .. info.name .. "'"
    end
    if info and info.what == "main" then
      return source .. ": in main chunk"
    end
    if info and type(info.linedefined) == "number" and info.linedefined > 0 then
      return source .. ": in function <" .. debugstack_source(info) .. ":" .. info.linedefined .. ">"
    end
    return source .. " ?"
  end

  function debugstack(level, count1, count2)
    if not debug or not debug.getinfo then
      return ""
    end
    local start = (tonumber(level) or 1) + 1
    local lines = {}
    local depth = start
    while true do
      local info = debug.getinfo(depth, "Sln")
      if not info then break end
      lines[#lines + 1] = debugstack_line(info)
      depth = depth + 1
    end

    if count1 or count2 then
      local top = tonumber(count1) or 12
      local bottom = tonumber(count2) or 10
      if #lines > top + bottom then
        local kept = {}
        for i = 1, top do kept[#kept + 1] = lines[i] end
        kept[#kept + 1] = "..."
        for i = #lines - bottom + 1, #lines do kept[#kept + 1] = lines[i] end
        return table.concat(kept, "\n") .. "\n"
      end
    end
    local stack = table.concat(lines, "\n")
    if stack ~= "" then stack = stack .. "\n" end
    return stack
  end
end

if debuglocals == nil then
  function debuglocals(level)
    if not debug or not debug.getinfo or not debug.getlocal then
      return ""
    end
    local start = (tonumber(level) or 1) + 1
    local info = debug.getinfo(start, "fS")
    if not info then return "" end
    local parts = {}
    local i = 1
    while true do
      local name, value = debug.getlocal(start, i)
      if not name then break end
      if not name:match("^%(") then
        parts[#parts + 1] = string.format("%s = %s", name, tostring(value))
      end
      i = i + 1
    end
    return table.concat(parts, "\n")
  end
end

if debug ~= nil and debug.getfenv ~= nil then
  local __wow_debug_getfenv = debug.getfenv
  local function __wow_is_frame_backed_table(obj)
    if type(obj) ~= "table" then
      return false
    end
    local mt = getmetatable(obj)
    local index = mt and mt.__index
    return type(index) == "table"
      and (
        type(index.GetObjectType) == "function"
        or type(index.IsObjectType) == "function"
        or type(index.GetName) == "function"
      )
  end

  function debug.getfenv(obj)
    if __wow_is_frame_backed_table(obj) then
      return { obj }
    end
    return __wow_debug_getfenv(obj)
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(DEBUG_ENVIRONMENT_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_debug_environment_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local marker = function() return "wrapped" end
                if CreateSecureDelegate(marker)() ~= "wrapped" then return "secure_delegate" end
                if type(GetButtonMetatable()) ~= "table" then return "button_metatable" end
                if type(GetEditBoxMetatable()) ~= "table" then return "editbox_metatable" end
                if secretwrap(marker)() ~= "wrapped" then return "secretwrap" end
                if GetCallstackHeight() ~= 0 then return "callstack_height" end
                if GetErrorCallstackHeight() ~= 0 then return "error_callstack_height" end
                if type(debugstack) ~= "function" then return "debugstack_type" end
                if not string.find(debugstack(1), "in main chunk", 1, true) then return "debugstack_value" end
                if type(debuglocals) ~= "function" then return "debuglocals_type" end
                if type(debuglocals(1)) ~= "string" then return "debuglocals_value" end
                SetErrorCallstackHeight(4)
                AddSourceLocationExclude("example.lua")
                return "ok"
                "#,
            )
            .expect("debug environment defaults probe should run");

        assert_eq!(result, "ok");
    }
}
