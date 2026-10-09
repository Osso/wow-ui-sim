//! Host-authored cast completion inputs; no spell classification is inferred.

use super::WowLuaEnv;
use crate::Result;
use rilua::Val;

/// Explicit host snapshot. Secrecy and instantness describe this cast, not all casts
/// of the spell. `caster_is_player` includes aliases of the local player.
#[derive(Clone, Debug)]
pub struct CastSuccess {
    pub unit: String,
    pub caster_is_player: bool,
    pub cast_guid: String,
    pub spell_id: u32,
    pub cast_bar_id: Option<u32>,
    pub instant: bool,
    pub spell_is_secret: bool,
}

impl WowLuaEnv {
    /// Publish one host completion through the real synchronous event dispatcher.
    /// This does not synthesize a cast, its effects, or secret payload wrappers.
    pub fn fire_cast_success(&self, input: &CastSuccess) -> Result<()> {
        let suppress = !input.caster_is_player && input.instant && input.spell_is_secret;
        if suppress {
            return Ok(());
        }
        let args = [
            self.lua_string(&input.unit),
            self.lua_string(&input.cast_guid),
            Val::Num(f64::from(input.spell_id)),
            input
                .cast_bar_id
                .map_or(Val::Nil, |id| Val::Num(f64::from(id))),
        ];
        self.fire_event_with_args("UNIT_SPELLCAST_SUCCEEDED", &args)
    }
}
