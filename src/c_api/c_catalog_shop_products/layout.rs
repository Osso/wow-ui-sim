//! Host-supplied bundle and section snapshots; no seeded storefront records.

use super::*;

#[derive(Clone, Debug, Default)]
pub struct CatalogShopBundleChildInfo {
    pub child_product_id: f64,
    pub display_order: f64,
    pub quantity_in_bundle: f64,
}

#[derive(Clone, Debug, Default)]
pub struct CatalogShopSectionInfo {
    pub id: f64,
    pub display_name: String,
    pub parent_catalog_shop_category_info_id: Option<f64>,
    pub card_type: Option<String>,
    pub scroll_grid_size: Option<f64>,
    pub should_show_recommendation_opt_out_disclaimer: bool,
}

pub(super) fn register(state: &mut LuaState, namespace: Val) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetProductIDsForBundle", bundle_children)?;
    table_set_rust_fn_static(state, namespace, "GetCategorySectionInfo", section_info)
}

fn bundle_children(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_public_id(state, 1)?;
    // INFERRED: unknown bundles have no children; no synthetic quantity of one.
    let children = borrow_state(state)?
        .catalog_shop_products
        .bundle_children
        .get(&id)
        .cloned()
        .unwrap_or_default();
    let result = create_table(state);
    state.push(result);
    for (index, child) in children.iter().enumerate() {
        let row = create_table(state);
        state.push(row);
        for (key, value) in [
            ("childProductID", child.child_product_id),
            ("displayOrder", child.display_order),
            ("quantityInBundle", child.quantity_in_bundle),
        ] {
            table_set_static(state, row, key, Val::Num(value));
        }
        set_table_array(state, result, index as i64 + 1, row);
        state.pop();
    }
    Ok(1)
}

fn section_info(state: &mut LuaState) -> LuaResult<u32> {
    let category = read_public_id(state, 1)?;
    let section = read_public_id(state, 2)?;
    // INFERRED: missing nonnullable DTO is an explicit input error, not a fake row.
    let input = borrow_state(state)?
        .catalog_shop_products
        .sections
        .get(&(category, section))
        .cloned()
        .ok_or_else(|| {
            rilua::runtime_error(&format!(
                "C_CatalogShop.GetCategorySectionInfo: missing host section {category}/{section}"
            ))
        })?;
    push_section(state, &input);
    Ok(1)
}

fn push_section(state: &mut LuaState, input: &CatalogShopSectionInfo) {
    let row = create_table(state);
    state.push(row);
    table_set_static(state, row, "ID", Val::Num(input.id));
    let name = create_string(state, &input.display_name);
    table_set_static(state, row, "displayName", name);
    for (key, value) in [
        (
            "parentCatalogShopCategoryInfoID",
            input.parent_catalog_shop_category_info_id,
        ),
        ("scrollGridSize", input.scroll_grid_size),
    ] {
        if let Some(value) = value {
            table_set_static(state, row, key, Val::Num(value));
        }
    }
    if let Some(value) = &input.card_type {
        let value = create_string(state, value);
        table_set_static(state, row, "cardType", value);
    }
    table_set_static(
        state,
        row,
        "shouldShowRecommendationOptOutDisclaimer",
        Val::Bool(input.should_show_recommendation_opt_out_disclaimer),
    );
}
