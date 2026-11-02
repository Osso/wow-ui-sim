# Secret auras

Umbrella contract for Retail 12.x aura secrecy. Complete accessor behavior under restriction remains unknown; the bounded WM61 findings below distinguish field publication from caller access. This spec records:

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
- [ ] Model the supplied restriction inputs and transitions described in WM61 S1 below. Server effects and scheduling remain UNKNOWN.
- [ ] `UNIT_AURA` payloads are secret when auras are restricted (DOCS: `SecretWhenAurasRestricted`). `UNIT_AURA_BLOCKED` carries a secret aura ID (DOCS: `SecretValue`).

### Who sees secrets
- [ ] Preserve source and secret markers through secure invocation as described in WM61 S2 below. Complete callback access remains UNKNOWN.
- [x] Error messages report taint separately from secrecy: the four secret-value variants and the table guards follow the native templates ([taint error text](taint-error-text.md)).
- [ ] Event handlers run under the taint the client gives them, not the taint of whoever caused the state change. UNKNOWN (S9). This decides issue #13's `CompactArenaFrame.lua:318` "secret access requires an untainted caller" case.

### Granularity
- [ ] Aura secrecy can be queried by spell, by unit+index, by unit+auraInstanceID and by unit+slot (STRING).
- [ ] Model field-level AuraData publication as bounded by WM61 S3 below. Complete accessor-to-publication behavior remains UNKNOWN.
- [ ] The 22 Retail functions annotated `SecretWhenUnitAuraRestricted` return secrets only under restriction (DOCS).
- [ ] `NeverSecret` returns are never secret (DOCS, 934 Retail annotations).
- [ ] Model the WM61 S8 pre-lookup rejection and data-driven spell classification below; do not generalize the fragment result to every matched aura.

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

## WM61 evidence (Retail 12.1.5 build 70688)

WM citations below identify clean-room reports in `native-evidence-70688/secrets/`. Local findings do not establish the full registered API path or apply to other builds.

### Restriction inputs and transitions

- **VERIFIED — WM61 S1** (`S1-restriction-triggers-wm61.md`): Six indexed types have states Inactive 0, Activating 1 and Active 2. Natural activation or the corresponding forced CVar selects Active. The forced CVars are `addonCombatRestrictionsForced`, `addonEncounterRestrictionsForced`, `addonChallengeModeRestrictionsForced`, `addonPvPMatchRestrictionsForced`, `addonMapRestrictionsForced`, and `addonChatRestrictionsForced`.
- **VERIFIED — WM61 S1:** Combat uses a transition input; Encounter uses a status flag or positive counter; ChallengeMode uses a running marker; PvPMatch is active in Waiting, StartUp, Engaged or PostRound; Map needs nonzero classification without flag 512. Chat requires its enabled policy, a client flag, the Map metadata condition, and an Active Encounter, ChallengeMode or PvPMatch. Combat alone does not activate Chat.
- **VERIFIED — WM61 S1:** Activation stores 1, optionally emits `ADDON_RESTRICTION_STATE_CHANGED(type, 1)`, then stores 2; it emits no state-2 public event. Deactivation stores 0 before the optional event `(type, 0)`. No transition occurs for an unchanged target or an Activating reentry.
- **UNKNOWN — WM61 S1:** Server transitions, callback effects, event-hook availability, scheduling, complete map classifications and the meanings of the Chat client flag and map flag 512. Missing map metadata bypasses an update; it is not proven to clear the state.
- **SOURCE CONTRADICTION — WM61 S1:** The simulator's explicit `unit_auras_restricted` input is not the six-state transition model and does not produce these transitions.

### Secure invocation

