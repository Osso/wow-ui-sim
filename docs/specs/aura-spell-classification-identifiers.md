# Aura spell classification identifiers

Batch48 exact rows361/363 change argument1 of `C_UnitAuras.AuraIsBigDefensive` and `C_UnitAuras.AuraIsPrivate` from `number` to `SpellIdentifier` ([retained source](../../data/patch-api/sources/12.0.5-api-changes.txt), lines360–363). Cached Retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:54–82` declares a required identifier, `SecretArguments = "AllowedWhenTainted"`, and one required boolean per API. It does **not** provide a classification catalog, acquisition mechanism or secret policy. C API-owned inputs live in `src/c_api/c_unit_aura_classification.rs`; [Lua API architecture](../lua-api.md) describes the boundary.

## What it must do

All model, representation, miss and security policies below are **INFERRED simulator choices**, not native-verified semantics. The bounded producer follows saved parent compiled RED. Both getters and their registration are implemented; parent owns GREEN and independent verification. Every contract remains unchecked.

### Explicit classification input

- [ ] Own a public `AuraSpellClassification` record with independent `is_big_defensive: bool` and `is_private: bool` fields and `Clone`, `Copy`, `Default` (both false). Own public `AuraSpellClassifications { pub spells: HashMap<u32, AuraSpellClassification> }`, empty `Default`, under `retail-12-0-5` with a public per-environment `SimState.aura_spell_classifications` field.
- [ ] Use only explicitly declared resolved spell-ID records. Fixture101=(true,false),202=(false,true),303=(true,true),404=(false,false) covers both flags independently. No manufactured production catalog.
- [ ] Each API returns exactly one **public boolean**, corresponding to its independent flag; missing resolved records and unseeded public strings return false.
- [ ] Host replacement/removal and alias mutation take effect immediately. Queries leave records, aliases and caller input unchanged; environments isolate their inputs.
- [ ] Generic `AuraInfo` buffs, private aura instance state and cooldown associations neither supply nor change spell classification. Private-instance presence does not imply `is_private`.

### Identifier boundary

- [ ] Accept only actual public UTF-8 STRING or finite integral u32 NUMBER, including0/u32MAX. Reject missing/nil, bool, table, actual backing Frame table, function, thread, nonfinite/fractional/negative/out-of-range numbers and invalid UTF-8 **before alias lookup**, including values that could otherwise coerce to seeded aliases.
- [ ] Share existing `c_spell::read_spell_identifier_at` resolution semantics after validation: explicit aliases first, numeric identity otherwise; uppercase names normalize to lowercase. Strings, including numeric strings and full colored links, require seeded aliases.
- [ ] Numeric alias overrides and seeded numeric strings resolve the same key. Full colored fixture link containing101 explicitly resolves202; no automatic link parsing or implicit string catalog.
- [ ] Unknown controls remain false alongside true records; no classification inferred from generic aura, private-instance or cooldown state.

### Caller context and conservative security

- [ ] Public numeric/name/full-link calls work in secure and tainted contexts, preserve caller taint and produce public results.
- [ ] **INFERRED conservative policy:** reject actual VM secret STRING/NUMBER, including hits and misses, and actual host-secret wrapped wrong Frame values even in secure context. This does not implement native secret `AllowedWhenTainted` permissions.
- [ ] Root real host-secret values and original Frame across allocation/GC; preserve identity, secrecy and caller taint through rejection. Subsequent public calls recover in both secure and tainted contexts. No Lua marker secrets or API replacement.

## How it works

- [Lua API architecture](../lua-api.md)
- [Classification implementation](../wiki/systems/lua-api.md#retail-1205-aura-spell-classifications)
- [Frame fixture representation](../wiki/investigations/frame-surrogate-identity-slot.md)
- [Shared cooldown identifier contract](cooldown-aura-spell-identifiers.md)
- [12.0.5 audit context](../wiki/investigations/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/c_api/c_unit_aura_classification.rs`: unchanged independent bool record/empty explicit map; sole two getters read a copied resolved-ID record or default and push the corresponding public boolean.
- `src/lua_api/globals/register.rs`: `retail-12-0-5` registration after existing aura namespace/state initialization.
- `src/c_api/mod.rs`: unchanged public `retail-12-0-5` module declaration.
- `src/lua_api/state/sim_state.rs`: public feature-gated classification input next to cooldown associations.
- `src/lua_api/state.rs`: empty default initializer next to cooldown associations.
- `src/c_api/c_spell.rs`: `read_public_spell_identifier_at` shares strict public validation/conservative actual-VM secret rejection across both classification getters and the cooldown getter, then calls the unchanged alias-first `read_spell_identifier_at` resolver.
- `src/c_api/c_unit_aura_cooldown_spells.rs`: existing getter now calls the shared validator; public number/nil results and API-specific errors preserved.

