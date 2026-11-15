//! R88k.2 conditional normal-return contract. Record writers remain UNKNOWN.
use crate::lua_api::methods::{borrow_state, create_table, table_set_static};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

const NAME: &str = "GetContainerItemPurchaseInfo";
pub(super) const USAGE: &str = "Usage: local info = C_Container.GetContainerItemPurchaseInfo(containerIndex, slotIndex, isEquipped)";

#[derive(Debug, Clone, Default)]
pub struct PurchaseInputs {
    /// Native zero-based inventory slots and container identifiers; identity is
    /// host-supplied, not item template ID. No guessed legacy bag conversion.
    pub inventory: HashMap<i32, u64>,
    pub containers: HashMap<(i32, i32), u64>,
    pub inventory_count: i32,
    pub carried_count: i32,
    pub container_counts: HashMap<i32, i32>,
    pub records: HashMap<u64, PurchaseRecord>,
    pub excluded_identity: Option<u64>,
    pub refund_clock: i64,
}

#[derive(Debug, Clone, Default)]
pub struct PurchaseRecord {
    pub money: u64,
    pub expiration: i64,
    pub item_quantities: [i64; 5],
    pub currency_quantities: [i64; 5],
    /// Each pair is the native modification value and its supplied flags check.
    /// The unknown item-flags predicate/writer is not reconstructed.
    pub modifications: [(i64, bool); 4],
}

impl PurchaseInputs {
    fn item(&self, container: i32, slot: i32, equipped: bool) -> Option<u64> {
        if equipped {
            return (0..30)
                .contains(&slot)
                .then(|| self.inventory.get(&slot).copied())
                .flatten();
        }
        if (0..16).contains(&container) {
            let count = self.container_counts.get(&container).copied().unwrap_or(0);
            return (0..count)
                .contains(&slot)
                .then(|| self.containers.get(&(container, slot)).copied())
                .flatten();
        }
        if container >= 16 || container == -2 {
            return None;
        }
        let (offset, limit) = match container {
            -1 => (35, self.carried_count),
            -3 => (63, 6),
            -4 => (100, 5),
            _ => (0, self.inventory_count),
        };
        if !(0..limit).contains(&slot) {
            return None;
        }
        let index = offset + slot;
        (index < self.inventory_count)
            .then(|| self.inventory.get(&index).copied())
            .flatten()
    }
}

fn get_purchase_info(state: &mut LuaState) -> LuaResult<u32> {
    let (container, slot, equipped) = super::container_purchase_args::read(state)?;
    let sim = borrow_state(state)?;
    if !sim.player.present {
        return Ok(0);
    }
    let input = &sim.purchase_inputs;
    let Some(identity) = input.item(container, slot, equipped) else {
        return Ok(0);
    };
    if input.excluded_identity == Some(identity) {
        return Ok(0);
    }
    let Some(record) = input.records.get(&identity) else {
        return Ok(0);
    };
    let seconds = i128::from(record.expiration) - i128::from(input.refund_clock) + 7200;
    if seconds <= 0 {
        return Ok(0);
    }
    let record = record.clone();
    drop(sim);
    publish_record(state, &record, seconds);
    Ok(1)
}

fn publish_record(state: &mut LuaState, record: &PurchaseRecord, seconds: i128) {
    let table = create_table(state);
    table_set_static(state, table, "money", Val::Num(record.money as f64));
    table_set_static(
        state,
        table,
        "itemCount",
        Val::Num(record.item_quantities.iter().filter(|&&q| q > 0).count() as f64),
    );
    table_set_static(state, table, "refundSeconds", Val::Num(seconds as f64));
    table_set_static(
        state,
        table,
        "currencyCount",
        Val::Num(
            record
                .currency_quantities
                .iter()
                .filter(|&&q| q > 0)
                .count() as f64,
        ),
    );
    table_set_static(
        state,
        table,
        "hasEnchants",
        Val::Bool(
            record
                .modifications
                .iter()
                .any(|&(value, allowed)| value != 0 && allowed),
        ),
    );
    state.push(table);
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Container")?;
    table_set_rust_fn_static(state, namespace, NAME, get_purchase_info)
}
