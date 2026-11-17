//! Host-backed Retail battle-pet TooltipData for the original bag-item accessor.
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set_static};
use rilua::Val;
use rilua::vm::state::LuaState;

const TOOLTIP_TYPE_BATTLE_PET: f64 = 6.0;

/// Explicit host snapshot; the native upstream writer remains UNKNOWN.
#[derive(Clone, Debug, PartialEq)]
pub struct BattlePetTooltipSnapshot {
    pub species_id: f64,
    pub level: f64,
    pub breed_quality: f64,
    pub max_health: f64,
    pub power: f64,
    pub speed: f64,
    pub name: String,
}

/// Build type 6 only when both host inputs and the addressed bag item exist.
pub(crate) fn tooltip_for_bag_item(state: &mut LuaState, bag: i32, slot: i32) -> Option<Val> {
    let snapshot = {
        let sim = borrow_state(state).ok()?;
        let key = (bag, slot);
        sim.get_bag_item(bag, slot)?;
        if sim.bag_battle_pet_tooltip_acceptance.get(&key) != Some(&true) {
            return None;
        }
        sim.bag_battle_pet_tooltip_snapshots.get(&key)?.clone()
    };
    Some(publish_battle_pet_tooltip(state, &snapshot))
}

fn publish_battle_pet_tooltip(state: &mut LuaState, snapshot: &BattlePetTooltipSnapshot) -> Val {
    let saved_top = state.top;
    let tooltip = create_table(state);
    state.push(tooltip);
    let lines = create_table(state);
    state.push(lines);
    table_set_static(state, tooltip, "lines", lines);
    table_set_static(state, tooltip, "type", Val::Num(TOOLTIP_TYPE_BATTLE_PET));
    for (field, value) in [
        ("battlePetSpeciesID", snapshot.species_id),
        ("battlePetLevel", snapshot.level),
        ("battlePetBreedQuality", snapshot.breed_quality),
        ("battlePetMaxHealth", snapshot.max_health),
        ("battlePetPower", snapshot.power),
        ("battlePetSpeed", snapshot.speed),
    ] {
        table_set_static(state, tooltip, field, Val::Num(value));
    }
    let name = create_string(state, &snapshot.name);
    table_set_static(state, tooltip, "battlePetName", name);
    state.top = saved_top;
    tooltip
}
