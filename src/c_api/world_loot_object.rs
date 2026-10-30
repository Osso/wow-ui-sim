//! Host-supplied world loot state; unobserved behavior is simulator policy.
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table_with_fields};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct WorldLootObject {
    pub unit_token: String,
    pub inventory_type: i32,
    pub at_max_quality: bool,
    pub is_upgrade: bool,
    pub distance_squared: f64,
    pub in_range: bool,
}

#[derive(Default)]
pub struct WorldLootObjects {
    pub objects: HashMap<String, WorldLootObject>,
    pub clicks: Vec<(String, bool)>,
    /// Explicit observations of inventory-slot compatibility.
    pub slot_matches: HashMap<(i32, i32), bool>,
}

impl WowLuaEnv {
    /// Replace a GUID's record before notifying listeners synchronously.
    pub fn supply_world_loot_object(
        &self,
        guid: &str,
        object: WorldLootObject,
    ) -> crate::Result<()> {
        {
            let mut sim = self.state.borrow_mut();
            sim.world_loot_objects
                .objects
                .retain(|key, value| key == guid || value.unit_token != object.unit_token);
            sim.world_loot_objects
                .objects
                .insert(guid.to_owned(), object);
        }
        self.fire_event_with_args("WORLD_LOOT_OBJECT_INFO_UPDATED", &[self.lua_string(guid)])
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_WorldLootObject")?;
    for (name, handler) in [
        (
            "DoesSlotMatchInventoryType",
            slot_matches as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("GetWorldLootObjectDistanceSquared", distance),
        ("GetWorldLootObjectInfo", info),
        ("GetWorldLootObjectInfoByGUID", info_guid),
        ("IsWorldLootObject", is_object),
        ("IsWorldLootObjectByGUID", is_object_guid),
        ("IsWorldLootObjectInRange", in_range),
        ("OnWorldLootObjectClick", click),
    ] {
        table_set_rust_fn_static(state, ns, name, handler)?;
    }
    Ok(())
}

fn by_unit<'a>(store: &'a WorldLootObjects, unit: &str) -> Option<&'a WorldLootObject> {
    store.objects.values().find(|obj| obj.unit_token == unit)
}
fn slot_matches(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let inventory = i32::from_stack(state, 2)?;
    let matches = borrow_state(state)?
        .world_loot_objects
        .slot_matches
        .get(&(slot, inventory))
        .copied()
        .unwrap_or(false);
    state.push(Val::Bool(matches));
    Ok(1)
}
fn distance(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let value =
        by_unit(&borrow_state(state)?.world_loot_objects, &unit).map(|o| o.distance_squared);
    state.push(value.map_or(Val::Nil, Val::Num));
    Ok(1)
}
fn push_info(state: &mut LuaState, obj: Option<WorldLootObject>) -> LuaResult<u32> {
    if let Some(obj) = obj {
        let table = create_table_with_fields(
            state,
            &[
                ("inventoryType", Val::Num(obj.inventory_type as f64)),
                ("atMaxQuality", Val::Bool(obj.at_max_quality)),
                ("isUpgrade", Val::Bool(obj.is_upgrade)),
            ],
        );
        state.push(table);
        Ok(1)
    } else {
        Ok(0)
    }
}
fn info(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let obj = by_unit(&borrow_state(state)?.world_loot_objects, &unit).cloned();
    push_info(state, obj)
}
fn info_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let obj = borrow_state(state)?
        .world_loot_objects
        .objects
        .get(&guid)
        .cloned();
    push_info(state, obj)
}
fn is_object(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let found = by_unit(&borrow_state(state)?.world_loot_objects, &unit).is_some();
    state.push(Val::Bool(found));
    Ok(1)
}
fn is_object_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let found = borrow_state(state)?
        .world_loot_objects
        .objects
        .contains_key(&guid);
    state.push(Val::Bool(found));
    Ok(1)
}
fn in_range(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let found =
        by_unit(&borrow_state(state)?.world_loot_objects, &unit).is_some_and(|o| o.in_range);
    state.push(Val::Bool(found));
    Ok(1)
}
fn click(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let left = bool::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .world_loot_objects
        .clicks
        .push((unit, left));
    Ok(0)
}
