//! Retail 12.1.5 and Forever 1.60.1 DiscordDocumentation.lua (:15-96) shapes:
//! per-server channel counts with `valid`, two-index channel names, the
//! three-value guild link status and the local user ID/name. The selected
//! consumer, Blizzard_GuildControlUI `GuildControlUI_Discord_Update`, reads the
//! same returns (Blizzard_GuildControlUI.lua:314-391 Retail, :317-394 Forever).
#![cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const SERVERS: &str = r#"
    A_Admin.SetDiscordServers({
        { name = "Guild Hall", channels = { { name = "general" }, { name = "raids", linkable = false } } },
        { name = "Empty", channels = {} },
    })
    A_Admin.SetDiscordUser("881", "Moira")
"#;

const SHAPES: &str = r#"
    local count, valid = C_Discord.GetNumDiscordChannels(1)
    assert(count == 2 and valid == true)
    count, valid = C_Discord.GetNumDiscordChannels(2)
    assert(count == 0 and valid == true)
    count, valid = C_Discord.GetNumDiscordChannels(3)
    assert(count == 0 and valid == false)
    assert(C_Discord.GetDiscordChannelName(1, 2) == "raids")
    assert(#C_Discord.GetServerLinkableChannels(1) == 1)
    assert(C_Discord.GetDiscordUserID() == "881")
    assert(C_Discord.GetDiscordUserName("881") == "Moira")
    local linked, channel, server = C_Discord.GetGuildLinkStatus()
    assert(linked == false and channel == "" and server == "")
    for _, case in ipairs({ { 1, 1, "general" }, { 1, 2, "raids" } }) do
        C_Discord.GuildLink(case[1], case[2])
        linked, channel, server = C_Discord.GetGuildLinkStatus()
        assert(linked and channel == case[3] and server == "Guild Hall", channel)
    end
    C_Discord.GuildUnlink()
    assert(C_Discord.GetGuildLinkStatus() == false)
"#;

#[test]
fn discord_guild_link_status_follows_the_supplied_servers() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(SERVERS).expect("supply Discord servers");
    env.exec(SHAPES).expect("Discord shapes follow the supplied servers");
}
