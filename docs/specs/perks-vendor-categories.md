# Perks vendor categories

Retail and PTR expose `C_PerksProgram.GetAvailableCategoryIDs` and `GetCategoryInfo` from the pinned `PerksVendorCategory` catalogue. The simulator's default scenario makes all six exported categories available. Each category query returns a fresh table with its ID, display name and default model-scene ID. Unknown category IDs return no value.

`python3 tools/gen_perks_categories.py` reads the wago.tools `PerksVendorCategory.csv` export for build 12.1.5.70391, locale enUS, SHA-256 `e27dada35b9379b75f1165d14a03ff45177837a2cf042103a2ed972351fbf1b6`. Retail and PTR `PerksProgramDocumentation.lua:19–25,37–49,403–409` declare the queries and fields. WoW Forever's PerksProgram TOC excludes camelot; this model is not published there.

## Best-guess basis (best-guess, native UNKNOWN)

The export provides category records, not current server availability. Making all exported categories available is a simulator scenario, not a claim about a live vendor. Native unknown-ID behavior is unknown. The PTR export was not obtained separately.

## Known gaps

Vendor items, currency, purchases and live availability are not modeled here. Default scene 430 is not among the three generated model scenes; this change does not claim 3D rendering or complete preview data.
