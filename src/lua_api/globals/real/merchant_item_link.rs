//! R88l.2 Retail merchant links. Offer population and eligibility predicate
//! writers remain host-supplied; no server events or data are fabricated.
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct MerchantLinks {
    /// Explicit failed availability/predicate inputs, neutral when absent.
    pub unavailable: bool,
    pub rejected_slots: HashSet<usize>,
    /// The existing list is a host-supplied selected offer ordering. Native raw
    /// storage filtering/renumbering writers remain UNKNOWN, not simulated.
    /// Overrides of the item path for slots in the existing merchant list.
    pub offers: HashMap<usize, MerchantLinkOffer>,
}

#[derive(Debug, Clone)]
pub enum MerchantLinkOffer {
    /// None means the selected builder produced no text (one nil).
    /// Some fields contain the host's raw positional payload, not guessed names.
    Item {
        fields: Option<Vec<i64>>,
    },
    Currency {
        record: Option<MerchantNamedLink>,
    },
    Spell {
        record: Option<MerchantNamedLink>,
    },
    /// The second spell route requires a resolved offer record first.
    ResolvedSpell {
        record: Option<MerchantNamedLink>,
    },
    Unsupported,
    Missing,
}

#[derive(Debug, Clone)]
pub struct MerchantNamedLink {
    pub id: i32,
    pub second: i32,
    /// Currency uses a named color token, spell uses an ARGB string.
    pub color: String,
    /// None represents a selected builder with no text; not a missing record.
    pub label: Option<String>,
}

fn get_merchant_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Num(n) => Some(n),
        Val::Str(r) => state
            .gc
            .string_arena
            .get(r)
            .and_then(|s| crate::c_api::parse_numeric_string(s.data())),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error("Usage: GetMerchantItemLink(index)"))?;
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number) {
        return Err(rilua::runtime_error(format!(
            "integer overflow attempting to store {number:.6}"
        )));
    }
    let Some(index) = usize::try_from((number.trunc() as i32).wrapping_sub(1)).ok() else {
        return Ok(0);
    };
    let sim = borrow_state(state)?;
    if sim.merchant_links.unavailable {
        return Ok(0);
    }
    let slot = index;
    if sim.merchant_links.rejected_slots.contains(&slot) {
        return Ok(0);
    }
    let Some(&item_id) = sim.merchant_items.get(slot) else {
        return Ok(0);
    };
    let offer = sim.merchant_links.offers.get(&slot).cloned();
    drop(sim);
    let text = match offer {
        None | Some(MerchantLinkOffer::Item { .. }) => {
            if item_id == 0 || crate::items::get_item(item_id).is_none() {
                return Ok(0);
            }
            match offer {
                Some(MerchantLinkOffer::Item { fields: None }) => None,
                Some(MerchantLinkOffer::Item {
                    fields: Some(fields),
                }) => crate::c_api::item_spell::item_link_with_fields(item_id, &fields),
                // Twelve unnamed fixed fields from the native template. No
                // bonus lists are supplied by the existing merchant model.
                _ => crate::c_api::item_spell::item_link_with_fields(item_id, &[0; 12]),
            }
        }
        Some(MerchantLinkOffer::Currency { record }) => {
            let Some(r) = record else {
                return Ok(0);
            };
            r.label.map(|label| {
                format!(
                    "|cn{}:|Hcurrency:{}:{}|h[{}]|h|r",
                    r.color, r.id, r.second, label
                )
            })
        }
        Some(MerchantLinkOffer::Spell { record } | MerchantLinkOffer::ResolvedSpell { record }) => {
            let Some(r) = record else {
                return Ok(0);
            };
            r.label.map(|label| {
                format!(
                    "|c{}|Hspell:{}:{}|h[{}]|h|r",
                    r.color, r.id, r.second, label
                )
            })
        }
        Some(MerchantLinkOffer::Unsupported | MerchantLinkOffer::Missing) => return Ok(0),
    };
    let value = text
        .as_deref()
        .map(|s| create_string(state, s))
        .unwrap_or(Val::Nil);
    state.push(value);
    Ok(1)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    // Native evidence is Retail-only; no speculative publication elsewhere.
    if cfg!(feature = "client-retail") {
        LuaApiMut::register_function(lua, "GetMerchantItemLink", get_merchant_item_link)?;
    }
    Ok(())
}
