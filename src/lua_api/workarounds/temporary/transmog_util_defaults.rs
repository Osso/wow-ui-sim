//! Temporary `TransmogUtil` location helpers.
//!
//! Retail location helpers belong to the original selected TransmogShared Lua.
//! Retain the historical location stand-ins only on other profiles pending their
//! own ownership proof. The unrelated best-item helper remains unresolved.

#[cfg(not(feature = "client-retail"))]
const TRANSMOG_UTIL_LOCATION_DEFAULTS_LUA: &str = r#"
local function __wow_make_transmog_util_location(slotName, slotID, transmogType, modification)
  local location = {
    slotName = tostring(slotName or ""),
    slotID = tonumber(slotID) or 0,
    transmogType = tonumber(transmogType) or 0,
    modification = tonumber(modification) or 0,
  }

  function location:IsAppearance()
    return true
  end

  function location:IsIllusion()
    return false
  end

  function location:IsEitherHand()
    return self.slotName == "MAINHANDSLOT" or self.slotName == "SECONDHANDSLOT"
  end

  function location:IsSecondary()
    return self.slotName == "SECONDHANDSLOT"
  end

  function location:IsMainHand()
    return self.slotName == "MAINHANDSLOT"
  end

  function location:GetSlotName()
    return self.slotName
  end

  function location:IsEqual(other)
    return type(other) == "table"
      and self.slotName == other.slotName
      and self.slotID == other.slotID
      and self.transmogType == other.transmogType
      and self.modification == other.modification
  end

  return location
end

TransmogUtil = type(TransmogUtil) == "table" and TransmogUtil or {}
if rawget(TransmogUtil, "GetTransmogLocation") == nil then
  function TransmogUtil.GetTransmogLocation(slotName, transmogType, modification)
    return __wow_make_transmog_util_location(slotName, 0, transmogType, modification)
  end
end
if rawget(TransmogUtil, "CreateTransmogLocation") == nil then
  function TransmogUtil.CreateTransmogLocation(slotID, transmogType, modification)
    return __wow_make_transmog_util_location("", slotID, transmogType, modification)
  end
end
"#;

// Ownership of this separate historical helper remains unresolved.
const TRANSMOG_UTIL_BEST_ITEM_DEFAULT_LUA: &str = r#"
TransmogUtil = type(TransmogUtil) == "table" and TransmogUtil or {}
if rawget(TransmogUtil, "GetBestItemModifiedAppearanceID") == nil then
  function TransmogUtil.GetBestItemModifiedAppearanceID(_itemID)
    return nil
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(feature = "client-retail"))]
    lua.exec(TRANSMOG_UTIL_LOCATION_DEFAULTS_LUA)?;
    lua.exec(TRANSMOG_UTIL_BEST_ITEM_DEFAULT_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(feature = "client-retail"))]
    #[test]
    fn installs_transmog_util_location_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("transmog defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                local head = TransmogUtil.GetTransmogLocation("HEADSLOT", 0, false)
                local head2 = TransmogUtil.GetTransmogLocation("HEADSLOT", 0, false)
                local mainHand = TransmogUtil.GetTransmogLocation("MAINHANDSLOT", 0, false)
                local created = TransmogUtil.CreateTransmogLocation(1, 0, 0)
                if head:GetSlotName() ~= "HEADSLOT" then return "slot_name" end
                if not head:IsAppearance() or head:IsIllusion() then return "kind" end
                if mainHand:IsEitherHand() ~= true or mainHand:IsMainHand() ~= true then return "hand" end
                if not head:IsEqual(head2) then return "equal" end
                if created.slotID ~= 1 then return "created_slot" end
                if TransmogUtil.GetBestItemModifiedAppearanceID(6948) ~= nil then return "appearance_id" end
                return "ok"
                "#,
            )
            .expect("transmog defaults should be callable");

        assert_eq!(result, "ok");
    }
    #[cfg(feature = "client-retail")]
    #[test]
    fn retail_defers_locations_to_original_lua_and_keeps_best_item_separate() {
        let env = WowLuaEnv::new().expect("environment");
        env.exec(
            r#"
            assert(TransmogUtil.GetTransmogLocation == nil)
            assert(TransmogUtil.CreateTransmogLocation == nil)
            assert(type(TransmogUtil.GetBestItemModifiedAppearanceID) == 'function')
            assert(TransmogUtil.GetBestItemModifiedAppearanceID(6948) == nil)
            originalGet = function() return 'original get' end
            originalCreate = function() return 'original create' end
            TransmogUtil.GetTransmogLocation = originalGet
            TransmogUtil.CreateTransmogLocation = originalCreate
        "#,
        )
        .unwrap();
        super::apply_bootstrap(&mut env.lua.borrow_mut()).unwrap();
        env.exec(
            r#"
            assert(TransmogUtil.GetTransmogLocation == originalGet)
            assert(TransmogUtil.CreateTransmogLocation == originalCreate)
        "#,
        )
        .unwrap();
    }
}