- **VERIFIED locally — WM61 S2** (`S2-secure-invocation-wm61.md`): `securecallfunction` saves the caller source and restores it after normal invocation when its metadata scope requires restoration. Entry does not replace that source with an untainted source. Surviving results receive the saved caller provenance without clearing the secret marker.
- **EMULATED — WM61 S2:** Entry selects access policy 1, not unrestricted policy 2. Argument and result markers survive the tested transfers when the marker gate is enabled; the disabled gate clears copied markers.
- **INFERRED — WM61 S2:** Source provenance and secrecy are separate; a secure invocation is not blanket permission to untaint or unsecret values. **UNKNOWN — WM61 S2:** callback access, full return connection and exceptional restoration.
- **SOURCE CONTRADICTION — WM61 S2:** Rilua registers `securecallfunction` as `securecall`, which clears active ancestor frame taints for the call. This is not the observed source-preserving entry behavior.

### AuraData publication and spell lookup

- **VERIFIED locally — WM61 S3** (`S3-aura-field-granularity-wm61.md`): Publication policy 2 marks `auraInstanceID`, `spellId`, `icon`, `applications`, `duration`, `expirationTime`, `name`, `sourceUnit`, `dispelName`, `isHelpful`, and `isHarmful` individually; optional nil values remain unmarked. `points` is separately marked and its numeric elements use the field policy.
- **EMULATED — WM61 S3:** Under the stated publication seeds the outer table value is unmarked by default, but an explicit table-policy override can mark it. **UNKNOWN — WM61 S3:** complete accessor reachability, `issecrettable`, populated points and table security flags. Policy 0 does not mark; policy 1 depends on current-frame operation state; policy 2 applies without that condition.
- **SOURCE CONTRADICTION — WM61 S3:** `finish_aura_data` wraps the entire table and secure indexing returns plain fields, rather than applying the observed per-field publication policy.
- **VERIFIED locally — WM61 S8** (`S8-requires-nonsecret-aura-wm61.md`): When tainted execution, the secrecy feature and at least one of four Active restriction states select the restrictive context, the ID lookups classify the requested spell before aura lookup; the name lookup resolves the name first. SpellMisc `Attributes[15]` mask 33554432 selects AlwaysSecret before mask 67108864 selects NeverSecret; otherwise the enabled feature yields ContextuallySecret. Schema names are inferred.
- **EMULATED — WM61 S8 and S8a** (`S8a-unitauras-secret-emulated.md`): Selected rejection fragments return zero results, not one nil; separate outer fragments forward zero under their assumptions. **INFERRED — WM61 S8:** The relationship of these branches to `RequiresNonSecretAura`. **UNKNOWN — WM61 S8/S8a:** Complete registered-entry behavior, matched-aura outcomes and allowed-path publication.
- **SOURCE CONTRADICTION — WM61 S8:** The simulator returns one nil for its tainted restricted spell-keyed rejection. Its ambiguous-flags error also conflicts with the locally verified AlwaysSecret precedence when both flags are set.

### Text aspect

- **VERIFIED locally — WM61 S6** (`S6-aspects-wm61.md`): Text aspect is 8. The marked-argument `SetText` path supplies it to a helper that checks prevention and script-handler policy before adding it to the direct set. `HasSecretAspect` checks either of two sets; `HasAnySecretAspect` checks whether either is nonempty. Prevention and anchoring use separate flags.
- **EMULATED — WM61 S6:** With publication enabled, a Text aspect prepares forced-secret `GetText` return policy; a separate helper marks eligible strings under that policy. **UNKNOWN — WM61 S6:** the complete setter/getter path, second-set origin, propagation and clearing on plain reset, `SetToDefaults` or `ClearAllPoints`.
- **SOURCE CONTRADICTION — WM61 S6:** The simulator derives object aspect state from prevention/protection and uses prevention for `IsAnchoringSecret`; these do not match the distinct native calculations. Its text sink does not establish the WM61 Text-aspect behavior.

## Current state (SOURCE, 2026-10-31)

