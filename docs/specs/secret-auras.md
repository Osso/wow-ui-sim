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
- [ ] `UNIT_AURA` payloads are secret when auras are restricted (DOCS: `SecretWhenUnitAuraRestricted`; the earlier spelling `SecretWhenAurasRestricted` is document drift, see S0). `UNIT_AURA_BLOCKED` carries a secret aura ID (DOCS: `SecretValue`).

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
- [ ] Apply `SecretArgumentsAddAspect` and `SecretReturnsForAspect` according to the bounded WM61 S6 findings below; complete getter publication remains UNKNOWN.
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
- **SOURCE — WM61 S1:** `restricted_actions.rs` supplies six indexed states, activation state 1 and deactivation state 0, and natural-or-forced CVar transitions. WM105 supersedes the former separate aura flag: the aura gate now reads entries 0..3 of this array.

### Secure invocation

- **VERIFIED locally — WM61 S2** (`S2-secure-invocation-wm61.md`): `securecallfunction` saves the caller source and restores it after normal invocation when its metadata scope requires restoration. Entry does not replace that source with an untainted source. Surviving results receive the saved caller provenance without clearing the secret marker.
- **EMULATED — WM61 S2:** Entry selects access policy 1, not unrestricted policy 2. Argument and result markers survive the tested transfers when the marker gate is enabled; the disabled gate clears copied markers.
- **INFERRED — WM61 S2:** Source provenance and secrecy are separate; a secure invocation is not blanket permission to untaint or unsecret values. **UNKNOWN — WM61 S2:** callback access, full return connection and exceptional restoration.
- **SOURCE — WM61 S2:** Retail overrides rilua's ancestor-clearing `securecallfunction` registration with a source-preserving invocation. The simulator does not yet model the native policy-1 selector, result-provenance rewrite or exceptional restoration; WM61 S2 leaves complete callback access and exceptional behavior UNKNOWN.
- **EMULATED — simulator boundary:** The simulator's internal forbidden-object projection and secure XML delegate machinery use a dedicated secure invocation supplied privately to the bootstrap. It is not the public `securecallfunction` contract. This models the native secure-client boundary; the exact native object-projection implementation is UNKNOWN.

### AuraData publication and spell lookup

- **VERIFIED locally — WM61 S3** (`S3-aura-field-granularity-wm61.md`): Publication policy 2 marks `auraInstanceID`, `spellId`, `icon`, `applications`, `duration`, `expirationTime`, `name`, `sourceUnit`, `dispelName`, `isHelpful`, and `isHarmful` individually; optional nil values remain unmarked. `points` is separately marked and its numeric elements use the field policy.
- **EMULATED — WM61 S3:** Under the stated publication seeds the outer table value is unmarked by default, but an explicit table-policy override can mark it. **UNKNOWN — WM61 S3:** complete accessor reachability, `issecrettable`, populated points and table security flags. Policy 0 does not mark; policy 1 depends on current-frame operation state; policy 2 applies without that condition.
- **SOURCE — WM61 S3:** Restricted, non-NeverSecret AuraData now marks the locally identified fields and leaves the outer value plain. The simulator's points list is currently empty; populated-list publication remains untested. Complete accessor reachability is still UNKNOWN.
- **VERIFIED locally — WM61 S8** (`S8-requires-nonsecret-aura-wm61.md`): When tainted execution, the secrecy feature and at least one of four Active restriction states select the restrictive context, the ID lookups classify the requested spell before aura lookup; the name lookup resolves the name first. SpellMisc `Attributes[15]` mask 33554432 selects AlwaysSecret before mask 67108864 selects NeverSecret; otherwise the enabled feature yields ContextuallySecret. Schema names are inferred.
- **EMULATED — WM61 S8 and S8a** (`S8a-unitauras-secret-emulated.md`): Selected rejection fragments return zero results, not one nil; separate outer fragments forward zero under their assumptions. **INFERRED — WM61 S8:** The relationship of these branches to `RequiresNonSecretAura`. **UNKNOWN — WM61 S8/S8a:** Complete registered-entry behavior, matched-aura outcomes and allowed-path publication.
- **SOURCE — WM61 S8:** ID-based spell queries return zero values at the restricted tainted pre-lookup gate. The SpellMisc classifier gives AlwaysSecret precedence when both flags are set. The spell-name query uses supplied spell-name aliases before lookup; if restricted tainted name resolution has no supplied alias, that branch raises a not-modeled error. Native name-provider behavior is UNKNOWN (WM61 S8/S8a).

