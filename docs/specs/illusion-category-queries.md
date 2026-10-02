# Illusion category queries

Exact patch12.0.5 source occurrences352/353 govern `C_TransmogCollection.GetIllusions(category)` and its `AllowedWhenUntainted` argument gate. This slice adds only explicit host inputs under `src/c_api/c_transmog_collection/illusion_info.rs` and behavioral tests; the existing empty query remains untouched. See [Lua API architecture](../wiki/lua-api.md).

## What it must do

### Documented boundary

Cached retail `AddOns/Blizzard_APIDocumentationGenerated/TransmogItemsDocumentation.lua:478–492` declares nullable `category:TransmogCollectionType`, `SecretArguments = "AllowedWhenUntainted"`, and exactly one nonnullable array of `TransmogIllusionInfo`. Lines1220–1232 declare exactly six nonnullable fields: `visualID:number`, `sourceID:number`, `icon:fileID`, `isCollected:bool`, `isUsable:bool`, `isHideVisual:bool`. These declarations do not establish native filtering, ordering, error text, or category partition.

- [ ] Return exactly one fresh dense public array; each fresh row has exactly those six keys with public numeric/boolean values, never the host selector `category`.
- [ ] Authenticate arg1 through the existing VM `unwrap_secret` AllowedWhenUntainted gate before inspecting its type or model state; never clear caller taint or replace security callbacks.
- [ ] Actual VM secret NUMBER is accepted by a secure caller and denied by a tainted caller; ordinary public categories work while tainted.
- [ ] Actual secret BOOL/STRING/Frame/table payloads fail category validation when secure, but hit the same VM access denial as secret NUMBER when tainted, without leaking private payloads.
- [ ] Retain rooted authentic wrapper identity, allocation sequence and VM secrecy across queries, failures, GC and ordinary-public recovery.

### Explicit inferred policies (not native-verified)

- [ ] Inputs are an environment-local ordered `Vec<IllusionInfo>`, empty by default. No synthesized records, appearance-source derivation or production catalog.
- [ ] Nil/omitted category selects all supplied rows in source order; an ordinary finite integral u32 category selects exactly matching rows; unknown/unmatched categories return an empty array.
- [ ] Reject public BOOL/STRING/Frame/table, negative, fractional, nonfinite and out-of-u32 selectors instead of coercing/truncating them.
- [ ] Preserve every supplied flag without inferred hidden/collected/usability filtering. Numeric fixture values501–503/601–603/132261–132262 are test data only.
- [ ] Secret NIL follows nil/all when secure and VM denial when tainted. The existing VM wrapper constructor supports nil; native nil-secret policy remains inferred.
- [ ] Queries are read-only; result/row mutations do not affect host inputs or other results. Live host replacement/clear affects subsequent queries without modifying prior results; separate environments remain isolated.

Existing enum assertions use `Enum.TransmogCollectionType.OneHAxe == 13` and `OneHSword == 14`, not an invented illusion-category enum. Associating the fixture rows with those categories is test input, not native catalog evidence.

## How it works

- [Lua API architecture](../wiki/lua-api.md)
- [Frame/model data flow](../frame-data-flow.md)

## Implementation inventory

- `src/c_api/c_transmog_collection/illusion_info.rs`: public host-input row with category plus six documented fields; no query producer.
- `src/c_api/c_transmog_collection.rs`: epoch125 module/type export only.
- `src/lua_api/state/sim_state.rs`: epoch125 explicit ordered input field.
- `src/lua_api/state.rs`: epoch125 empty vector initialization.
- `src/lua_api/globals/missing_surface/transmog_collection.rs`: unchanged legacy query/registration; still empty.

## Tests asserting this spec

`tests/illusion_category_queries.rs` adds16 focused tests through existing `build.rs` top-level grouped integration auto-discovery; no new Cargo target. They cover default control, nil/omitted selection, category matching/misses, order/schema/flags, read-only inputs, freshness/mutation, live replacement/clear, isolation, strict invalid selectors, ordinary-public taint, authentic secure NUMBER, authentic NIL, secure invalid secret payloads, tainted gate ordering/recovery, and GC/metadata.

## Known gaps (current cycle)

- [ ] Parent must compile and observe actual behavioral RED before any producer/registration edits. No compilation, test, check, readability or acceptance commands ran in this inputs-only slice.
- [ ] Expected RED: unchanged stub ignores seeded inputs and returns empty, and accepts invalid/tainted secret selectors. The empty-default control is expected to pass; no fabricated RED for already-correct empty behavior. These are predictions, not observed results.
- [ ] Parent owns producer, fresh checks and acceptance. Added state field invalidates older compiled proof for the new combined source; row311 verifier463 active runtime and `c_spell` are untouched.
- [ ] Native category partition, nil/unknown behavior, ordering, filtering, error details, acquisition, result secrecy and full taint parity remain unknown.

Accounting remains210 pending/138 bounded/14 partial,64 capabilities; rows352/353 remain uncredited. No final-goal or native-parity claim.

## Out of scope

- Row357 pending transmog cost; row355 outfit metadata naming is separate.
- Query producers, registrations, catalogs, discovery/acquisition and native-client probes.
- Existing aura-duration work, spell/glyph runtime changes, operational/deployment work, push and broad gates.
