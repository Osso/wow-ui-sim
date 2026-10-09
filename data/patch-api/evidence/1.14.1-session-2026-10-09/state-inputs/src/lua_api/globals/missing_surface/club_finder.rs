//! Deterministic `C_ClubFinder` surface for Communities UI defaults.

use super::ensure_namespace;
use super::set_table_array;
use crate::lua_api::methods::{create_string, create_table, table_set};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, RustFn, Val};

const GUILD_FINDER_GUID: &str = "guild-finder-heroes";
const PENDING_GUILD_FINDER_GUID: &str = "guild-finder-pending";
const COMMUNITY_FINDER_GUID: &str = "community-finder-raiders";
const PENDING_COMMUNITY_FINDER_GUID: &str = "community-finder-pending";
const REQUEST_TYPE_GUILD: f64 = 1.0;
const REQUEST_TYPE_COMMUNITY: f64 = 2.0;
const PLAYER_CLUB_REQUEST_STATUS_PENDING: f64 = 1.0;
const CLUB_FINDER_FLAG_DUNGEONS: f64 = 1.0;

pub(super) fn register_club_finder_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_ClubFinder")?;
    register_club_finder_default_handlers(state, table_ref)?;
    register_club_finder_query_handlers(state, table_ref)?;
    Ok(())
}

fn register_club_finder_default_handlers(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_same_handler(state, table_ref, EMPTY_TABLE_METHODS, empty_table)?;
    register_same_handler(state, table_ref, NOOP_METHODS, noop)?;
    register_same_handler(state, table_ref, TRUE_METHODS, return_true)?;
    register_same_handler(state, table_ref, FALSE_METHODS, return_false)?;
    register_same_handler(state, table_ref, ZERO_METHODS, return_zero)?;
    Ok(())
}

fn register_club_finder_query_handlers(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetClubRecruitmentSettings",
        club_recruitment_settings,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetPlayerApplicantSettings",
        player_applicant_settings,
    )?;
    register_club_finder_status_handlers(state, table_ref)?;
    register_club_finder_list_handlers(state, table_ref)?;
    Ok(())
}

fn register_club_finder_status_handlers(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, table_ref, "GetClubTypeFromFinderGUID", get_club_type)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetFocusIndexFromFlag",
        get_focus_index_from_flag,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetPlayerClubApplicationStatus",
        get_player_club_application_status,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetTotalMatchingCommunityListSize",
        get_total_matching_community_list_size,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetTotalMatchingGuildListSize",
        get_total_matching_guild_list_size,
    )?;
    Ok(())
}

fn register_club_finder_list_handlers(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "PlayerReturnPendingCommunitiesList",
        pending_communities,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "PlayerReturnPendingGuildsList",
        pending_guilds,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "ReturnMatchingCommunityList",
        matching_communities,
    )?;
    table_set_rust_fn_static(state, table_ref, "ReturnMatchingGuildList", matching_guilds)?;
    Ok(())
}