| Area | Today | Gap |
|---|---|---|
| Restriction trigger | `unit_auras_restricted` (Retail 12.1 feature) set only by tests | No production trigger; combat, encounters and M+ never restrict |
| AuraData | Whole table wrapped; tainted access denied; secure indexing returns plain fields | No per-field secrets; no `ConditionalSecret` contents |
| Spell-keyed lookups | Never-secret spells stay plain; others return one nil to tainted callers (inferred) | WM61 S8 rejection fragments return zero results, not one nil; complete entry behavior remains UNKNOWN |
| VM operations | Guarded truth/equality for wrapped booleans and nil, guarded ordering for numbers | Arithmetic, concat, `tostring`, `format` and table-key rules missing |
| Widget sinks | `SetText`, `SetShown`, geometry, texture and cooldown on gated paths | No general argument-mode or aspect model |
| Event taint | `A_Admin.FireEvent` clears caller taint during dispatch (4.3.4-based comment) | Retail rule UNKNOWN; other dispatch paths unchecked |
| Helpers | Bounded shallow checks; `canaccesstable` recursive | Several helpers missing or pass-through |

## How it works

- [Lua API: taint](../wiki/systems/lua-api.md)
- [Secret values](retail-secret-values.md), [classic policy](classic-secret-policy.md), [secret display](aura-secret-display.md)

## Related narrow specs

[unit-aura-slot-secret-arguments](unit-aura-slot-secret-arguments.md), [unit-aura-index-secret-arguments](unit-aura-index-secret-arguments.md), [spell-aura-secrecy](spell-aura-secrecy.md), [secret-selector-aura-duration](secret-selector-aura-duration.md), [secret-selector-aura-caster](secret-selector-aura-caster.md), [tooltip-aura-instance-security](tooltip-aura-instance-security.md), [script-object-secret-aspects](script-object-secret-aspects.md), [widget-secret-aspects-12-0-7](widget-secret-aspects-12-0-7.md), [secret-number-ordering](secret-number-ordering.md), [secret-string-formatting](secret-string-formatting.md), [closure-taint-lifetime](closure-taint-lifetime.md), [taint-error-text](taint-error-text.md).

## Implementation inventory

- `src/c_api/unit_aura_access.rs`: restriction flag and the aura access guard.
- `src/lua_api/globals/auras.rs`: AuraData builder and table wrapping.
- `src/c_api/c_unit_auras.rs`, `c_unit_aura_*.rs`, `aura_duration.rs`: aura queries and per-function guards.
- `src/lua_api/globals/security/{secret_values,value_access}.rs`: helpers and the wrap/unwrap bridge.
- `src/lua_api/frame/methods/{secret_origin.rs,misc/secret.rs}`: widget sinks and aspects.
- `src/lua_api/globals/admin.rs`, `admin_events.rs`: admin event dispatch taint.
- `rilua/src/{table_security.rs,stdlib/taint.rs}`: VM secret wrappers and taint.

## Tests asserting this spec

None yet at umbrella level. The narrow specs above list their own tests. Each bullet here gains a test that runs a shipped Blizzard consumer when its lane lands.

## Casting and threat predicate surface

The pinned Retail and PTR `Blizzard_APIDocumentationGenerated/SecretPredicateAPIDocumentation.lua` declares `ShouldUnitSpellCastingBeSecret` at lines 372–385 and `ShouldUnitThreatValuesBeSecret` at lines 415–429. Both return one boolean. Casting requires one unit token; threat values requires two. `ShouldUnitThreatStateBeSecret` declares its second token nilable at lines 398–412. These declarations specify required arguments, not exact error text or coercion behavior.

The simulator rejects missing, nil and non-string required tokens (best-guess validation, native UNKNOWN). Casting follows the existing public player-cast policy. Threat values follows the existing supplied `threat_state_restricted` policy. Both triggers are **best-guess, native UNKNOWN**. `CanCompareUnitTokens` uses the existing comparison-permission policy, not the inverse of comparison secrecy. Retail and PTR both enable the `retail-12-0-5` feature that gates the existing threat-state predicate.

