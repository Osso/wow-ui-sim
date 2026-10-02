# Aura spell classification identifiers

Batch48 exact rows361/363 change argument1 of `C_UnitAuras.AuraIsBigDefensive` and `C_UnitAuras.AuraIsPrivate` from `number` to `SpellIdentifier` ([retained source](../../data/patch-api/sources/12.0.5-api-changes.txt), lines360–363). Cached Retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:54–82` declares a required identifier, `SecretArguments = "AllowedWhenTainted"`, and one required boolean per API. It does **not** provide a classification catalog, acquisition mechanism or secret policy. C API-owned inputs live in `src/c_api/c_unit_aura_classification.rs`; [Lua API architecture](../lua-api.md) describes the boundary.

## What it must do

All model, representation, miss and security policies below are **INFERRED simulator choices**, not native-verified semantics. This stage supplies inputs, spec and fixtures only. Parent owns compiled RED; no producer, getter or Lua registration is added here. Every contract remains unchecked.

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
- [Frame fixture representation](../wiki/investigations/frame-surrogate-identity-slot.md)
- [Shared cooldown identifier contract](cooldown-aura-spell-identifiers.md)
- [12.0.5 audit context](../wiki/investigations/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/c_api/c_unit_aura_classification.rs`: public independent bool record and empty explicit map; inputs only.
- `src/c_api/mod.rs`: public `retail-12-0-5` module declaration; no registration.
- `src/lua_api/state/sim_state.rs`: public feature-gated classification input next to cooldown associations.
- `src/lua_api/state.rs`: empty default initializer next to cooldown associations.
- `src/c_api/c_spell.rs`: unchanged existing alias resolver defining shared semantics; not called by a new producer at this stage.

## Tests asserting this spec

`tests/aura_spell_classification_identifiers.rs`: 16 grouped feature-gated integration fixtures under existing autodiscovery, no new Cargo target. Covers empty defaults, independent numeric flags, miss controls, uppercase names, full colored alias, no link parsing, immediate alias mutation, numeric override/string alias, replacement/removal, numeric endpoints, strict invalid representations, generic buff/cooldown independence, private-instance independence, read-only/environment isolation, public tainted calls and GC-rooted actual-secret rejection/recovery in both contexts. Fixtures are uncompiled/unrun at this stage; no passing or RED claims.

## Known gaps (current cycle)

- [ ] Parent compiled RED against these inputs/fixtures.
- [ ] Implement producer/getter and registration only after parent compiled RED.
- [ ] Parent-owned GREEN and independent verification; no audit accounting or acceptance changes here.
- [ ] Native `AllowedWhenTainted` secret permissions, result secrecy, classification catalog and acquisition remain unknown. Chosen conservative rejection and public results are informed models, not native evidence.

## Out of scope

- Native probe gate: approved full12.0.5 audit permits explicitly informed models.
- Production spell classifications, new catalogs/parsers, classification acquisition, deductions from generic buffs/private instances/cooldown associations.
- Producer/getter/Lua registration before parent compiled RED; tests, builds, checks, lint, readability, coverage, deployment, operations and delegation in this stage.
- Wiki acceptance/coverage, PLAN and proof-ledger changes; `src/c_api/aura_duration.rs` is protected and untouched.
