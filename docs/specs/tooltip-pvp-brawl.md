# PvP brawl tooltip

## Publication and selection

Retail, PTR and Forever pinned `TooltipInfoDocumentation.lua:710` declare
`C_TooltipInfo.GetPvpBrawl([isSpecial])`. PTR and Forever reuse the same supplied
brawl records; that is **INFERRED**, not separate native proof.

- The optional flag uses Lua truthiness and defaults false (WM94 §6). False or
  omitted selects `state.active_brawl`; true selects
  `state.special_event_brawl`.
- A missing record uses [the shared UNKNOWN failure
  seam](tooltip-appearance-item.md): zero return values are **EMULATED**
  simulator policy.
- The active record rejects a false `canQueue` before any row. The special-event
  path forces queue permission true (WM107 §5).

## Rows

Tooltip type 16 (`PvPBrawl`), no identity id (WM107).

| # | Row | Type | Colour | Gate |
|---|---|---|---|---|
| 1 | record's raw localized name | 0 | highlight (published white) | none |
| 2 | `BRAWL_TOOLTIP_ENDS` with the localized duration | 42 | `DISABLED_FONT_COLOR` / trivial gray | remaining-time field present |
| 3 | blank (`" "`) | 1 | normal | map count nonzero |
| 4 | `BRAWL_TOOLTIP_MAPS` for all-arena or several maps, `BRAWL_TOOLTIP_MAP` for one | 0 | highlight (published white) | map count nonzero |
| 5 | `PVP_BRAWL_TOOLTIP_ALL_ARENA_MAPS`, or one `BRAWL_TOOLTIP_MAP_ENTRY` per map | 0 | normal | map count nonzero |
| 6 | blank (`" "`) | 1 | normal | description nonempty |
| 7 | `BRAWL_TOOLTIP_RULES` | 0 | highlight (published white) | description nonempty |
| 8 | long description | 0 | normal | description nonempty |

- The name is never formatted through a global; the pinned build contains no
  `BRAWL_TOOLTIP_NAME` usage (WM107 §3).
- The ends-in row is emitted for the active record even when the remaining value
  is zero, because the gate tests presence only. The special-event record never
  emits it (WM107 §5).
- There is no queue row, no short-description row and no separator (WM107 §1).
- Row 2 is requested as a type-0 line but serializes as 42 because its colour
  value equals `DISABLED_FONT_COLOR` (WM107 §4); the producer publishes 42
  directly.

**EMULATED:** WM107 leaves the exact localized ends-in text and the loaded
GlobalColor values UNKNOWN. The ends-in text formats the pinned
`BRAWL_TOOLTIP_ENDS` global with the simulator's existing localized duration
formatter (the Totem formatter, rounded upward), and the ends-in row publishes
the measured `0.5019608` trivial/disabled gray explicitly.

**Named-colour gap:** the simulator's `runtime_surface_bootstrap.lua` synthesizes
every unmodelled `*_COLOR` constant, including `NORMAL_FONT_COLOR` and
`DISABLED_FONT_COLOR`, as white. Passing an explicit highlight colour would
therefore equal the disabled placeholder and be re-typed to 42 by the native
value-based remap. Plain and blank rows pass no explicit colour, so they publish
the default normal value (white), which matches the measured highlight value;
the WM107 normal-colour rows (map entries, blank lines, long description)
publish white instead of the data-table gold until the named-colour surface is
modelled.

## Evidence and proof

Clean-room native source: WM107
`native-evidence-70688/WM107-pvp-brawl-tooltip.md`.
`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:577` supplies the original
`GameTooltip:SetPvpBrawl` accessor.

`tests/tooltip_pvp_brawl.rs` runs that consumer for an active record with one
map and remaining 7200s (eight rows, types 0/42/1, trivial colour, singular
heading, explicit map row) and for a special record with two maps, all-arena
true and `canQueue` false (seven rows, no type 42, forced queue permission,
plural heading and the all-arena row). It also proves that a nonqueueable active
record and a missing record take the missing route. Native protected-return arity
and the localized ends-in text are not claimed.