Retail's `Blizzard_UIPanels_Game_Mainline.toc:6,10` selects `Shared/CastingBarFrame.lua`; its `IsInterruptable` method reaches the predicate via lines 2, 5 and 604–605. The consumer test observes interruptible and uninterruptible results, but a constant false predicate also permits both: it proves the missing-call boundary and preserves consumer behavior, not nonconstant predicate semantics.

## Native restriction surface (clean-room S10–S13, 2026-10-31)

Source: `native-evidence-70688/secrets/S13-restriction-surface-summary.md` (behavioural specs only; builds 12.1.5.70688, Forever 70707, 12.1.7 PTR).

- VERIFIED, all three builds: the restricted helper set is closed. It has 25 names: 24 at permission class 7 and `GetForbiddenObjectTable` at class 6. The set is `canaccessallvalues`, `canaccesssecrets`, `canaccesstable`, `canaccessvalue`, `hasanysecretvalues`, `issecrettable`, `issecretvalue`, `mapvalues`, `scrub`, `scrubsecretvalues`, `secretunwrap`, `secretwrap`, `securecopy`, `CreateFromMixins`, `CreateSecureDelegate`, `Mixin`, `RegisterEventCallback`, `RegisterUnitEventCallback`, `UnregisterEventCallback`, `UnregisterUnitEventCallback`, `dropsecretaccess`, `dumpobject`, `securecallmethod`, `settablesecurity` and `GetForbiddenObjectTable`. The simulator registers all 25. `securecopy` and `CreateSecureDelegate` are still temporary fallbacks; the secure-helpers lane replaces them. One implementation is correct for every pinned build.
- VERIFIED: `GetAddOnRestrictionState(type)` returns one byte of a per-type state array, and `IsAddOnRestrictionActive(type)` is `state[type] == 2`. See [restricted-actions-state](restricted-actions-state.md).
- VERIFIED structure, two builds: `canaccessvalue` returns its fallback `1` when the argument is missing or fails its check. INFERRED meaning: fail-open (access allowed). The simulator matches this: `canaccessvalue()` returns `true`. Do not apply one fail-closed default to the whole surface.
- VERIFIED, three builds: `CreateSecureDelegate`, `secretunwrap` and `settablesecurity` refuse while a native singleton is set. They report an error and return `nil`; they do not raise. `secretwrap` has no guard. The singleton and its writer are native UNKNOWN, so the simulator does not model this refusal.
- Earlier S3's type-name-only inference is superseded by the bounded WM61 S3 converter evidence below. No complete accessor path is established.

## Known gaps (current cycle)

- [ ] Casting and threat restriction triggers: **best-guess, native UNKNOWN**; the shared simulated threat flag is not evidence of native threat-values behavior.
- [ ] Native S1 §7 describes a generic indexed state-byte predicate path comparing the byte to 2. The mapping of restriction indices and any pairing of a particular predicate with a getter remain unverified; existing `GetSpellCastSecrecy` and `ShouldUnitSpellCastBeSecret` are computed independently.

- [ ] Complete accessor-to-AuraData publication and the remaining secret-operation behavior are UNKNOWN; see the WM61 boundaries below.
- [ ] Restricted-helper refusal guard (`CreateSecureDelegate`, `secretunwrap`, `settablesecurity` return `nil` while a native singleton is set): not modelled; the singleton is UNKNOWN.
- [ ] Production restriction inputs and transitions (WM61 S1).
- [ ] Per-field AuraData secrecy (WM61 S3).
- [ ] General operation rules for secrets in tainted code (after S4).
- [ ] Event-dispatch taint rule, which fixes the issue #13 `CompactArenaFrame` case (after S9).

## Out of scope

- Classic profiles beyond [classic-secret-policy](classic-secret-policy.md).
- Non-aura `SecretWhen*` families (unit identity, power, stats, casts, cooldowns), except where they share the same machinery.
- Patching Blizzard Lua to avoid secrets.
