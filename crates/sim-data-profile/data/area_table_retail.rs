//! Auto-generated Retail AreaTable lookup from the pinned DB2 CSV.
//! VERIFIED source: data/db2/retail-12.1.5.70783/provenance.json.
//! INFERRED: the native AreaTable u16 result is the DBD ContinentID field.
//! Do not edit manually - regenerate with: wow-cli generate area-table

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AreaTableRow {
    pub id: u32,
    pub continent_id: u16,
}

#[rustfmt::skip]
pub static AREA_TABLE_RETAIL: &[AreaTableRow] = &[
    AreaTableRow {
        id: 14620,
        continent_id: 2444,
    },
];

#[rustfmt::skip]
pub fn get_continent_id(area_id: u32) -> Option<u16> {
    let index = AREA_TABLE_RETAIL
        .binary_search_by_key(&area_id, |row| row.id)
        .ok()?;
    Some(AREA_TABLE_RETAIL[index].continent_id)
}
