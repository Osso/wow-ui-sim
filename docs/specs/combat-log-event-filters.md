# Combat-log event filter records

Stage Retail/PTR AddEventFilter and sibling ClearEventFilters in `src/c_api/c_combat_log.rs`. This is an opaque filter-record model, not a native event matcher. [Consumer selection and RED discipline](../wiki/investigations/consumer-selection-and-red-proof.md) govern acceptance.

## What it must do

- [ ] Original TOC-selected secure Blizzard_CombatLogProcessor SetFilterSettings/ApplyFilterSettings adds supplied filter records and replaces them after clear; empty settings clear them all.
- [ ] Public C_CombatLog and C_CombatLogSecure each retain independent filter records; public clear does not erase secure records. Independence is simulator staging policy, not established native behavior.
- [ ] Preserve argument order, count, nil positions and values without coercion or inferred defaults. A simulator-only diagnostic returns a fresh packed-record snapshot for the supplied namespace; it is not a WoW API.
- [ ] Do not reset filtered-event enablement when clearing filter records: these are distinct state domains.

## How it works

- [Lua API subsystem](../lua-api.md)
- [Original-consumer proof rules](../wiki/investigations/consumer-selection-and-red-proof.md)

## Implementation inventory

- `src/c_api/c_combat_log.rs`: namespace filter-record ownership and diagnostic query.
- `src/lua_api/env_init/mod.rs`: register before secure environment capture and legacy alias creation.
- `src/lua_api/workarounds/temporary/combat_log_state.rs`: exclude Retail/PTR from legacy no-op Add and enablement-resetting Clear; preserve other profiles.
- `src/lua_api/workarounds/temporary/startup_service_capability_defaults.rs`: remove replaced secure-clear no-op.

## Tests asserting this spec

`tests/combat_log_event_filters.rs`, selector `combat_log_event_filters::original_processor_replaces_and_clears_filter_records`, compiles only for Retail/PTR. It loads the real dependency closure, checks secure function provenance and executes the original method without overrides.

### Static contradiction table (each listed selector stands alone)

| Selector | Required observations | Constant producer contradicted by | No-op producer contradicted by |
| --- | --- | --- | --- |
| `combat_log_event_filters::original_processor_replaces_and_clears_filter_records` | secure 2 distinct rows -> 1 different row -> 1 empty-event row -> 0; public 1 -> 0 with secure unchanged | one fixed snapshot cannot satisfy counts 2, 1, 0 and different event/GUID values; constant add/clear return values do not alter required state | add no-op fails first 2-row observation; clear no-op accumulates 3 rows instead of replacement 1, or leaves 1 after empty settings; public add/clear no-op fails 1 -> 0 |

case032/033 settings-identity checks are **preservation only**, not independent producer RED. No new runtime RED/GREEN was executed: edit-only authorization. Missing-member failure and a static contradiction are not executed semantic RED.

## Known gaps (current cycle)

- [ ] Execute exact selector RED on test-only commit, GREEN on producer commit, Retail and PTR separately; prove registration before execution.
- [ ] Run stock startup and compare exact message/count maps, not unique counts alone.

## Out of scope

Cached `CombatLogSecureDocumentation.lua` declares AddEventFilter and ClearEventFilters with `HasRestrictions = true`, `Environment = SecureOnly`, but **no Arguments, Returns or SecretArguments declarations**. Public `CombatLogDocumentation.lua` does not declare these members. Therefore argument validation/coercion/arity, return values, secret handling and restriction enforcement are **UNKNOWN**, not inferred from the lack of declarations. Accepting opaque arguments and returning no values is staging policy, not native parity. Consumer evidence supplies only nil/string event lists and nil/number/GUID source/destination selectors; no undocumented rejection rules are introduced.

Event matching (including event-list syntax, object-mask conjunctions, zero/empty semantics and filter combination), native history ingestion/traversal and public/secure native sharing are **UNKNOWN**. **INFERRED (simulator mapping):** Retail/Forever now use an explicit host-supplied Rust history model that preserves the previous fixture's traversal; it does not synthesize events or wire filter records into matching. See the [history contract](combat-log-history-state.md). Retail/PTR select this mainline processor for filter-record staging. Forever also selects `mainline`, but this filter-record stage still has no dedicated Forever consumer test. Mists/Era/Anniversary reject the mainline-only TOC despite cached files; Wrath rejects mainline and has no cached processor. No vendor changes or decompiled sources are used.
