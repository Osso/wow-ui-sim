use super::*;

pub(super) fn default_guild_ranks() -> Vec<GuildRank> {
    vec![
        GuildRank {
            name: "Guild Leader".into(),
            flags: vec![true; 21],
        },
        GuildRank {
            name: "Officer".into(),
            flags: vec![true; 21],
        },
        GuildRank {
            name: "Member".into(),
            flags: vec![false; 21],
        },
    ]
}

pub(super) fn default_guild_members() -> Vec<GuildMember> {
    vec![
        GuildMember {
            name: "Uther".into(),
            guid: None,
            rank_index: 1,
            online: true,
        },
        GuildMember {
            name: "Jaina".into(),
            guid: None,
            rank_index: 2,
            online: false,
        },
    ]
}

pub(super) fn default_guild_events() -> Vec<GuildEvent> {
    vec![
        GuildEvent {
            event_type: "join".into(),
            player1: "Uther".into(),
            player2: None,
            rank_name: None,
            year: 24,
            month: 9,
            day: 1,
            hour: 18,
        },
        GuildEvent {
            event_type: "invite".into(),
            player1: "Uther".into(),
            player2: Some("Jaina".into()),
            rank_name: None,
            year: 24,
            month: 10,
            day: 4,
            hour: 21,
        },
        GuildEvent {
            event_type: "join".into(),
            player1: "Jaina".into(),
            player2: None,
            rank_name: None,
            year: 24,
            month: 10,
            day: 4,
            hour: 22,
        },
        GuildEvent {
            event_type: "promote".into(),
            player1: "Uther".into(),
            player2: Some("Jaina".into()),
            rank_name: Some("Officer".into()),
            year: 24,
            month: 11,
            day: 12,
            hour: 19,
        },
        GuildEvent {
            event_type: "quit".into(),
            player1: "Thrall".into(),
            player2: None,
            rank_name: None,
            year: 25,
            month: 1,
            day: 6,
            hour: 14,
        },
        GuildEvent {
            event_type: "remove".into(),
            player1: "Uther".into(),
            player2: Some("Sylvanas".into()),
            rank_name: None,
            year: 25,
            month: 2,
            day: 18,
            hour: 23,
        },
    ]
}

pub(super) fn default_guild_motd() -> String {
    "Raid invites tonight at 20:00 server. Repairs are on for progression.".into()
}

pub(super) fn default_guild_info_text() -> String {
    "Mythic-focused guild recruiting healers and a warlock for weekend raids.".into()
}

pub(super) fn default_guild_challenges() -> Vec<GuildChallenge> {
    vec![
        GuildChallenge {
            challenge_type: 1,
            current: 5,
            max: 7,
            gold: 1250000,
            max_gold: 1750000,
        },
        GuildChallenge {
            challenge_type: 2,
            current: 1,
            max: 1,
            gold: 5000000,
            max_gold: 5000000,
        },
        GuildChallenge {
            challenge_type: 3,
            current: 1,
            max: 3,
            gold: 1000000,
            max_gold: 3000000,
        },
        GuildChallenge {
            challenge_type: 4,
            current: 2,
            max: 7,
            gold: 500000,
            max_gold: 1750000,
        },
    ]
}

pub(super) fn default_world_pvp_areas() -> Vec<WorldPvpBattlegroundInfo> {
    vec![
        WorldPvpBattlegroundInfo {
            bg_id: 571,
            can_enter: true,
            can_queue: true,
            is_active: true,
            max_level: 80,
            min_level: 80,
            name: "Wintergrasp".into(),
            start_time: 900,
        },
        WorldPvpBattlegroundInfo {
            bg_id: 607,
            can_enter: false,
            can_queue: false,
            is_active: false,
            max_level: 85,
            min_level: 80,
            name: "Tol Barad".into(),
            start_time: 0,
        },
    ]
}

pub(super) fn default_holiday_bg_info() -> RandomBGInfo {
    default_bg_info(108, 2, "Warsong Scramble")
}

pub(super) fn default_random_bg_info() -> RandomBGInfo {
    default_bg_info(0, 0, "Random Battleground")
}

pub(super) fn default_random_epic_bg_info() -> RandomBGInfo {
    default_bg_info(0, 0, "Random Epic Battleground")
}

fn default_bg_info(bg_id: i32, bg_index: i32, name: &str) -> RandomBGInfo {
    RandomBGInfo {
        bg_id,
        bg_index,
        can_queue: true,
        has_random_win_today: false,
        max_level: 80,
        min_level: 10,
        name: name.into(),
    }
}

pub(super) fn apply_collection_defaults(ws: &mut WorldState) {
    let heirlooms = default_heirlooms();
    ws.collected_heirlooms = heirlooms.iter().map(|h| h.item_id).collect();
    ws.heirlooms = heirlooms;
    ws.transmog_appearances = default_transmog_appearances();
    ws.transmog_collected_shown = true;
    ws.transmog_uncollected_shown = true;
    ws.transmog_all_factions_shown = false;
    ws.transmog_all_races_shown = false;
    ws.transmog_class_filter = 2;
    ws.transmog_source_type_filters = (1..=7).collect();
    ws.transmog_search_text.clear();
    ws.mounts = default_mounts();
    ws.pets = default_pets();
    ws.toys = default_toys();
    ws.warband_scenes = default_warband_scenes();
    ws.premade_listings = default_premade_listings();
}
