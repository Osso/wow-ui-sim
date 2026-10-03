# Raid roster unknown name

`GetRaidRosterInfo(index)` returns a nonnil localized unknown name for an existing member whose name is not cached. [12.0.5 register](../../data/patch-api/sources/12.0.5-register.json) rows `prose-2026-03-25-123` and `prose-2026-03-31-180` state: “Instead, it will return "Unknown" during that period.” The later row finalizes the earlier proposal. Neither row identifies a localization global. Cached `Blizzard_RaidUI/Classic/Blizzard_RaidUI.lua:342–343` replaces an absent roster name with `UNKNOWN`, corroborating that binding.

## What it must do

- [ ] Cached roster members retain their real name and existing twelve-result tuple.
- [ ] An existing uncached member returns the localized `UNKNOWN` string as a plain, non-secret first result; all eleven other returns and exact arity remain unchanged. INFERRED localization binding: `UNKNOWN` is already registered as `Unknown`; `UNKNOWNOBJECT` also exists, but neither post identifies which global the native API uses. Public sentinel policy follows this slice's explicit requirement, not a secrecy claim made by the posts.
- [ ] Cache arrival changes subsequent reads to the stored real name without rebuilding the roster; other members and the player remain unaffected.
- [ ] Invalid indices and an inactive group retain twelve nil results, rather than reporting an unknown-name member. This preserves observed simulator implementation, not native miss-policy evidence.
- [ ] INFERRED initialization: existing named records start with `name_cached = true`; the synthesized local-player roster record is always cached. `name_cached = false` explicitly models the transient missing-cache state without discarding the stored name.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/group_queries.rs` — existing roster provider reads cache state live and uses the localized global only for unresolved members.
- `src/lua_api/game_data.rs` — required `PartyMember.name_cached` field and named-record initialization; exact integration insertions are in the slice handoff, not applied by its author.

## Tests asserting this spec

- `tests/raid_roster_unknown_name.rs` — cached tuple, unresolved public sentinel, live arrival and peer isolation, absent indices and inactive group.

## Known gaps (current cycle)

- [ ] Integrate the model field and all construction-site initializers from the handoff; compile, format and execute tests in the main session. No execution proof exists for this slice.
- [ ] Native localization-global identity and native absent-index behavior remain unverified.

## Out of scope

- Automatic asynchronous name-cache population, events, cached-name secrecy changes and other name APIs.
- Profile-specific behavior: the existing provider has no epoch gating, so this change adds none, as required by the slice. INFERRED applicability beyond 12.0.5 is not native-verified; tests are scoped to `retail-12-0-5`.
- Existing tuple limitations (including online always true and fixed zone/roles) are preserved, not repaired.
