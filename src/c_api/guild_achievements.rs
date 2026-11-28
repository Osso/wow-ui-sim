//! `C_AchievementInfo.AreGuildAchievementsEnabled` (AchievementInfoDocumentation,
//! byte-identical Retail/Forever blocks). Original consumers: Forever
//! GuildInviteFrame.lua (invite point display), InstanceDifficulty.lua and
//! Blizzard_Communities GuildNews/CommunitiesFrame. Host input:
//! `A_Admin.SetGuildAchievementsEnabled(bool)`. The enabled default is INFERRED
//! from Retail's guild achievement UI; Forever's realm default is R9-8.

use super::helpers::ensure_namespace;
use crate::lua_api::globals::real::admin_fields as fields;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_AchievementInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "AreGuildAchievementsEnabled",
        are_guild_achievements_enabled,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetGuildAchievementsEnabled", |state| {
        let enabled = fields::boolean_arg(state, 1, "A_Admin.SetGuildAchievementsEnabled")?;
        borrow_state_mut(state)?.guild_achievements_enabled = enabled;
        Ok(0)
    })
}

fn are_guild_achievements_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.guild_achievements_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}
