# Forever partial spell names

The Forever profile needs the spell names listed by ID in `data/forever-spell-ids.txt`. The list covers the class buffs in issue 13; it is not a complete spell catalog. No spell metadata is inferred from the name table.

## Source and regeneration

The names were exported from `SpellName` file data ID 1990283 in the installed Forever product `wowf`, build 1.60.1.70685, build key `002a8047a7c3a53de5e981936db76cce`. The extracted WDC5 file has layout `782EE721` and SHA-256 `2fab24d9a044b733aa8ded3691006c4947776bc6dbca9f145bae9e3ac5e494ce`. The matching WoWDBDefs `SpellName.dbd` has SHA-256 `5f3cbcdf957152668d5e033c1c4ff79a7b436e2e45b8521270ca7e64cc028629`. Extraction reported encrypted, zero-filled chunks; the exporter reported 624 unreadable records. All 72 selected IDs have readable names.

Export with the game-engine `scripts/export_db2_csv.py SpellName <db2> <csv> --definitions <definitions-directory>`, then run `wow-cli generate forever-spell-names <csv>` from the repository root. The generator rejects missing or empty selected names. The exported CSV SHA-256 is `e410d002ba5991fad50c0ae4ceaf1eae3019295323b331f55e8abac3b6352499`.
