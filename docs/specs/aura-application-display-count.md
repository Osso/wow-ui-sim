# Aura application display count

Bounded Retail 12.0.5 contract for `C_UnitAuras.GetAuraApplicationDisplayCount`, rows 367–369 only. Batch43 adds the sole epoch-gated C API producer after parent compiled RED. Saved parent GREEN is established for the committed producer plus preserved unowned dirty source; independent gates and parent acceptance remain pending. Existing model mechanics: [Lua API system](../wiki/systems/lua-api.md).

## What it must do

### Documented signature and bounded output

- [ ] Accept required `auraInstanceUnit: string` and `auraInstanceID: number`; omitted `minDisplayCount` defaults to 2. Explicit nil minimum rejects (`Nilable = false`); omitted/nil maximum is unbounded (`Nilable = true`).
- [ ] Return exactly one nonnil string: below minimum `""`, strictly above supplied maximum `"*"`, otherwise decimal applications count. Decimal formatting/public output and equality behavior are bounded **inferred** policies, not native formatting/output-secrecy proof.
- [ ] Read concrete `AuraInfo.applications` in existing player and party helpful/harmful stores. Preserve current blocked-inclusive instance lookup (`find_aura_by_instance_id`); blocked enumeration must not hide an instance from this count query.
- [ ] Apply minimum comparison before maximum, including maximum below minimum (**inferred ordering**). Accept actual finite fractional, negative and large thresholds; no invented integral/positive/i32 threshold cap (**inferred numeric validation**).
- [ ] Return `""` for unknown unit/missing instance (**inferred**). Accept signed-i32 ID endpoints/negative integers as representations, not necessarily existing records.

### Argument authentication and validation

- [ ] Validate all supplied arguments before lookup, including unknown units and mixed inputs. Require actual string unit and actual finite integral signed-i32 number ID; reject coercible strings, fractions, nonfinite and out-of-range IDs (**inferred representation/error policy**).
- [ ] Require actual finite numbers for supplied minimum/maximum; reject numeric strings, wrong types and nonfinite thresholds before record lookup (**inferred**). Error wording is unspecified.
- [ ] `AllowedWhenUntainted`: securely accept authentic host-secret STRING unit and NUMBER ID independently and together; deny them under tainted callers, including secret unknown unit and secret ID with public unknown unit. Authenticated underlying representations must still validate.
- [ ] `NeverSecret`: reject secret minimum and maximum each/mixed even for untainted callers, secret unit/ID combinations and absent records. Do not unwrap NeverSecret thresholds merely because caller is secure.
- [ ] Preserve original input secrecy and caller taint across success, denial and public recovery. Root real VM secrets across GC; retain identity through secure → tainted denial → public recovery → secure access.

### Read-only state

- [ ] Leave records, order, DTO counts, block list and alternate provider selection unchanged across success/miss/error. Mutating a returned AuraData count must not affect the typed count provider; display queries must not modify that DTO.
- [ ] Keep counts/missing records independent across two environments. Public outputs use ordinary typed simulator data only; no secret application payload or native permission bypass is modeled.

## How it works

- [Lua API system](../wiki/systems/lua-api.md).
- [Instance enumeration](unit-aura-instance-enumeration.md).
- [Instance filter query](unit-aura-filter-query.md) — existing blocked-inclusive lookup/security fixture conventions.

## Implementation inventory

- `tests/aura_application_display_count.rs`: fourteen retail-12-0-5-gated tests, grouped through existing integration harness; no Cargo/build target changes.
- `src/lua_api/game_data.rs`: existing `AuraInfo.applications: i32`; unchanged.
- `src/lua_api/globals/auras.rs`: existing instance lookup, DTO serialization, blocking and provider controls; unchanged.
- `src/c_api/c_unit_aura_display_count.rs`: sole `retail-12-0-5` count producer; actual VM `unwrap_secret` authenticates unit/ID before strict validation. VM `is_secret_value` rejects NeverSecret thresholds without declassification, even for secure callers. Supplied thresholds validate before blocked-inclusive lookup; ordinary typed `applications` produces one public string.
- `src/c_api/mod.rs` and `src/lua_api/globals/register.rs`: epoch-gated module and registration after aura namespace/state initialization, alongside indexed queries. No alternate/fallback provider or adjacent getter changes.

## Tests asserting this spec

| Capability | Fixture tests | Proof level |
| --- | ---: | --- |
| Counts 0/1/2/5/6, optional defaults, exact public string arity, player/party polarity | 2 | Saved parent GREEN; independent acceptance pending |
| Equality, fractions/negative/large thresholds, min-first ordering, unknown/missing IDs | 3 | Saved parent GREEN; inferred policies; independent acceptance pending |
| Required inputs, strict i32 ID and finite thresholds before miss | 3 | Saved parent GREEN; inferred policies; independent acceptance pending |
| Secure each/mixed host secrets, NeverSecret thresholds, tainted denial/recovery, rooted GC identity | 4 | Saved parent GREEN; actual VM secrets; independent acceptance pending |
| DTO/store/block/provider immutability and per-environment isolation | 2 | Saved parent GREEN; independent acceptance pending |

Saved parent GREEN covers the fourteen fixtures and controls/startup below. Independent Rust/security/readability gates and parent acceptance remain pending; checklist items remain unchecked, not accounting credit. This docs-only reconciliation ran no tests, builds, checks or independent gates.

### Saved parent batch43 RED — 2026-10-01

