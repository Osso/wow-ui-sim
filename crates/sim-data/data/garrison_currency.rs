//! Generated from GarrType.csv build 12.1.5.70391; run tools/gen_garrison_currency.py.

pub static GARRISON_CURRENCIES: &[(i32, i32, i32)] = &[
    (2, 824, 1101),
    (3, 1220, 0),
    (9, 1560, 0),
    (111, 1810, 1813),
];

pub fn currency_types(garrison: i32) -> Option<(i32, i32)> {
    let index = GARRISON_CURRENCIES
        .binary_search_by_key(&garrison, |row| row.0)
        .ok()?;
    let (_, primary, secondary) = GARRISON_CURRENCIES[index];
    Some((primary, secondary))
}
