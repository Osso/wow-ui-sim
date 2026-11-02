//! Currency data for the WoW UI simulator.
//!
//! Provides a static list of currencies with quantities for the token frame UI.
//! The list is hierarchical: headers group currencies into categories.

/// A currency entry in the currency list.
#[derive(Clone, Copy)]
pub struct CurrencyEntry {
    pub currency_id: i32,
    pub name: &'static str,
    pub quantity: i32,
    pub max_quantity: i32,
    pub icon_file_id: u32,
    pub quality: i32,
    pub is_header: bool,
    pub is_header_expanded: bool,
    pub depth: i32,
    pub is_discovered: bool,
    pub is_show_in_backpack: bool,
}

const fn header(name: &'static str) -> CurrencyEntry {
    CurrencyEntry {
        currency_id: 0,
        name,
        quantity: 0,
        max_quantity: 0,
        icon_file_id: 0,
        quality: 0,
        is_header: true,
        is_header_expanded: true,
        depth: 0,
        is_discovered: true,
        is_show_in_backpack: false,
    }
}

const fn currency(
    currency_id: i32,
    name: &'static str,
    quantity: i32,
    max_quantity: i32,
    icon_file_id: u32,
    quality: i32,
) -> CurrencyEntry {
    CurrencyEntry {
        currency_id,
        name,
        quantity,
        max_quantity,
        icon_file_id,
        quality,
        is_header: false,
        is_header_expanded: false,
        depth: 1,
        is_discovered: true,
        is_show_in_backpack: false,
    }
}

const fn watched(mut c: CurrencyEntry) -> CurrencyEntry {
    c.is_show_in_backpack = true;
    c
}

/// Static currency list (headers + entries).
static CURRENCY_LIST: &[CurrencyEntry] = &[
    header("The War Within"),
    watched(currency(2245, "Valorstones", 1847, 0, 5868905, 3)),
    currency(2806, "Weathered Harbinger Crest", 42, 90, 5868904, 2),
    currency(2807, "Carved Harbinger Crest", 15, 90, 5868906, 3),
    currency(2809, "Runed Harbinger Crest", 3, 90, 5868907, 4),
    watched(currency(3089, "Resonance Crystals", 620, 4000, 3528287, 3)),
    header("Player vs. Player"),
    watched(currency(1792, "Honor", 4350, 15000, 1140617, 0)),
    currency(1901, "Honor", 0, 0, 1140617, 0),
    currency(390, "Conquest Points", 0, 0, 463448, 0),
    currency(483, "Conquest from Arena", 0, 0, 463448, 0),
    currency(484, "Conquest from Rated Battlegrounds", 0, 0, 463449, 0),
    currency(1602, "Conquest", 880, 0, 1140616, 0),
    header("Miscellaneous"),
    currency(1191, "Valor", 0, 0, 5868908, 0),
    currency(1813, "Reservoir Anima", 23500, 35000, 3528287, 0),
    currency(1767, "Stygia", 140, 0, 134418, 0),
    currency(824, "Garrison Resources", 0, 0, 1042294, 0),
    currency(1101, "Oil", 0, 0, 1391724, 0),
];

/// Number of items in the currency list.
pub fn currency_list_size() -> i32 {
    CURRENCY_LIST.len() as i32
}

/// Get a currency list entry by 1-based index.
pub fn get_currency_list_entry(index: i32) -> Option<&'static CurrencyEntry> {
    CURRENCY_LIST.get((index - 1) as usize)
}

/// Get currency info by currency ID.
pub fn get_currency_by_id(currency_id: i32) -> Option<&'static CurrencyEntry> {
    CURRENCY_LIST
        .iter()
        .find(|c| !c.is_header && c.currency_id == currency_id)
}

/// Backpack (watched) currencies, returned as (index, entry) pairs.
pub fn backpack_currencies() -> impl Iterator<Item = &'static CurrencyEntry> {
    CURRENCY_LIST
        .iter()
        .filter(|c| c.is_show_in_backpack && !c.is_header)
}

/// Build the initial `SimState.currency_info` map from generated
/// `CurrencyTypes` DB2 rows, then overlay the small visible-token list with
/// simulator quantities/backpack flags.
pub fn seeded_currency_info_map()
-> std::collections::HashMap<i32, crate::lua_api::state::CurrencyInfo> {
    let mut currencies = crate::currencies::CURRENCY_TYPES
        .entries()
        .map(|(currency_id, currency_type)| {
            (
                *currency_id,
                currency_type_to_info(*currency_id, currency_type),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();

    for entry in CURRENCY_LIST.iter().filter(|c| !c.is_header) {
        overlay_currency_list_entry(&mut currencies, entry);
    }

    currencies
}

fn currency_type_to_info(
    currency_id: i32,
    currency_type: &crate::currencies::CurrencyTypeInfo,
) -> crate::lua_api::state::CurrencyInfo {
    crate::lua_api::state::CurrencyInfo {
        currency_id,
        name: currency_type.name.to_string(),
        description: currency_type.description.to_string(),
        icon_file_id: currency_type.icon_file_id,
        max_quantity: currency_type.max_quantity,
        max_weekly_quantity: currency_type.max_weekly_quantity,
        quality: currency_type.quality,
        transfer_percentage: currency_type.transfer_percentage,
        discovered: true,
        ..crate::lua_api::state::CurrencyInfo::default()
    }
}

fn overlay_currency_list_entry(
    currencies: &mut std::collections::HashMap<i32, crate::lua_api::state::CurrencyInfo>,
    entry: &CurrencyEntry,
) {
    let info = currencies.entry(entry.currency_id).or_insert_with(|| {
        crate::lua_api::state::CurrencyInfo {
            currency_id: entry.currency_id,
            name: entry.name.to_string(),
            discovered: entry.is_discovered,
            ..crate::lua_api::state::CurrencyInfo::default()
        }
    });

    info.quantity = entry.quantity;
    info.name = entry.name.to_string();
    info.max_quantity = entry.max_quantity;
    info.icon_file_id = entry.icon_file_id;
    info.quality = entry.quality;
    info.is_show_in_backpack = entry.is_show_in_backpack;
    info.discovered = entry.is_discovered;
    info.currency_list_depth = entry.depth;
}
