use super::*;

/// Seed the `SimState.bnet_friends` list with two representative
/// entries: one online Alliance Paladin with two game accounts, and
/// one offline friend. Provides coverage for all five C_BattleNet
/// probes out of the box.
pub(in crate::lua_api::state) fn default_bnet_friends() -> Vec<BnetFriend> {
    vec![uther_online_friend(), thrall_offline_friend()]
}

fn uther_online_friend() -> BnetFriend {
    BnetFriend {
        friend_index: 1,
        bnet_account_guid: "BNet-0-100001".into(),
        bnet_account_id: 100001,
        battle_tag: "Uther#1000".into(),
        account_name: "Uther".into(),
        note: String::new(),
        custom_title_friend_name: None,
        friend_tags: Vec::new(),
        custom_message: String::new(),
        custom_message_time: 0,
        appear_offline: false,
        is_battle_tag_friend: true,
        friend_level: 1,
        is_friend: true,
        is_favorite: false,
        is_afk: false,
        is_dnd: false,
        last_online_time: 0,
        raf_link_type: 0,
        game_accounts: vec![uther_stormwind_account(), lightbringer_alt_account()],
    }
}

fn uther_stormwind_account() -> BnetGameAccount {
    BnetGameAccount {
        wow_account_guid: SEEDED_LOCAL_CHARACTER_GUID.into(),
        game_account_id: 200001,
        character_name: SEEDED_LOCAL_CHARACTER_NAME.into(),
        realm_name: "Stormwind".into(),
        realm_display_name: "Stormwind".into(),
        realm_id: 1,
        class_id: 2,
        class_name: "Paladin".into(),
        character_level: 70,
        area_name: "Stormwind City".into(),
        is_online: true,
        is_game_afk: false,
        is_game_busy: false,
        client_program: "WoW".into(),
        faction_name: "Alliance".into(),
        race_name: "Human".into(),
        rich_presence: "In Stormwind City".into(),
        can_summon: true,
        is_in_current_region: true,
        has_focus: true,
        wow_project_id: 1,
        timerunning_season_id: 0,
        region_id: 1,
        player_guid: String::new(),
    }
}

fn lightbringer_alt_account() -> BnetGameAccount {
    BnetGameAccount {
        wow_account_guid: "Player-1-00000002".into(),
        game_account_id: 200002,
        character_name: "Lightbringer".into(),
        realm_name: "Stormwind".into(),
        realm_display_name: "Stormwind".into(),
        realm_id: 1,
        class_id: 2,
        class_name: "Paladin".into(),
        character_level: 60,
        area_name: "Ironforge".into(),
        is_online: false,
        is_game_afk: false,
        is_game_busy: false,
        client_program: "WoW".into(),
        faction_name: "Alliance".into(),
        race_name: "Dwarf".into(),
        rich_presence: String::new(),
        can_summon: false,
        is_in_current_region: true,
        has_focus: false,
        wow_project_id: 1,
        timerunning_season_id: 0,
        region_id: 1,
        player_guid: String::new(),
    }
}

fn thrall_offline_friend() -> BnetFriend {
    BnetFriend {
        friend_index: 2,
        bnet_account_guid: "BNet-0-100002".into(),
        bnet_account_id: 100002,
        battle_tag: "Thrall#2000".into(),
        account_name: "Thrall".into(),
        note: "old friend".into(),
        custom_title_friend_name: None,
        friend_tags: Vec::new(),
        custom_message: String::new(),
        custom_message_time: 0,
        appear_offline: false,
        is_battle_tag_friend: true,
        friend_level: 1,
        is_friend: true,
        is_favorite: true,
        is_afk: false,
        is_dnd: false,
        last_online_time: 1700000000,
        raf_link_type: 0,
        game_accounts: vec![],
    }
}

/// Seed the `SimState.social_friends` list with three representative
/// WoW friends: two online and one offline. Provides coverage for all
/// C_Social probes out of the box.
pub(in crate::lua_api::state) fn default_social_friends() -> Vec<SocialFriend> {
    vec![
        SocialFriend {
            name: "Arthax".into(),
            level: 70,
            area: "Stormwind City".into(),
            class_name: "Paladin".into(),
            note: String::new(),
            is_online: true,
            guid: "Player-1-0000A001".into(),
        },
        SocialFriend {
            name: "Durotan".into(),
            level: 65,
            area: "Orgrimmar".into(),
            class_name: "Shaman".into(),
            note: "old guildie".into(),
            is_online: false,
            guid: "Player-2-0000A002".into(),
        },
        SocialFriend {
            name: "Sylvara".into(),
            level: 60,
            area: "Ironforge".into(),
            class_name: "Mage".into(),
            note: String::new(),
            is_online: true,
            guid: "Player-1-0000A003".into(),
        },
    ]
}

/// Default items in bag 0 (backpack) at startup. Slots are 1-based (WoW convention).
pub(in crate::lua_api::state) fn default_backpack_items() -> HashMap<(i32, i32), BagItem> {
    [
        (1, default_bag_item(6948, 1)), // Hearthstone
        (2, default_bag_item(159, 5)),  // Refreshing Spring Water
        (3, default_bag_item(4540, 5)), // Tough Hunk of Bread
        (4, default_bag_item(7005, 1)), // Skinning Knife
    ]
    .into_iter()
    .map(|(slot, item)| ((0, slot), item))
    .collect()
}

fn default_bag_item(item_id: u32, stack_count: i32) -> BagItem {
    BagItem {
        item_id,
        stack_count,
        hyperlink: None,
    }
}
