# Supplied exact-link item stats

**Producer stage: bounded Retail runtime GREEN; native acceptance unclaimed.**

`C_Item.GetItemStats(itemLink)` and `C_Item.GetItemStatDelta(itemLink1, itemLink2)` expose environment-local supplied numeric maps. Genuine behavioral RED: all nine item-stat tests failed against sealed executable revision `76cf33ef327bbd9305706d8dd3f375b609e824d4`. The retained private report is pre-producer proof, not validation of this implementation. Tests target `client-retail`/`client-ptr`; backing state and producer use their shared `retail-12-0-0` module floor. Registration follows existing C_Item setup; compiled Bounded Retail GREEN passes.

Cached Retail evidence recorded by the accepted draft: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:965–995` declares required `cstring` arguments, `MayReturnNothing = true`, `SecretArguments = "AllowedWhenUntainted"`, and `LuaValueVariant` named `statTable`. Numeric maps, fresh allocation, arithmetic, and missing-link rules below are accepted simulator assumptions, not native facts established by that declaration. No native/PTR parity proof.

## What it must do

### Supplied state and queries

- [x] Initialize each environment with empty supplied maps. Retail/PTR retain Rust population. Forever 70707 declares both functions in `ItemDocumentation.lua:986,1003`; `A_Admin.SetItemStats(link, stats)` accepts an exact string link and a table of string keys with finite numeric values, replacing that link's observations. See the [admin API](../admin-api/README.md).
- [x] Match complete strings exactly; distinct full links for one item ID remain distinct. Unknown strings, bare IDs/links, changed names or whitespace yield nil unless supplied independently.
- [x] Return exact string-keyed numeric values, including explicit zeros, in fresh tables; caller mutation cannot affect other results or later queries.
- [x] Compute first minus second over the union of supplied keys, treating missing keys as zero. Preserve positive/negative/fractional differences; omit exactly zero differences. Identical supplied links yield an empty table.
- [x] Return nil if either delta operand is absent, including two identical absent strings. Missing-result tests do not constrain zero-versus-one-nil return arity.

### Original-argument security and validation

- [x] Authenticate all original required arguments before parsing/type validation; an invalid plain first operand cannot mask a secret second operand's denial.
- [x] Untainted callers may unwrap genuine VM secrets, then apply ordinary required-string validation, lookup and arithmetic.
- [x] Tainted callers must reject secrets before lookup/type validation, including malformed/missing payloads; retain wrappers and caller taint, without payload disclosure. Permission errors identify `untainted caller`.
- [x] Missing/nil and non-string plain values, including numbers, must error explicitly with the required string contract. Tainted callers may query ordinary strings; no blanket addon denial.

## How it works

- [Lua API overview](../lua-api.md).
- [Runtime architecture](../wiki/systems/lua-api.md).

## Implementation inventory

- `src/c_api/item_stats.rs`: public `ItemStatsState { by_link: HashMap<String, HashMap<String, f64>> }`, empty default; `pub(crate) register(state)` ensures existing `C_Item` and installs exactly `GetItemStats` / `GetItemStatDelta` using Rust functions `c_item_get_item_stats` / `c_item_get_item_stat_delta`. Queries authenticate original required operands before strict string validation, clone supplied maps while borrowing state, then project independent results after releasing the borrow. Delta uses the key union and omits zero differences; result tables remain stack-rooted across key allocations.
- `src/c_api/mod.rs`: existing C_API model re-export and modern-floor module gate; unchanged in this producer stage.
- `src/c_api/item_spell/mod.rs`: registers both stat getters after existing C_Item setup under the shared modern feature floor.
- `src/lua_api/state/sim_state.rs`: public `item_stats` fixture field.
- `src/lua_api/state.rs`: independent empty initialization.
- `src/c_api/item_spell/c_item.rs`: existing C_Item registration boundary; unchanged in this stage.

## Tests asserting this spec

Current bounded proof: **9/9 PASS**, zero selected failures, fixture failures, skips or timeouts. [Integrated current proof](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#five-producer-and-receiver-source-checkpoint--2026-10-12) owns aggregate gates and retention status.

[tests/item_stats_supplied.rs](../../tests/item_stats_supplied.rs): nine existing behavioral tests, all observed failing against sealed pre-producer revision `76cf33ef`; current bounded Retail run passes all nine. Covers exact supplied maps; bidirectional union delta; misses in either position; zero omission/empty delta; fresh results; environment isolation; required-string errors; secure genuine-secret validation; tainted permission-before-validation and public-call recovery. Fixtures use same item ID with distinct full links and concrete fractional/zero stats.

## Known gaps (current cycle)

Genuine compiled behavioral RED is recorded in the report above: supplied-map/union/freshness assertions failed on nil output; plain invalid-type and tainted-secret assertions failed because calls did not error. Later assertions beyond each failure are not executed proof. Current bounded GREEN reaches these modeled requirements.

- [x] Current bounded Retail GREEN: 9/9 tests, exit0.
- [ ] Native catalog/arithmetic/error wording/return arity/output security and PTR equivalence remain unverified.

## Out of scope

Catalog/cache seeding, equipment/item-level/scaling inference, canonicalization/item-ID parsing, supplied output-table arguments, Admin commands, fallbacks, native probes and other profiles. Non-finite stat values, embedded-NUL handling and extra-argument policy are unspecified. No vendor/config/WTF edits.
