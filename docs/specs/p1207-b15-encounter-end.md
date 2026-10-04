# 12.0.7 encounter-end

Proposed bounded contract for `prose-undated-009`, `events-ENCOUNTER_END-163`. Source: [retained API excerpt](../../data/patch-api/sources/12.0.7-api-changes.txt). Later cached declarations may postdate 12.0.7; no native historical proof.

## What it must do

- [ ] Preserve exact six-argument success tuple, multiple concrete creature IDs/names/health percentages and ordering.
- [ ] Public fields remain nonsecret in a genuinely tainted event handler; output/input edits and garbage collection do not alter subsequent snapshots.
- [ ] Omitted/nil lists produce separate empty snapshots; earlier epochs retain existing five-argument payload.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- src/lua_api/globals/admin_encounter.rs:20–57 — synchronous dispatch and snapshot rooting.
- src/lua_api/globals/admin_encounter/unit_status.rs — dense list validation and copying.

## Tests asserting this spec

`tests/p1207_b15_encounter_end.rs` — One new test predicted PASS by reading: copied Val::Num/Str fields are public, and stamped OnEvent callback runs tainted. Constant empty status, shared table aliasing, secret fields or five-argument payload fail concrete assertions. With snapshot producer withheld, exact arity/data fail. Existing malformed-list and BOSS_KILL-order tests stay unchanged. No code edits needed.

Source-reading predictions only; no test execution or accepted coverage.

## Known gaps (current cycle)

- [ ] Integrate and run staged tests under strict retail-12-0-7.
- [ ] Automatic encounter engagement history is not modeled; do not call supplied fixture list exhaustive native engaged-boss tracking. A_Admin argument/malformed policies are simulator inputs, not declaration-level C API policy. No encounter loss producer proof.

## Out of scope

Native parity, automatic server synchronization, production datasets, vendor/cache edits and adjacent API rows.
