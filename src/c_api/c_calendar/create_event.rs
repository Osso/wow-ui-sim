//! Local creation of a new editable Calendar event (Retail 70688 WM40).
use super::{
    CalendarEditableAccess, CalendarInviteRecord, club::parse_record_club_id,
    editable_time::report_invalid,
};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::SimState;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Native -1 marks every unset date component.
const UNSET_DATE: i32 = -1;
/// Native -1 in the model's unsigned 32-bit time components.
const UNSET_TIME: u32 = u32::MAX;
const UNSET_TEXTURE: u32 = u32::MAX;

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "CreatePlayerEvent", create_player)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "CreateGuildSignUpEvent",
        create_guild_signup,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "CreateGuildAnnouncementEvent",
        create_guild_announcement,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "CreateCommunitySignUpEvent",
        create_community_signup,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Player,
    GuildSignUp,
    GuildAnnouncement,
    CommunitySignUp,
}

impl Kind {
    fn flags(self) -> u32 {
        match self {
            Kind::Player => 0x1,
            Kind::GuildSignUp => 0xc00,
            Kind::GuildAnnouncement => 0x40,
            Kind::CommunitySignUp => 0x400,
        }
    }

    fn self_invite_type(self) -> Option<i32> {
        match self {
            Kind::Player => Some(0),
            Kind::GuildSignUp | Kind::CommunitySignUp => Some(1),
            Kind::GuildAnnouncement => None,
        }
    }

    fn requires_guild(self) -> bool {
        matches!(self, Kind::GuildSignUp | Kind::GuildAnnouncement)
    }
}

fn create_player(state: &mut LuaState) -> LuaResult<u32> {
    create(state, Kind::Player)
}
fn create_guild_signup(state: &mut LuaState) -> LuaResult<u32> {
    create(state, Kind::GuildSignUp)
}
fn create_guild_announcement(state: &mut LuaState) -> LuaResult<u32> {
    create(state, Kind::GuildAnnouncement)
}
fn create_community_signup(state: &mut LuaState) -> LuaResult<u32> {
    create(state, Kind::CommunitySignUp)
}

fn create(state: &mut LuaState, kind: Kind) -> LuaResult<u32> {
    let guild_club_id = if kind.requires_guild() {
        let id = find_guild_club_id(&*borrow_state(state)?);
        let Some(id) = id else {
            return report_invalid(state, "ERR_GUILD_PLAYER_NOT_IN_GUILD");
        };
        Some(id)
    } else {
        None
    };
    {
        let mut host = borrow_state_mut(state)?;
        let record = new_record(&host, kind, guild_club_id);
        host.calendar_context_policy.editable = Some(record);
        host.calendar_opened_selection = None;
    }
    if kind == Kind::Player {
        dispatch_event_now(state, "CALENDAR_UPDATE_INVITE_LIST", &[Val::Bool(true)])?;
    }
    Ok(0)
}

/// The first kind-2 club record supplies the guild association. A record whose ID is
/// not a numeric club ID (the legacy roster's simulator key) supplies none.
fn find_guild_club_id(host: &SimState) -> Option<u64> {
    host.clubs
        .clubs
        .values()
        .find(|club| club.club_type == 2)
        .and_then(|club| parse_record_club_id(&club.id))
}

/// INFERRED: native self-invites require nonzero GUID type bits. Every valid GUID kind
/// has a nonzero type field and an empty GUID is zero, so a present player GUID maps
/// to that condition.
fn local_player_guid(host: &SimState) -> Option<String> {
    host.player
        .present
        .then(|| existing_guid_for_unit(host, "player"))
        .flatten()
}

fn new_record(host: &SimState, kind: Kind, guild_club_id: Option<u64>) -> CalendarEditableAccess {
    let player = local_player_guid(host);
    let invites = kind
        .self_invite_type()
        .zip(player.as_deref())
        .map(|(invite_type, guid)| self_invite(host, guid, invite_type))
        .into_iter()
        .collect();
    // Native initial clubId is UNKNOWN for every kind; 0 is the simulator placeholder.
    // Guild assignment marks the record dirty only when it changes that prior value.
    let club_id = guild_club_id.unwrap_or_default();
    CalendarEditableAccess {
        event_id: "0".into(),
        category_flags: kind.flags(),
        creator_identity: player.unwrap_or_default(),
        date: Some((UNSET_DATE, UNSET_DATE, UNSET_DATE)),
        time: Some((UNSET_TIME, UNSET_TIME)),
        texture_id: Some(UNSET_TEXTURE),
        invites,
        club_id,
        dirty: club_id != 0,
        ..Default::default()
    }
}

fn self_invite(host: &SimState, guid: &str, invite_type: i32) -> CalendarInviteRecord {
    let level = if host.player.present {
        host.player.level as u8
    } else {
        0
    };
    CalendarInviteRecord {
        invite_id: "0".into(),
        level: f64::from(level),
        invite_status: Some(3),
        mod_status: Some("CREATOR".into()),
        invite_is_mine: true,
        invite_type,
        guid: guid.into(),
        ..Default::default()
    }
}
