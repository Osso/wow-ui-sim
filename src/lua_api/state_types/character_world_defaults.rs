use super::{
    GuildChallenge, GuildEvent, GuildMember, GuildRank, RandomBGInfo, WorldPvpBattlegroundInfo,
    WorldState,
};

use crate::lua_api::state_defaults::*;

struct GuildEventSeed {
    event_type: &'static str,
    player1: &'static str,
    player2: Option<&'static str>,
    rank_name: Option<&'static str>,
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
}

/// Build the sim's seeded default `WorldState` — Stormwind zone,
/// "Heroes of Azeroth" guild (except Wrath), populated collections. `WorldState::default`
/// still returns a fully zero/empty state (the derived `Default`); this
/// function is what `SimState::Default` reaches for.
pub fn seeded_world_state() -> WorldState {
    let mut ws = WorldState {
        zone_name: "Stormwind City".into(),
        zone_id: 1519,
        sub_zone_name: "Trade District".into(),
        instance_type: "none".into(),
        instance_difficulty_name: String::new(),
        guild_name: Some("Heroes of Azeroth".into()),
        guild_rank: Some("Member".into()),
        guild_num_members: 2,
        guild_ranks: default_guild_ranks(),
        guild_members: default_guild_members(),
        guild_motd: default_guild_motd(),
        guild_info_text: default_guild_info_text(),
        guild_challenges: default_guild_challenges(),
        guild_events: default_guild_events(),
        pvp_type: "contested".into(),
        guild_can_speak_in_chat: true,
        world_pvp_areas: default_world_pvp_areas(),
        holiday_bg_info: Some(default_holiday_bg_info()),
        random_bg_info: Some(default_random_bg_info()),
        random_epic_bg_info: Some(default_random_epic_bg_info()),
        ..WorldState::default()
    };
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        // Native initialized cache, no membership, and no received rank rights.
        ws.guild_info_text.clear();
        ws.guild_name = None;
        ws.guild_rank = None;
        ws.guild_num_members = 0;
        ws.guild_members.clear();
        ws.guild_ranks.clear();
        // Initial world-entry image; received host rows replace these seeds.
        ws.guild_bank_rank_permissions.insert(
            0,
            super::GuildBankRankPermissions {
                money_per_day: 0,
                tab_rights: [-1; 6],
                tab_stacks_per_day: [-1; 6],
            },
        );
    }
    apply_collection_defaults(&mut ws);
    ws
}

fn default_guild_ranks() -> Vec<GuildRank> {
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

fn default_guild_members() -> Vec<GuildMember> {
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

fn default_guild_events() -> Vec<GuildEvent> {
    default_guild_event_seeds()
        .into_iter()
        .map(build_guild_event)
        .collect()
}

fn default_guild_event_seeds() -> [GuildEventSeed; 6] {
    [
        guild_event_seed("join", "Uther", None, None, (24, 9, 1, 18)),
        guild_event_seed("invite", "Uther", Some("Jaina"), None, (24, 10, 4, 21)),
        guild_event_seed("join", "Jaina", None, None, (24, 10, 4, 22)),
        guild_event_seed(
            "promote",
            "Uther",
            Some("Jaina"),
            Some("Officer"),
            (24, 11, 12, 19),
        ),
        guild_event_seed("quit", "Thrall", None, None, (25, 1, 6, 14)),
        guild_event_seed("remove", "Uther", Some("Sylvanas"), None, (25, 2, 18, 23)),
    ]
}

fn guild_event_seed(
    event_type: &'static str,
    player1: &'static str,
    player2: Option<&'static str>,
    rank_name: Option<&'static str>,
    date: (i32, i32, i32, i32),
) -> GuildEventSeed {
    let (year, month, day, hour) = date;
    GuildEventSeed {
        event_type,
        player1,
        player2,
        rank_name,
        year,
        month,
        day,
        hour,
    }
}

fn build_guild_event(seed: GuildEventSeed) -> GuildEvent {
    GuildEvent {
        event_type: seed.event_type.into(),
        player1: seed.player1.into(),
        player2: seed.player2.map(String::from),
        rank_name: seed.rank_name.map(String::from),
        year: seed.year,
        month: seed.month,
        day: seed.day,
        hour: seed.hour,
    }
}

fn default_guild_motd() -> String {
    "Raid invites tonight at 20:00 server. Repairs are on for progression.".into()
}

fn default_guild_info_text() -> String {
    "Mythic-focused guild recruiting healers and a warlock for weekend raids.".into()
}

fn default_guild_challenges() -> Vec<GuildChallenge> {
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

fn default_world_pvp_areas() -> Vec<WorldPvpBattlegroundInfo> {
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

fn default_holiday_bg_info() -> RandomBGInfo {
    default_bg_info(108, 2, "Warsong Scramble")
}

fn default_random_bg_info() -> RandomBGInfo {
    default_bg_info(0, 0, "Random Battleground")
}

fn default_random_epic_bg_info() -> RandomBGInfo {
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

fn apply_collection_defaults(ws: &mut WorldState) {
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
