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

  __wow_temporary_player_location_defaults = PlayerLocation
end
"#;

const REMOVE_PLAYER_LOCATION_DEFAULTS_LUA: &str = r#"
if rawget(_G, "__wow_temporary_player_location_defaults") == rawget(_G, "PlayerLocation") then
  rawset(_G, "PlayerLocation", nil)
end
rawset(_G, "__wow_temporary_player_location_defaults", nil)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PLAYER_LOCATION_DEFAULTS_LUA)?;
    Ok(())
}

pub(crate) fn update_for_screen_mode(
    lua: &mut rilua::Lua,
    screen: crate::screen::ScreenKind,
) -> crate::Result<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return Ok(());
    }

    if screen.is_glue() {
        return apply_bootstrap(lua);
    }

    lua.exec(REMOVE_PLAYER_LOCATION_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(all(feature = "optional-tests", feature = "client-retail"))]
    #[test]
    fn retail_glue_keeps_player_location_for_friend_invite_report() {
        use crate::screen::ScreenKind;
        use std::path::Path;

        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.set_screen_mode(ScreenKind::Login);

        let ui = crate::client_profile::blizzard_ui_addons_dir_under(Path::new(env!(
            "CARGO_MANIFEST_DIR"
        )));
        let addons = crate::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Login);
        assert!(
            addons
                .iter()
                .any(|(name, _)| name == "Blizzard_FriendsFrame")
        );
        assert!(!addons.iter().any(|(name, _)| name == "Blizzard_ObjectAPI"));

        let source =
            std::fs::read_to_string(ui.join("Blizzard_FriendsFrame/Mainline/FriendsFrame.lua"))
                .expect("selected FriendsFrame source should read");
        env.loader_env()
            .exec(&source)
            .expect("original FriendsFrame consumer should load");

        let result: (bool, bool, bool, String, String) = env
            .eval(
                r#"
                local callbacks = {}
                local received = {}
                C_BattleNet.GetFriendInviteInfo = function(index)
                    return { inviteID = 1000 + index, accountName = "Friend" .. index }
                end
                ReportInfo = { CreateReportInfoFromType = function(_, reportType) return reportType end }
                ReportFrame = {
                    InitiateReport = function(_, _, name, playerLocation, isBNetReport)
                        received[#received + 1] = {
                            name = name,
                            playerLocation = playerLocation,
                            isBNetReport = isBNetReport,
                        }
                    end,
                }
                local rootDescription = {
                    SetTag = function() end,
                    CreateButton = function(_, _, callback)
                        callbacks[#callbacks + 1] = callback
                        return {}
                    end,
                }
                local button = {
                    inviteIndex = 1,
                    DeclineButton = {
                        SetupMenu = function(_, setup)
                            setup(nil, rootDescription)
                        end,
                    },
                }

                FriendsFrameInviteTemplateMixin.OnLoad(button)
                callbacks[2]()
                button.inviteIndex = 2
                callbacks[2]()
                return received[1].playerLocation ~= nil,
                    received[2].playerLocation ~= nil,
                    received[1].playerLocation ~= received[2].playerLocation,
                    received[1].name,
                    received[2].name
                "#,
            )
            .expect("original deferred friend-invite report callback should run");

        assert_eq!(
            result,
            (
                true,
                true,
                true,
                "Friend1".to_string(),
                "Friend2".to_string()
            )
        );
    }

    #[cfg(all(feature = "optional-tests", feature = "client-retail"))]
    #[test]
    fn retail_game_uses_original_player_location_getter_in_channel_roster() {
        use crate::screen::ScreenKind;
        use std::path::Path;

        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.set_screen_mode(ScreenKind::Game);

        let fallback_absent: bool = env
            .eval("return PlayerLocation == nil")
            .expect("game screen PlayerLocation probe should run");
        assert!(
            fallback_absent,
            "Retail Game must leave PlayerLocation to the selected ObjectAPI source"
        );

        let ui = crate::client_profile::blizzard_ui_addons_dir_under(Path::new(env!(
            "CARGO_MANIFEST_DIR"
        )));
        let addons = crate::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
        assert!(addons.iter().any(|(name, _)| name == "Blizzard_ObjectAPI"));
        assert!(addons.iter().any(|(name, _)| name == "Blizzard_Channels"));

        for relative_path in [
            "Blizzard_ObjectAPI/Mainline/PlayerLocation.lua",
            "Blizzard_Channels/RosterButton.lua",
        ] {
            let source = std::fs::read_to_string(ui.join(relative_path)).unwrap_or_else(|error| {
                panic!("failed to read selected source {relative_path}: {error}")
            });
            env.loader_env().exec(&source).unwrap_or_else(|error| {
                panic!("failed to load original source {relative_path}: {error}")
            });
        }

        let result: (bool, bool, String, String) = env
            .eval(
                r#"
                local seen = {}
                C_AccountInfo.IsGUIDRelatedToLocalAccount = function(guid)
                    seen[#seen + 1] = guid
                    return guid == "Player-3676-00000001"
                end

                local function check(guid)
                    local button = {
                        playerLocation = PlayerLocation:CreateFromGUID(guid),
                        GetMemberPlayerLocation = ChannelRosterButtonMixin.GetMemberPlayerLocation,
                        IsLocalPlayer = ChannelRosterButtonMixin.IsLocalPlayer,
                    }
                    return button:IsLocalPlayer() and true or false
                end

                local first = check("Player-3676-00000001")
                local second = check("Player-3676-00000002")
                return first, second, seen[1], seen[2]
                "#,
            )
            .expect("original channel roster consumer should read PlayerLocation GUIDs");

        assert_eq!(
            result,
            (
                true,
                false,
                "Player-3676-00000001".to_string(),
                "Player-3676-00000002".to_string()
            )
        );
    }

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
