# Unit aura indexed getter secret arguments

Bounded Retail 12.0.5 fixture contract for `C_UnitAuras.GetAuraDataByIndex`, `GetBuffDataByIndex` and `GetDebuffDataByIndex`. Only six argument-delta rows are selected. Source facts come from [the committed delta ledger](../../data/patch-api/sources/12.0.5-api-changes.txt) and the complete retail cache declaration at `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua`. Existing model context: [Lua API system](../wiki/systems/lua-api.md). Bounded argument-boundary producer implemented after genuine parent RED; parent owns GREEN, regression, startup, independent acceptance and six-row accounting. Saved parent GREEN recorded below; independent gates and parent acceptance remain pending.

## What it must do

### Exact source contract

| Getter | Ledger rows / exact delta | Complete cached declaration lines |
| --- | --- | --- |
| `GetAuraDataByIndex` | 373: `- arg1 NeverSecret`; 374: `AllowedWhenTainted -> AllowedWhenUntainted` | 188–205 |
| `GetBuffDataByIndex` | 384: `- arg1 NeverSecret`; 385: `AllowedWhenTainted -> AllowedWhenUntainted` | 304–321 |
| `GetDebuffDataByIndex` | 389: `- arg1 NeverSecret`; 390: `AllowedWhenTainted -> AllowedWhenUntainted` | 338–355 |

Source IDs are `global api-C_UnitAuras-GetAuraDataByIndex-373` / `-374`, `global api-C_UnitAuras-GetBuffDataByIndex-384` / `-385`, and `global api-C_UnitAuras-GetDebuffDataByIndex-389` / `-390`. These remove the unit annotation, not add it. The complete declarations give all three getters required `unit: UnitTokenRestrictedForAddOns`, required `index: luaIndex`, optional/nilable `filter: AuraFilters`, one nullable `AuraData` return, `SecretArguments = "AllowedWhenUntainted"`, `RequiresUnitAuraAccess = true` and `SecretWhenUnitAuraRestricted = true`. None of these three argument lists retains a `NeverSecret` marker. The delta ledger alone does not describe the complete argument/return contract.

- [ ] Authenticate all three supplied argument positions, including optional filter, under the VM's untainted-only secret access policy. Accept real secret STRING unit/filter and NUMBER index for untainted callers; do not reject secret unit as NeverSecret.
- [ ] Omitted/nil filter remains valid. General getter defaults to existing helpful selection; buff/debuff wrappers retain their helpful/harmful polarity. Return exactly one table or nil, including misses.
- [ ] Keep native unit-aura permission and conditional output-secrecy annotations explicit as unmodeled boundaries; argument acceptance is not permission enforcement or output-declassification proof.

### Inferred representations and retained indexed lookup

- [ ] **Inferred:** require actual string unit, actual string-or-nil filter and finite integral signed-i32 numeric index; reject missing/nil required arguments, wrong types, numeric-string indices, fractions, nonfinite and out-of-range numbers. Validate before lookup, including public unknown unit. Exact invalid-input/native error wording is unproved.
- [ ] Preserve existing one-based selection from concrete helpful/harmful player records and existing seeded party buff/debuff stores. Valid nonpositive integer indices, past-end indices and unknown units return one nil rather than gaining invented positive-index restrictions.
- [ ] Preserve existing blocked-record exclusion before indexing; excluding first records compacts later indices independently in player/party helpful/harmful lists.
- [ ] Preserve existing DTO fields: identity, count aliases, timing, flags, nilability and independent empty points table. Player query `sourceUnit` currently serializes as `"player"` even when stored source is pet/party1; party query preserves stored source. Do not change DTO serialization to satisfy a source-predicate assertion.
- [ ] Preserve observed index selection with recognized HELPFUL/HARMFUL/PLAYER combinations and case/order controls. Existing index selection uses polarity only, unlike the instance filter query's PLAYER predicate: second non-player-source records remain selected with `|PLAYER`. Their DTO `isFromPlayerOrPlayerPet` flags remain false. Buff/debuff wrappers retain forced polarity even with the opposite optional filter. This is existing simulator behavior, not native filter-parity proof or authorization to redesign shared helpers.
- [ ] Leave every stored aura field/order, block-list identity/content and provider selection unchanged across success, missing result, malformed input, secret success and tainted denial. Returned DTO mutation must not change subsequent results or store data.

