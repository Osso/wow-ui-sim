# Secret auras

Umbrella contract for Retail 12.x aura secrecy. When aura data is restricted, the client hands addon (tainted) code secret values instead of plain aura data, while Blizzard's untainted UI keeps working. This spec is the single place that records:

- what the simulator must do;
- what it does today;
- what is still unknown.

Narrow, already-specified pieces are linked rather than restated. For how taint and secret values work, see [Lua API](../wiki/systems/lua-api.md) and [retail secret values](retail-secret-values.md).

Evidence levels used below:
- **DOCS**: shipped `Blizzard_APIDocumentationGenerated` annotations.
- **UI**: shipped Blizzard Lua consumers.
- **STRING**: a string that is present in the 12.1.5.70688 client image. Presence alone is not proof of behaviour.
- **NATIVE**: a clean-room behavioural spec traced from the client.
- **SOURCE**: simulator source as inspected on 2026-10-31.
- **UNKNOWN**: no evidence yet.

## What it must do

### Restriction trigger
- [ ] Aura restriction is driven by game state, not by a test-only flag. STRING: the client's help text names its guard states as combat, lockdown, active encounter, challenge mode / Mythic+, current map type and PvP match state, and frames the outcome as "restricted, or may return secrets". The exact inputs, and whether the state is per unit, per aura or global, are UNKNOWN (native question S1).
- [ ] `UNIT_AURA` payloads are secret when auras are restricted (DOCS: `SecretWhenAurasRestricted`). `UNIT_AURA_BLOCKED` carries a secret aura ID (DOCS: `SecretValue`).

### Who sees secrets
- [ ] Secrecy applies to tainted (addon) execution. Untainted Blizzard code keeps working on the same data. STRING: `DenyTaintedAccessWhenAurasAreSecret`; "Auras cannot be accessed when secret while tainted by '%s'". Exact rules UNKNOWN (S2).
- [ ] Error messages report taint separately from secrecy. STRING: one access-error family adds "(execution tainted by '%s')".
- [ ] Event handlers run under the taint the client gives them, not the taint of whoever caused the state change. UNKNOWN (S9). This decides issue #13's `CompactArenaFrame.lua:318` "secret access requires an untainted caller" case.

### Granularity
- [ ] Aura secrecy can be queried by spell, by unit+index, by unit+auraInstanceID and by unit+slot (STRING).
- [ ] Under restriction, `C_UnitAuras.GetAuraDataByIndex` / `GetAuraDataByAuraInstanceID` return AuraData with secret fields. `GetUnitAuras` declares `ConditionalSecretContents` (DOCS), and the client has a `ConditionalSecret<AuraData>` type (STRING). Which fields stay plain is UNKNOWN (S3).
- [ ] The 22 Retail functions annotated `SecretWhenUnitAuraRestricted` return secrets only under restriction (DOCS).
- [ ] `NeverSecret` returns are never secret (DOCS, 934 Retail annotations).
- [ ] The `RequiresNonSecretAura` lookups (`GetAuraDataBySpellName`, `GetPlayerAuraBySpellID`, `GetUnitAuraBySpellID`) define their result for a secret aura. Whether that is nil, an error or a secret is UNKNOWN (S8).

### Operations on secret values
- [ ] Tainted code gets defined results or errors for arithmetic, comparison, concatenation, `tostring`, `string.format`, `#`, table keys, table storage, truth tests and `type`. UNKNOWN (S4).
- [ ] Untainted code may operate on secrets where the client allows it. UNKNOWN (S5).
- [ ] Secret values render as `<secret>`, `<secret table>`, or `nil --[[ secret value ]]` where the client renders them (STRING). Tables cannot be indexed with secret keys where the client forbids it (STRING: "attempted to %s a table that cannot be indexed with secret keys").

### Widget sinks and aspects
- [ ] Widget methods apply their `SecretArguments` mode. STRING names three modes: never allowed, not allowed, and only allowed during untainted execution.
- [ ] `SecretArgumentsAddAspect` setters mark the widget aspect secret, and `SecretReturnsForAspect` getters then return secrets (DOCS). Aspect semantics UNKNOWN (S6).
- [ ] Shipped aura buttons can pass secret duration, text, icon, visibility and geometry to their widgets and render correctly (UI).

### Helpers
- [ ] `issecretvalue`, `issecrettable`, `hasanysecretvalues`, `canaccessvalue`, `canaccessallvalues`, `canaccesstable`, `canaccesssecrets`, `scrubsecretvalues`, `dropsecretaccess`, `secretwrap`, `secretunwrap` and the widget `HasSecretValues` / `HasSecretAspect` / `HasAnySecretAspect` / `IsPreventingSecretValues` / `IsAnchoringSecret` match the client's usage strings and return counts. Usage strings are STRING; semantics UNKNOWN (S7).

## Current state (SOURCE, 2026-10-31)

