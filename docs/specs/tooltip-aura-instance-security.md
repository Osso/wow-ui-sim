# Tooltip aura-instance argument security

Retail12.0.5 exact source rows **339/340/344/345/349/350** cover three `C_TooltipInfo` namespace queries. This bounded producer specifies a chosen simulator input boundary and meaningful host-backed payload controls; saved pre-producer compiled RED is recorded below. Producer GREEN and acceptance remain pending. [Lua API architecture](../lua-api.md) describes the runtime boundary.

## What it must do

### Exact source scope

[`data/patch-api/sources/12.0.5-api-changes.txt`](../../data/patch-api/sources/12.0.5-api-changes.txt) supplies only these six changes:

| Rows | Exact namespace query | Change |
|---|---|---|
| 339 / 340 | `C_TooltipInfo.GetUnitAuraByAuraInstanceID` | Remove arg1 `NeverSecret`; `SecretArguments AllowedWhenTainted -> AllowedWhenUntainted` |
| 344 / 345 | `C_TooltipInfo.GetUnitBuffByAuraInstanceID` | Same two changes |
| 349 / 350 | `C_TooltipInfo.GetUnitDebuffByAuraInstanceID` | Same two changes |

Cached retail `AddOns/Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua` documents all three positions: required `unitToken:UnitTokenRestrictedForAddOns`, required `auraInstanceID:number`, nullable `filter:AuraFilters`. Its current blocks declare `SecretArguments="AllowedWhenUntainted"`, `MayReturnNothing`, `RequiresUnitAuraAccess`, `SecretWhenUnitAuraRestricted`, and a `TooltipData` return. These declarations do not establish native lookup, invalid-input errors, miss shape, or public result semantics. Aura's documented HELPFUL/HARMFUL filter description does not receive implementation credit here.

- [ ] Call each exact registered namespace function, independently, without fake registration or replacement security/query callbacks.
- [ ] Authenticate actual VM secret payloads at positions1–3 using `unwrap_secret`'s AllowedWhenUntainted boundary **before any argument type validation or model lookup**. Preserve ordinary-public calls from tainted callers; do not implement a blanket caller-taint ban.
- [ ] Plan production registration under epoch125 (`retail-12-0-5`); retain older handlers only for earlier profiles/epochs, not as an active fallback. Production changes and earlier-profile verification are pending, not delivered by this commit.

Only arg1 NeverSecret removal and the three argument-policy transitions may receive these six rows' credit. Secure secret arg2/arg3 tests exercise that policy across documented positions, **not new arg2/arg3 NeverSecret-removal credit**.

### Chosen security behavior (not native-verified)

- [ ] Secure authentic secret STRING `player` at arg1 returns the same meaningful public DTO as public `player`; tainted use is denied without declassification.
- [ ] Secure authentic secret NUMBER1/2 at arg2 selects the same concrete instance/class result as public1/2, including a meaningful match for every query; tainted use is denied.
- [ ] Secure authentic secret STRING and NIL at optional arg3 preserve the existing ignored-option payload. Tainted use is denied even though the provider would otherwise ignore it. Omitted/public nil and public filter strings remain ignored; **no filter semantics credit**.
- [ ] Secure authentic secret BOOL/table/actual Frame payloads at unit or ID positions fail type validation with nonempty nonleaking errors. Tainted secrets hit VM access denial before invalid public arguments, unsupported units, or missing instances can short-circuit the query.
- [ ] Errors identify the exact namespace API and do not expose private string payloads. VM denial is distinguished by the pinned rilua `requires an untainted caller` diagnostic, not claimed native wording. Public recovery succeeds after errors; caller taint stays unchanged and outer secure context is restored.
- [ ] Root authentic host wrappers throughout secure acceptance, tainted rejection and forced GC. After fresh host entry, native secret metadata, userdata identity/allocation sequence and rooted-list identity remain unchanged. Never compare authentic secret BOOL identities from tainted Lua.

### Inferred host lookup and preserved payload

The fixture uses the **existing** `SimState.player.buffs:Vec<AuraInfo>`, including both polarities via `is_helpful`; no new aura store or producer. Helpful fixture: Flash of Light, spell19750, icon135987, instance1, duration/expiration3600, helpful=true. Harmful fixture: Corruption, spell172, icon136118, instance2, duration/expiration3600, helpful=false, Magic dispel. Both are explicit test inputs, not production acquisition/catalog evidence.

| Query | Instance1 helpful | Instance2 harmful |
|---|---|---|
| Aura | Flash of Light payload | Corruption payload |
| Buff | Flash of Light payload | Empty UnitAura DTO |
| Debuff | Empty UnitAura DTO | Corruption payload |

