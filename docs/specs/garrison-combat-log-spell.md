# Garrison combat-log spell information

Retail and PTR expose `C_Garrison.GetCombatLogSpellInfo`, declared in pinned
`Blizzard_APIDocumentationGenerated/GarrisonInfoDocumentation.lua:116-128`.

## What it must do

- [ ] Look up the requested spell in generated 12.1.5.70391 enUS GarrAutoSpell data.
- [ ] Return its ID, name, cooldown, duration, school and icon, and an empty
  description. WM69 verifies the native builder's empty description.
- [ ] Derive preview mask, tutorial flag and thorns using WM69's related-record
  comparisons. The identification of the related table as GarrAutoSpellEffect
  and of its `Effect`, `Points` and `TargetType` columns is **INFERRED**.
- [ ] Return nil for a missing ID; WM69 labels this missing-record return **INFERRED**.

## Implementation inventory

- `tools/gen_garrison_auto_spell.py` — generated 70391 data and derived fields.
- `src/c_api/c_garrison_combat_log_spell.rs` — C API publication.

## Tests asserting this spec

- `tests/garrison_combat_log_spell_info.rs` — concrete catalog and missing-ID queries.

## Out of scope

Native argument conversion, secret handling and provider availability are not
established by WM69. The generated table is an offline supplied catalog; it
is not proof of live provider availability.