| Area | Today | Gap |
|---|---|---|
| Restriction trigger | `unit_auras_restricted` (Retail 12.1 feature) set only by tests | No production trigger; combat, encounters and M+ never restrict |
| AuraData | Whole table wrapped; tainted access denied; secure indexing returns plain fields | No per-field secrets; no `ConditionalSecret` contents |
| Spell-keyed lookups | Never-secret spells stay plain; others nil for tainted callers (inferred) | Native result UNKNOWN |
| VM operations | Guarded truth/equality for wrapped booleans and nil, guarded ordering for numbers | Arithmetic, concat, `tostring`, `format` and table-key rules missing |
| Widget sinks | `SetText`, `SetShown`, geometry, texture and cooldown on gated paths | No general argument-mode or aspect model |
| Event taint | `A_Admin.FireEvent` clears caller taint during dispatch (4.3.4-based comment) | Retail rule UNKNOWN; other dispatch paths unchecked |
| Helpers | Bounded shallow checks; `canaccesstable` recursive | Several helpers missing or pass-through |

## How it works

- [Lua API: taint](../wiki/systems/lua-api.md)
- [Secret values](retail-secret-values.md), [classic policy](classic-secret-policy.md), [secret display](aura-secret-display.md)

## Related narrow specs

[unit-aura-slot-secret-arguments](unit-aura-slot-secret-arguments.md), [unit-aura-index-secret-arguments](unit-aura-index-secret-arguments.md), [spell-aura-secrecy](spell-aura-secrecy.md), [secret-selector-aura-duration](secret-selector-aura-duration.md), [secret-selector-aura-caster](secret-selector-aura-caster.md), [tooltip-aura-instance-security](tooltip-aura-instance-security.md), [script-object-secret-aspects](script-object-secret-aspects.md), [widget-secret-aspects-12-0-7](widget-secret-aspects-12-0-7.md), [secret-number-ordering](secret-number-ordering.md), [secret-string-formatting](secret-string-formatting.md), [closure-taint-lifetime](closure-taint-lifetime.md).

## Implementation inventory

- `src/c_api/unit_aura_access.rs`: restriction flag and the aura access guard.
- `src/lua_api/globals/auras.rs`: AuraData builder and table wrapping.
- `src/c_api/c_unit_auras.rs`, `c_unit_aura_*.rs`, `aura_duration.rs`: aura queries and per-function guards.
- `src/lua_api/globals/security/{secret_values,value_access}.rs`: helpers and the wrap/unwrap bridge.
- `src/lua_api/frame/methods/{secret_origin.rs,misc/secret.rs}`: widget sinks and aspects.
- `src/lua_api/globals/admin.rs`, `admin_events.rs`: admin event dispatch taint.
- `/home/osso/Repos/rilua/src/{table_security.rs,stdlib/taint.rs}`: VM secret wrappers and taint.

## Tests asserting this spec

None yet at umbrella level. The narrow specs above list their own tests. Each bullet here gains a test that runs a shipped Blizzard consumer when its lane lands.

## Casting and threat predicate surface

The pinned Retail and PTR `Blizzard_APIDocumentationGenerated/SecretPredicateAPIDocumentation.lua` declares `ShouldUnitSpellCastingBeSecret` at lines 372–385 and `ShouldUnitThreatValuesBeSecret` at lines 415–429. Both return one boolean. Casting requires one unit token; threat values requires two. `ShouldUnitThreatStateBeSecret` declares its second token nilable at lines 398–412. These declarations specify required arguments, not exact error text or coercion behavior.

The simulator rejects missing, nil and non-string required tokens (best-guess validation, native UNKNOWN). Casting follows the existing public player-cast policy. Threat values follows the existing supplied `threat_state_restricted` policy. Both triggers are **best-guess, native UNKNOWN**. `CanCompareUnitTokens` uses the existing comparison-permission policy, not the inverse of comparison secrecy. Retail and PTR both enable the `retail-12-0-5` feature that gates the existing threat-state predicate.

Retail's `Blizzard_UIPanels_Game_Mainline.toc:6,10` selects `Shared/CastingBarFrame.lua`; its `IsInterruptable` method reaches the predicate via lines 2, 5 and 604–605. The consumer test observes interruptible and uninterruptible results, but a constant false predicate also permits both: it proves the missing-call boundary and preserves consumer behavior, not nonconstant predicate semantics.

## Known gaps (current cycle)

- [ ] Casting and threat restriction triggers: **best-guess, native UNKNOWN**; the shared simulated threat flag is not evidence of native threat-values behavior.
- [ ] Native S1 §7 describes a generic indexed state-byte predicate path comparing the byte to 2. The mapping of restriction indices and any pairing of a particular predicate with a getter remain unverified; existing `GetSpellCastSecrecy` and `ShouldUnitSpellCastBeSecret` are computed independently.

- [ ] Native behaviour questions S1–S9 (clean-room, 12.1.5.70688). The string inventory is done; behavioural traces are pending.
- [ ] Production restriction trigger (after S1).
- [ ] Per-field AuraData secrecy (after S3).
- [ ] General operation rules for secrets in tainted code (after S4).
- [ ] Event-dispatch taint rule, which fixes the issue #13 `CompactArenaFrame` case (after S9).

## Out of scope

- Classic profiles beyond [classic-secret-policy](classic-secret-policy.md).
- Non-aura `SecretWhen*` families (unit identity, power, stats, casts, cooldowns), except where they share the same machinery.
- Patching Blizzard Lua to avoid secrets.
