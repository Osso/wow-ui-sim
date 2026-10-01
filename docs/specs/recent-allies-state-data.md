# Recent Allies state-data rename

Bounded 12.0.5 audit slice `structures-RecentAllyStateData-669`: replace `hasFriendRequestPending` with `friendRequestSentThisSession` in explicit Recent Allies snapshots. Primary contract: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/RecentAlliesDocumentation.lua`, `GetRecentAllies` and its nested structures. [Audit context](../wiki/investigations/patch-12-0-5-api-audit.md).

## What it must do

- [x] Own an explicit C API input, initially disabled with no rows; never synthesize production allies.
- [x] Query only `C_RecentAllies.GetRecentAllies()` with zero arguments. Disabled input returns zero values; enabled empty input returns exactly one empty table.
- [x] Publish complete `RecentAllyData` rows with `stateData`, `characterData`, and `interactionData`, including explicit interaction sequences and context tables.
- [x] Publish distinct true/false `friendRequestSentThisSession` values with no `hasFriendRequestPending` alias. Preserve all other required values and optional nils.
- [x] Return independent nested snapshots; Lua mutations must not alter input or subsequent results.
- [x] Limit this modeled surface and fixtures to `retail-12-0-5` plus mainline `profile-retail`/`client-ptr`.

### Declared nested fields

| Structure | Required fields | Optional fields |
| --- | --- | --- |
| `RecentAllyStateData` | `isOnline`, `isDND`, `isAFK`, `isConvertedLegacyFriend`, `friendRequestSentThisSession`: bool | `pinExpirationDate`: time_t; `currentLocation`: string |
| `RecentAllyCharacterData` | `guid`: WOWGUID; `name`, `fullName`, `realmName`: string; `level`, `classID`, `raceID`: number; `sex`: UnitSex | none |
| `RecentAllyInteractionData` | `interactions`: table of RecentAllyInteraction | `note`: string |
| `RecentAllyInteraction` | `type`: RolodexType; `description`: cstring; `timestamp`: time_t; `contextData`: RecentAllyInteractionContextData | none |
| `RecentAllyInteractionContextData` | none | `itemID`, `activityDifficultyID`, `activityDifficultyLevel`: number; `locationName`: cstring |

**Source evidence:** `GetRecentAllies` has no declared arguments, returns nonnil `table<RecentAllyData>`, and declares `RequiresRecentAllies = true`. The predicate declares `FailureMode = "ReturnNothing"`.

**Inference, not native evidence:** one explicit `enabled` boolean models predicate eligibility; disabled is initial simulator policy. Enabled with no supplied rows returns an empty table. Stored vector order, independent snapshots, and integer Rust representations (`i64` timestamps/item IDs; `i32` numbers/enum values) are bounded model choices, not native ordering, numeric-limit or ownership claims. No support/readiness state is invented.

## How it works

- [C API boundary and environment state](../lua-api.md).
- [12.0.5 audit evidence boundaries](../wiki/investigations/patch-12-0-5-api-audit.md).

## Implementation inventory

- `src/c_api/c_recent_allies.rs`: explicit input, all nested rows, and bounded `GetRecentAllies` snapshot producer; implemented; four bounded actual-query fixtures GREEN at `3666902bf`.
- `src/c_api/mod.rs`: gated public input module.
- `src/c_api/registration.rs`: query registration under `retail-12-0-5` and mainline retail/PTR gates; other profiles unchanged.
- `src/lua_api/state/sim_state.rs`: per-environment input storage.
- `src/lua_api/state.rs`: disabled, empty input initialization.
- `tests/recent_allies_state_data.rs`: four actual-query cases in the existing generated integration harness; no new Cargo target.

## Tests asserting this spec

`tests/recent_allies_state_data.rs` covers disabled populated input, enabled empty input, two populated rows with opposite renamed flags and concrete nested/optional data, and deep result mutation versus input and later snapshots.

Proof ledger: input/fixtures committed in `d0495e796`; parent build at `ee3e2717213972f9c99d3ee26c3220ac57cfa983` succeeded (`/tmp/patch-12.0.5-batch12-red-build.json` and `.log`). Actual RED (`/tmp/patch-12.0.5-batch12-red-run.log` and `.json`) is 0/4: callable query returned one nil, failing disabled return count, enabled empty-table type, populated length, and independent snapshot identity. Fixtures call the actual registered query without test-time replacement.

Producer `3666902bf` passes all four actual-query fixtures: `/tmp/patch-12.0.5-batch12-green-runs.json` and `/tmp/patch-12.0.5-batch12-green-run-0.log` record exit 0, 4/4 at full revision `3666902bfc0838d394a7a1ab6ca324d13395a1d9`. Run-1 records six passing source/TOC controls, not addon runtime integration. Saved `/tmp/patch-12.0.5-batch12-green-startup-run.json`, `-startup.json` and `-startup.log` record the same revision's `--no-addons --no-saved-vars lua-errors` run: exit 0, `[]`, zero Lua errors.

Independent bounded PASS: `/tmp/patch-12.0.5-recent-allies-independent-proof.md` confirms reused actual RED 0/4, GREEN 4/4, six source/TOC controls and saved startup `[]`; wiring, nested rooting and scoped readability audits pass. Default `cargo fmt --check` and `cargo check` each exit 0 without warnings over `f442b0913` → `a956dfdd3`: before/after manifests cover 3,194 tracked source/test/crate/build/Cargo configuration files with zero changed hashes. Gate artifacts: `/tmp/patch-12.0.5-recent-allies-independent-gates.json`, `-fmt.log`, `-check.log`; source manifests: `-before.json`, `-after.json`.

This is bounded producer acceptance, not current full-source verification: newer cast input `c14076510` is not covered. Default profile compiled/executed only; mainline/epoch gates remain source-accounted, not an all-profile execution proof. Rooting is audited, not forced-GC stress-tested. No populated Blizzard-addon integration, native parity, full-system or whole-page acceptance.

Old-provider trace: no explicit `C_RecentAllies`/`GetRecentAllies` registration existed in `src`. `init_lua_state` runs Rust globals registration before `init_runtime_surface_bootstrap`; that bootstrap's `_G.__index` creates missing `C_*` namespaces, then `__wow_namespace_mt.__index` installs `function() return nil end` for missing methods. New registration supplies the concrete method before bootstrap. No targeted obsolete provider exists to remove; generic other-namespace fallback remains untouched.

Rooting: each newly allocated output/nested table is pushed immediately, stays rooted while fields and descendants allocate, and loses its temporary stack root only after attachment to a rooted parent with the table write barrier. `table_set_static` temporarily roots string values while field names intern. Enabled queries leave only the output sequence as the result; disabled queries allocate no snapshots. Each query copies explicit Rust rows and builds fresh nested tables; absent optional fields stay nil. No secret decoding, taint clearing or synthetic records are introduced.

## Known gaps (current cycle)

- [x] Independent bounded Rust gates and scoped readability: recorded in [proof ledger](#tests-asserting-this-spec), limited to the unchanged-source interval stated there.
- [ ] Future native probe: query disabled/enabled empty/populated states; record return counts, nested nils, renamed true/false flags, ordering and mutations across repeated queries. Native eligibility and default state remain unknown.

## Out of scope

Lookup, search, mutation, readiness/support queries, events, location preference, production records, and earlier/nonmainline behavior. No full-system, full-page audit completion or native parity credit.
