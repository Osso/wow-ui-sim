//! Temporary SecureTypes compatibility defaults.
//!
//! The simulator has taint primitives, but does not model Blizzard's full
//! secure container implementation yet. Keep these compatibility containers in
//! the workaround layer until the secure environment owns them.

const SECURE_TYPES_DEFAULTS_LUA: &str = r#"
if SecureTypes == nil then
  SecureTypes = {}
end

do
  local __wow_original_securecall = securecall
  if type(__wow_original_securecall) == "function" and not rawget(_G, "__wow_securecall_accepts_names") then
    function securecall(fn, ...)
      if type(fn) == "string" then
        fn = _G[fn]
      end
      return __wow_original_securecall(fn, ...)
    end

    rawset(_G, "__wow_securecall_accepts_names", true)
  end
end

local function __wow_securetypes_call(fn, ...)
  if type(securecallfunction) == "function" then
    return securecallfunction(fn, ...)
  end
  return fn(...)
end

SecureTypes.CreateSecureMap = SecureTypes.CreateSecureMap or function(mixin)
  local SecureMap = {}

  function SecureMap:GetValue(key)
    return __wow_securetypes_call(rawget, self.tbl, key)
  end

  function SecureMap:SetValue(key, value)
    assert(not issecretvalue(key), "attempted to store a secret key in a SecureMap")
    assert(not issecretvalue(value), "attempted to store a secret value in a SecureMap")
    self.tbl[key] = value
  end

  function SecureMap:ClearValue(key)
    self.tbl[key] = nil
  end

  function SecureMap:HasKey(key)
    return self:GetValue(key) ~= nil
  end

  function SecureMap:GetNext(key)
    return __wow_securetypes_call(next, self.tbl, key)
  end

  function SecureMap:GetSize()
    local count = 0
    for _ in pairs(self.tbl) do
      count = count + 1
    end
    return count
  end

  function SecureMap:IsEmpty()
    return self:GetNext() == nil
  end

  function SecureMap:Wipe()
    for key in pairs(self.tbl) do
      self.tbl[key] = nil
    end
  end

  function SecureMap:Enumerate()
    local iterator, tbl, index = next, self.tbl, nil
    local function Iterator(_, key)
      return __wow_securetypes_call(iterator, tbl, key)
    end

    return Iterator, nil, index
  end

  function SecureMap:ExecuteRange(func, ...)
    return secureexecuterange(self.tbl, func, ...)
  end

  function SecureMap:ExecuteTable(func)
    return __wow_securetypes_call(func, self.tbl)
  end

  function SecureMap:Insert(key, value)
    self:SetValue(key, value)
  end

  function SecureMap:Remove(key)
    local value = self:GetValue(key)
    self:ClearValue(key)
    return value
  end

  function SecureMap:Find(key)
    return self:GetValue(key)
  end

  function SecureMap:Contains(key)
    return self:HasKey(key)
  end

  function SecureMap:Clear()
    self:Wipe()
  end

  SecureMap.__index = function(t, key)
    local mapValue = SecureMap[key]
    if mapValue then
      return mapValue
    end
    return SecureMap.GetValue(t, key)
  end

  SecureMap.__newindex = function(t, key, value)
    t:SetValue(key, value)
  end

  local map = { tbl = {} }
  setmetatable(map, SecureMap)

  if mixin and type(Mixin) == "function" then
    Mixin(map, mixin)
  end

  return map
end
SecureTypes.CreateSecureFunction = SecureTypes.CreateSecureFunction or function(fn) return fn end
SecureTypes.CreateSecureNumber = SecureTypes.CreateSecureNumber or function(value) return value or 0 end
SecureTypes.CreateSecureArray = SecureTypes.CreateSecureArray or function()
  local array = {}
  local methods = {}
  function methods:Insert(value)
    self[#self + 1] = value
  end
  function methods:Remove(value)
    for index, existing in ipairs(self) do
      if existing == value then
        table.remove(self, index)
        return true
      end
    end
    return false
  end
  function methods:Clear()
    for index = #self, 1, -1 do
      self[index] = nil
    end
  end
  function methods:Enumerate()
    local index = 0
    return function()
      index = index + 1
      if index <= #self then
        return self[index]
      end
    end
  end
  function methods:FindInTableIf(predicate)
    return FindInTableIf(self, predicate)
  end
  return setmetatable(array, { __index = methods })
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SECURE_TYPES_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_secure_type_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                function __secure_types_named_add(a, b) return a + b end
                if securecall("__secure_types_named_add", 2, 3) ~= 5 then return "securecall_name" end
                local map = SecureTypes.CreateSecureMap()
                map:Insert("alpha", 7)
                if map:GetValue("alpha") ~= 7 then return "map_get" end
                if map:GetSize() ~= 1 then return "map_size" end
                if map:Remove("alpha") ~= 7 or map:GetSize() ~= 0 then return "map_remove" end
                if SecureTypes.CreateSecureFunction(__secure_types_named_add) ~= __secure_types_named_add then return "secure_function" end
                if SecureTypes.CreateSecureNumber(nil) ~= 0 or SecureTypes.CreateSecureNumber(4) ~= 4 then return "secure_number" end
                local array = SecureTypes.CreateSecureArray()
                array:Insert("a")
                array:Insert("b")
                local index, value = array:FindInTableIf(function(v) return v == "b" end)
                if index ~= 2 or value ~= "b" then return "array_find" end
                if array:Remove("a") ~= true or array[1] ~= "b" then return "array_remove" end
                array:Clear()
                if array[1] ~= nil then return "array_clear" end
                return "ok"
                "#,
            )
            .expect("secure types defaults should be callable");

        assert_eq!(result, "ok");
    }
}
