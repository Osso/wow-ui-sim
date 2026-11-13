//! Temporary fallbacks for small SharedXML utility globals.
//!
//! Blizzard normally supplies these from SharedXMLBase or Settings definition
//! files. Keep shallow compatibility defaults here for isolated addon closure
//! tests that do not load those providers.

const SHARED_XML_UTILITY_DEFAULTS_LUA: &str = r##"
if type(CreateAnchor) ~= "function" then
  function CreateAnchor(point, relativeTo, relativePoint, x, y)
    return {
      point = point,
      relativeTo = relativeTo,
      relativePoint = relativePoint or point,
      x = x or 0,
      y = y or 0,
    }
  end
end

if type(GetFinalNameFromTextureKit) ~= "function" then
  function GetFinalNameFromTextureKit(formatString, textureKit)
    if type(formatString) ~= "string" then
      return nil
    end
    if textureKit == nil or textureKit == "" then
      return (formatString:gsub("%%s_?", ""):gsub("_$", ""))
    end
    return formatString:gsub("%%s", textureKit)
  end
end

if TextureKitConstants == nil then
  TextureKitConstants = {
    SetVisibility = true,
    DoNotSetVisibility = false,
    UseAtlasSize = true,
    IgnoreAtlasSize = false,
    AddressModeClamp = 1,
    AddressModeWrap = 2,
    AddressModeAllowAssetToDetermine = 3,
  }
end

if type(CopyValuesAsKeys) ~= "function" then
  function CopyValuesAsKeys(values)
    local result = {}
    if type(values) ~= "table" then
      return result
    end
    for _, value in pairs(values) do
      result[value] = true
    end
    return result
  end
end

if EnumUtil == nil then
  EnumUtil = {}
end

if EnumUtil.MakeEnum == nil then
  function EnumUtil.MakeEnum(...)
    local enum = {}
    for index = 1, select("#", ...) do
      local name = select(index, ...)
      enum[name] = index
    end
    return enum
  end
end

if CreateCounter == nil then
  function CreateCounter()
    local nextID = 0
    return function()
      nextID = nextID + 1
      return nextID
    end
  end
end

if GetOrCreateTableEntry == nil then
  function GetOrCreateTableEntry(tbl, key)
    local value = tbl[key]
    if value == nil then
      value = {}
      tbl[key] = value
    end
    return value
  end
end

if GenerateClosure == nil then
  function GenerateClosure(fn, ...)
    local bound = { n = select("#", ...), ... }
    return function(...)
      local args = {}
      local argCount = 0
      for i = 1, bound.n do
        argCount = argCount + 1
        args[argCount] = bound[i]
      end
      for i = 1, select("#", ...) do
        argCount = argCount + 1
        args[argCount] = select(i, ...)
      end
      return fn(unpack(args, 1, argCount))
    end
  end
end

if type(GetMicroIconForRole) ~= "function" then
  function GetMicroIconForRole(role)
    if type(role) ~= "string" then
      return "roleicon"
    end
    return "roleicon-" .. role:lower()
  end
end

if type(PingSystemInitializer) ~= "function" then
  function PingSystemInitializer(_category)
  end
end
"##;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SHARED_XML_UTILITY_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_shared_xml_utility_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("shared XML utility defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                local anchor = CreateAnchor("TOPLEFT", UIParent, nil, 10, -20)
                if anchor.point ~= "TOPLEFT" or anchor.relativePoint ~= "TOPLEFT" then return "anchor" end
                if GetFinalNameFromTextureKit("%s-topper", "ardenweald") ~= "ardenweald-topper" then return "texture_kit" end
                if GetFinalNameFromTextureKit("%s_topper", "") ~= "topper" then return "texture_kit_empty" end
                local keys = CopyValuesAsKeys({ "a", "b" })
                if keys.a ~= true or keys.b ~= true then return "copy_values" end
                if type(TextureKitConstants) ~= "table" then return "texture_constants" end
                if TextureKitConstants.SetVisibility ~= true then return "texture_set_visibility" end
                if TextureKitConstants.DoNotSetVisibility ~= false then return "texture_do_not_set_visibility" end
                if TextureKitConstants.UseAtlasSize ~= true then return "texture_use_atlas" end
                if TextureKitConstants.IgnoreAtlasSize ~= false then return "texture_ignore_atlas" end
                if TextureKitConstants.AddressModeClamp ~= 1 then return "texture_clamp" end
                if TextureKitConstants.AddressModeWrap ~= 2 then return "texture_wrap" end
                if TextureKitConstants.AddressModeAllowAssetToDetermine ~= 3 then return "texture_asset_address" end
                local enum = EnumUtil.MakeEnum("Foo", "Bar")
                if enum.Foo ~= 1 or enum.Bar ~= 2 then return "enum" end
                local counter = CreateCounter()
                if counter() ~= 1 or counter() ~= 2 then return "counter" end
                local tableEntryStore = {}
                local entry = GetOrCreateTableEntry(tableEntryStore, "entry")
                entry.value = 9
                if GetOrCreateTableEntry(tableEntryStore, "entry").value ~= 9 then return "table_entry" end
                local closure = GenerateClosure(function(first, second, third)
                  return first .. second .. third
                end, "a", "b")
                if closure("c") ~= "abc" then return "closure" end
                if GetMicroIconForRole("TANK") ~= "roleicon-tank" then return "role_icon" end
                PingSystemInitializer({})
                return "ok"
                "#,
            )
            .expect("shared XML utility defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_shared_xml_utility_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function CreateAnchor() return "existing_anchor" end
            function GetFinalNameFromTextureKit() return "existing_texture" end
            function CopyValuesAsKeys() return "existing_copy" end
            EnumUtil = { MakeEnum = function() return "existing_enum" end }
            function CreateCounter() return "existing_counter" end
            function GetOrCreateTableEntry() return "existing_entry" end
            function GenerateClosure() return "existing_closure" end
            TextureKitConstants = { UseAtlasSize = "existing_texture_constant" }
            "#,
        )
        .expect("fixture should install existing utility globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("shared XML utility defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if CreateAnchor() ~= "existing_anchor" then return "overwrote_anchor" end
                if GetFinalNameFromTextureKit() ~= "existing_texture" then return "overwrote_texture" end
                if CopyValuesAsKeys() ~= "existing_copy" then return "overwrote_copy" end
                if EnumUtil.MakeEnum() ~= "existing_enum" then return "overwrote_enum" end
                if CreateCounter() ~= "existing_counter" then return "overwrote_counter" end
                if GetOrCreateTableEntry() ~= "existing_entry" then return "overwrote_table_entry" end
                if GenerateClosure() ~= "existing_closure" then return "overwrote_closure" end
                if TextureKitConstants.UseAtlasSize ~= "existing_texture_constant" then return "overwrote_texture_constants" end
                if type(GetMicroIconForRole) ~= "function" then return "missing_role_icon" end
                if type(PingSystemInitializer) ~= "function" then return "missing_ping" end
                return "ok"
                "#,
            )
            .expect("shared XML utility preservation probe should run");

        assert_eq!(result, "ok");
    }
}
