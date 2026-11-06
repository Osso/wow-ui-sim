//! Temporary portrait texture helpers.
//!
//! Real WoW derives bag portrait art from equipped inventory data, live unit
//! display data, or creature display IDs. The simulator only has shallow class
//! and inventory icon paths today, so these helpers keep Blizzard portrait
//! textures rendering while richer model/portrait state is still missing.

const CONTAINER_PORTRAIT_TEXTURE_LUA: &str = r#"
if SetPortraitTexture == nil then
  function SetPortraitTexture(texture, unit, _disablePortraitMask)
    if not texture then
      return
    end

    if UnitIsPlayer ~= nil and UnitIsPlayer(unit) then
      local _, classFile = UnitClass(unit)
      if classFile then
        local coords = CLASS_ICON_TCOORDS and CLASS_ICON_TCOORDS[classFile]
        if coords and texture.SetTexture and texture.SetTexCoord then
          texture:SetTexture("Interface\\TargetingFrame\\UI-Classes-Circles")
          texture:SetTexCoord(unpack(coords))
          return
        end

        local atlas = GetClassAtlas and GetClassAtlas(classFile)
        if atlas and texture.SetAtlas then
          texture:SetAtlas(atlas)
          return
        end
      end
    end

    if texture.SetTexture then
      texture:SetTexture("Interface\\ICONS\\INV_Misc_QuestionMark")
    end
  end
end

if SetPortraitTextureFromCreatureDisplayID == nil then
  function SetPortraitTextureFromCreatureDisplayID(texture, _creatureDisplayID)
    if texture and texture.SetTexture then
      texture:SetTexture("Interface\\ICONS\\INV_Misc_QuestionMark")
    end
  end
end

-- C_Container's metatable can expose generated method stubs through __index;
-- rawget keeps this workaround tied to the explicit table slot.
if C_Container ~= nil and type(rawget(C_Container, "SetBagPortraitTexture")) ~= "function" then
  function C_Container.SetBagPortraitTexture(texture, bagID)
    if texture ~= nil then
      local inventoryID = C_Container.ContainerIDToInventoryID and C_Container.ContainerIDToInventoryID(bagID)
      if inventoryID == nil and type(bagID) == "number" then
        inventoryID = 20 + bagID
      end
      local portraitTexture = GetInventoryItemTexture("player", inventoryID or 20)
      texture:SetTexture(portraitTexture)
    end
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CONTAINER_PORTRAIT_TEXTURE_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_bag_portrait_texture_helper_when_missing() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            rawset(C_Container, "SetBagPortraitTexture", nil)
            C_Container.ContainerIDToInventoryID = function()
              return 21
            end
            GetInventoryItemTexture = function()
              return "portrait_texture"
            end
            "#,
        )
        .expect("fixture should reset helper and inventory texture");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("container portrait helper should apply");
        }
        let helper_type: String = env
            .eval("return type(C_Container.SetBagPortraitTexture)")
            .expect("helper type probe should run");
        assert_eq!(helper_type, "function");

        let texture: String = env
            .eval(
                r#"
                local portrait = {
                  value = nil,
                  SetTexture = function(self, texture)
                    self.value = texture
                  end,
                }
                C_Container.SetBagPortraitTexture(portrait, Enum.BagIndex.Bag_1)
                return portrait.value
                "#,
            )
            .expect("bag portrait helper should run");

        assert_eq!(texture, "portrait_texture");
    }

    modern_engine_namespace_fixture! {
    #[test]
    fn installs_unit_portrait_texture_helpers_when_missing() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            rawset(_G, "SetPortraitTexture", nil)
            rawset(_G, "SetPortraitTextureFromCreatureDisplayID", nil)
            UnitIsPlayer = function()
              return true
            end
            UnitClass = function()
              return "Paladin", "PALADIN"
            end
            CLASS_ICON_TCOORDS = {
              PALADIN = { 0, 0.25, 0, 0.25 },
            }
            "#,
        )
        .expect("fixture should reset portrait helpers");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("portrait helpers should apply");
        }

        let result: (i64, f64, f64, f64, f64, i64) = env
            .eval(
                r#"
                local frame = CreateFrame("Frame")
                local playerPortrait = frame:CreateTexture(nil, "ARTWORK")
                SetPortraitTexture(playerPortrait, "player")
                local left, top, _, bottom, right = playerPortrait:GetTexCoord()

                local creaturePortrait = frame:CreateTexture(nil, "ARTWORK")
                SetPortraitTextureFromCreatureDisplayID(creaturePortrait, 1)

                return playerPortrait:GetTexture(),
                    left,
                    right,
                    top,
                    bottom,
                    creaturePortrait:GetTexture()
                "#,
            )
            .expect("portrait helpers should run");

        assert_eq!(result.0, 237669);
        assert_eq!(
            (result.1, result.2, result.3, result.4),
            (0.0, 0.25, 0.0, 0.25)
        );
        assert_eq!(result.5, 134400);
    }
    }

    #[test]
    fn preserves_existing_bag_portrait_texture_helper() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_Container.SetBagPortraitTexture = function(texture)
              texture:SetTexture("custom")
            end
            "#,
        )
        .expect("fixture should install custom helper");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("container portrait helper should apply");
        }

        let texture: String = env
            .eval(
                r#"
                local frame = CreateFrame("Frame")
                local portrait = frame:CreateTexture(nil, "ARTWORK")
                C_Container.SetBagPortraitTexture(portrait, Enum.BagIndex.Bag_1)
                return portrait:GetTexture()
                "#,
            )
            .expect("custom bag portrait helper should run");

        assert_eq!(texture, "custom");
    }
}
