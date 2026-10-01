# Unit aura instance filter query

Bounded Retail 12.0.5 contract for `C_UnitAuras.IsAuraFilteredOutByInstanceID`. Source facts: committed `data/patch-api/sources/12.0.5-api-changes.txt:397–399` and profile-cached `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:472–487`. Existing aura model/filter mechanics are described in [Lua API](../wiki/systems/lua-api.md). Fixtures/spec only at this checkpoint; parent owns producer after compiled RED.

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
- `src/lua_api/globals/auras.rs`: current predicate, unfiltered lookup, shared filter helpers and public blocked enumeration; unchanged by this inputs-only slice.
- `src/c_api/c_unit_auras.rs`: existing C aura surface; parent owns eventual producer/registration, not this slice.

## Tests asserting this spec

`tests/unit_aura_filter_query.rs` contains fourteen tests:

| Coverage | Tests | Proof at input checkpoint |
| --- | ---: | --- |
| Exact public boolean, player/party polarity and PLAYER source; recognized case/order; MAW/EXTERNAL_DEFENSIVE | 5 | Written, not compiled/run |
| Unknown units/IDs including negative i32; blocked stored instances | 2 | Written, not compiled/run |
| Required strings and finite integral i32; validation before unknown lookup | 2 | Written, not compiled/run |
| Secure host-secret access; tainted denial/public recovery; GC-root retention | 3 | Written, not compiled/run |
| Read-only state/provider/block-list behavior; environment isolation | 2 | Written, not compiled/run |

No RED/GREEN, check, lint, readability, startup or acceptance claim. Parent must compile and inspect actual RED before implementing the producer. All requirement checkboxes remain unverified.

## Known gaps (current cycle)

- [ ] Compiled RED and parent producer/registration, followed by parent-owned GREEN/controls and verification. Current input baseline is `79f2be1f9`; no source-accounting, PLAN or current330-verification edits.
- [ ] Native invalid-input/error wording, missing-instance, output secrecy, visibility and restriction semantics remain inferred/unproved.
- [ ] Complete native filter vocabulary/syntax remains unproved. Existing helper defaults empty/unknown filter strings to HELPFUL; this is retained implementation behavior, **not a supported API contract or general parity claim**. No empty/unknown filter fixture or whitelist/parser redesign.
- [ ] Actual cached consumer closure remains unproved: `Blizzard_AuraContainerUtil.lua:3–9` calls this query for non-private records, but this file also constructs comparator tables from real AuraUtil functions/container enums and exports secure delegates through its addon table. `Blizzard_AuraContainerShared.lua` additionally initializes formatters/curves; AuraUtil itself requires CVar/Enum/Event registry support. Complete unchanged loading with real support was not established in this no-execution slice. No extracted/fake callback, query override, invented support table, or private consumer claim.

## Out of scope

Producer/module/registration changes, shared filter changes, new backing state/maps, namespace/default changes for other APIs, vendor edits, other profiles, native probes, permission enforcement without a grounded model, complete AuraContainer/nameplate closure, audit-row promotion, PLAN/source-accounting changes, builds/tests/checks/lint/readability/gates/delegation/push/deployment. Only this test file and spec are changed; formatting and coherent commit precede parent RED.
