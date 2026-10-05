//! Rich host source inputs and an adapter for the existing compact world catalog.

use crate::lua_api::SimState;

pub(crate) fn read_source_info(sim: &SimState, id: i64) -> Option<AppearanceSourceInfo> {
    if let Some(record) = sim.transmog_appearance_sources.get(&id) {
        return Some(record.clone());
    }
    let source = sim
        .world
        .transmog_appearances
        .iter()
        .find(|row| i64::from(row.source_id) == id)?;
    let item = crate::items::get_item(source.item_id as u32);
    // INFERRED compact-record enrichment: plain item link, no transmoglink or
    // numeric subclass metadata. Rich host records supply those missing fields.
    Some(AppearanceSourceInfo {
        category: source.category_id,
        item_appearance_id: i64::from(source.visual_id),
        can_have_illusion: item
            .is_some_and(|item| matches!(item.inventory_type, 13 | 17 | 21 | 22)),
        icon: item
            .map(|item| i64::from(item.icon_file_data_id))
            .unwrap_or(0),
        is_collected: source.is_collected,
        item_link: format!("item:{}", source.item_id),
        transmoglink: String::new(),
        source_type: Some(source.source_type),
        item_subclass: 0,
        ignore_model_attachment_checks_for_illusion: false,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppearanceSourceInfo {
    pub category: i32,
    pub item_appearance_id: i64,
    pub can_have_illusion: bool,
    pub icon: i64,
    pub is_collected: bool,
    pub item_link: String,
    pub transmoglink: String,
    pub source_type: Option<i32>,
    pub item_subclass: i32,
    pub ignore_model_attachment_checks_for_illusion: bool,
}
