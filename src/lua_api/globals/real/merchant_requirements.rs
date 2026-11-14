//! R88i.5 verified requirement gates; inferred faction/rating adapters.
use crate::lua_api::methods::table_get_static;
use crate::lua_api::state::{MerchantItemRequirement, SimState};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(super) fn item_gate(
    category: i32,
    record: Option<&MerchantItemRequirement>,
    test: impl FnOnce(u32, i32) -> bool,
) -> Option<bool> {
    if category != 1 {
        return Some(true);
    }
    let record = record?;
    Some(record.requirement_id == 0 || test(record.requirement_id, record.required_value))
}

pub(super) fn cost_gate(value: u32, mode: i32, test: impl FnOnce(u32, i32) -> bool) -> bool {
    value == 0 || !(0..9).contains(&mode) || test(value, mode)
}

// INFERRED: ItemSparse MinFactionID/MinReputation identify a faction and its
// minimum reaction rank. Reuse the existing reputation owner rows; do not seed
// duplicate faction state. Comparison direction is not established by R88i.5.
pub(super) fn reputation_test(state: &mut LuaState, id: u32, required: i32) -> bool {
    let root = table_get_static(state, Val::Table(state.global), "__wow_reputation_state");
    let Val::Table(rows) = table_get_static(state, root, "factions") else {
        return false;
    };
    let mut index = 1;
    loop {
        let row = state
            .gc
            .tables
            .get(rows)
            .map_or(Val::Nil, |t| t.get_int(index));
        if row == Val::Nil {
            return false;
        }
        if table_get_static(state, row, "factionID") == Val::Num(f64::from(id))
            && table_get_static(state, row, "isHeader") != Val::Bool(true)
        {
            return matches!(table_get_static(state, row, "reaction"), Val::Num(rank)
                if rank >= f64::from(required));
        }
        index += 1;
    }
}

// INFERRED: ItemExtendedCost RequiredArenaRating/ArenaBracket and >= mapping.
// Missing scenario rating is not a fabricated successful requirement check.
pub(super) fn rating_test(sim: &SimState, required: u32, mode: i32) -> bool {
    sim.merchant_player_ratings
        .get(&mode)
        .is_some_and(|&rating| i64::from(rating) >= i64::from(required))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r88i5_item_gate_distinguishes_missing_record_skip_and_failed_test() {
        assert_eq!(item_gate(1, None, |_, _| panic!("no record")), None);
        assert_eq!(
            item_gate(2, None, |_, _| panic!("wrong category")),
            Some(true)
        );
        let mut record = MerchantItemRequirement {
            requirement_id: 0,
            required_value: -4,
        };
        assert_eq!(
            item_gate(1, Some(&record), |_, _| panic!("zero ID")),
            Some(true)
        );
        record.requirement_id = 42;
        assert_eq!(
            item_gate(1, Some(&record), |id, value| {
                assert_eq!((id, value), (42, -4));
                false
            }),
            Some(false)
        );
        assert_eq!(item_gate(1, Some(&record), |_, _| true), Some(true));
    }

    #[test]
    fn r88i5_cost_gate_tests_only_nonzero_value_with_signed_mode_zero_through_eight() {
        assert!(cost_gate(0, 0, |_, _| panic!("zero value")));
        for mode in [i32::MIN, -1, 9, i32::MAX] {
            assert!(cost_gate(5, mode, |_, _| panic!("ineligible mode")));
        }
        for mode in 0..9 {
            assert!(!cost_gate(5, mode, |value, actual_mode| {
                assert_eq!((value, actual_mode), (5, mode));
                false
            }));
            assert!(cost_gate(5, mode, |_, _| true));
        }
    }
}