Unchanged fixture/spec input `f2743342d2f33a722ef1c068dab4c92c4f81fb27`; compiled revision `ad83ca7d0e4fa401234c2185c28a438ee53d4648`. Parent build exit0 in 464.307s; selected run exit101 in 3.614s, **14 genuine FAIL / 0 PASS**. Evidence: `/tmp/patch-12.0.5-batch43-red-build-result.json`, `-run.json`, `-run.stdout`, `-run.stderr` and full build stdout/stderr. Binary `integration-a11e89d240f9bd0c` SHA256 `23406119db24b56c5cbd54371235458189a6d5ae9fce484aca2ea3a983e94f58`.

Compilation included preserved unowned `src/c_api/aura_duration.rs` changes adding unrelated `DoesAuraHaveExpirationTime`; source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`, saved `/tmp/patch-12.0.5-batch43-red-build-source-diff.txt`. That file is not producer-owned and is excluded from formatting/staging/commit. No native or exact-row accounting credit follows RED.

### Reconciled batch43 parent GREEN — 2026-10-01

Producer `04cf872ce98af702758e0e704b540d9228e1700a`; unchanged fixture input `f2743342d2f33a722ef1c068dab4c92c4f81fb27`. **Proof covers committed producer PLUS preserved UNOWNED dirty `src/c_api/aura_duration.rs` adding `DoesAuraHaveExpirationTime`, not a clean revision.** Exact saved diff `/tmp/patch-12.0.5-batch43-green-build-source-diff.txt` SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`. That file remains excluded from touch/format/stage.

Saved `cargo test --test integration --no-run --message-format=json`: **exit0 / 336.675s**. Integration binary `integration-a11e89d240f9bd0c` SHA256 `27897cba2e7f173000609f6edb759b70f1bd62b833e105ef623905a975366362`. Six saved serial filtered runs, each under `timeout 90`, each exit0:

| Filter | Unique PASS | Elapsed seconds |
| --- | ---: | ---: |
| `aura_application_display_count::` | 14 | 12.963 |
| `next125aura::` | 12 | 10.440 |
| `unit_aura_filter_query::` | 14 | 11.265 |
| `aura_table_shape::` | 7 | 4.395 |
| `aura_api::` | 29 | 15.801 |
| `admin_buff_api::` | 18 | 8.330 |

**14 display + 80 controls = 94 unique PASS, zero failures.** Saved startup `timeout 90 target/debug/wow-sim --no-addons --no-saved-vars lua-errors`: **exit0 / 17.512s / stdout `[]`**; wow-sim SHA256 `835aa59e8da15620bb2badf122a5c26c25c6db60ba3780c9fd591c919224642e`.

Artifacts share `/tmp/patch-12.0.5-batch43-green-`: `build-result.json`, `build.jsonl` (full build stdout), `build.log` (build stderr), `runs.json`, `run-0.stdout`/`.stderr` through `run-5.stdout`/`.stderr` (full selected outputs), `startup-run.json`, `startup.stdout`, `startup.stderr`, and `build-source-diff.txt`. Metadata and saved selected stdout were reconciled without rerunning commands.

Security fixtures cover authentic host-secret unit/ID independently and mixed, NeverSecret thresholds, tainted denial before misses, recovery and rooted GC identity. Typed-state fixtures cover player/party counts, unchanged DTO/store/block/provider state, blocked-inclusive lookup and environment isolation. Strict representations, decimal/public output, missing-record and min-first policies remain inferred. Native access/valid-instance enforcement and restricted output secrecy remain excluded. Independent gates and parent acceptance remain pending; exact rows367–369 stay uncredited and accounting/PLAN unchanged.

### Sources and exact row boundary

Actual runtime declaration verified at `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:127–146`: `RequiresUnitAuraAccess`, `RequiresValidUnitAuraInstance`, `SecretWhenUnitAuraRestricted`, `SecretArguments = "AllowedWhenUntainted"`, required unit/ID, minimum default2/non-nil/NeverSecret, maximum nilable/NeverSecret and one nonnil string result. This supersedes the **wrong runtime citation** to `vendor/wow-ui-source-mists` in `/tmp/patch-12.0.5-display-count-provider-map.md`; that map is planning context only, not authoritative runtime provenance.

Retained [source plaintext](../../data/patch-api/sources/12.0.5-api-changes.txt):366–369, [register](../../data/patch-api/sources/12.0.5-register.json) and [coverage](../../data/patch-api/sources/12.0.5-page-coverage.json) establish exactly:

- `global api-C_UnitAuras-GetAuraApplicationDisplayCount-367`: `+ arg3 NeverSecret`.
- `global api-C_UnitAuras-GetAuraApplicationDisplayCount-368`: `+ arg4 NeverSecret`.
- `global api-C_UnitAuras-GetAuraApplicationDisplayCount-369`: `AllowedWhenTainted -> AllowedWhenUntainted`.

All three coverage rows remain `audit-pending`; source IDs, source/register/coverage contents and accounting unchanged. Row376 and other aura APIs are excluded.

## Known gaps (current cycle)

- [ ] Independent Rust/security/readability gates and parent acceptance pending. Saved parent GREEN below is dirty-source-bound simulator evidence, not clean-revision or native proof.
- [ ] Strict representation, missing-record response, min-first ordering, decimal/public-output and error policies are informed guesses, not native-verified semantics.

## Out of scope

Native `RequiresUnitAuraAccess`/`RequiresValidUnitAuraInstance` enforcement, conditional restricted output secrecy, actual secret application data, native permission/error/formatting parity, production data ingestion and cached consumer closure. No vendor, model, DTO, predicate, provider-control, Cargo/build.rs, accounting or other-profile changes; only the count producer, its registration and relevant spec/wiki inventory change. No ops/delegation/push/deployment. Unowned dirty `src/c_api/aura_duration.rs` (`DoesAuraHaveExpirationTime`) preserved; no ownership assumed.
