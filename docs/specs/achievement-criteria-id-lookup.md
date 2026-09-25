# Achievement criteria ID lookup

Retail's `GetAchievementCriteriaInfoByID(achievementID, criteriaID)` queries the existing achievement criteria model by criterion ID, rather than by ordinal index. The modeled criteria and legacy query live in `src/lua_api/globals/missing_surface/achievement_info/`.

## What it must do

- [ ] Return the same 13 values as `GetAchievementCriteriaInfo(achievementID, index)` for a matching stored criterion, including its actual ID and completion-dependent progress.
- [ ] Distinguish criterion ID from ordinal index; match only criteria belonging to the requested achievement.
- [ ] Return nil for unknown criterion IDs, unknown achievements, and achievements without modeled criteria.

## How it works

- [Lua API](../lua-api.md) describes global registration and queries.

## Implementation inventory

- `src/lua_api/globals/missing_surface/achievement_info/categories.rs` — stored criterion IDs and lookups.
- `src/lua_api/globals/missing_surface/achievement_info.rs` — shared criteria tuple and missing-result behavior.
- `src/lua_api/globals/missing_surface/achievement_info/registration.rs` — legacy global registration.

## Tests asserting this spec

- `tests/achievements_api.rs` — index/ID tuple equivalence, completion and missing-result tests.

## Known gaps (current cycle)

- [ ] Only criteria already present in the simulator model are queryable; the complete game criteria catalog is not modeled.

## Out of scope

- Native-client tuple verification and bulk criteria import; this contract covers only stored simulator criteria.
