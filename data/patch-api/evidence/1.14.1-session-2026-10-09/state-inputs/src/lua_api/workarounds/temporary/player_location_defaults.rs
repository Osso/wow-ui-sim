//! Temporary `PlayerLocation` ObjectAPI fallback.
//!
//! Glue screens can use `PlayerLocation` before Blizzard_ObjectAPI has loaded
//! its game-screen constructor table. Keep that fallback explicit here until
//! the simulator has a proper ObjectAPI/glue loading boundary.

const PLAYER_LOCATION_DEFAULTS_LUA: &str = r#"
if PlayerLocation == nil then
  PlayerLocation = {}

  local function create_player_location(kind, payload)
    local location = {
      kind = kind,
      payload = payload,
    }

    function location:Clear()
      self.kind = nil
      self.payload = nil
    end

    function location:IsGUID()
      return self.kind == "guid"
    end

    function location:IsUnit()
      return self.kind == "unit"
    end

    function location:IsCommunityData()
      return self.kind == "community"
    end

    function location:IsBattleNetID()
      return self.kind == "battle_net"
    end

    function location:GetGUID()
      if self.kind == "guid" and self.payload then
        return self.payload.guid
      end
      return nil
    end

    function location:IsValid()
      if self.kind == "guid" and self.payload then
        if type(C_AccountInfo) == "table"
          and type(C_AccountInfo.IsGUIDBattleNetAccountType) == "function"
          and C_AccountInfo.IsGUIDBattleNetAccountType(self.payload.guid)
        then
          return true
        end
        return type(C_PlayerInfo) == "table"
          and type(C_PlayerInfo.GUIDIsPlayer) == "function"
          and C_PlayerInfo.GUIDIsPlayer(self.payload.guid)
          or false
      elseif self.kind == "unit" and self.payload then
        return type(UnitIsHumanPlayer) == "function" and UnitIsHumanPlayer(self.payload.unit) or false
      elseif self.kind == "community" and self.payload then
        return type(C_Club) == "table"
          and type(C_Club.CanResolvePlayerLocationFromClubMessageData) == "function"
          and C_Club.CanResolvePlayerLocationFromClubMessageData(
            self.payload.clubID,
            self.payload.streamID,
            self.payload.epoch,
            self.payload.position
          )
          or false
      elseif self.kind == "battle_net" and self.payload then
        return self.payload.battleNetID ~= nil
      elseif self.kind == "voice" and self.payload then
        return self.payload.memberID ~= nil and self.payload.channelID ~= nil
      end
      return false
    end

    return location
  end

  function PlayerLocation:CreateFromGUID(guid)
    return create_player_location("guid", { guid = guid })
  end

  function PlayerLocation:CreateFromUnit(unit)
    return create_player_location("unit", { unit = unit })
  end

  function PlayerLocation:CreateFromCommunityChatData(clubID, streamID, epoch, position)
    return create_player_location("community", {
      clubID = clubID,
      streamID = streamID,
      epoch = epoch,
      position = position,
    })
  end

  function PlayerLocation:CreateFromBattleNetID(battleNetID)
    return create_player_location("battle_net", { battleNetID = battleNetID })
  end

  function PlayerLocation:CreateFromVoiceID(memberID, channelID)
    return create_player_location("voice", {
      memberID = memberID,
      channelID = channelID,
    })
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PLAYER_LOCATION_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_player_location_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                C_PlayerInfo.GUIDIsPlayer = function(guid)
                    return guid == "Player-3676-00000001"
                end
                UnitIsHumanPlayer = function(unit)
                    return unit == "player"
                end
                C_Club.CanResolvePlayerLocationFromClubMessageData = function(clubID, streamID, epoch, position)
                    return clubID == 7 and streamID == 11 and epoch == 13 and position == 17
                end

                local guid = PlayerLocation:CreateFromGUID("Player-3676-00000001")
                if not guid:IsGUID() or guid:GetGUID() ~= "Player-3676-00000001" or not guid:IsValid() then
                    return "guid"
                end

                local unit = PlayerLocation:CreateFromUnit("player")
                if not unit:IsUnit() or not unit:IsValid() then return "unit" end

                local community = PlayerLocation:CreateFromCommunityChatData(7, 11, 13, 17)
                if not community:IsCommunityData() or not community:IsValid() then return "community" end

                local bnet = PlayerLocation:CreateFromBattleNetID(42)
                if not bnet:IsBattleNetID() or not bnet:IsValid() then return "bnet" end

                local voice = PlayerLocation:CreateFromVoiceID(3, 9)
                if not voice:IsValid() then return "voice" end
                voice:Clear()
                if voice:IsValid() then return "clear" end

                return "ok"
                "#,
            )
            .expect("PlayerLocation defaults probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_player_location_table() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            PlayerLocation = {
                CreateFromGUID = function()
                    return "existing"
                end,
            }
            "#,
        )
        .expect("fixture should install existing PlayerLocation table");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("PlayerLocation defaults should apply");
        }

        let result: String = env
            .eval(r#"return PlayerLocation:CreateFromGUID("Player-3676-00000001")"#)
            .expect("existing PlayerLocation table should survive");

        assert_eq!(result, "existing");
    }
}
