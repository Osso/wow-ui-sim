//! Explicit world rectangles for C_Map.GetMapPosFromWorldPos.
//! No geography is inferred from art pixels. Overlapping snapshots require an
//! override map ID; native automatic hierarchy selection remains unmodeled.
use crate::c_api::ensure_namespace;
use crate::lua_api::methods::{borrow_state, table_get};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Clone, Copy)]
pub struct MapWorldRect {
    pub continent_id: i32,
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

impl MapWorldRect {
    fn project(&self, continent: i32, x: f64, y: f64) -> Option<(f64, f64)> {
        let coordinates = [self.left, self.right, self.top, self.bottom, x, y];
        let finite = coordinates.into_iter().all(f64::is_finite);
        if self.continent_id != continent || !finite {
            return None;
        }
        let width = self.right - self.left;
        let height = self.bottom - self.top;
        if width == 0.0 || height == 0.0 {
            return None;
        }
        let normalized_x = (x - self.left) / width;
        let normalized_y = (y - self.top) / height;
        let inside = (0.0..=1.0).contains(&normalized_x) && (0.0..=1.0).contains(&normalized_y);
        inside.then_some((normalized_x, normalized_y))
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Map")?;
    table_set_rust_fn_static(state, namespace, "GetMapPosFromWorldPos", get_map_pos)
}

fn read_coordinate(state: &mut LuaState, vector: Val, key: &str) -> LuaResult<f64> {
    let Val::Num(value) = table_get(state, vector, key) else {
        return Err(runtime_error(format!(
            "worldPosition.{key} must be a number"
        )));
    };
    Ok(value)
}

fn get_map_pos(state: &mut LuaState) -> LuaResult<u32> {
    let continent = i32::from_stack(state, 1)?;
    let vector = stack_val(state, 2);
    if !matches!(vector, Val::Table(_)) {
        return Err(runtime_error("worldPosition must be a vector2 table"));
    }
    let x = read_coordinate(state, vector, "x")?;
    let y = read_coordinate(state, vector, "y")?;
    let override_id = Option::<i32>::from_stack(state, 3)?;
    let position = find_map_position(state, continent, x, y, override_id)?;
    let Some((id, (x, y))) = position else {
        return Ok(0);
    };
    let vector = super::c_map::create_world_position_vector(state, x, y)?;
    state.push(Val::Num(f64::from(id)));
    state.push(vector);
    Ok(2)
}

fn find_map_position(
    state: &LuaState,
    continent: i32,
    x: f64,
    y: f64,
    override_id: Option<i32>,
) -> LuaResult<Option<(i32, (f64, f64))>> {
    let sim = borrow_state(state)?;
    let mut matches = sim.map_world_rects.iter().filter_map(|(&id, rect)| {
        if override_id.is_some_and(|selected| selected != id) || !sim.maps.contains_key(&id) {
            return None;
        }
        rect.project(continent, x, y).map(|point| (id, point))
    });
    let first = matches.next();
    if matches.next().is_some() {
        return Ok(None);
    }
    Ok(first)
}
