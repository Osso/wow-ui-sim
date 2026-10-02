# Aura spell classification identifiers

Batch48 exact rows361/363 change argument1 of `C_UnitAuras.AuraIsBigDefensive` and `C_UnitAuras.AuraIsPrivate` from `number` to `SpellIdentifier` ([retained source](../../data/patch-api/sources/12.0.5-api-changes.txt), lines360–363). Cached Retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:54–82` declares a required identifier, `SecretArguments = "AllowedWhenTainted"`, and one required boolean per API. It does **not** provide a classification catalog, acquisition mechanism or secret policy. C API-owned inputs live in `src/c_api/c_unit_aura_classification.rs`; [Lua API architecture](../lua-api.md) describes the boundary.

## What it must do

All model, representation, miss and security policies below are **INFERRED simulator choices**, not native-verified semantics. The bounded producer follows saved parent compiled RED. Both getters and their registration are implemented; saved parent GREEN is reconciled below. Independent381 gate remains pending. Every contract remains unchecked.

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

`tests/aura_spell_classification_identifiers.rs`: 16 grouped feature-gated integration fixtures under existing autodiscovery, no new Cargo target. Covers empty defaults, independent numeric flags, miss controls, uppercase names, full colored alias, no link parsing, immediate alias mutation, numeric override/string alias, replacement/removal, numeric endpoints, strict invalid representations, generic buff/cooldown independence, private-instance independence, read-only/environment isolation, public tainted calls and GC-rooted actual-secret rejection/recovery in both contexts. Fixtures remain unchanged from inputs `baf81dfec6704b80d5d17e6d42b2adac3c856bea`; saved parent compiled RED is recorded below. Saved parent GREEN below is development evidence, not independent acceptance.

## Saved parent compiled RED and producer boundary — 2026-10-02

Input revision `baf81dfec6704b80d5d17e6d42b2adac3c856bea`. `/tmp/patch-12.0.5-batch48-red-build-result.json` records `cargo test --test integration --no-run --message-format=json`, exit0/273.2447727450635s. `/tmp/patch-12.0.5-batch48-red-run.json` records `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c aura_spell_classification_identifiers:: --test-threads=1`, **16 selected / 0 PASS / 16 genuine FAIL**, exit101/2.6088224769337103s. Full outputs: `/tmp/patch-12.0.5-batch48-red-build.{jsonl,log}` and `/tmp/patch-12.0.5-batch48-red-run.{stdout,stderr}`. Binary SHA256 `4bb4f34d30aee634b33b8c52c6e1d233ef27c99235087127ccc4993e19b1eb70`; saved dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`. Saved evidence includes preserved unowned dirty source, not clean-revision proof; that source was not inspected or modified here.

| Capability | Saved pre-producer RED | Implemented scope / proof limit |
|---|---|---|
| Independent flags, default/miss controls, numeric/name/link aliases, mutations, endpoints, read-only/isolation | Required boolean failures | Copied explicit record/default; exactly one corresponding public boolean; parent GREEN pending |
| Public secure/tainted calls | Required boolean failure | Shared strict boundary without caller-taint changes; parent GREEN pending |
| Invalid public representations and actual-secret GC/rooting/recovery | Argument rejection failures | Validation before aliases; actual VM secret rejection without unwrap/payload inspection; parent GREEN pending |
| Generic buff/cooldown and private-instance independence | Required boolean failures | Only classification map queried; private fixture reached classification assertion, no observed setup failure; fixtures unchanged |

The narrow shared validator is extracted from the existing cooldown getter because these three callbacks require the identical boundary. Other alias-resolver callers remain unchanged. No input record/map/default, parser, catalog, acquisition or state changes. Native `AllowedWhenTainted`, secret acquisition and result secrecy remain unknown; public results and conservative rejection are inferred simulator policies.

## Reconciled batch48 parent GREEN — 2026-10-02

