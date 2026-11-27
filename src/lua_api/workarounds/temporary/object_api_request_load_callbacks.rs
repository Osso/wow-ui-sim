//! Temporary synchronous ObjectAPI request-load callbacks.
//!
//! Blizzard_ObjectAPI registers listener objects for async item/spell cache
//! loads. The simulator does not model those caches yet, so item and spell
//! request-load calls immediately flush their listener callbacks.

const OBJECT_API_REQUEST_LOAD_CALLBACKS_LUA: &str = r#"
local function __wow_fire_listener_callbacks(listener, id)
  if listener ~= nil and type(listener.FireCallbacks) == "function" then
    listener:FireCallbacks(id)
  end
end

if C_Spell ~= nil and type(rawget(C_Spell, "RequestLoadSpellData")) ~= "function" then
  function C_Spell.RequestLoadSpellData(spellID)
    __wow_fire_listener_callbacks(SpellEventListener, spellID)
    return true
  end
end
"#;

/// Retail and Forever register the modeled Rust `C_Item.RequestLoadItemDataByID`.
const ITEM_REQUEST_LOAD_CALLBACK_LUA: &str = r#"
if C_Item ~= nil and type(rawget(C_Item, "RequestLoadItemDataByID")) ~= "function" then
  function C_Item.RequestLoadItemDataByID(itemID)
    if ItemEventListener ~= nil and type(ItemEventListener.FireCallbacks) == "function" then
      ItemEventListener:FireCallbacks(itemID)
    end
    return true
  end
end
"#;

fn item_request_workaround_applies() -> bool {
    use crate::client_profile::{ACTIVE, ClientProfile};
    !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(OBJECT_API_REQUEST_LOAD_CALLBACKS_LUA)?;
    if item_request_workaround_applies() {
        lua.exec(ITEM_REQUEST_LOAD_CALLBACK_LUA)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    /// Retail and Forever register the Rust item request instead.
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_item_and_spell_request_load_callbacks_when_missing() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            rawset(C_Item, "RequestLoadItemDataByID", nil)
            rawset(C_Spell, "RequestLoadSpellData", nil)
            item_fired = nil
            spell_fired = nil
            ItemEventListener = {
              FireCallbacks = function(self, itemID)
                item_fired = itemID
              end,
            }
            SpellEventListener = {
              FireCallbacks = function(self, spellID)
                spell_fired = spellID
              end,
            }
            "#,
        )
        .expect("fixture should reset request-load callbacks");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("request-load callbacks should apply");
        }

        let (item_result, item_fired, spell_result, spell_fired): (bool, i32, bool, i32) = env
            .eval(
                r#"
                local itemResult = C_Item.RequestLoadItemDataByID(6948)
                local spellResult = C_Spell.RequestLoadSpellData(35395)
                return itemResult, item_fired, spellResult, spell_fired
                "#,
            )
            .expect("request-load callbacks should run");

        assert!(item_result);
        assert_eq!(item_fired, 6948);
        assert!(spell_result);
        assert_eq!(spell_fired, 35395);
    }

    #[test]
    fn preserves_existing_request_load_callbacks() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_Item.RequestLoadItemDataByID = function()
              return "item-existing"
            end
            C_Spell.RequestLoadSpellData = function()
              return "spell-existing"
            end
            "#,
        )
        .expect("fixture should install existing callbacks");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("request-load callbacks should apply");
        }

        let (item_result, spell_result): (String, String) = env
            .eval(
                r#"
                return C_Item.RequestLoadItemDataByID(1), C_Spell.RequestLoadSpellData(2)
                "#,
            )
            .expect("existing callbacks should run");

        assert_eq!(item_result, "item-existing");
        assert_eq!(spell_result, "spell-existing");
    }
}