fn register_same_handler(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
    names: &[&'static str],
    handler: RustFn,
) -> LuaResult<()> {
    for name in names {
        table_set_rust_fn_static(state, table_ref, name, handler)?;
    }
    Ok(())
}

const EMPTY_TABLE_METHODS: &[&str] = &[
    "PlayerGetClubInvitationList",
    "ReturnClubApplicantList",
    "ReturnPendingClubApplicantList",
    "GetStatusOfPostingFromClubId",
];

const TRUE_METHODS: &[&str] = &[
    "IsEnabled",
    "IsCommunityFinderEnabled",
    "IsValidSearchString",
];

const FALSE_METHODS: &[&str] = &[
    "DoesPlayerBelongToClubFromClubGUID",
    "HasAlreadyAppliedToLinkedPosting",
    "HasPostingBeenDelisted",
    "IsListingEnabledFromFlags",
    "IsPostingBanned",
];

const ZERO_METHODS: &[&str] = &["GetClubFinderDisableReason"];

const NOOP_METHODS: &[&str] = &[
    "ApplicantAcceptClubInvite",
    "ApplicantDeclineClubInvite",
    "CancelMembershipRequest",
    "LookupClubPostingFromClubFinderGUID",
    "PlayerRequestPendingClubsList",
    "PostClub",
    "RequestApplicantList",
    "RequestClubsList",
    "RequestMembershipToClub",
    "RequestNextCommunityPage",
    "RequestNextGuildPage",
    "RequestPostingInformationFromClubId",
    "RequestSubscribedClubPostingIDs",
    "ResetClubPostingMapCache",
    "RespondToApplicant",
    "SendChatWhisper",
    "SetAllRecruitmentSettings",
    "SetPlayerApplicantLocaleFlags",
    "SetPlayerApplicantSettings",
    "SetRecruitmentLocale",
    "SetRecruitmentSettings",
];

const PLAY_STYLE_SETTINGS: &[&str] = &[
    "playStyleDungeon",
    "playStyleRaids",
    "playStylePvp",
    "playStyleRP",
    "playStyleSocial",
];

const PLAYER_APPLICANT_FALSE_SETTINGS: &[&str] = &[
    "roleTank",
    "roleHealer",
    "roleDps",
    "sizeSmall",
    "sizeMedium",
    "sizeLarge",
    "sortMembers",
    "sortNewest",
    "crossFaction",
];

fn club_recruitment_settings(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table(state);
    set_false_fields(state, table, PLAY_STYLE_SETTINGS);
    table_set(state, table, "maxLevelOnly", Val::Bool(false));
    table_set(state, table, "enableListing", Val::Bool(false));
    state.push(table);
    Ok(1)
}

fn player_applicant_settings(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table(state);
    set_false_fields(state, table, PLAY_STYLE_SETTINGS);
    set_false_fields(state, table, PLAYER_APPLICANT_FALSE_SETTINGS);
    table_set(state, table, "sortRelevance", Val::Bool(true));
    state.push(table);
    Ok(1)
}

fn set_false_fields(state: &mut LuaState, table: Val, fields: &[&str]) {
    for field in fields {
        table_set(state, table, field, Val::Bool(false));
    }
}

fn empty_table(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table(state);
    state.push(table);
    Ok(1)
}

fn matching_guilds(state: &mut LuaState) -> LuaResult<u32> {
    push_card_list(
        state,
        &[ClubFinderCard {
            guid: GUILD_FINDER_GUID,
            name: "Heroes of Azeroth",
            comment: "Mists guild recruitment posting",
            leader: "Uther",
            members: 12.0,
            is_guild: true,
        }],
    )
}

fn pending_guilds(state: &mut LuaState) -> LuaResult<u32> {
    push_card_list(
        state,
        &[ClubFinderCard {
            guid: PENDING_GUILD_FINDER_GUID,
            name: "Pandaria Vanguard",
            comment: "Pending guild application",
            leader: "Jaina",
            members: 18.0,
            is_guild: true,
        }],
    )
}

fn matching_communities(state: &mut LuaState) -> LuaResult<u32> {
    push_card_list(
        state,
        &[ClubFinderCard {
            guid: COMMUNITY_FINDER_GUID,
            name: "Timeless Isle Raiders",
            comment: "Community finder posting",
            leader: "Chen",
            members: 24.0,
            is_guild: false,
        }],
    )
}

fn pending_communities(state: &mut LuaState) -> LuaResult<u32> {
    push_card_list(
        state,
        &[ClubFinderCard {
            guid: PENDING_COMMUNITY_FINDER_GUID,
            name: "Celestial Tournament",
            comment: "Pending community application",
            leader: "Lorewalker Cho",
            members: 31.0,
            is_guild: false,
        }],
    )
}

struct ClubFinderCard {
    guid: &'static str,
    name: &'static str,
    comment: &'static str,
    leader: &'static str,
    members: f64,
    is_guild: bool,
}

fn push_card_list(state: &mut LuaState, cards: &[ClubFinderCard]) -> LuaResult<u32> {
    let list = create_table(state);
    for (index, card) in cards.iter().enumerate() {
        let card_info = create_card_info(state, card);
        set_table_array(state, list, index as i64 + 1, card_info);
    }
    state.push(list);
    Ok(1)
}

fn create_card_info(state: &mut LuaState, card: &ClubFinderCard) -> Val {
    let table = create_table(state);
    let guid = create_string(state, card.guid);
    let name = create_string(state, card.name);
    let comment = create_string(state, card.comment);
    let leader = create_string(state, card.leader);
    let recruiting_spec_ids = create_table(state);
    let tabard_info = create_table(state);

    set_table_field(state, table, "clubFinderGUID", guid);
    set_table_field(state, table, "name", name);
    set_table_field(state, table, "comment", comment);
    set_table_field(state, table, "guildLeader", leader);
    set_table_field(state, table, "numActiveMembers", Val::Num(card.members));
    set_table_field(
        state,
        table,
        "recruitmentFlags",
        Val::Num(CLUB_FINDER_FLAG_DUNGEONS),
    );
    set_table_field(state, table, "recruitingSpecIds", recruiting_spec_ids);
    set_table_field(state, table, "isReported", Val::Bool(false));
    set_table_field(state, table, "isCrossFaction", Val::Bool(false));
    set_table_field(state, table, "emblemInfo", Val::Num(0.0));
    set_table_field(state, table, "tabardInfo", tabard_info);
    set_table_field(state, table, "clubType", club_type_value(card.is_guild));
    table
}

fn set_table_field(state: &mut LuaState, table_val: Val, key: &'static str, value: Val) {
    let Val::Table(table_ref) = table_val else {
        return;
    };
    let key_ref = state.gc.intern_string_static(key.as_bytes());
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}

fn club_type_value(is_guild: bool) -> Val {
    let request_type = if is_guild {
        REQUEST_TYPE_GUILD
    } else {
        REQUEST_TYPE_COMMUNITY
    };
    Val::Num(request_type)
}

fn get_total_matching_guild_list_size(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

fn get_total_matching_community_list_size(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

fn get_player_club_application_status(state: &mut LuaState) -> LuaResult<u32> {
    let guid = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    if is_pending_guid(&guid) {
        state.push(Val::Num(PLAYER_CLUB_REQUEST_STATUS_PENDING));
    } else {
        state.push(Val::Nil);
    }
    Ok(1)
}

fn is_pending_guid(guid: &str) -> bool {
    matches!(
        guid,
        PENDING_GUILD_FINDER_GUID | PENDING_COMMUNITY_FINDER_GUID
    )
}

fn get_club_type(state: &mut LuaState) -> LuaResult<u32> {
    let guid = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let is_guild = guid == GUILD_FINDER_GUID || guid == PENDING_GUILD_FINDER_GUID;
    state.push(club_type_value(is_guild));
    Ok(1)
}

fn get_focus_index_from_flag(state: &mut LuaState) -> LuaResult<u32> {
    let focus_flag = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0);
    state.push(Val::Num(focus_flag));
    Ok(1)
}

fn noop(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn return_true(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn return_false(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn return_zero(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}
