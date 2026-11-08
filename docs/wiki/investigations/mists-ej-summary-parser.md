# Mists encounter-journal summary parser

The mists-panels triage reported a Mists encounter-journal failure caused by a
summary-parser defect. On master `f915f72b6` the defect is already fixed: the
failure was a test-helper false positive, not a simulator or Blizzard-Lua
defect. The encounter-journal test and the other six affected tests pass.

## Root cause

`tests/common/lua_error_output.rs::assert_no_lua_errors` used
`!stderr.contains("Lua error")`. The simulator's clean summary line is
`Lua errors: 0 unique, 0 occurrence(s)` (`src/lua_errors.rs:94-121`), so a clean
run matched the substring and the helper panicked with
`... flow emitted Lua errors`.

The triage row is cluster A in
[mists-panels-triage](../../../../.local/state/wow-ui-sim/handoff/mists-panels-triage/REPORT.md):
seven tests, including
`mists_encounter_journal_panel::mists_encounter_journal_opens_and_displays_instances`.

## Fix (already on master)

- `fe05a05f4` added the shared helper and parsed the structured summary fields
  (`Status:`, `Lua errors: <n> unique`) instead of the substring.
- `224474adf` converted the remaining panel tests, including
  `tests/mists_encounter_journal_panel.rs`.

`tests/lua_error_output_helper.rs` covers the helper (10 tests), including
rejection of a failed summary, non-empty stdout, a missing summary, and
`[exec-lua] error` text.

## Encounter-journal summary parser is correct

The Mists Blizzard file parses `$bullet;` markup in
`EncounterJournal_SetBullets`:

- `~/.cache/wow-ui-sim/blizzard-ui/mists/AddOns/Blizzard_EncounterJournal/Cata/Blizzard_EncounterJournal.lua:1277-1346`
  (`string.find(description, "$bullet;")`, `string.match(description, "(.-)$bullet;")`,
  `string.gmatch(description, "$bullet;([^$]+)")`).
- `:1348-1360` (`EncounterJournal_SetDescriptionWithBullets`).

A direct probe on the Mists build produced the native result: the intro text
was `Intro text.`, the parent `Bullets` table held `First bullet.` and
`Second bullet.`, and a second call with one bullet cleaned the extra bullet
into `BulletCache`. The simulator's Lua pattern engine handles the literal `$`
in the middle of the pattern correctly.

## Remaining failures

The `mists_` filter has 40 failures on master. None is caused by this defect:
the seven cluster-A tests pass. The `encounter` filter has five pre-existing
failures caused by missing Mists cache files (`Blizzard_POIButton` TOC and
`Blizzard_Collections_Bootstrap.lua`), not by the summary parser.

## Sources

- [mists-panels triage report](../../../../.local/state/wow-ui-sim/handoff/mists-panels-triage/REPORT.md)
  — cluster A row and the seven affected tests.
- [lua-error helper report](../../../../.local/state/wow-ui-sim/handoff/mists-lua-error-helper/REPORT.md)
  — conversion list and proof.
- `tests/common/lua_error_output.rs`, `tests/lua_error_output_helper.rs`,
  `tests/mists_encounter_journal_panel.rs`.
- `~/.cache/wow-ui-sim/blizzard-ui/mists/AddOns/Blizzard_EncounterJournal/Cata/Blizzard_EncounterJournal.lua:1277-1360`.

## See Also

- [[lua-api]] — `C_EncounterJournal` and `EJ_*` surface.
