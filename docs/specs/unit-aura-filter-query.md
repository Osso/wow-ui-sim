# Unit aura instance filter query

Bounded Retail 12.0.5 contract for `C_UnitAuras.IsAuraFilteredOutByInstanceID`. Source facts: committed `data/patch-api/sources/12.0.5-api-changes.txt:397–399` and profile-cached `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:472–487`. Existing aura model/filter mechanics are described in [Lua API](../wiki/systems/lua-api.md). Bounded producer implemented after parent-owned corrected compiled RED; parent owns GREEN and integration acceptance.

## What it must do

### Source contract

- [ ] Accept required `(unit: string, auraInstanceID: number, filter: string)` and return exactly one nonnil public boolean in this bounded simulator contract. Cached declaration specifies one boolean; public output is an inferred simulator policy, not native output-secrecy proof.
- [ ] Honor `SecretArguments = AllowedWhenUntainted` for all three documented arguments through VM-authenticated access. Row398 literally `- arg1 NeverSecret` **removes** the unit marker; row399 changes `AllowedWhenTainted` to `AllowedWhenUntainted`. This declaration has no remaining argument `NeverSecret` marker; do not copy one from `GetUnitAuras`.
- [ ] Retain `RequiresUnitAuraAccess = true` as a source limitation: absent a grounded permission model, these fixtures cannot prove native authorization or a valid bypass.

### Inferred representation and existing model policies

- [ ] Validate/authenticate every required argument before lookup, even for an unknown unit. Require actual strings for unit/filter and a finite integral signed-i32 number for instance ID. Reject missing/nil/wrong representations, numeric strings, fractions, nonfinite values and out-of-range numbers; avoid current lossy casts. Error text is not native-verified.
- [ ] Unknown unit or instance returns `true` under existing simulator policy. Negative integers within i32 remain valid representations and return `true` when absent; no invented positive-ID validation.
- [ ] Resolve stored instances without public enumeration's blocked-ID exclusion. Blocked instances remain retrievable and obey the same polarity/source predicate as unblocked records.
- [ ] Preserve existing modeled HELPFUL/HARMFUL polarity and PLAYER combinations, case-insensitivity and recognized token order. PLAYER uses the existing `is_from_player_or_player_pet` field, including concretely seeded pet sources, not merely the literal `sourceUnit` token.
- [ ] Preserve existing MAW/EXTERNAL_DEFENSIVE match-none behavior for modeled records. Do not expand native vocabulary, negation or RAID rules in this slice.
- [ ] Read concrete helpful/harmful records from player and existing seeded party stores without fabricating production data, adding state fields/maps or changing shared helpers.
- [ ] Leave aura fields/order, block list and provider selection unchanged on successful/missing/error queries. Separate environments retain independent records.

### Secret inputs

- [ ] An untainted caller accepts authentic host-secret string unit, string filter and number ID independently and together, yielding one public boolean without declassifying the original inputs or clearing taint.
- [ ] A tainted caller is denied each secret argument, including secret ID/filter with public unknown unit and secret unknown-unit string. Error recovery with public arguments succeeds while caller taint and original secrecy remain intact.
- [ ] Rooted secret unit/filter strings retain identity and secrecy across GC, remain securely usable, deny tainted access and permit public recovery. Number ID secrecy is also retained in the fixture.

## How it works

- [Lua API system](../wiki/systems/lua-api.md).
- [Aura classification contract](aura-classification-flags.md).
- [Aura instance enumeration contract](unit-aura-instance-enumeration.md).

## Implementation inventory

- `tests/unit_aura_filter_query.rs`: retail-12-0-5-gated fixtures using actual host state and VM secrets; auto-discovered by the existing grouped integration harness. No Cargo/target changes.
- `src/c_api/c_unit_aura_filter_query.rs`: sole bounded producer; VM `unwrap_secret` authenticates each required argument, strict UTF-8 strings and finite integral signed-i32 ID validate before unfiltered lookup, then one public boolean is pushed.
- `src/c_api/mod.rs` and `src/lua_api/globals/register.rs`: module and registration gated by `retail-12-0-5`. Cargo's existing cumulative epoch makes `aura-instance-enumeration` helpers available; no profile/feature definitions changed.
- `src/lua_api/globals/auras.rs`: exposes the existing `aura_matches_filter_string` predicate without semantic changes; reuses already exposed `find_aura_by_instance_id`. Old `retail-12-1-0` registration and producer removed, leaving no alternate provider. Store/filter/DTO and blocked enumeration behavior unchanged.
- `src/c_api/c_unit_auras.rs`: existing enumeration surface unchanged.

