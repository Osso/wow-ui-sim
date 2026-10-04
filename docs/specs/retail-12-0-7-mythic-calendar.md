# Retail 12.0.7 Mythic+ CalendarTime — B22

[Source row `prose-undated-016`](../../data/patch-api/sources/12.0.7-api-changes.txt) changes completion-date DTOs in three APIs. Cached `TimeDocumentation.lua` declares `monthDay`, `month`, `weekday`, `year`, `hour`, `minute`; cached MythicPlusInfo documentation gives tuple/record shapes. Cache may postdate 12.0.7; historical fields/indexing require authenticated pinned evidence before full acceptance.

## What it must do

- [ ] Under `retail-12-0-7`, all three APIs read explicit completion dates live and encode exactly six public CalendarTime fields, with no historical `day` field.
- [ ] Run history preserves run metadata and returns one detached array. Weekly best returns six values: duration, level, date, affixes, members, score. Season best returns two independently nilable records with duration, level, date, affixes, members and dungeonScore.
- [ ] Nested dates, affixes and members are detached; host state updates/removals are visible; environments remain independent.
- [ ] Authenticate every argument/extra via `unwrap_secret` before validation. Tainted secret flags/maps/extras fail even after malformed arguments or unknown maps; caller taint and host secrets survive GC.
- [ ] INFERRED: malformed exact-i32 map selectors miss. Undated history rows are omitted, undated/missing weekly best returns no values, undated/missing season sides return nil independently. Default history/weekly/season host inputs are empty; no dates are fabricated.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_mythic_plus_calendar.rs` — host DTOs, rooted fresh encoders and three producers.
- `src/c_api/patch_12_0_7_inputs.rs` — shared all-argument authentication and selector parsing.
- `src/c_api/mod.rs` — feature-gated module.
- `src/lua_api/state_types/mythic_plus_scenario.rs` — optional dates, affixes/members, independent season-best sides.
- `src/lua_api/globals/missing_surface/mythic_plus.rs` — registration routes retail to the C API producers; existing older-profile producers remain unchanged.

## Tests asserting this spec

- `tests/p1207_mythic_calendar.rs` — five public API cases.
- `tests/c_mythic_plus_probes.rs` — existing intersecting probes, now supplying explicit dates for populated fixtures.

## Known gaps (current cycle)

- [ ] History flags are authenticated but existing unfiltered history behavior is unchanged; no filtering credit.
- [ ] Weekday/month indexing, date validity/ranges/timezone and native/historical field provenance remain unproved. Fields are host inputs, not computed dates.

## Out of scope

- Native run derivation, missing-date recovery, date arithmetic, history filtering, UI interaction, other-profile execution and full page acceptance.
- Cached ChallengesUI guards nil weekly/season best; weekly reward consumers iterate history arrays. Empty defaults do not require excluded producers. Hosts with undated records lose those records from these outputs by explicit INFERRED policy.