### Authentic secret inputs and caller state

- [ ] On both populated player and party stores, secure callers succeed with each secret argument separately and all combined, including secret unit/index with nil filter. Secure unknown-unit combined arguments return one nil. Original inputs remain secret and caller taint unchanged.
- [ ] Tainted callers reject each secret position and combinations for all three getters, including secret unknown-unit string and public unknown unit with secret index/filter. Public populated lookup and missing-result recovery succeed in the same tainted closure; caller taint remains unchanged, original secret inputs remain secret and inaccessible through `secretunwrap`.
- [ ] Global-rooted host-secret strings and number retain identity/secrecy across forced GC. Retained references support secure → tainted denial/public recovery → secure lookup without input declassification or taint reset.

## How it works

- [Lua API system](../wiki/systems/lua-api.md).
- [Aura instance filter query](unit-aura-filter-query.md): separate instance lookup/predicate contract, not an indexed selection requirement.
- [Aura classification flags](aura-classification-flags.md): existing DTO context.

## Implementation inventory

- `tests/next125aura.rs`: independent retail-12-0-5-gated behavioral fixtures; discovered automatically by existing `build.rs` grouped integration mechanism. No Cargo targets or harness edits.
- `src/c_api/c_unit_aura_index_queries.rs`: sole three indexed argument-boundary providers under `retail-12-0-5`; VM authentication and strict representations precede existing lookup.
- `src/c_api/mod.rs`: epoch-gated module declaration.
- `src/lua_api/globals/register.rs`: C API registrar follows existing aura namespace/state initialization.
- `src/lua_api/globals/auras.rs`: former providers/registration remain only before `retail-12-0-5`; existing `AuraFilter`, parser and indexed push helper exposed crate-wide without selection, DTO or store changes.
- `docs/specs/unit-aura-index-secret-arguments.md`: source facts, inferred contract and proof limits; [Lua API system](../wiki/systems/lua-api.md#retail-1205-indexed-aura-arguments) documents implementation.

## Tests asserting this spec

`tests/next125aura.rs`: **12 fixtures, unchanged by producer**. Grouped integration filter: `next125aura::`; requires `retail-12-0-5` enabled. Saved parent RED compiled and ran before producer. Saved parent GREEN covers unchanged fixtures and controls; independent gates and parent acceptance remain pending.

| Coverage | Fixture count | Saved parent RED / post-producer proof |
| --- | ---: | --- |
| Player/party full DTO, polarity and source flags; optional filter defaults; retained filter selection | 4 | 4 PASS retained controls / GREEN 4 PASS |
| Blocked compaction; valid index/unknown misses and exact nullable arity | 2 | 2 PASS retained controls / GREEN 2 PASS |
| Strict unit/filter/index representations before unknown lookup | 2 | 2 FAIL genuine boundary failures / GREEN 2 PASS; policy inferred |
| Each/combined authentic secrets, tainted denial/recovery, GC-rooted roundtrip | 3 | 3 FAIL genuine secret failures / GREEN 3 PASS |
| Store/block/provider immutability and independent DTO mutation | 1 | 1 FAIL at authentic secret input / GREEN 1 PASS |

The secure/tainted matrices cover all three getters; general getter covers both polarity filters. VM secrets are host-created, rooted before global insertion and observed using real security helpers. No simulated secret marker or overridden query/vendor implementation.

### Corrected batch42 parent RED — 2026-10-01

Saved at `414f87346c2c836a849dee2eaaff9ff0bdb62ad2`: `cargo test --test integration --no-run --message-format=json` exit **0**, **322.047s**; selected binary filter `next125aura::` exit **101**, **7.871s**, **12 selected = 6 PASS / 6 FAIL**. Integration executable SHA256 `a837b7f6877a43b811a79416509b3cfe0b4f8d24b57ee6d498645f02cc4560dd`. Artifacts: `/tmp/patch-12.0.5-batch42-red-build-result.json`, `/tmp/patch-12.0.5-batch42-red-run.json`, and corresponding `.stdout`/`.stderr` files.

Strict unit/filter/index fixtures fail on existing coercion/defaults. Secure, GC and immutability fixtures fail with `expected string, got userdata at argument 1` on authentic host-secret STRING. Tainted fixture fails at `GetBuffDataByIndex` because the old wrapper ignores supplied secret filter. These are callable-provider behavior failures, not missing registration or compile failures. Six passing fixtures establish retained modeled behavior only; they do not prove secret authentication. This supersedes the earlier written-only status.

### Reconciled batch42 parent GREEN — 2026-10-01

Producer `8e04eaa335b4df36b213c92842ef4242d81d2025`, unchanged fixtures `414f87346c2c836a849dee2eaaff9ff0bdb62ad2`: saved `cargo test --test integration --no-run --message-format=json` exit **0**, **238.27s**. `/tmp/patch-12.0.5-batch42-green-build-result.json` binds `target/debug/deps/integration-a11e89d240f9bd0c` to SHA256 `362a5bab54821d4499128f955517b66d1411bc45b60f237700ab39235b81022a`. Build transcripts: `/tmp/patch-12.0.5-batch42-green-build.jsonl` and `/tmp/patch-12.0.5-batch42-green-build.log`.

`/tmp/patch-12.0.5-batch42-green-runs.json` records the same revision/binary hash for **80 unique PASS**: `next125aura::` **12**, `unit_aura_filter_query::` **14**, `aura_table_shape::` **7**, `aura_api::` **29**, corrected `admin_buff_api::` **18**, each exit **0**. Raw artifacts: `/tmp/patch-12.0.5-batch42-green-run-{0,1,2,3,5}.{stdout,stderr}` respectively. Original `admin_buff::` (run4) selected **zero tests** and supplies no control proof; its exit0 is excluded.

`/tmp/patch-12.0.5-batch42-green-startup-run.json` records startup exit **0**, **5.638s**, stdout `[]`, at producer revision; wow-sim SHA256 `5c13e22ae410eb80bce3c75d229a42c9a93bb0cab792f43216662334c55d0554`. Raw artifacts: `/tmp/patch-12.0.5-batch42-green-startup.stdout` and `.stderr`. Startup is bounded error-scan evidence, not native access/output secrecy or consumer closure proof.

Saved parent runs only; no new executions by this docs audit. Independent Rust/security/readability gates and parent acceptance remain pending. Strict representations inferred; selection/DTO/store and earlier-profile providers retained. Exact373/374/384/385/389/390 remain uncredited; no accounting/PLAN change follows from GREEN alone.

## Known gaps (current cycle)

- [ ] Saved parent GREEN/controls and startup recorded above; independent acceptance and final Rust/security/readability gates pending. Prior RED is invalid as post-producer proof. Six source rows373/374/384/385/389/390 remain uncredited here; accounting/PLAN unchanged.
- [ ] `RequiresUnitAuraAccess` and `SecretWhenUnitAuraRestricted` lack grounded aura permission/restriction state in this scope. No fixture proves native authorization, conditional secret outputs, restricted DTO fields or arbitrary declassification.
- [ ] Strict signed-i32 index policy and invalid-input/missing-result/error behavior are inferred simulator contracts, not native-client-verified semantics.
- [ ] Native filter semantics and cached consumer closure remain unproved. Tests deliberately retain current indexed selection instead of promoting the instance-query PLAYER predicate into it.

## Out of scope

Instance-ID getter additions or changed lookup; store/DTO/predicate changes or PLAYER-index enhancement; full filter vocabulary/parser redesign; fabricated restriction/access state, toggles or output-declassification; native probes, vendor edits, all-profile parity; accounting, PLAN or unrelated documentation changes; producer-side builds/tests/gates, delegation, operational changes, push or deployment. Producer scope is three argument boundaries, narrow epoch ownership/registration, existing helper visibility and exact spec/relevant wiki. Parent owns integration and six-row accounting.
