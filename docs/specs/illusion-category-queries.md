# Illusion category queries

Exact patch12.0.5 source occurrences352/353 govern `C_TransmogCollection.GetIllusions(category)` and its `AllowedWhenUntainted` argument gate. The epoch125 query consumes explicit host inputs under `src/c_api/c_transmog_collection/illusion_info.rs`; prior epochs retain their legacy empty query. Independently accepted within the chosen bounded contract; global formatting remains failed on preserved unowned source. No fabricated production catalog or native parity claim. See [Lua API architecture](../lua-api.md).

## What it must do

### Documented boundary

Cached retail `AddOns/Blizzard_APIDocumentationGenerated/TransmogItemsDocumentation.lua:478–492` declares nullable `category:TransmogCollectionType`, `SecretArguments = "AllowedWhenUntainted"`, and exactly one nonnullable array of `TransmogIllusionInfo`. Lines1220–1232 declare exactly six nonnullable fields: `visualID:number`, `sourceID:number`, `icon:fileID`, `isCollected:bool`, `isUsable:bool`, `isHideVisual:bool`. These declarations do not establish native filtering, ordering, error text, or category partition.

- [x] Return exactly one fresh dense public array; each fresh row has exactly those six keys with public numeric/boolean values, never the host selector `category`.
- [x] Authenticate arg1 through the existing VM `unwrap_secret` AllowedWhenUntainted gate before inspecting its type or model state; never clear caller taint or replace security callbacks.
- [x] Actual VM secret NUMBER is accepted by a secure caller and denied by a tainted caller; ordinary public categories work while tainted.
- [x] Actual secret BOOL/STRING/Frame/table payloads fail category validation when secure, but hit the same VM access denial as secret NUMBER when tainted, without leaking private payloads.
- [x] Retain rooted authentic wrapper identity, allocation sequence and VM secrecy across queries, failures, GC and ordinary-public recovery.

### Explicit inferred policies (not native-verified)

- [x] Inputs are an environment-local ordered `Vec<IllusionInfo>`, empty by default. No synthesized records, appearance-source derivation or production catalog.
- [x] Nil/omitted category selects all supplied rows in source order; an ordinary finite integral u32 category selects exactly matching rows; unknown/unmatched categories return an empty array.
- [x] Reject public BOOL/STRING/Frame/table, negative, fractional, nonfinite and out-of-u32 selectors instead of coercing/truncating them.
- [x] Preserve every supplied flag without inferred hidden/collected/usability filtering. Numeric fixture values501–503/601–603/132261–132262 are test data only.
- [x] Secret NIL follows nil/all when secure and VM denial when tainted. The existing VM wrapper constructor supports nil; native nil-secret policy remains inferred.
- [x] Queries are read-only; result/row mutations do not affect host inputs or other results. Live host replacement/clear affects subsequent queries without modifying prior results; separate environments remain isolated.

Existing enum assertions use `Enum.TransmogCollectionType.OneHAxe == 13` and `OneHSword == 14`, not an invented illusion-category enum. Associating the fixture rows with those categories is test input, not native catalog evidence.

## How it works

- [Lua API architecture](../lua-api.md)
- [Frame/model data flow](../frame-data-flow.md)

## Implementation inventory

- `src/c_api/c_transmog_collection/illusion_info.rs`: public host-input row with category plus six documented fields.
- `src/c_api/c_transmog_collection/illusions.rs`: epoch125 authenticated selector and ordered fresh six-field query producer; selected-input snapshot released before VM allocation, stack-rooted array owns rows before key allocations.
- `src/c_api/c_transmog_collection.rs`: epoch125 module/type export and producer registration into the already-rooted namespace.
- `src/lua_api/state/sim_state.rs`: epoch125 explicit ordered input field.
- `src/lua_api/state.rs`: epoch125 empty vector initialization.
- `src/lua_api/globals/missing_surface/transmog_collection.rs`: legacy empty query/registration only under inverse epoch125; no duplicate active publication or fallback.

## Tests asserting this spec