This polarity selection and wrong-class empty policy are **inferred**, not native-verified. Debuff's current unconditional empty result is insufficient as a positive security fixture; the meaningful harmful lookup is required for this bounded contract.

- [ ] Preserve the existing meaningful helpful Aura/Buff content; Aura finds either polarity, Buff only helpful, Debuff only harmful using the explicit host instance IDs and `is_helpful` flag.
- [ ] Retain exact `player`-only lookup. Unsupported `target`, `party1`, and unknown tokens, and missing instance999, return fresh line-empty UnitAura DTOs under current inferred simulator miss policy. Do not expand unit coverage.
- [ ] Return exactly one public table with `type=Enum.TooltipDataType.UnitAura` and `lines`, without inventing ID/width fields. Matched records preserve three meaningful name/duration/description lines: SpellName title, SpellName `1 hr`, wrapped nonempty SpellDescription. Compare colors through public numeric RGBA, never color object/method identity.
- [ ] Preserve the shared builder's **hardcoded `1 hr` limitation**. Duration3600 fixtures deliberately avoid requiring a builder change; dynamic duration/native payload parity is not asserted. No parser, identifier-width, or tooltip-width policy change is requested.
- [ ] Queries are read-only over every fixture field. DTO/lines/line tables are fresh; mutating one result cannot affect another result or host inputs. Live host replacement/clear changes subsequent queries without changing old DTOs; separate environments remain isolated.

## How it works

- [Lua API boundary](../lua-api.md)
- [Frame/model data flow](../frame-data-flow.md)
- [Adjacent tooltip identifier contract](tooltip-spell-mount-identifiers.md)
- [Authentic-wrapper precedent](illusion-category-queries.md)

## Implementation inventory

