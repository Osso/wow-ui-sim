#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_vendor(env: &WowLuaEnv, path: &str) {
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.exec(&std::fs::read_to_string(root.join(path)).unwrap())
        .unwrap();
}

#[test]
fn forever_social_enums_match_documented_values_and_metadata() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        checked = 0
        local wanted = {ClubStreamType=true, BattleNetFriendTag=true, RecentAlliesInteractionCategoryFilter=true, SocialUIPresenceType=true, SocialSystemType=true, SocialUIBlockType=true, RolodexType=true}
        APIDocumentation = {AddDocumentationTable=function(_, doc)
            for _, definition in ipairs(doc.Tables or {}) do
                if wanted[definition.Name] then
                    local values = assert(Enum[definition.Name], definition.Name)
                    for _, field in ipairs(definition.Fields) do
                        assert(values[field.Name] == field.EnumValue, definition.Name .. '.' .. field.Name)
                    end
                    local meta = assert(Enum[definition.Name .. 'Meta'])
                    assert(meta.MinValue == definition.MinValue)
                    assert(meta.MaxValue == definition.MaxValue)
                    assert(meta.NumValues == definition.NumValues)
                    checked = checked + 1
                end
            end
        end}
    "#).unwrap();
    for file in [
        "ClubDocumentation.lua",
        "BattleNetSharedDocumentation.lua",
        "RecentAlliesConstantsDocumentation.lua",
        "SocialUIDocumentation.lua",
        "RolodexConstantsDocumentation.lua",
    ] {
        load_vendor(&env, &format!("Blizzard_APIDocumentationGenerated/{file}"));
    }
    assert_eq!(env.eval::<i32>("return checked").unwrap(), 7);
}

#[test]
fn forever_social_presence_consumer_maps_icons_and_account_states() {
    let env = WowLuaEnv::new().unwrap();
    load_vendor(&env, "Blizzard_SocialUIShared/SocialUIUtil.lua");
    env.exec(r#"
        assert(SocialUIUtil.GetIconForPresenceType(Enum.SocialUIPresenceType.Busy) == "friends-status-busy")
        assert(SocialUIUtil.GetIconForPresenceType(999) == "friends-status-offline")
        assert(SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo(nil) == Enum.SocialUIPresenceType.Offline)
        local account = {gameAccountInfo={isOnline=true}}
        assert(SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo(account) == Enum.SocialUIPresenceType.Online)
        account.isAFK = true
        assert(SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo(account) == Enum.SocialUIPresenceType.Away)
    "#).unwrap();
}

#[test]
fn forever_communities_sorts_discord_between_officer_and_other() {
    let env = WowLuaEnv::new().unwrap();
    load_vendor(&env, "Blizzard_FrameXMLUtil/CommunitiesUtil.lua");
    env.exec(
        r#"
        local streams = {
            {streamType=4, creationTime=2}, {streamType=3, creationTime=2},
            {streamType=2, creationTime=2}, {streamType=0, creationTime=2},
            {streamType=1, creationTime=2}, {streamType=3, creationTime=1},
        }
        CommunitiesUtil.SortStreams(streams)
        local expected = {1,0,2,3,3,4}
        for i, kind in ipairs(expected) do assert(streams[i].streamType == kind) end
        assert(streams[4].creationTime == 1 and streams[5].creationTime == 2)
    "#,
    )
    .unwrap();
}