`tests/illusion_category_queries.rs` adds16 focused tests through existing `build.rs` top-level grouped integration auto-discovery; no new Cargo target. They cover default control, nil/omitted selection, category matching/misses, order/schema/flags, read-only inputs, freshness/mutation, live replacement/clear, isolation, strict invalid selectors, ordinary-public taint, authentic secure NUMBER, authentic NIL, secure invalid secret payloads, tainted gate ordering/recovery, and GC/metadata.

## Independent bounded acceptance — 2026-10-02

Parent accepted [independent470 audit](/tmp/patch-12.0.5-illusion-category-independent-proof.md) and [471 follow-up](/tmp/patch-12.0.5-illusion-category-readability-followup.md). Three new-code readability findings accepted and resolved inbf9d6e74bf86d88456be985afb5dfda356ab3f0b: named pure category predicates, split type-safe dense-index and error assertions. Follow-up independently proves unchanged policy/security/GC/schema and resolves findings; fresh two-filefmt0/check0 (152.372544002s including build-lock wait). Initial eight-filefmt0/security/wiring audit remains source-scoped. Globalfmt still FAILED on preserved unowned source.

Follow-up compile0/230.89876293297857s and16 focusedPASS/4.612545692012645s refresh changed files.72 unchanged historical controls and startup0 `[]` reused through source-equivalence audit; **88 distinct accepted PASS names**, not104 tests or freshly rerun88. Earlier135.072315269s compile/88PASS and corrected2PASS14genuineFAIL remain historical; initialE0603 not behavioral proof. All proof dirty-combined, not clean revision; later cost scaffolding is outside this snapshot.

Only352/353 promote:206 pending/141 bounded/14 partial/1 metadata-only =362 ordered unique IDs;66 capabilities.360 unrelated rows/65 prior capabilities and source provenance retained; parent owns postcommit identity validation. Native/catalog/filtering/large-width/result secrecy/full-taint/profile/runtime gaps remain open. No broader illusion API ecosystem or whole-page/goal acceptance.

## Known gaps (current cycle)

- Parent-reported corrected compiled RED at `f83e073f93a8a55b9eba365afa7daea0dc4ac518`: 16 selected, 2 PASS, 14 genuine behavioral FAIL, exit101, execution3.596897289s; compilation exit0,171.680163353s. Initial E0603 was not behavioral RED; the corrected revision publicly exported `IllusionInfo`. Inputs originate at `b22604af0`. No commands were rerun for this producer slice.
- [ ] Saved parent GREEN atcd6eb9a2f148ffab413d0861ed1fac3392c07045: compile0/135.07231526903342s;16 focused+67 transmog/heirloom+5 outfit catalog =88 distinctPASS,three exits0; runtime12.8127168919891119s. Startup0 `[]`/5.0203285770257935s. `/tmp/patch-12.0.5-batch59-green-{build-result,runs,startup-run}.json` and full outputs bind revision and executable hashes. Independent470 fresh source/security/readability/ownedfmt/defaultcheck pending; requirement/source accounting unchanged. Inputs and all16 tests unchanged by producer; initial compiler error and corrected genuine RED retained separately.

All evidence is dirty-combined with preserved supplied unowned diffSHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`, not clean-revision proof. Runtime+startup17.8330454690149054s is bounded development evidence below final partition target; no padding/rerun, full-profile/native/full-page/goal acceptance claim. Prior known globalfmt failure on protected source remains.
- [ ] Native category partition, nil/unknown behavior, ordering, filtering, error details, acquisition, result secrecy and full taint parity remain unknown.

Parent-supplied current accounting remains208 pending/139 bounded/14 partial/1 metadata,362 IDs,65 capabilities; rows352/353 remain uncredited. No final-goal or native-parity claim.

## Out of scope

- Row357 pending transmog cost; row355 outfit metadata naming is separate.
- Catalogs, discovery/acquisition and native-client probes; only the bounded query producer and registration are included.
- Existing aura-duration work, spell/glyph runtime changes, operational/deployment work, push and broad gates.
