//! Forever local channel membership and unchanged installed LockInTools consumer.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_permanent_channel_returns_kind_and_committed_membership() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(_G, 'JoinPermanentChannel')) == 'function')
        assert(GetChannelName('SimulatorFriends') == 0)
        local kind, name = JoinPermanentChannel('SimulatorFriends', 'local-password', 2, 1)
        assert(kind == Enum.PermanentChatChannelType.Custom and kind == 3)
        assert(name == 'SimulatorFriends')
        local id, canonical = GetChannelName(name)
        assert(id > 0 and canonical == name, 'a constant/no-op join must fail')
        local info = C_ChatInfo.GetChannelInfoFromIdentifier(id)
        assert(info.name == name and info.localID == id and info.channelType == kind)
        assert(select('#', JoinPermanentChannel(name)) == 2)
        local list = {GetChannelList()}
        assert(#list == 3 and list[1] == id and list[2] == name)
        local displayed = GetChatWindowChannels(2)
        assert(#displayed == 1 and displayed[1] == name)
        JoinPermanentChannel(name, nil, 2)
        assert(#GetChatWindowChannels(2) == 1)
        local zoneKind, zoneName = JoinPermanentChannel('General')
        assert(zoneKind == Enum.PermanentChatChannelType.Zone and zoneKind == 1)
        assert(zoneName == 'General' and GetChannelName(zoneName) ~= id)
        assert(JoinPermanentChannel('') == nil)
        assert(JoinPermanentChannel() == nil)
        assert(#{GetChannelList()} == 6, 'invalid names must not add membership')
        ChannelLeave(name)
        assert(GetChannelName(name) == 0)
        JoinPermanentChannel(name)
        assert(GetChannelName(name) > 0)
        "#,
    )
    .unwrap();
    let other = WowLuaEnv::new().unwrap();
    other
        .exec("assert(GetChannelName('SimulatorFriends') == 0)")
        .expect("joined channels must remain local to their environment");
}

#[test]
fn forever_permanent_channel_notifies_once_after_membership_and_routing_commit() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ChannelJoinNotices, ChannelJoinUpdates = 0, 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('CHAT_MSG_CHANNEL_NOTICE')
        listener:RegisterEvent('CHANNEL_UI_UPDATE')
        listener:SetScript('OnEvent', function(_, event, ...)
            local id, name = GetChannelName('SimulatorFriends')
            assert(id > 0 and name == 'SimulatorFriends')
            assert(GetChatWindowChannels(3)[1] == name)
            if event == 'CHAT_MSG_CHANNEL_NOTICE' then
                ChannelJoinNotices = ChannelJoinNotices + 1
                assert(select('#', ...) == 18)
                assert((...) == 'YOU_JOINED')
                assert(select(4, ...) == id .. '. ' .. name)
                assert(select(7, ...) == 0)
                assert(select(8, ...) == id and select(9, ...) == name)
                local discord = select(18, ...)
                assert(type(discord) == 'table' and discord.fromDiscord == false)
            else
                ChannelJoinUpdates = ChannelJoinUpdates + 1
                assert(select('#', ...) == 0)
            end
        end)
        JoinPermanentChannel('SimulatorFriends', nil, 3)
        assert(ChannelJoinNotices == 1 and ChannelJoinUpdates == 1)
        JoinPermanentChannel('SimulatorFriends', nil, 3)
        JoinPermanentChannel('')
        assert(ChannelJoinNotices == 1 and ChannelJoinUpdates == 1)
        "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
    env.apply_post_load_workarounds();
    env.exec(
        r#"
        assert(type(rawget(_G, 'JoinPermanentChannel')) == 'function')
        ChannelLeave('SimulatorFriends')
        JoinPermanentChannel('SimulatorFriends', nil, 3)
        assert(ChannelJoinNotices == 2 and ChannelJoinUpdates == 2)
        "#,
    )
    .expect("cleanup must preserve joining and transition notifications");
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn forever_permanent_channel_selected_lockintools_consumer_confirms_real_membership() {
    use std::path::{Path, PathBuf};
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::toc::TocFile;

    let Some(addon_dir) = std::env::var_os("LOCKINTOOLS_UI_PATH").map(PathBuf::from) else {
        eprintln!(
            "skipping installed LockInTools consumer: set LOCKINTOOLS_UI_PATH to its addon directory"
        );
        return;
    };
    let toc_path = wow_ui_sim::loader::find_toc_file(&addon_dir)
        .expect("installed LockInTools must have a selected TOC");
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert_eq!(
        toc.metadata.get("Interface").map(String::as_str),
        Some("16001")
    );
    assert!(!toc.is_game_type_restricted());
    assert!(toc.allows_screen(ScreenKind::Game));
    let index = toc
        .files
        .iter()
        .position(|path| path == Path::new("Chat.lua"))
        .expect("selected TOC must load original Chat.lua");
    assert!(toc.file_allows_screen(index, ScreenKind::Game));
    let source = std::fs::read_to_string(addon_dir.join(&toc.files[index])).unwrap();
    assert!(source.contains("JoinPermanentChannel(name,"));

    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        LockInChannelConsumer = {
            ready = true,
            chatState = {wanted = true, attempts = 0},
            GetGroupChannelName = function() return 'SimulatorLockInTools' end,
            GetGroupChannelPassword = function() return 'local-password' end,
            IsGroupSwitching = function() return false end,
            Notify = function() end,
            Print = function(_, message) error(message) end,
        }
        -- Outer addon and chat-frame scaffolding only; no query/join API changes.
        DEFAULT_CHAT_FRAME = {
            channelList = {},
            GetID = function() return 1 end,
            AddChannel = function(self, name)
                self.channelList[#self.channelList + 1] = name
            end,
        }
        "#,
    )
    .unwrap();
    let mut separator = String::from("=");
    while source.contains(&format!("]{separator}]")) {
        separator.push('=');
    }
    env.exec(&format!(
        "assert(loadstring([{separator}[{source}]{separator}], '@LockInTools/Chat.lua'))('LockInTools', LockInChannelConsumer)"
    ))
    .expect("load entire unchanged selected Chat.lua");
    env.exec(
        r#"
        local consumer = LockInChannelConsumer
        local name = consumer:GetGroupChannelName()
        assert(consumer:ConfirmChatMembership() == false)
        assert(consumer:JoinChatChannel(false) == true)
        assert(GetChannelName(name) > 0, 'reject constant success/no-op producer')
        assert(consumer.chatState.phase == 'joined' and consumer.chatState.pending == false)
        assert(consumer.chatState.attempts == 0)
        assert(DEFAULT_CHAT_FRAME.channelList[1] == name)
        assert(GetChatWindowChannels(1)[1] == name)
        ChannelLeave(name)
        assert(consumer:ConfirmChatMembership() == false, 'reject constant membership')
        consumer.chatState.wanted = true
        assert(consumer:JoinChatChannel(false) == true)
        assert(consumer.chatState.phase == 'joined')
        "#,
    )
    .expect("original LockInTools join and confirmation consume actual channel state");
    assert!(env.state().borrow().lua_errors.is_empty());
}