## Tests asserting this spec

`tests/aura_spell_classification_identifiers.rs`: 16 grouped feature-gated integration fixtures under existing autodiscovery, no new Cargo target. Covers empty defaults, independent numeric flags, miss controls, uppercase names, full colored alias, no link parsing, immediate alias mutation, numeric override/string alias, replacement/removal, numeric endpoints, strict invalid representations, generic buff/cooldown independence, private-instance independence, read-only/environment isolation, public tainted calls and GC-rooted actual-secret rejection/recovery in both contexts. Fixtures remain unchanged from inputs `baf81dfec6704b80d5d17e6d42b2adac3c856bea`; saved parent compiled RED is recorded below. No GREEN or acceptance claim.

## Saved parent compiled RED and producer boundary — 2026-10-02

Input revision `baf81dfec6704b80d5d17e6d42b2adac3c856bea`. `/tmp/patch-12.0.5-batch48-red-build-result.json` records `cargo test --test integration --no-run --message-format=json`, exit0/273.2447727450635s. `/tmp/patch-12.0.5-batch48-red-run.json` records `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c aura_spell_classification_identifiers:: --test-threads=1`, **16 selected / 0 PASS / 16 genuine FAIL**, exit101/2.6088224769337103s. Full outputs: `/tmp/patch-12.0.5-batch48-red-build.{jsonl,log}` and `/tmp/patch-12.0.5-batch48-red-run.{stdout,stderr}`. Binary SHA256 `4bb4f34d30aee634b33b8c52c6e1d233ef27c99235087127ccc4993e19b1eb70`; saved dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`. Saved evidence includes preserved unowned dirty source, not clean-revision proof; that source was not inspected or modified here.

| Capability | Saved pre-producer RED | Implemented scope / proof limit |
|---|---|---|
| Independent flags, default/miss controls, numeric/name/link aliases, mutations, endpoints, read-only/isolation | Required boolean failures | Copied explicit record/default; exactly one corresponding public boolean; parent GREEN pending |
| Public secure/tainted calls | Required boolean failure | Shared strict boundary without caller-taint changes; parent GREEN pending |
| Invalid public representations and actual-secret GC/rooting/recovery | Argument rejection failures | Validation before aliases; actual VM secret rejection without unwrap/payload inspection; parent GREEN pending |
| Generic buff/cooldown and private-instance independence | Required boolean failures | Only classification map queried; private fixture reached classification assertion, no observed setup failure; fixtures unchanged |

The narrow shared validator is extracted from the existing cooldown getter because these three callbacks require the identical boundary. Other alias-resolver callers remain unchanged. No input record/map/default, parser, catalog, acquisition or state changes. Native `AllowedWhenTainted`, secret acquisition and result secrecy remain unknown; public results and conservative rejection are inferred simulator policies.

## Known gaps (current cycle)

- [ ] Parent-owned GREEN and independent verification; no audit accounting or acceptance changes here.
- [ ] Native `AllowedWhenTainted` secret permissions, result secrecy, classification catalog and acquisition remain unknown. Chosen conservative rejection and public results are informed models, not native evidence.

## Out of scope

- Native probe gate: approved full12.0.5 audit permits explicitly informed models.
- Production spell classifications, new catalogs/parsers, classification acquisition, deductions from generic buffs/private instances/cooldown associations.
- Tests, builds, checks, lint, readability, coverage, deployment, operations and delegation in this producer stage; parent owns GREEN and independent gates.
- Wiki acceptance/coverage, PLAN and proof-ledger changes; `src/c_api/aura_duration.rs` is protected and untouched.
