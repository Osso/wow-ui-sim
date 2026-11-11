//! R60 shared GameMode record selection. Rows are host input, never fabricated.
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GameModeRow {
    pub glue_screen_name: String,
    pub logo: i32,
    pub logo_height: i32,
    pub logo_vertical_offset: i32,
    pub logo_shrunken_height: i32,
    pub logo_uses_dark_backdrop: bool,
    pub character_create_extra_height: i32,
    pub character_create_outer_border: i32,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GameModeDisplayState {
    pub rows: HashMap<i32, GameModeRow>,
    temporary_override: Option<i32>,
}
impl GameModeDisplayState {
    /// Bounded host counterpart of the native auto-connect selection path.
    pub fn set_temporary_override(&mut self, record: i32) {
        self.temporary_override = Some(record);
    }
    /// Native ClearRealmList lifecycle effect; not a fabricated Lua API.
    pub fn clear_realm_list(&mut self) {
        self.temporary_override = None;
    }
    fn selected_record(&self, cvars: &crate::cvars::CVarStorage) -> LuaResult<Option<i32>> {
        if let Some(record) = self.temporary_override {
            return Ok(Some(record));
        }
        let record = if cvars.get("currentGameMode").is_none() {
            -1
        } else {
            cvars.cached_integer("currentGameMode").ok_or_else(|| rilua::runtime_error("currentGameMode: cached integer outside simulator coverage (native behavior unknown)"))?
        };
        Ok((record != -1).then_some(record))
    }
    fn selected_row(&self, cvars: &crate::cvars::CVarStorage) -> LuaResult<Option<&GameModeRow>> {
        Ok(self
            .selected_record(cvars)?
            .and_then(|record| self.rows.get(&record)))
    }
}
pub(crate) fn modern_profile() -> bool {
    use crate::client_profile::{ACTIVE, ClientProfile};
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
}
pub(crate) fn cvar_set_callback(state: &mut LuaState, name: &str) -> LuaResult<()> {
    if modern_profile() && name.eq_ignore_ascii_case("currentGameMode") {
        borrow_state_mut(state)?
            .game_rules
            .display
            .clear_realm_list();
        // Synchronous dispatch follows the simulator's existing CVar callback path;
        // exact native timing relative to assignment/other events is UNKNOWN.
        dispatch_event_now(state, "GAME_MODE_DISPLAY_INFO_UPDATED", &[])?;
    }
    Ok(())
}
pub(crate) fn glue_screen(state: &mut LuaState) -> LuaResult<u32> {
    let name = {
        let sim = borrow_state(state)?;
        sim.game_rules
            .display
            .selected_row(&sim.cvars)?
            .map(|row| row.glue_screen_name.clone())
    };
    let value = match name.filter(|name| !name.is_empty()) {
        Some(name) => create_string(state, &name),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}
pub(crate) fn current_display(state: &mut LuaState) -> LuaResult<u32> {
    let row = {
        let sim = borrow_state(state)?;
        if sim.game_rules.display.selected_record(&sim.cvars)? == Some(0) {
            None
        } else {
            sim.game_rules.display.selected_row(&sim.cvars)?.cloned()
        }
    };
    let Some(row) = row else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = create_table(state);
    for (name, value) in [
        ("logo", row.logo),
        ("logoHeight", row.logo_height),
        ("logoVerticalOffset", row.logo_vertical_offset),
        ("logoShrunkenHeight", row.logo_shrunken_height),
        (
            "characterCreateExtraHeight",
            row.character_create_extra_height,
        ),
        (
            "characterCreateOuterBorder",
            row.character_create_outer_border,
        ),
    ] {
        if value != 0 || !matches!(name, "logo" | "characterCreateOuterBorder") {
            table_set(state, table, name, Val::Num(f64::from(value)));
        }
    }
    table_set(
        state,
        table,
        "logoUsesDarkBackdrop",
        Val::Bool(row.logo_uses_dark_backdrop),
    );
    state.push(table);
    Ok(1)
}
