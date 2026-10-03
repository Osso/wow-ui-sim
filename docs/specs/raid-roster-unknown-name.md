# Raid roster unknown name

`GetRaidRosterInfo(index)` returns a nonnil localized unknown name for an existing member whose name is not cached. [12.0.5 register](../../data/patch-api/sources/12.0.5-register.json) rows `prose-2026-03-25-123` and `prose-2026-03-31-180` state: “Instead, it will return "Unknown" during that period.” The later row finalizes the earlier proposal. Neither row identifies a localization global. Cached `Blizzard_RaidUI/Classic/Blizzard_RaidUI.lua:342–343` replaces an absent roster name with `UNKNOWN`, corroborating that binding.

## What it must do

- [x] Cached roster members retain their real name and existing twelve-result tuple.
- [x] An existing uncached member returns the localized `UNKNOWN` string as a plain, non-secret first result; all eleven other returns and exact arity remain unchanged. INFERRED localization binding: `UNKNOWN` is already registered as `Unknown`; `UNKNOWNOBJECT` also exists, but neither post identifies which global the native API uses. Public sentinel policy follows this slice's explicit requirement, not a secrecy claim made by the posts.
- [x] Cache arrival changes subsequent reads to the stored real name without rebuilding the roster; other members and the player remain unaffected.
- [x] Invalid indices and an inactive group retain twelve nil results, rather than reporting an unknown-name member. This preserves observed simulator implementation, not native miss-policy evidence.
- [x] INFERRED initialization: existing named records start with `name_cached = true`; the synthesized local-player roster record is always cached. `name_cached = false` explicitly models the transient missing-cache state without discarding the stored name.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/group_queries.rs` — existing roster provider reads cache state live and uses the localized global only for unresolved members.
- `src/lua_api/game_data.rs` — required `PartyMember.name_cached` field and named-record initialization; exact integration insertions are in the slice handoff, not applied by its author.

## Tests asserting this spec

- `tests/raid_roster_unknown_name.rs` — cached tuple, unresolved public sentinel, live arrival and peer isolation, absent indices and inactive group.

## Development proof and independent bounded acceptance — 2026-10-03

Inputs and producer landed together in `a0e23199d`. RED with the producers withheld from the working tree: 2 PASS / 2 FAIL with the producer forced to the cached branch. GREEN: 4/4; the combined run was 396 PASS / 1 FAIL, the failure being `c_system_api::test_c_console_get_all_commands_empty` on an untouched console command count. `cargo fmt --check` exit0; startup `lua-errors` `[]`.

Main accepts an independent GPT-6.1-sol review: **ACCEPT WITH QUALIFICATIONS** (report SHA256 `caf669b3a28143b86dd45ae1f8eeca25e4b0a66992b5ce70640e3520ee1432bf`, scratchpad-only), own rerun 4/4 exit0. RED was produced by forcing the cached branch, not by reverting to the parent producer. Checked requirements are bounded simulator proof on the tested fixtures, not native parity. No `cargo check`, broad suite or older-profile run.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows prose-2026-03-25-123, prose-2026-03-31-180 under new capability `raid-roster-unknown-name`; **107 capabilities/362 IDs; 77 pending /239 bounded /13 partial /33 metadata**.

## Known gaps (current cycle)

- [ ] Native identity with the localized `UNKNOWN` global and absent-index behavior are inferred, not native-verified.
- [ ] Native localization-global identity and native absent-index behavior remain unverified.

## Out of scope

- Automatic asynchronous name-cache population, events, cached-name secrecy changes and other name APIs.
- Profile-specific behavior: the existing provider has no epoch gating, so this change adds none, as required by the slice. INFERRED applicability beyond 12.0.5 is not native-verified; tests are scoped to `retail-12-0-5`.
- Existing tuple limitations (including online always true and fixed zone/roles) are preserved, not repaired.