- `src/c_api/c_tooltip_info_aura_instance.rs`: epoch125 first-class registration of the three queries into the existing rooted namespace. All documented arguments authenticate through pinned rilua `unwrap_secret` before parsing/model access. Errors identify API/position and retain the VM diagnostic without payloads. UTF-8 unit parsing uses `val_to_string`; IDs require finite exact integral i32 values. Arg3 remains ignored after authentication. Public tainted calls remain permitted; no NeverSecret guard or generic declassification.
- `src/c_api/mod.rs`: epoch125 module gate.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs`: one registration into existing `ensure_namespace`; minimal crate-visible bridge uses existing player-only instance lookup, inferred polarity selection and unchanged builder. Lookup clones/relinquishes the model borrow before VM allocation; returned Val is pushed immediately. Old three registration entries are inverse-gated.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs`: old three callbacks and lookup import inverse-gated for earlier epochs/profiles; no active fallback.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs`: existing instance lookup and shared aura payload builder with hardcoded `1 hr`; preserve payload limitations.
- `src/lua_api/game_data.rs`: existing `AuraInfo` fields; accessible through `lua_api::state::AuraInfo` for grouped integration fixtures.
- `tests/tooltip_aura_instance_security.rs`: unchanged grouped tests through existing integration autodiscovery; no fixture/test edits or new Cargo target.

Research inputs: `/tmp/patch-12.0.5-aura-tooltip-security-provider-map.md` and `/tmp/patch-12.0.5-aura-tooltip-input-payload-boundary.md`. They are read-only local source maps, not execution/native evidence. Protected unowned `src/c_api/aura_duration.rs` is excluded from inspection, edits, formatting, staging and reverts.

## Tests asserting this spec

`tests/tooltip_aura_instance_security.rs` defines **24 substantive tests**, eight independent tests per query (`aura`, `buff`, `debuff` modules):

| Test in each module | Behavioral scope | Proof level |
|---|---|---|
| `public_payload_classification_misses_and_ignored_filter` | Both concrete IDs/class selection; public secure/tainted calls; unsupported units/misses; ignored public option; fresh misses | Pre-producer RED: Aura PASS; Buff/Debuff FAIL |
| `secure_authentic_unit_string_returns_meaningful_payload` | Actual secret `player` against both helpful/harmful IDs, including each query's meaningful match | Pre-producer RED: three wrapper-conversion FAIL |
| `secure_authentic_numeric_instance_ids_select_concrete_rows` | Actual secret NUMBER1/2 and concrete public controls | Pre-producer RED: three wrapper-conversion FAIL |
| `secure_authentic_optional_string_and_nil_remain_ignored` | Actual secret optional STRING/NIL, plus combined secure secrets | Pre-producer RED: Aura/Buff combined-unit conversion FAIL; Debuff baseline payload FAIL |
| `tainted_secret_gate_precedes_lookup_and_public_type_errors` | Each documented position; actual VM denial before miss/public invalid inputs; wrong-type secret denial | Pre-producer RED: three missing-context FAIL; downstream pending |
| `secure_wrong_type_bool_table_frame_reject_with_context_and_recovery` | Actual secret BOOL/table/Frame type errors and ordinary public invalid controls; contextual errors/recovery | Pre-producer RED: three missing-context FAIL; downstream pending |
| `fresh_dtos_read_only_live_replacement_clear_and_environment_isolation` | DTO mutation, replacement/clear, old results and two-environment isolation | Pre-producer RED: Aura/Buff PASS; Debuff missing-payload FAIL |
| `rooted_secret_gc_failure_recovery_retains_payload_and_trust` | Authentic roots across GC, repeated failure/public recovery, secure meaningful payload after tainted failures | Pre-producer RED: three combined-unit conversion FAIL; downstream GC pending |

Fixture helpers register only pinned rilua's real table-security helpers and construct secrets using `wrap_host_secret_string`, `wrap_host_secret_number`, `wrap_host_secret_bool`, and `wrap_secret`. Frame wrapping verifies real host backing. Each query gets its own environment/test outcome; an Aura failure cannot hide Buff/Debuff results. Host assertions compare all aura fields and native wrapper metadata, not source structure.

## Saved pre-producer compiled RED — 2026-10-02

Main-owned artifacts cover revision `6864b23eeed557b29b27c69fc14e2b1371c40999`, dirty-combined provenance, not clean-revision proof. Compile: `cargo test --test integration --no-run --message-format=json`, exit0, **114.38749977899715s**; `/tmp/patch-12.0.5-batch61-red-build-result.json`, `/tmp/patch-12.0.5-batch61-red-build.stderr` and `/tmp/patch-12.0.5-batch61-red-build.stdout.jsonl`. Integration binary `target/debug/deps/integration-a11e89d240f9bd0c` SHA256 `7834eb8ebc9a8ab06499ed03dfc38a1454c7829e2af195c772ae17dc528afa5a`.

Run: `timeout 90 target/debug/deps/integration-a11e89d240f9bd0c tooltip_aura_instance_security:: --nocapture --test-threads=1`, exit101, **5.795002096099779s** external elapsed (harness5.27s). Artifacts: `/tmp/patch-12.0.5-batch61-red-run.json`, `/tmp/patch-12.0.5-batch61-red-run.stdout`, `/tmp/patch-12.0.5-batch61-red-run.stderr`. **24 tests:3PASS/21FAIL**. Read-only482 classification reconciled against saved stdout/stderr:

| Group | Actual earliest failure / proof limit |
|---|---|
| Twelve secure-acceptance/optional/GC tests | Eleven stop at wrapper conversion (`expected string/number, got userdata`); Debuff optional stops at missing baseline payload. Combined-unit failures hide later optional/GC assertions. Not twelve independently proven authentication or GC failures. |
| Six contextual-error tests | Missing exact namespace API context; not yet proof of VM gate behavior or downstream recovery. |
| Three provider tests | Buff public classification fails wrong-class empty expectation; Debuff public and live-state tests fail missing meaningful payload. |
| Three passes | Aura public classification, Aura live-state/freshness and Buff live-state/freshness controls only. |

Producer source is later than this RED evidence. No GREEN/check/startup/independent/native/profile/full-page proof, checked requirement, output-policy credit or source-accounting change follows. Batch60 acceptance remains independent and untouched.

## Known gaps (current cycle)

- [ ] Main-owned compiled RED is recorded below; no build/check/test command executed by this producer slice. No producer GREEN evidence yet.
- [ ] Epoch125 production gate, authenticated provider and inferred polarity selection are implemented but unverified. GREEN, scoped checks/readability, startup regression and independent acceptance remain pending.
- [ ] Source-row/capability accounting remains pending and untouched: user-supplied checkpoint **206 pending /141 bounded /14 partial /1 metadata-only;362 IDs;66 capabilities**. Six requested rows remain pending; no other accounting change or acceptance claim.
- [ ] Native error wording/permissions, output restriction/secrecy, acquisition and full native payload semantics remain unknown. All requirements stay unchecked until main-owned evidence supports them.

## Out of scope

- `RequiresUnitAuraAccess` enforcement and `SecretWhenUnitAuraRestricted` result secrecy/output restriction; neither receives credit from this input-policy slice.
- Native parity, all-profile coverage, Tooltip frame forwarding, aura acquisition/catalog completeness, real filter behavior and unit expansion.
- Production edits outside the bounded three-query producer/minimal bridge, additional Cargo targets, protected unowned aura-duration work, accounting/wiki updates, build/check/test commands by this producer slice, delegation, push, deployment or operations.