Saved evidence binds unchanged 16 fixtures at input `baf81dfec6704b80d5d17e6d42b2adac3c856bea` and producer `11eca0c6d78fc148ad3f679ab43ea3df3145a17f`, **plus preserved unowned dirty source**, not clean-revision proof. Saved dirty diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`; source/diff contents were not inspected or modified here. Historical compiled RED exit0/273.2447727450635s and runtime exit101/2.6088224769337103s, 0PASS/16 genuineFAIL above are retained.

`/tmp/patch-12.0.5-batch48-green-build-result.json` records `cargo test --test integration --no-run --message-format=json`: exit0/**289.3887384700356s**. Full outputs `/tmp/patch-12.0.5-batch48-green-build.{jsonl,log}` record successful compilation. Integration executable `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c`, SHA256 `02e3962f21df0ba3ece48cdef109cec525420203e64215e4bbc70ba27d4d6c14`.

`/tmp/patch-12.0.5-batch48-green-runs.json` records all16 commands as `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c <filter> --test-threads=1`. Corresponding full outputs `/tmp/patch-12.0.5-batch48-green-run-{0..15}.{stdout,stderr}` follow table order; all exits0, all stderr empty. Saved full test names establish **16 new +210 controls +18 private anchors =244 unique PASS** with no duplicated names. Private-anchor results are controls only, not row359 or private-anchor documentation acceptance.

| Exact filter | PASS | Saved seconds |
|---|---:|---:|
| `aura_spell_classification_identifiers::` | 16 | 3.9198297100374475 |
| `cooldown_aura_spell_identifiers::` | 14 | 3.5792808020487428 |
| `aura_dispel_color_arguments::` | 16 | 8.03681391198188 |
| `unit_aura_slot_enumeration_arguments::` | 12 | 3.3598275440745056 |
| `c_unit_auras_admin::` | 14 | 3.1972014729399234 |
| `userdata_proxy::color_curve_` | 18 | 3.9796690000221133 |
| `unit_aura_slot_secret_arguments::` | 12 | 3.1126869439613074 |
| `aura_application_display_count::` | 14 | 3.2329896120354533 |
| `next125aura::` | 12 | 2.900904139969498 |
| `unit_aura_filter_query::` | 14 | 3.9504475500434637 |
| `aura_table_shape::` | 7 | 1.9443296330282465 |
| `aura_api::` | 29 | 7.60804703400936 |
| `admin_buff_api::` | 18 | 4.789165845955722 |
| `aura_refresh_duration::` | 18 | 5.274159140069969 |
| `aura_spell_identifier::` | 12 | 3.225799681036733 |
| `private_aura_anchors::` | 18 | 9.270167073933408 |

Exact decimal sum of saved runtime records: **71.3813190951477735s** (binary-float sum `71.38131909514777s`), without padding or repeated executions. Startup adds9.945783533039503s for **81.3271026281872765s** total runtime, excluding compilation. This is bounded development proof, not final whole-page/goal/native acceptance.

`/tmp/patch-12.0.5-batch48-green-startup-run.json` records `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-sim --no-addons --no-saved-vars lua-errors`: exit0/9.945783533039503s, stdout `[]`. Full `/tmp/patch-12.0.5-batch48-green-startup.{stdout,stderr}` reports zero unique/occurrence Lua errors. Simulator SHA256 `a74414bd5953a51d2e2e0747801eb30f2794fc4710f53ca9cc87cd3852b5d220`.

| Capability (all policies INFERRED) | Current saved proof | Limit |
|---|---|---|
| Independent defensive/private flags; explicit empty map; false misses | Numeric independent records, empty-default and unknown controls PASS | No production catalog or acquisition evidence |
| Strict identifiers, inclusive u32 endpoints, explicit name/numeric/full-link aliases | Invalid-before-alias, endpoints, uppercase, numeric override/string seed, link override/no parsing PASS | Representation/resolution policy, not native parity |
| Immediate alias/map mutation; read-only queries; environment isolation | Mutation/removal and read-only/isolation PASS | Explicit host inputs only |
| Public secure/tainted queries; public results | Public-query and secret recovery fixtures PASS | Native `AllowedWhenTainted` and result secrecy unknown |
| Authentic secret/wrong Frame wrapper rejection; GC-rooted identity, secrecy, taint and recovery | Actual-secret fixture PASS in both contexts | Conservative denial, not native secret permissions |
| Generic buff, cooldown and private-instance noninterference | Both independence fixtures PASS; refreshed cooldown14 and private-anchor18 controls PASS | No inference from instance state; no new acceptance for controls |

Independent381 verification remains pending; source361/363 and359 remain pending. No requirements checked, coverage promoted, audit accounting changed, or private-anchor docs accepted. Parent owns separate accounting and independent gate. Earlier GREEN-pending entries are historical checkpoints, not current status.

## Known gaps (current cycle)

- [ ] Independent381 gate and parent acceptance remain pending; source rows361/363 and359 remain pending. No audit accounting, coverage or acceptance changes here.
- [ ] Native `AllowedWhenTainted` secret permissions, result secrecy, classification catalog and acquisition remain unknown. Chosen conservative rejection and public results are informed models, not native evidence.

## Out of scope

- Native probe gate: approved full12.0.5 audit permits explicitly informed models.
- Production spell classifications, new catalogs/parsers, classification acquisition, deductions from generic buffs/private instances/cooldown associations.
- Tests, builds, checks, lint, readability, coverage, deployment, operations and delegation in this producer stage; parent owns GREEN and independent gates.
- Wiki acceptance/coverage, PLAN and proof-ledger changes; `src/c_api/aura_duration.rs` is protected and untouched.