### Text aspect

- **VERIFIED locally — WM61 S6** (`S6-aspects-wm61.md`): Text aspect is 8. The marked-argument `SetText` path supplies it to a helper that checks prevention and script-handler policy before adding it to the direct set. `HasSecretAspect` checks either of two sets; `HasAnySecretAspect` checks whether either is nonempty. Prevention and anchoring use separate flags.
- **EMULATED — WM61 S6:** With publication enabled, a Text aspect prepares forced-secret `GetText` return policy; a separate helper marks eligible strings under that policy. **UNKNOWN — WM61 S6:** the complete setter/getter path, second-set origin, propagation and clearing on plain reset, `SetToDefaults` or `ClearAllPoints`.
- **SOURCE — WM61 S6:** Retail `SetText` adds the Text aspect for a secret input and rejects a frame that prevents secrets. The script-handler policy is not yet modeled. The simulator still derives object aspect state from prevention/protection and uses prevention for `IsAnchoringSecret`; these differ from the distinct native calculations.

## Native evidence folded (Retail 12.1.5.70688)

Clean-room reports: `H/native-evidence-70688/secrets/` (S0–S11, S15, S18, and the WM61 follow-ups) and `H/native-evidence-70688/` (S14, S16–S21), where `H` is the wow-ui-sim handoff directory. Labels are the reports' own. Behaviour read from a region that a report marks contaminated, truncated or overlapping is not used, and the reports' own retractions are carried as stated.

| Unit | Subject | Label | Answer |
|---|---|---|---|
| S0 | client strings | VERIFIED (presence) | Four secret-value templates, three argument-mode diagnostics, table guards, aspect vocabulary, `<secret>` display forms |
| S1 | `C_Secrets` surface | VERIFIED (surface) | 23 predicates with arities; registered bodies are validation trampolines; booleans are `state == 2` |
| S2 | who sees a secret | VERIFIED (aura path only) | Aura rejection needs a taint source **and** a flag **and** one of four Active states; no context permits |
| S3 | AuraData types | VERIFIED (types), INFERRED (fields) | Four AuraData types, four accessors, a swappable provider; field-level marking is WM61 S3 |
| S4 | helper surface | VERIFIED (surface) | The restricted helper set is closed; behaviour reads blocked |
| S4s7a | `securecopy` | EMULATED | One result on success; secret scalars stay secret; tainted arguments rejected |
| S4s7b | `CreateSecureDelegate` | INFERRED + EMULATED | One C closure with three upvalues; invocation captures argument provenance |
| S5 | operations on secrets | VERIFIED (check), UNKNOWN (modes) | Marked operands consult one shared check; mode 0 rejects without a taint source |
| S6 | sinks and aspects | VERIFIED (strings only) | Three argument diagnostics; the text aspect is 8 (WM61 S6) |
| S7, S7a | wrap/unwrap | EMULATED | Both are variadic transforms, one result per argument |
| S8 | `RequiresNonSecretAura` | VERIFIED (enum, classifier) | `NeverSecret` 0, `AlwaysSecret` 1, `ContextuallySecret` 2; flags 33554432 wins over 67108864 |
| S9 | taint | VERIFIED (messages, raise), UNKNOWN (dispatch) | Taint is named provenance; violations raise; dispatch inheritance UNKNOWN |
| S10 | restriction storage | VERIFIED | `IsAddOnRestrictionActive(type)` is `state[type] == 2` |
| S11 | tooling limits | VERIFIED (limits) | Anti-disassembly filler; real code intact; only verified entry points are admissible |
| S14 | named predicates | VERIFIED | 16 `Predicate check failed: <Name>` guards; `RequiresUnitAuraAccess` guards `GetUnitAura` |
| S15 | protected globals | VERIFIED (negatives) | `ADDON_ACTION_*` and `MACRO_ACTION_*` exist; the protected set is UNKNOWN |
| S16 | registration families | VERIFIED | One 38-function family holds every secrecy helper |
| S18 | secrecy predicates | VERIFIED | Permissive no-argument defaults; question/answer pairs; `scrub` is scalar-only |

