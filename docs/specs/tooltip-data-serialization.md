# Tooltip data serialization

## Scope

Retail/PTR common populated tooltip builders follow WM82 and WM63 D56 (Retail
12.1.5 build 70688). Applying the shared rules to other getters using these
builders is **INFERRED**, not independent native getter proof. Other profiles and
existing miss shapes are unchanged.

Pinned API declarations on both profiles: `Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:439`
(`GetItemByID`), `:1045` (`GetSpellByID`), `:1181` (`GetUnit`). Existing getter
publication is not expanded by this change.

## Requirements

- [x] Populated publications have fresh tables/lines and an incrementing per-state `dataInstanceID`.
- [x] Common text is copied with markup intact, without synthetic segment arrays.
- [x] Supplied sides, including empty strings, publish text and RGB color; absent sides publish neither.
- [x] RGB uses packed bytes multiplied in single precision by `1/255`, without alpha; preserve usable color methods.
- [x] Populated identity omits zero `id`; later common identity assignments append ordered type-19 rows with `tooltipType`, nonzero-only `tooltipID`, empty left text and supplied left color.
- [x] Original `ProcessInfo`, GameTooltip line handlers and TooltipUtil retain their payloads; presentation parses inline markup independently.

## Evidence limits

WM82 and D56 establish the publication boundary, not complete protected getter
connections or misses. Existing misses retain their prior shapes. No GUID is
fabricated. No getter is assigned new nested-row eligibility: the generic builder
supports later identity assignment, but no current producer requests one.

`scenario_color_byte` is the single **EMULATED** float-to-packed-byte input seam:
round to the nearest saturated byte. Native float-to-byte conversion is **UNKNOWN**.
The existing color factory is retained; full native `ColorMixin` contents remain
**UNKNOWN**. Default side colors use the supplied `NORMAL_FONT_COLOR` scenario
value, not an assertion about every native producer.

WM72 establishes that empty strings on both sides append no displayed row and
right-only rows can exist. It does not add TooltipData fields or imply geometry
changes here.

## Proof

- `tests/tooltip_d56_serialization.rs`: direct getters and original consumers.
- `src/c_api/tooltip_data_tests.rs`: nested identity order and present/absent sides.
- Existing tooltip builder, GC and presentation tests remain regression coverage.

## Related

- [Loot/buyback](tooltip-loot-buyback.md)
- [Lua API](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation)
