# Item dressability

Retail/PTR `C_Item.IsDressableItemByID(itemInfo)` projects an explicit host-supplied per-item boolean. Its C API home is `src/c_api/item_spell/c_item.rs`; the historical test/spec checkpoint `13a818ba5` added no producer. After reached supplied-data/original-consumer RED in both profiles, native predicate source now authenticates the original argument, uses existing numeric/item-link identity parsing and projects only the explicit boolean map. GREEN remains pending. Profile-scoped native declarations establish a required `ItemInfo`, `AllowedWhenUntainted`, and one non-nil boolean result. Empty defaults and supplied classification are user-approved simulator policy, not a native item catalog claim.

## What it must do

### API and security

- [ ] Expose the Retail/PTR public API and return exactly one boolean for numeric identities and supported item links, including the existing generated hyperlink consumed by PlayerChoice.
- [ ] Reject absent/nil arguments and values outside the supported ItemInfo input types. Shared numeric, link and generated-name resolution follows [ItemInfo conversion](item-info-conversion.md); other coercion and malformed-link details remain unknown.
- [ ] Authenticate original VM secrets before parsing or lookup: untainted callers may query wrapped supported inputs; tainted callers must be denied wrapped inputs, including malformed/nil payloads. Ordinary inputs remain usable by tainted callers. Preserve wrapper identity and caller taint through denial and subsequent secure recovery.

### Approved host input

- [ ] Each environment starts with empty explicit per-item boolean records; an absent record yields false. Generated metadata, equipment/consumable sets, ownership and mount relations do not seed or infer this classification.
- [ ] Supplied true and supplied false project directly through numeric identity and corresponding supported item links. Neither projection requires a generated catalog entry, appearance record, or 3D model.
- [ ] Host insertion, replacement and removal are visible on the next query; removal restores the false miss. Inputs remain isolated between environments and do not mutate mount relations or equipment/consumable classification.

### Real consumer

- [ ] Preserve the whole profile-scoped cached PlayerChoice/ItemButton/SharedXML closure, original XML reward template, original ItemButton SetItem/GetItemInfo path and original IsDressupReward identity/provenance. Supplied classification must drive the original consumer without replacing globals or vendor methods.
- [ ] With the mount relation absent, supplier-controlled false/true/false classification for the existing real item yields the corresponding original consumer contrast. This asserts the supplied policy, not native Hearthstone dressability.
- [ ] Preserve independent supplied mount-relation behavior: false dressability must not suppress a mapped mount; changing the mount relation must not change the direct dressability projection.

## How it works

- [Lua/C API integration](../lua-api.md).
- [Independent mount relation contract](mount-item-relations.md).

## Implementation inventory

- `src/c_api/item_spell/c_item.rs` — Retail/PTR-only native predicate publication, original-VM-secret authentication, required numeric/string input validation, existing identity parser and explicit map lookup with false miss.
- `src/c_api/item_spell/helpers.rs` — existing metadata projections; not authoritative dressability inputs.
- `src/lua_api/state/sim_state.rs` / `src/lua_api/state.rs` — uniform empty `item_dressability: HashMap<u32, bool>` storage; query publication is decided once at C_Item registration for Retail/PTR. Uniform storage avoids repeated profile cfg lists; it does not publish or seed an earlier-profile API.
- `tests/item_dressability.rs` — five authored API/input cases; initial two-case RED checkpoint below. Public lookup, not raw registration-table shape, is the contract.
- `tests/mount_item_player_choice.rs` — unchanged whole cached consumer helper plus authored supplied false/true/false and independent mount contrast; execution pending.

## Tests asserting this spec

Initial `13a818ba5` tests contained exactly two cases:

- `item_dressability_empty_input_returns_exactly_one_boolean_false` — numeric ID and supported bare/generated item-link defaults, type and arity.
- `item_dressability_requires_item_info_and_authenticates_vm_secrets` — required argument/type rejection, authentic wrapped numeric/link queries, malformed wrapped payloads, tainted denial and secure recovery.