### Helper contracts (S18, S7, S7a)

- **VERIFIED:** `issecretvalue()` with no argument returns `false`. With one argument it returns a boolean from the secret test.
- **VERIFIED:** `canaccessvalue()` with a missing or malformed argument returns `true`. The permissive default is a design property: the "is it secret" question defaults to false and the "may I use it" question defaults to true. A general rule for all 25 helpers is **INFERRED**, not verified.
- **VERIFIED:** `issecrettable` and `canaccesstable` both gate on Lua type code 5 (`table`). The non-table return is **INFERRED** false.
- **VERIFIED:** `hasanysecretvalues` and `canaccessallvalues` are variadic scans from argument 1; fewer than one argument skips the scan. Result accumulation is **INFERRED**.
- **VERIFIED:** `canaccesssecrets` does not type-gate. `secretunwrap` carries the singleton entry guard; `secretwrap` has none.
- **VERIFIED:** `scrub` inspects only scalar arguments (Lua types 1, 3 and 4: boolean, number, string). Tables, userdata and functions take a separate path. `scrubsecretvalues` shares the iteration skeleton; whether it shares the filter is **UNKNOWN**.
- **EMULATED (S7a):** `secretwrap` and `secretunwrap` are transforms, not checks or grants. Both take variable arguments and return one result per argument, in order, with the same type and payload; no arguments return zero results. `secretwrap` marks every returned value when the marker gate is enabled and marks none when it is disabled. `secretunwrap` unmarks marked argument slots without changing payloads. Guard denial returns zero results.
- **CORRECTED (S7a):** the earlier "two arguments" reading of the wrap pair is wrong. The constant 2 in those handlers is an operation-policy setting, not an arity.
- **UNKNOWN:** success values of `secretunwrap`, mutation of referenced tables, `canaccesssecrets` signature, and the contracts of `dropsecretaccess`, `mapvalues`, `settablesecurity` and `securecallmethod` (S7).

### Operation check (S5)

**VERIFIED:** arithmetic, the identified comparison path, concatenation, length and the identified boolean path call one shared operation check on their operands. That check reads an operand marker; a zero marker returns false. For a marked operand the outcomes are conditional:

- if an additional state is absent, the check sets a flag and returns true;
- if that state is present, decision mode 0 rejects regardless of taint;
- mode 1 rejects only when a current taint source exists, and otherwise passes;
- other mode values pass.

**VERIFIED:** rejection raises through the secret-error formatter with the caller's operation word, not a returned status. **UNKNOWN:** the meaning of the flag and the additional state, the mode's origin, and the actual mode at any boundary. `tostring`, `string.format`, table key, table value storage, `type`, `select`, function pass-through and `pcall` outcomes are **UNKNOWN**. The check's presence does not establish that untainted code always passes or that tainted code always errors.

### Taint (S9)

