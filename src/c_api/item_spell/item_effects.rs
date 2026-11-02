//! Item-effect catalog selection for item spell queries.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use sim_data::item_enchants::ENCHANT_SPELLS;
use sim_data_profile::item_effects::{ITEM_EFFECTS, ITEM_SPELL_NAMES};

type ItemEffect = (u32, u32, i32, i32, i32);

fn use_spell_for_item(item: u32, effects: &[ItemEffect], profile: ClientProfile) -> Option<u32> {
    let start = effects.partition_point(|row| row.0 < item);
    let end = effects.partition_point(|row| row.0 <= item);
    let effects = &effects[start..end];
    if profile != ClientProfile::Retail {
        // Preserve the profile catalogs' selection order. Only Retail has
        // evidence for rejecting slots occupied by non-use effects.
        return effects
            .iter()
            .filter(|row| row.3 == 0 && row.2 > 0)
            .min_by_key(|row| (row.4, row.1))
            .map(|row| row.2 as u32);
    }
    let mut slots = [None; 5];
    for &(_, _, spell, trigger, slot) in effects {
        let slot = usize::try_from(slot)
            .ok()
            .filter(|slot| *slot < slots.len())?;
        if slots[slot].is_some() {
            return None;
        }
        slots[slot] = Some((spell, trigger));
    }
    slots
        .into_iter()
        .flatten()
        .find_map(|(spell, trigger)| (trigger == 0 && spell > 0).then_some(spell as u32))
}

pub(super) fn selected_use_spell(item: u32) -> Option<(u32, Option<&'static str>)> {
    selected_use_spell_from_catalog(item, ITEM_EFFECTS, ITEM_SPELL_NAMES, ACTIVE)
}

fn selected_use_spell_from_catalog<'a>(
    item: u32,
    effects: &[ItemEffect],
    names: &[(u32, &'a str)],
    profile: ClientProfile,
) -> Option<(u32, Option<&'a str>)> {
    let spell = use_spell_for_item(item, effects, profile)?;
    let name = names
        .binary_search_by_key(&spell, |entry| entry.0)
        .ok()
        .map(|index| names[index].1);
    Some((spell, name))
}

fn equipped_enchant_spell(
    state: &LuaState,
    item: u32,
) -> LuaResult<Option<(u32, Option<&'static str>)>> {
    let sim = borrow_state(state)?;
    let mut equipment: Vec<_> = sim.player.equipped_items.iter().collect();
    equipment.sort_unstable_by_key(|(slot, _)| *slot);
    let Some((_, owned)) = equipment
        .into_iter()
        .find(|(_, owned)| owned.item_id == item)
    else {
        return Ok(None);
    };
    let Ok(index) = ENCHANT_SPELLS.binary_search_by_key(&owned.enchant_id, |entry| entry.0) else {
        return Ok(None);
    };
    let (_, spell, name) = ENCHANT_SPELLS[index];
    Ok((spell != 0).then_some((spell, name)))
}

pub(super) fn selected_item_spell(
    state: &LuaState,
    item: u32,
) -> LuaResult<Option<(u32, Option<&'static str>)>> {
    match selected_use_spell(item) {
        Some(selection) => Ok(Some(selection)),
        None if ACTIVE == ClientProfile::Retail => equipped_enchant_spell(state, item),
        None => Ok(None),
    }
}

pub(crate) fn get_item_spell(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(value, Val::Num(_) | Val::Str(_)) {
        return Err(runtime_error("C_Item.GetItemSpell: expected itemInfo"));
    }
    let Some(item) = super::parse_item_id_key(state, value) else {
        return Ok(0);
    };
    let Some((spell, Some(name))) = selected_item_spell(state, item)? else {
        return Ok(0);
    };
    let name = create_string(state, name);
    state.push(name);
    state.push(Val::Num(spell as f64));
    Ok(2)
}

#[cfg(test)]
mod tests {
    use super::{selected_use_spell_from_catalog, use_spell_for_item};
    use crate::client_profile::ClientProfile;

    #[test]
    fn use_selection_keeps_passive_rows_and_follows_legacy_slot_order() {
        let effects = [
            (10, 1, 100, 1, 0),
            (10, 2, 200, 0, 3),
            (10, 3, 300, 0, 1),
            (20, 4, 400, 0, 0),
        ];
        assert_eq!(
            use_spell_for_item(10, &effects, ClientProfile::Retail),
            Some(300)
        );
        assert_eq!(
            use_spell_for_item(20, &effects, ClientProfile::Retail),
            Some(400)
        );
        assert_eq!(
            use_spell_for_item(30, &effects, ClientProfile::Retail),
            None
        );
    }

    #[test]
    fn invalid_or_duplicate_slots_reject_the_whole_association_set() {
        assert_eq!(
            use_spell_for_item(
                10,
                &[(10, 1, 100, 0, 0), (10, 2, 200, 1, 0)],
                ClientProfile::Retail
            ),
            None
        );
        for invalid_slot in [-1, 5] {
            assert_eq!(
                use_spell_for_item(
                    10,
                    &[(10, 1, 100, 0, 0), (10, 2, 200, 1, invalid_slot)],
                    ClientProfile::Retail
                ),
                None
            );
        }
    }

    #[test]
    fn nonpositive_spell_ids_remain_data_but_cannot_be_selected() {
        let effects = [(10, 1, -100, 0, 0), (10, 2, 0, 0, 1), (10, 3, 300, 0, 2)];
        assert_eq!(
            use_spell_for_item(10, &effects, ClientProfile::Retail),
            Some(300)
        );
        assert_eq!(
            use_spell_for_item(10, &effects[..2], ClientProfile::Retail),
            None
        );
    }

    #[test]
    fn missing_name_preserves_the_selected_spell() {
        let effects = [(10, 1, 100, 0, 0)];
        assert_eq!(
            selected_use_spell_from_catalog(
                10,
                &effects,
                &[(200, "Another spell")],
                ClientProfile::Retail
            ),
            Some((100, None))
        );
        assert_eq!(
            selected_use_spell_from_catalog(
                10,
                &effects,
                &[(100, "Use spell")],
                ClientProfile::Retail
            ),
            Some((100, Some("Use spell")))
        );
    }

    #[test]
    fn non_retail_use_selection_ignores_shared_non_use_slots() {
        let effects = [
            (239695, 208251, 55884, 0, 1),
            (239695, 227869, 1229308, 6, 1),
        ];
        for profile in [
            ClientProfile::Ptr,
            ClientProfile::Wrath,
            ClientProfile::Mists,
            ClientProfile::Era,
            ClientProfile::Anniversary,
            ClientProfile::WowForever,
        ] {
            assert_eq!(
                selected_use_spell_from_catalog(239695, &effects, &[(55884, "Learning")], profile),
                Some((55884, Some("Learning"))),
                "{profile:?}"
            );
        }
        assert_eq!(
            use_spell_for_item(239695, &effects, ClientProfile::Retail),
            None
        );
    }

    #[test]
    fn non_retail_use_selection_orders_by_slot_then_effect_id() {
        let effects = [
            (10, 1, 100, 0, 3),
            (10, 2, 200, 6, 0),
            (10, 3, 300, 0, 1),
            (10, 4, 400, 0, 1),
            (10, 5, -1, 0, 0),
            (10, 6, 0, 0, 0),
        ];
        assert_eq!(
            use_spell_for_item(10, &effects, ClientProfile::WowForever),
            Some(300)
        );
        assert_eq!(
            use_spell_for_item(10, &effects, ClientProfile::Retail),
            None
        );
    }
}