Initial PTR API RED at `13a818ba5` executed two cases: 0 PASS / 2 FAIL, exit101, independently classified with successful environment initialization. First numeric miss passed exact-one-return arity then failed `empty supplied classification must return boolean false`; actual value/type was not printed. First no-argument call failed the combined rejection/string-message assertion `required ItemInfo must reject input`; successful return versus non-string error is not distinguished by output. Later selectors and all secret/taint assertions were unreached. Retail at source `c7ba479fe` subsequently executed the same 0 PASS / 2 FAIL, exit101 and first assertion messages; independently classified as reached API behavior RED after successful environment initialization, with the same first-failure limits and unreached security assertions. No passing contract checkboxes or modeled/native consumer coverage follow from initial RED.

The input-only successor authors three additional direct cases for supplied numeric/bare/generated-link booleans and uncatalogued identity, replacement/removal/isolation, and independent mount/equipment/consumable inputs. The security case now exercises supplied false and true. One added original PlayerChoice case preserves whole closure/template/method provenance and supplies false/true/false plus independent mount controls. Retail at `76e8b8309` now executed direct 0 PASS / 5 FAIL and unchanged whole cached PlayerChoice 0 PASS / 1 FAIL, each exit101. Direct supplied queries fail nonboolean results; the consumer preserves original methods/SetItem/link/provenance, accepts supplied-false baseline then returns false for supplied true. Independent Retail classification confirms five direct feature REDs plus the original-consumer feature RED. PTR at `733399c38` subsequently executed the same direct 0 PASS / 5 FAIL and original-consumer 0 PASS / 1 FAIL, each exit101; direct nonboolean/required-input and original false-versus-supplied-true boundaries retained. Independent PTR classification confirms five direct feature REDs plus the original-consumer feature RED; later secret/state/mount controls remain unreached. Native predicate source follows those reached REDs; GREEN remains pending.

[Initial selected packet](../../data/patch-api/evidence/bullet-dress-initial-api-red-2026-10-13/README.md) retains whole Retail/PTR initial API streams and independent classification; corrected bounded public-copy/privacy/link gate PASS at `a7439281a`, original invocation-ID privacy FAIL retained.

### Verified bounded runtime — 2026-10-13

Retail and PTR each pass five explicit-boolean/input/security/isolation cases and the original PlayerChoice dressability/mount contrast. These prove the supplied model, not native item classification.

The [whole host-record packet](../../data/patch-api/evidence/bullet-dress-host-records-2026-10-13/README.md) owns the mixed-revision matrix, preservation, complete diagnostics, startup and check/format evidence. Public-copy and retained-gate copy checks pass at their recorded scopes; native/render parity, earlier profiles and the parent handoff remain open.

## Known gaps (current cycle)

- [x] Independently classify initial Retail RED; both profiles executed before passive input changes. Classification does not establish passing API, supplied-record, security or consumer coverage.
- [x] Execute reached host-state RED on Retail/PTR before predicate source; later state/link/security assertions remain unreached at that checkpoint.
- [ ] Execute complete GREEN for explicit true/false, uncatalogued/link identity, replacement/removal/isolation and independent mount/equipment/consumable state.
- [x] Execute real-consumer supplied-true RED on both profiles through unchanged whole closure/methods/link path, before predicate source. No sliced/recreated/mocked consumer.
- [x] Independently classify PTR supplied-input and original-consumer RED; no GREEN or later mount-control proof.
- [ ] Execute original-consumer GREEN and later mount controls.
- [ ] Verify committed producer source, mapped-secret behavior, earlier-profile preservation and broader acceptance; integration remains main-owned.

## Out of scope

Native catalog/classification, ownership or transmog eligibility inference and 3D rendering: not authorized or established by declarations. No inference from Hearthstone metadata, inventory type, equipment sets or mount membership. Numeric-domain/coercion/name grammar and malformed-string outcomes are not specified by this bounded declaration evidence. Historical input-only successor added passive storage/tests; current predicate follows actual RED with no catalog or metadata alternative path. Native classification, 3D and broader acceptance remain separate gaps.
