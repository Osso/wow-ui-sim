//! Host-seeded R79 publisher. The unseeded collector remains unmodeled.
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::{LuaApiMut, LuaResult, Val, vm::state::LuaState};

/// Already-published host values. Their native conversions/shapes are UNKNOWN.
#[derive(Clone, Debug)]
pub enum PublishedValue {
    Nil,
    Number(f64),
    Text(Vec<u8>),
    Boolean(bool),
    Object(Vec<(String, PublishedValue)>),
    Array(Vec<PublishedValue>),
}
#[derive(Clone, Debug, Default)]
pub struct Icon {
    pub id: u32,
    pub resolved: Option<Vec<u8>>,
}
#[derive(Clone, Debug, Default)]
pub struct Category {
    pub fields: Vec<(String, PublishedValue)>,
    pub icon: Icon,
    pub selected_icon: Icon,
    pub camera_zoom_level: Option<i64>,
    pub camera_distance_offset: Option<f64>,
    pub options: Vec<OptionRecord>,
}
#[derive(Clone, Debug, Default)]
pub struct OptionRecord {
    pub fields: Vec<(String, PublishedValue)>,
    pub choices: Vec<Choice>,
}
#[derive(Clone, Debug, Default)]
pub struct Choice {
    pub fields: Vec<(String, PublishedValue)>,
}

pub(crate) fn install(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();
    table_set_rust_fn_static(state, state.global, "__wow_host_customizations", publish)?;
    lua.exec(
        r#"
        local publish = __wow_host_customizations
        __wow_host_customizations = nil
        local unseeded = C_CharacterCreation.GetAvailableCustomizations
        C_CharacterCreation.GetAvailableCustomizations = function()
            local result = publish()
            if result ~= nil then return result end
            return unseeded()
        end
    "#,
    )?;
    Ok(())
}

fn publish(state: &mut LuaState) -> LuaResult<u32> {
    if borrow_state(state)?.character_customizations.is_some() {
        // Explicit RED scaffolding, not a guessed native publisher result.
        return Err(rilua::runtime_error(
            "host-seeded R79 publisher not implemented",
        ));
    }
    state.push(Val::Nil);
    Ok(1)
}
