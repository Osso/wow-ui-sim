# Unit aura indexed getter secret arguments

Bounded Retail 12.0.5 fixture contract for `C_UnitAuras.GetAuraDataByIndex`, `GetBuffDataByIndex` and `GetDebuffDataByIndex`. Only six argument-delta rows are selected. Source facts come from [the committed delta ledger](../../data/patch-api/sources/12.0.5-api-changes.txt) and the complete retail cache declaration at `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua`. Existing model context: [Lua API system](../wiki/systems/lua-api.md). Fixtures/spec only; parent owns compiled RED, producer, GREEN and acceptance. No execution proof claimed.

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
- `src/lua_api/globals/auras.rs`: existing three indexed providers, blocked/polarity selection and DTO builder; read-only in this slice. Parent owns argument-boundary producer work.
- `docs/specs/unit-aura-index-secret-arguments.md`: bounded source facts, inferred representation contract and exclusions.

## Tests asserting this spec

`tests/next125aura.rs`: **12 unexecuted fixtures**. Grouped integration filter: `next125aura::`; requires `retail-12-0-5` enabled. No builds, tests, lint, checks, readability or acceptance gates run in this slice. Formatting is not compiled proof.

| Coverage | Fixture count | Proof level |
| --- | ---: | --- |
| Player/party full DTO, polarity and source flags; optional filter defaults; retained filter selection | 4 | Written only |
| Blocked compaction; valid index/unknown misses and exact nullable arity | 2 | Written only |
| Strict unit/filter/index representations before unknown lookup | 2 | Written only; policy inferred |
| Each/combined authentic secrets, tainted denial/recovery, GC-rooted roundtrip | 3 | Written only |
| Store/block/provider immutability and independent DTO mutation | 1 | Written only |

The secure/tainted matrices cover all three getters; general getter covers both polarity filters. VM secrets are host-created, rooted before global insertion and observed using real security helpers. No simulated secret marker or overridden query/vendor implementation.

## Known gaps (current cycle)

- [ ] Parent-owned compiled RED → argument-boundary producer → GREEN/controls and final gates. A fixture that passes existing behavior is a retained-selection control, not evidence of implemented secret authentication.
- [ ] `RequiresUnitAuraAccess` and `SecretWhenUnitAuraRestricted` lack grounded aura permission/restriction state in this scope. No fixture proves native authorization, conditional secret outputs, restricted DTO fields or arbitrary declassification.
- [ ] Strict signed-i32 index policy and invalid-input/missing-result/error behavior are inferred simulator contracts, not native-client-verified semantics.
- [ ] Native filter semantics and cached consumer closure remain unproved. Tests deliberately retain current indexed selection instead of promoting the instance-query PLAYER predicate into it.

## Out of scope

Production/helper/store/DTO changes in this fixture slice; instance-ID getter additions or changed lookup; full filter vocabulary/parser redesign; fabricated restriction/access state, toggles or output-declassification; native probes, vendor edits, all-profile parity; accounting, PLAN, wiki or other concurrent documentation updates; builds/tests/gates, delegation, operational changes, push or deployment. Only exact test/spec paths are staged; parent owns integration and source-row accounting.