- **VERIFIED:** taint is provenance-bearing state that can print its origin. The message templates name `'%s'`, and one path uses the fixed phrase `an AddOn`.
- **VERIFIED:** secrecy, restricted-table access and forbidden aspects are three separate protected categories. Secrets are typed, so a violation names `a secret <lua type> value`.
- **VERIFIED:** the secret-value violation **raises**; the raise sites carry unreachable padding after the call. A simulator that returns `nil` or `false` diverges, because execution unwinds.
- **VERIFIED:** the four secret-value templates are selected by two independent booleans: a resolved name for the offending value, and a present taint origin. All four combinations are reachable. An unnamed temporary cannot produce the quoted form, and a tainted execution with no recorded origin yields the untainted wording.
- **VERIFIED:** the quoted name is resolved from the enclosing value's container, and the type slot uses standard Lua type names.
- **VERIFIED:** the tainted-table guard binds `index` (5 sites), `get length of`, `iterate` and `perform indexed assignment on`. The secret-value guard binds `perform boolean test on`, `negate`, `index`, `perform indexed assignment on`, plus one unresolved site. Neither family shows `perform arithmetic on` or `concatenate`.
- **UNKNOWN (and this decides issue #13's headline):** what propagates taint across an event dispatch. `TaintEvent` exists as a map value keyed by a string-like type; the dispatch path was not read. Whether a handler inherits the dispatcher's taint, and whether a secure context is entered for protected handlers, remain open.

### Registration, predicates and protection (S4, S14, S15, S16)

- **VERIFIED:** the registration class byte takes only 3, 6 and 7. Class 7 covers every secret/secure helper and also `Mixin` and the event-callback registrations, so the class does not separate secret from ordinary functions. `GetForbiddenObjectTable` is the only class 6 entry. Class meanings are **UNKNOWN**.
- **VERIFIED:** the client carries exactly 16 `Predicate check failed: <Name>` messages. Each maps to one guarded API; `SetFont` carries two. `RequiresUnitAuraAccess` guards `GetAuraDataByIndex`/`GetUnitAura`. WM105 corrects the earlier reading: there is no numeric code 46. The predicate text is pushed with an explicit string length of 46, exactly the character count of `Predicate check failed: RequiresUnitAuraAccess`.
- **VERIFIED:** these named predicates are a separate mechanism from the restriction state array. **UNKNOWN:** the numeric codes and any shared storage.
- **VERIFIED:** `addonCombatRestrictionsForced`, `addonEncounterRestrictionsForced`, `addonChallengeModeRestrictionsForced`, `addonPvPMatchRestrictionsForced`, `addonMapRestrictionsForced` and `addonChatRestrictionsForced` exist with their help text; `taintLogObjectSecrets` enables aspect logging.
- **VERIFIED:** the 38-function registration family contains every restricted/secrecy helper. Membership does **not** encode protection: `GetMoney`, `RepopMe` and `RetrieveCorpse` sit in the large general family yet behave as protected.
- **VERIFIED (S15):** `ADDON_ACTION_FORBIDDEN`, `ADDON_ACTION_BLOCKED` and the `MACRO_ACTION_*` equivalents exist as event names. The player-facing message text is composed Lua-side. The **protected set** is **UNKNOWN**: four candidate mechanisms were eliminated and the remaining one is inside a contaminated body.

### Evidence limits (S11, S4)

**VERIFIED:** the image carries a 25-byte anti-disassembly filler block 38,230 times, all inside the code section. Real code is intact and is chained by branches that jump over junk zones. Static claims are admissible only from verified entry points (registration sites, call targets, exception directory); any read that arrives through fall-through into junk is discarded. The secret-helper bodies, `securecopy` and `CreateSecureDelegate` statically, and the aura field layout remain outside admissible static evidence. S4s7a, S4s7b and S7a therefore report **EMULATED** results under explicit seeds, not native behaviour.

### Non-secrecy units (S17, S19, S20, S21)

The file-by-file pass also produced answers for units that are not secrecy surface. Their contracts belong to their own specs: targeting commands (S17) and cursor-item commands (S19) have no secrecy behaviour established; time queries (S20) are folded into [get-local-game-time](get-local-game-time.md); number formatting (S21) is folded into [abbreviated-number-formatter](abbreviated-number-formatter.md). The one repeated secrecy-adjacent observation is the singleton entry guard: `TargetUnit` returns `nil` while the singleton is set, the same refusal S13 records for `CreateSecureDelegate`, `secretunwrap` and `settablesecurity`. It is **not** the protected-action mechanism.

## Native behaviour vs simulator

`Native` gives the strongest admissible label; `Simulator` is source behaviour on 2026-11-04.

| Behaviour | Native | Simulator | Verdict |
|---|---|---|---|
| `issecretvalue()` no argument | `false` (VERIFIED) | `false` | matches |
| `canaccessvalue()` missing/malformed | `true` (VERIFIED) | `true` | matches |
| `issecrettable` / `canaccesstable` non-table | not a table → false (INFERRED) | `issecrettable` uses the VM table flag; `canaccesstable` falls back to a shallow value check | unverified, not contradicted |
| `hasanysecretvalues()` no arguments | `false` (INFERRED) | `false` | matches |
| `scrub` type filter | scalar-only (VERIFIED) | scalar-only; non-scalars are replaced with `nil` | replacement rule is **UNKNOWN** natively |
| `secretwrap` / `secretunwrap` arity | variadic (EMULATED) | variadic | matches |
| `secretwrap` / `secretunwrap` results | one per argument; zero for no arguments (EMULATED) | one per argument; zero for no arguments | matches |
| `secretunwrap` on a tainted caller | **UNKNOWN** | raises `secret access requires an untainted caller without revoked access` | best guess |
| Secret-value violation | raises, never returns (VERIFIED) | raises | matches |
| Violation wording | four templates selected by resolved name × taint origin (VERIFIED) | same four forms ([taint error text](taint-error-text.md)) | matches |
| Rejection at the aura path | context + recorded source + policy flag + any of entries 0..3 `Active` (VERIFIED, WM105) | same order over restriction entries 0..3; forced taint names its source | matches; suffix separator best-guess |
| `GetAuraDataByIndex` denial | raises `<ApiName>(): Auras cannot be accessed when secret while tainted by '<origin>' Lua Taint: <origin>` (WM105 §6) | same form | matches; suffix separator is best-guess |
| Marked operand in untainted code | conditional on mode and additional state (VERIFIED) | untainted callers pass the VM guard | partially matches; modes unmodelled |
| Event-dispatch taint | UNKNOWN | admin dispatch clears caller taint | unverified best guess |

## Current state (SOURCE, 2026-10-31)

| Area | Today | Gap |
|---|---|---|
| Restriction trigger | Aura secrecy reads `restricted_actions[0..3] == Active` plus the WM105 policy flag; `restricted_actions.rs` models supplied indexed transitions and forced CVars | The complete WM61 natural conditions are still supplied inputs |
| AuraData | Restricted, non-NeverSecret records mark identified fields; the outer value is plain | Full policy selection and `ConditionalSecret` contents are not established |
| Spell-keyed lookups | Never-secret spells stay plain; others return one nil to tainted callers (inferred) | WM61 S8 rejection fragments return zero results, not one nil; complete entry behavior remains UNKNOWN |
| VM operations | Guarded truth/equality for wrapped booleans and nil, guarded ordering for numbers; violations raise with the native templates | Arithmetic, concat, `tostring`, `format` and table-key rules missing; S5's operation-check modes and additional state are unmodelled |
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
- [ ] `RequiresUnitAuraAccess` predicate string: WM105 §6 shows it is only the fallback when the pending-error lookup finds no report, so it never appears on the aura-denial path. The simulator does not model a no-report denial arm; no native route to it is known.
- [x] Aura rejection condition (WM105): context, then a recorded taint source (a null source permits), then the generated-secrecy-policy flag, then any of restriction entries 0..3 equal to `Active`. `Map` and `Chat` are never read, `Activating` does not match, and the gate shares `GetAddOnRestrictionState`'s array. `forceinsecure()` installs `*** TaintForced ***`, so a forced caller is denied.
- [ ] Helper contracts left open by S7/S18: `canaccesssecrets` signature, `dropsecretaccess`, `mapvalues`, `settablesecurity`, `securecallmethod`, and success values of `secretunwrap`.

## Out of scope

- Classic profiles beyond [classic-secret-policy](classic-secret-policy.md).
- Non-aura `SecretWhen*` families (unit identity, power, stats, casts, cooldowns), except where they share the same machinery.
- Patching Blizzard Lua to avoid secrets.
