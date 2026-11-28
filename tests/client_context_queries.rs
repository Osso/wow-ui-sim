#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! C_UI engine frames, C_ChatInfo chat-type helpers and C_PvP match-type predicates.
//! The PvP predicates run through Blizzard_UnitFrame GroupFrameVisibility.

use crate::common::blizzard_addon_harness as harness;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn ui_frames_resolve_engine_ui_parent_and_world_frame() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        local parent, world = C_UI.GetUIParent(), C_UI.GetWorldFrame()
        assert(parent == UIParent and parent:GetName() == "UIParent")
        assert(world == WorldFrame and world:GetName() == "WorldFrame")
        assert(parent ~= world)
    "#,
        )
        .unwrap();
}

#[test]
fn chat_type_names_invert_chat_type_indices() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        for _, name in ipairs({ "SYSTEM", "SAY", "GUILD_ACHIEVEMENT", "PING", "CHANNEL1", "CHANNEL20" }) do
            local id = GetChatTypeIndex(name)
            assert(id > 0 and C_ChatInfo.GetChatTypeName(id) == name, name)
        end
        assert(C_ChatInfo.GetChatTypeName(GetChatTypeIndex("SAY")) ~= C_ChatInfo.GetChatTypeName(GetChatTypeIndex("YELL")))
        assert(C_ChatInfo.GetChatTypeName(0) == nil and C_ChatInfo.GetChatTypeName(9999) == nil)
        assert(C_ChatInfo.GetChatTypeName(1.5) == nil)
        assert(not pcall(C_ChatInfo.GetChatTypeName, "SAY"))

        local kinds = Enum.ChatChannelType
        assert(C_ChatInfo.IsPartyChannelType(kinds.PrivateParty))
        assert(C_ChatInfo.IsPartyChannelType(kinds.PublicParty))
        assert(not C_ChatInfo.IsPartyChannelType(kinds.Custom))
        assert(not C_ChatInfo.IsPartyChannelType(kinds.Communities))
        assert(not C_ChatInfo.IsPartyChannelType(kinds.None))
    "#,
        )
        .unwrap();
}

const PVP_MATCH: &str = r#"
    local function flags(brawl, shuffle, factional)
        A_Admin.SetPvPMatch({ brawl = brawl, soloShuffle = shuffle, factional = factional })
    end
    flags(true, true, true)
    assert(not C_PvP.IsPVPMap() and not C_PvP.IsInBrawl(), "flags need a PvP map")
    assert(not C_PvP.IsSoloShuffle() and not C_PvP.IsMatchFactional())

    A_Admin.SetBattlefieldArena(true, false)
    assert(C_PvP.IsArena() and C_PvP.IsPVPMap() and not C_PvP.IsBattleground())
    assert(C_PvP.IsSoloShuffle() and C_PvP.IsInBrawl())
    assert(not ShouldShowArenaParty(), "a brawl arena hides the arena party frames")
    flags(false, false, false)
    assert(ShouldShowArenaParty() and not C_PvP.IsSoloShuffle())

    A_Admin.SetBattlefieldArena(false, false)
    A_Admin.SetInstanceInfo("Warsong Gulch", "pvp", 0, 10)
    A_Admin.SetInInstance(true)
    flags(false, true, true)
    assert(C_PvP.IsBattleground() and C_PvP.IsPVPMap() and not C_PvP.IsArena())
    assert(C_PvP.IsMatchFactional() and not C_PvP.IsSoloShuffle(), "solo shuffle is an arena mode")
    assert(not ShouldShowArenaParty())
    assert(not pcall(A_Admin.SetPvPMatch, { brawl = 1 }))
    return true
"#;

#[test]
fn pvp_match_type_drives_group_frame_visibility() {
    harness::with_blizzard_addon_startup_shape(&["Blizzard_UnitFrame"], &[], |env, _| {
        let result: bool = env.eval(PVP_MATCH).expect("original consumer runs");
        assert!(result);
    });
}
