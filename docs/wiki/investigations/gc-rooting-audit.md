# Host-held Lua values across collection

Rust `Val`/`GcRef` locals are not VM roots. Callback execution can collect DTOs that have not been pushed onto the VM stack or attached to a reachable table. Audit starts at `16682b415` (item tooltip rooting).

## Confirmed sites

| Site | Vulnerable window | Fix | Forced-GC regression |
|---|---|---|---|
| `c_allied_races::build_allied_race_info_table` | Strings and achievement sequence created before `CreateColor` | Run callback first, then assemble DTO without Lua execution | `race_info_survives_collection_in_color_callback` |
| `c_artifact_ui::helpers::build_artifact_art_info_table` | Earlier colors and title strings survive later `CreateColor` calls only in locals | Stack-root DTO, attach each color before next callback | `artifact_art_survives_collection_in_each_color_callback` |
| `c_major_factions::build_major_faction_data_table` | DTO and descriptive strings survive color callback only in locals | Stack-root DTO, attach strings before callback | `faction_data_survives_collection_in_color_callback` |

All three regressions failed at baseline with `table has been collected` / `invalid table reference` when the color callback performed `collectgarbage('collect')`.

## Tooltip follow-up

Eleven additional construction sites need roots: `color_segment_table`, spell, aura, toy, mount, unit, currency, companion-pet DTO builders, action binding append, achievement and shapeshift probes. Rooting `lines` alone does not keep its containing tooltip alive: reachability is parent-to-child, not child-to-parent. Root each tooltip for the entire fill; root the segment array and attach entries before color callbacks.

`tests/tooltip_gc_rooting.rs` forces collection in ten API calls, checking IDs, text, width hints, GUIDs, line counts and segment RGB/text. Mutable pet/mount/achievement/form names carry markup to reach the color callback even when their normal text is plain. All ten initially fail with collected-table errors; the action test limits collection to the final green binding line.

C API subsystem filters at `7f6d28454`: allied races 13/13, artifact panel 42/42, major factions 14/14.

## Sources

- Commit `16682b415` — established stack/parent rooting idiom.
- [Allied races](../../../src/c_api/c_allied_races.rs)
- [Artifact art](../../../src/c_api/c_artifact_ui/helpers.rs)
- [Major factions](../../../src/c_api/c_major_factions.rs)

## See Also

- [Lua API](../../lua-api.md)
- [Event system](../../event-system.md)
