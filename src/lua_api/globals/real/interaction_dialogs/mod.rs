//! Server-driven interaction dialogs: item text, guild invites, petitions,
//! duels, area spirit healers, and the instance boot/lock and death countdowns.
//!
//! Each dialog is opened by a validated `A_Admin` host input that stands in for
//! the server packet, and the documented client requests (`AcceptGuild`,
//! `CancelDuel`, `ItemTextNextPage`, ...) answer it by changing the modeled
//! dialog and firing the event the original Blizzard consumer listens to.

mod duel;
mod guild_invite;
mod item_text;
mod petition;
mod session_timers;
mod spirit_healer;

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::create_string;
use crate::lua_bridge::TableBuilder;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub use guild_invite::GuildInvite;
pub use item_text::ItemText;
pub use petition::Petition;
pub use session_timers::{InstanceLock, LockEncounter, SessionTimers};
pub use spirit_healer::SpiritHealer;

/// Open server dialogs. `None` means the server has no dialog open.
#[derive(Clone, Debug, Default)]
pub struct DialogState {
    pub item_text: Option<ItemText>,
    pub guild_invite: Option<GuildInvite>,
    pub petition: Option<Petition>,
    pub spirit_healer: Option<SpiritHealer>,
    pub timers: SessionTimers,
}

/// One event payload value; strings are interned and rooted at dispatch.
pub(crate) enum Arg {
    Str(String),
    Num(f64),
    Bool(bool),
}

/// Dispatch synchronously, keeping interned string payloads rooted on the stack.
pub(crate) fn fire(state: &mut LuaState, event: &str, args: Vec<Arg>) -> LuaResult<()> {
    let saved_top = state.top;
    let values: Vec<Val> = args
        .into_iter()
        .map(|arg| match arg {
            Arg::Str(text) => {
                let value = create_string(state, &text);
                state.push(value);
                value
            }
            Arg::Num(number) => Val::Num(number),
            Arg::Bool(flag) => Val::Bool(flag),
        })
        .collect();
    let result = dispatch_event_now(state, event, &values);
    state.top = saved_top;
    result
}

pub(crate) fn push_optional_string(state: &mut LuaState, text: Option<&str>) {
    let value = text.map_or(Val::Nil, |text| create_string(state, text));
    state.push(value);
}

/// Item text, petitions and `CancelDuel` are declared only by Retail secure
/// publication rows; Forever declares none of them (R-ag10-9).
const RETAIL_ONLY: bool = matches!(ACTIVE, ClientProfile::Retail);

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    if RETAIL_ONLY {
        item_text::register_all(lua)?;
        petition::register_all(lua)?;
        duel::register_all(lua)?;
    }
    guild_invite::register_all(lua)?;
    session_timers::register_all(lua)?;
    spirit_healer::register_all(lua)
}

pub(crate) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    let b = if RETAIL_ONLY {
        let b = item_text::register_admin(b)?;
        let b = petition::register_admin(b)?;
        duel::register_admin(b)?
    } else {
        b
    };
    let b = guild_invite::register_admin(b)?;
    let b = session_timers::register_admin(b)?;
    spirit_healer::register_admin(b)
}
