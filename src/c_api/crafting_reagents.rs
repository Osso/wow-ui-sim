//! Nested profession reagent identities and host-provided recipe/order state.
use crate::lua_api::globals::profession_data;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CraftingReagent {
    Item(u32),
    Currency(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegularReagentInfo {
    pub reagent: CraftingReagent,
    pub quantity: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftingReagentInfo {
    pub reagent: CraftingReagent,
    pub data_slot_index: i32,
    pub quantity: i32,
}

#[derive(Debug, Clone)]
pub struct ReagentSlotInfo {
    pub mcr_slot_id: i32,
    pub required_skill_rank: i32,
    pub slot_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReagentSlotSchematic {
    pub reagents: Vec<CraftingReagent>,
    pub reagent_type: i32,
    pub variable_quantities: Vec<RegularReagentInfo>,
    pub quantity_required: i32,
    pub slot_info: Option<ReagentSlotInfo>,
    pub data_slot_type: i32,
    pub data_slot_index: i32,
    pub slot_index: i32,
    pub order_source: Option<i32>,
    pub required: bool,
    pub hidden_in_crafting_form: bool,
}

impl ReagentSlotSchematic {
    pub fn required_item(index: i32, item_id: u32, quantity: i32) -> Self {
        Self {
            reagents: vec![CraftingReagent::Item(item_id)],
            reagent_type: 1,
            variable_quantities: vec![],
            quantity_required: quantity,
            slot_info: None,
            data_slot_type: 0,
            data_slot_index: index,
            slot_index: index,
            order_source: None,
            required: true,
            hidden_in_crafting_form: false,
        }
    }
    pub fn quantity_for(&self, reagent: CraftingReagent) -> i32 {
        self.variable_quantities
            .iter()
            .find(|entry| entry.reagent == reagent)
            .map_or(self.quantity_required, |entry| entry.quantity)
    }
}

#[derive(Debug, Clone)]
pub struct ItemSlotModification {
    pub data_slot_index: i32,
    pub reagent: CraftingReagent,
}

#[derive(Debug, Clone)]
pub struct NewCraftingOrderInfo {
    pub skill_line_ability_id: i32,
    pub order_type: i32,
    pub order_duration: i32,
    pub tip_amount: f64,
    pub customer_notes: String,
    pub reagent_infos: Vec<RegularReagentInfo>,
    pub crafting_reagent_items: Vec<CraftingReagentInfo>,
    pub min_crafting_quality_id: Option<i32>,
    pub order_target: Option<String>,
    pub recraft_item: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CraftingOrderReagentInfo {
    pub reagent_info: CraftingReagentInfo,
    pub slot_index: i32,
    pub source: i32,
    pub is_basic_reagent: bool,
}

#[derive(Debug, Clone)]
pub struct CraftingOrder {
    pub order_id: u32,
    pub recipe_id: i32,
    pub request: NewCraftingOrderInfo,
    pub reagents: Vec<CraftingOrderReagentInfo>,
}

#[derive(Debug, Clone)]
pub struct CraftingInputs {
    pub recipe_slots: HashMap<i32, Vec<ReagentSlotSchematic>>,
    pub item_modifications: HashMap<String, Vec<ItemSlotModification>>,
    /// Deterministic host resourcefulness outcomes per cast, not a simulated proc chance.
    pub resource_returns: HashMap<i32, Vec<RegularReagentInfo>>,
    /// Explicit host mapping; ability IDs are not recipe IDs.
    pub order_recipes: HashMap<i32, i32>,
    pub orders: BTreeMap<u32, CraftingOrder>,
    pub next_order_id: u32,
}

impl Default for CraftingInputs {
    fn default() -> Self {
        let recipe_slots = profession_data::BLACKSMITHING_RECIPES
            .iter()
            .map(|recipe| {
                let slots = recipe
                    .reagents
                    .iter()
                    .enumerate()
                    .map(|(index, reagent)| {
                        ReagentSlotSchematic::required_item(
                            index as i32 + 1,
                            reagent.item_id,
                            reagent.quantity,
                        )
                    })
                    .collect();
                (recipe.recipe_id, slots)
            })
            .collect();
        Self {
            recipe_slots,
            item_modifications: HashMap::new(),
            resource_returns: HashMap::new(),
            order_recipes: HashMap::new(),
            orders: BTreeMap::new(),
            next_order_id: 1,
        }
    }
}

/// Validate a complete allocation before any inventory/order mutation.
pub(crate) fn validate_allocations(
    slots: &[ReagentSlotSchematic],
    allocations: &[CraftingReagentInfo],
) -> Result<(), String> {
    let mut quantities = HashMap::new();
    for allocation in allocations {
        let slot = slots
            .iter()
            .find(|slot| slot.data_slot_index == allocation.data_slot_index)
            .ok_or_else(|| "unknown crafting data slot".to_string())?;
        if !slot.reagents.contains(&allocation.reagent) || allocation.quantity <= 0 {
            return Err("invalid crafting reagent identity or quantity".into());
        }
        let quantity = quantities.entry(allocation.data_slot_index).or_insert(0i32);
        *quantity = quantity
            .checked_add(allocation.quantity)
            .ok_or("crafting quantity overflow")?;
    }
    for slot in slots.iter().filter(|slot| slot.required) {
        let expected: i32 = allocations
            .iter()
            .filter(|entry| entry.data_slot_index == slot.data_slot_index)
            .map(|entry| slot.quantity_for(entry.reagent))
            .max()
            .unwrap_or(slot.quantity_required);
        if quantities.get(&slot.data_slot_index).copied().unwrap_or(0) < expected {
            return Err("missing required crafting reagent quantity".into());
        }
    }
    Ok(())
}
