//! Currency data for the WoW UI simulator.
//!
//! Provides a static list of currencies with quantities for the token frame UI.
//! The list is hierarchical: headers group currencies into categories.

/// A currency entry in the currency list.
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
    currency(1602, "Conquest", 880, 0, 1140616, 0),
    header("Miscellaneous"),
    currency(1191, "Valor", 0, 0, 5868908, 0),
    currency(1813, "Reservoir Anima", 23500, 35000, 3528287, 0),
    currency(1767, "Stygia", 140, 0, 134418, 0),
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