## Tests asserting this spec

`tests/unit_aura_filter_query.rs` contains fourteen tests:

| Coverage | Tests | Corrected parent RED |
| --- | ---: | --- |
| Exact public boolean, player/party polarity and PLAYER source; recognized case/order; MAW/EXTERNAL_DEFENSIVE | 5 | PASS |
| Unknown units/IDs including negative i32; blocked stored instances | 2 | PASS |
| Required strings and finite integral i32; validation before unknown lookup | 2 | FAIL |
| Secure host-secret access; tainted denial/public recovery; GC-root retention | 3 | FAIL |
| Read-only state/provider/block-list behavior; environment isolation | 2 | Immutability FAIL (fraction rejection); isolation PASS |

Parent-reported corrected RED at `1e912a88b88e633dc5ca3bebca671260ae5c20e6`: `/tmp/patch-12.0.5-batch41-red-fixed-build-result.json` exit0 (206.149s), saved run JSON/log **14 selected, 8 PASS / 6 FAIL**, exit101 (3.362s). Original 7/7 RED included an invalid player DTO `sourceUnit == pet` assertion; parent corrected the fixture to actual host pet state plus `isFromPlayerOrPlayerPet`. Producer leaves unrelated DTO serialization unchanged; tests untouched in this implementation. Implementation-time handoff had no producer GREEN or gates. Parent subsequently built `a881d04bb596ae087aad12e75cc7d277d6dcde95` once: exit0, 360.590s. Selected GREEN: fourteen filter tests plus ten altered-form, twelve identifier, eighteen duration and seven aura-shape controls = **61 unique PASS**, all exit0; startup exit0 `[]`. Evidence: `/tmp/patch-12.0.5-batch41-green-build-result.json`, `-green-runs.json` and `-green-startup.json`. Independent335 Rust/security/readability verification and rows398/399 acceptance remain pending; requirement checkboxes await that gate.

## Known gaps (current cycle)

- [ ] Independent335 integration verification and exact rows398/399 acceptance. Parent GREEN/controls/startup established above; implementation formatted and committed before GREEN. No source-accounting promotion yet.
- [ ] `RequiresUnitAuraAccess` remains an unmodeled permission boundary; accepting untainted authentic secrets is not a native authorization/bypass claim.
- [ ] Native invalid-input/error wording, missing-instance, output secrecy, visibility and restriction semantics remain inferred/unproved.
- [ ] Complete native filter vocabulary/syntax remains unproved. Existing helper defaults empty/unknown filter strings to HELPFUL; this is retained implementation behavior, **not a supported API contract or general parity claim**. No empty/unknown filter fixture or whitelist/parser redesign.
- [ ] Actual cached consumer closure remains unproved: `Blizzard_AuraContainerUtil.lua:3–9` calls this query for non-private records, but this file also constructs comparator tables from real AuraUtil functions/container enums and exports secure delegates through its addon table. `Blizzard_AuraContainerShared.lua` additionally initializes formatters/curves; AuraUtil itself requires CVar/Enum/Event registry support. Complete unchanged loading with real support was not established in this no-execution slice. No extracted/fake callback, query override, invented support table, or private consumer claim.

## Out of scope

Shared filter semantics, store/DTO changes, new backing state/maps, namespace/default changes for other APIs, vendor edits, other profiles, native probes, permission enforcement without a grounded model, complete AuraContainer/nameplate closure, audit-row promotion, PLAN/source-accounting/concurrent-doc changes, builds/tests/checks/lint/readability/gates/delegation/push/deployment. Implementation changes only the bounded producer, registration/provider retirement, helper visibility and this spec/relevant wiki; formatting and coherent commit precede parent GREEN.
