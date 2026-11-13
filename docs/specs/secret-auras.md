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
- **SOURCE**: simulator source at base `e29763f8e`, audited on 2026-11-12.
- **VERIFIED**: the cited report establishes the stated local calculation or bytes, not a complete client invocation.
- **INFERRED**: a conclusion from verified facts, not a verified contract.
- **EMULATED**: a seeded fragment result; it is not promoted to VERIFIED or used alone to authorize a fix.
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
- [ ] Apply the conditional shared operation check documented under S5 below. **VERIFIED:** mode 0 rejects a marked operand even without taint when the additional state is present; mode 1 rejects only with a recorded source; other modes permit. **UNKNOWN:** the additional state's meaning and the mode selected at each public boundary. Do not replace this with “all untainted callers pass.”
- [ ] Leave complete arithmetic, comparison, concatenation, `tostring`, `string.format`, length, table storage and truth-test outcomes UNKNOWN where S5 does not establish them.
- [ ] Secret values render as `<secret>`, `<secret table>`, or `nil --[[ secret value ]]` where the client renders them (STRING). Tables cannot be indexed with secret keys where the client forbids it (STRING: "attempted to %s a table that cannot be indexed with secret keys").

### Widget sinks and aspects
- [ ] Widget methods apply their `SecretArguments` mode. STRING names three modes: never allowed, not allowed, and only allowed during untainted execution.
- [x] Under tainted execution, a marked string passed to `Texture:SetTexture` raises `Cannot set texture to a secret string value.` (**VERIFIED — R88c Q1, Retail 12.1.5.70688**; full native error context is UNKNOWN).
- [ ] A marked numeric file identifier can pass `Texture:SetTexture` from tainted code when the target permits it (**INFERRED — R88c Q1**; Retail docs declare `AllowedWhenTainted` at `Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:604–607`).
- [ ] An accepted marked input sets the region's secret-values flag (**VERIFIED — R88c Q2**). Getter marking is **INFERRED**; whether the asset renders is **UNKNOWN**.
- [ ] A target that prevents secret values rejects marked `SetTexture` input with `Attempt to set secret values on an object that prevents secret values` (**INFERRED — R88c Q1**; full native error context is UNKNOWN).
- [ ] Apply `SecretArgumentsAddAspect` and `SecretReturnsForAspect` according to the bounded WM61 S6 findings below; complete getter publication remains UNKNOWN.
- [ ] Shipped aura buttons can pass secret duration, text, icon, visibility and geometry to their widgets and render correctly (UI).

### Helpers
- [ ] `issecretvalue`, `issecrettable`, `hasanysecretvalues`, `canaccessvalue`, `canaccessallvalues`, `canaccesstable`, `canaccesssecrets`, `scrubsecretvalues`, `dropsecretaccess`, `secretwrap`, `secretunwrap` and the widget `HasSecretValues` / `HasSecretAspect` / `HasAnySecretAspect` / `IsPreventingSecretValues` / `IsAnchoringSecret` match the client's usage strings and return counts. Use the bounded S7/S7a/S18 contracts below: `issecretvalue()` is VERIFIED false, and `canaccessvalue()` is VERIFIED true. Do not treat the remaining incomplete contracts as implemented native behavior.

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
- **SOURCE — WM61 S2:** Retail and PTR override rilua's ancestor-clearing `securecallfunction` registration with a source-preserving invocation; Forever does not build the `retail-12-1-0` secret surface. The simulator does not yet model the native policy-1 selector, result-provenance rewrite or exceptional restoration; WM61 S2 leaves complete callback access and exceptional behavior UNKNOWN.
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
- **SOURCE — WM61 S6:** Retail `SetText` adds the Text aspect for a secret input and rejects a frame that prevents secrets. Retail/PTR now query recorded aspect bits independently of prevention/protection, and `IsAnchoringSecret` reads a separate supplied flag. No native writer or second-set propagation is invented. Other profiles retain their existing behavior. **INFERRED PTR compatibility:** `SetText` records Text=8 on the same profiles that query recorded bits; native PTR equivalence is not established by S6. This writer extension is a separate droppable commit. Script-handler policy remains unmodelled.

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
| S4s7b | `CreateSecureDelegate` | VERIFIED (bounded helper regions) + EMULATED | Construction gate, source selection, shallow argument provenance and callback wrapping; complete return and target behavior remain unknown |
| S5 | operations on secrets | VERIFIED (conditional mode calculation), UNKNOWN (boundary selection) | With additional state present, mode 0 rejects regardless of source, mode 1 rejects only with source, and other modes permit; absent state permits |
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
- **UNKNOWN:** complete live `secretunwrap` success behavior, mutation outside the seeded fragments, `canaccesssecrets` signature, and the contracts of `dropsecretaccess`, `mapvalues`, `settablesecurity` and `securecallmethod` (S7). S7a supplies EMULATED success results, not a complete live contract.

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
- **VERIFIED:** the named diagnostic sites differ from the indexed-state getter. **UNKNOWN:** any shared storage or complete connection between these mechanisms; no numeric error code is established.
- **VERIFIED:** `addonCombatRestrictionsForced`, `addonEncounterRestrictionsForced`, `addonChallengeModeRestrictionsForced`, `addonPvPMatchRestrictionsForced`, `addonMapRestrictionsForced` and `addonChatRestrictionsForced` exist with their help text; `taintLogObjectSecrets` enables aspect logging.
- **VERIFIED:** the 38-function registration family contains every restricted/secrecy helper. Membership does **not** encode protection: `GetMoney`, `RepopMe` and `RetrieveCorpse` sit in the large general family yet behave as protected.
- **VERIFIED (S15):** `ADDON_ACTION_FORBIDDEN`, `ADDON_ACTION_BLOCKED` and the `MACRO_ACTION_*` equivalents exist as event names. The player-facing message text is composed Lua-side. The **protected set** is **UNKNOWN**: four candidate mechanisms were eliminated and the remaining one is inside a contaminated body.

### Evidence limits (S11, S4)

**VERIFIED:** the image carries a 25-byte anti-disassembly filler block 38,230 times, all inside the code section. Real code is intact and is chained by branches that jump over junk zones. Static claims are admissible only from verified entry points (registration sites, call targets, exception directory); any read that arrives through fall-through into junk is discarded. The secret-helper bodies, `securecopy` and `CreateSecureDelegate` statically, and the aura field layout remain outside admissible static evidence. S4s7a, S4s7b and S7a therefore report **EMULATED** results under explicit seeds, not native behaviour.

### Non-secrecy units (S17, S19, S20, S21)

The file-by-file pass also produced answers for units that are not secrecy surface. Their contracts belong to their own specs: targeting commands (S17) and cursor-item commands (S19) have no secrecy behaviour established; time queries (S20) are folded into [get-local-game-time](get-local-game-time.md); number formatting (S21) is folded into [abbreviated-number-formatter](abbreviated-number-formatter.md). The one repeated secrecy-adjacent observation is the singleton entry guard: `TargetUnit` returns `nil` while the singleton is set, the same refusal S13 records for `CreateSecureDelegate`, `secretunwrap` and `settablesecurity`. It is **not** the protected-action mechanism.

## Native behaviour vs simulator

`Native` gives the strongest admissible label; `Simulator` is source behavior at `e29763f8e`. The audit below supersedes stale implementation summaries.

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

## Current state (SOURCE, lane implementation 2026-11-12)

| Area | Today | Gap |
|---|---|---|
| Restriction trigger | Aura secrecy reads `restricted_actions[0..3] == Active` plus the WM105 policy flag; `restricted_actions.rs` models supplied indexed transitions and forced CVars | The complete WM61 natural conditions are still supplied inputs |
| AuraData | Restricted, non-NeverSecret records mark identified fields; the outer value is plain | Full policy selection and `ConditionalSecret` contents are not established |
| Spell-keyed lookups | ID rejection returns zero values; the name path requires a supplied alias | Complete matched-aura publication and native name-provider behavior remain UNKNOWN (WM61 S8/S8a) |
| VM operations | Guarded truth/equality for wrapped booleans and nil, guarded ordering for numbers; violations raise with the native templates | Arithmetic, concat, `tostring`, `format` and table-key rules missing; S5's operation-check modes and additional state are unmodelled |
| Widget sinks | `SetText`, `SetShown`, geometry, texture and cooldown on gated paths; Retail/PTR recorded aspects and anchoring flag are independent of prevention/protection | No general argument-mode model, second recorded-set writers or native anchoring writer |
| Event taint | `A_Admin.FireEvent` clears caller taint during dispatch (4.3.4-based comment) | Retail rule UNKNOWN; other dispatch paths unchecked |
| Helpers | Bounded shallow checks; `canaccesstable` recursive | Several helpers missing or pass-through |

## Evidence index and corrections (2026-11-12)

All paths below are relative to `/home/osso/.local/state/wow-ui-sim/handoff/native-evidence-70688/`. A citation identifies a clean-room report, not client source. UNKNOWN selections stay unmodelled; this fold adds no guessed values.

| Answer | Evidence and exact limit |
|---|---|
| S2 access and invocation | **VERIFIED:** the aura gate needs context, recorded source, policy flag and one Active entry 0..3 (`secrets/S2-who-sees-secret.md`, Answer; corrected by `secrets/S2-four-state-wm105.md`, §§1–7). **VERIFIED locally:** secure invocation saves/restores caller source and rewrites surviving result provenance without clearing secrecy (`secrets/S2-secure-invocation-wm61.md`, Answer). **UNKNOWN:** complete callback execution and exceptional restoration. |
| S3 publication | **VERIFIED locally:** the converter marks individual identified fields, not the outer table by default (`secrets/S3-aura-field-granularity-wm61.md`, Native mechanism). This supersedes the whole-struct **INFERRED** reading of `secrets/S3-aura-field-granularity.md`, §1.1. **UNKNOWN:** complete accessor linkage and populated points. |
| S4 registration | **VERIFIED:** closed helper registration set and classes; class is not a protection decision (`secrets/S4-restriction-helpers.md`, Registration; `S16-global-api-families.md`, Notes). **UNKNOWN:** full bodies in contaminated regions. Usage strings alone do not establish behavior (`secrets/S4s7-securecopy-and-delegate.md`, Usage strings). |
| S4s7a copying | **VERIFIED at helper scope:** recursive copying handles keys and values, creates a new table without the metatable, and assigns saved execution provenance (`secrets/S4s7a-securecopy-emulated.md`, input table and provenance rule). **EMULATED:** successful secret scalars retain secrecy; nested tables are copied. **UNKNOWN:** complete protected wrapper, live propagation counter and error unwinding. Do not assert that copying always removes taint. |
| S4s7b delegate | **VERIFIED locally:** construction publishes a function with three captured values; invocation assigns captured provenance to arguments and selects/restores captured source; nonempty entry source rejects before dispatch (`secrets/S4s7b-createsecuredelegate-emulated.md`, §§1–3). **INFERRED:** exactly one public callable and forwarding all target results. **UNKNOWN:** actual Lua target security and inhibit-flag reachability. |
| S5 operations | **VERIFIED:** the shared check has the conditional mode branches stated above (`secrets/S5-untainted-operations.md`, Shared operation check and Answers). **UNKNOWN:** mode origin and boundary selection, result secrecy and operations whose continuation is unread. |
| S6 aspects | **VERIFIED locally:** aspect queries read recorded sets; prevention and anchoring use independent flags; Text is 8 (`secrets/S6-aspects-wm61.md`, Native mechanism). **UNKNOWN:** complete Lua publication, second-set writers, clearing and propagation. `secrets/S6-sinks-and-aspects.md` establishes three diagnostic strings, not complete argument-mode tables. |
| S7/S7a helpers | **VERIFIED:** registration and marker transfer rules; **EMULATED:** variadic wrap/unwrap transforms, one result per argument, zero for no arguments, no persistent access grant (`secrets/S7a-secretwrap-emulated.md`, §§1–2). This corrects S18's two-argument reading. **UNKNOWN:** remaining full helper contracts (`secrets/S7-secret-helpers.md`, per-helper sections). |
| S8/S8a lookup | **VERIFIED:** secrecy enum and classifier precedence, and bounded pre-lookup rejection (`secrets/S8-requires-nonsecret-aura.md`, NeverSecret mechanism; `secrets/S8-requires-nonsecret-aura-wm61.md`, Answer). **EMULATED:** rejection fragments return zero results (`secrets/S8a-unitauras-secret-emulated.md`). **UNKNOWN:** complete registered entry and matched-aura results. |
| S9 taint | **VERIFIED:** named provenance, separate protected categories and raising secret violations (`secrets/S9-taint-model.md`, §§1–4; `secrets/S9-taint-templates.md`, formatter findings). **UNKNOWN:** event-dispatch inheritance; RTTI map membership is not a dispatch trace (§7). |
| S11 admissibility | **VERIFIED:** unreachable 25-byte filler repeats 38,230 times; earlier nine-byte description was incomplete (`secrets/S11-deobfuscation.md`, §§0–0.1). **UNKNOWN:** behavior beyond admitted windows. |
| S14 predicates | **VERIFIED:** 16 diagnostic strings and co-located API names; attribution of each co-located name to its enclosing API is **INFERRED** (`S14-predicate-names.md`, §§5–5.2). WM105 corrects “code 46” to string length and makes the predicate text an alternative to raising the pending aura report. |
| S15 protection | **VERIFIED:** addon/macro event names and distinct combat/non-combat log wording (`secrets/S15-protected-globals.md`, Q1/Q4). **INFERRED:** combat input to the decision. **UNKNOWN:** protected set, complete emitter and event payload order. Later evidence belongs to [restriction reporting](restricted-actions-state.md). |
| S16 registration | **VERIFIED:** a 38-function family includes secrecy and ordinary helpers; family membership does not establish protection (`S16-global-api-families.md`, Notes). |
| S17 targeting | **VERIFIED:** 22-name registration family and `TargetUnit` optional name/exact-match input with empty-name default (`S17-targeting-unit.md`, §§1–2). **UNKNOWN:** complete target mutation and protection. No aura contract follows from this unit. |
| S18 predicates | **VERIFIED:** `issecretvalue()` false; supplied value uses a secret predicate; `canaccessvalue()` true for missing/malformed input (`S18-secrecy-predicates.md`, §§1–2). **VERIFIED:** table-type gates and scalar-only `scrub` scan; **INFERRED:** remaining result accumulation and non-table defaults (`S18-secrecy-predicates.md`, §§6–12; `secrets/S18-secrecy-predicates.md`, §§13–14). The latter's empty-scan false is not promoted over the former's INFERRED result. |
| S19 cursor | **VERIFIED:** 18-name family and `CursorHasItem` equality-to-1 flag test; **UNKNOWN:** flag meaning and `GetCursorInfo` returned tuple (`S19-cursor-unit.md`, §§1–4). See [cursor model](cursor-model.md); no secrecy behavior is established. |
| S20 time | **VERIFIED:** local time returns two numbers, initialized to -1; server time is a computed double; session time scales a counter by 0.001. Counter units are **INFERRED** (`S20-time-unit.md`, §§5–10). See [time contract](get-local-game-time.md). |
| S21 formatting | **VERIFIED:** named separator lookup, double thresholds and two configuration tables; **UNKNOWN:** configuration selector meaning (`S21-formatting-unit.md`, §§5,15). See [number formatter](abbreviated-number-formatter.md); no secrecy rule follows. |

## VERIFIED contradiction audit at `e29763f8e`

This table records established contradictions in the audited S2–S9 helper, aura and widget paths. It does not classify UNKNOWN behavior as a contradiction. Native citations use the evidence root above.

| Case | VERIFIED native claim | Simulator source | Disposition |
|---|---|---|---|
| Aspect queries | Only recorded sets determine intersection/nonempty (`secrets/S6-aspects-wm61.md`, Native mechanism) | `src/lua_api/frame/methods/misc/secret.rs:175–184` derives ObjectSecurity from prevention, forbidden or protected state | Fix the Retail/PTR calculation; do not assert second-set propagation or complete native publication. |
| Anchoring query | Separate anchoring flag, not prevention (`secrets/S6-aspects-wm61.md`, Native mechanism) | `src/lua_api/frame/methods/misc/secret.rs:101–105,187–191` aliases prevention | Supply a separate host input; leave native writers UNKNOWN. |
| Explicit aspect addition | **VERIFIED locally:** prevention blocks adding requested bits (`secrets/S6-aspects-wm61.md`, Native mechanism); final Lua error publication remains UNKNOWN | `src/lua_api/frame/methods/misc/secret.rs` returns before recording a new bit on Retail 12.1+ when prevention is set | Retail prevention guard implemented; script-handler policy and final native diagnostic publication remain UNKNOWN. |
| Secure delegate entry | Nonempty execution source rejects before dispatch (`secrets/S4s7b-createsecuredelegate-emulated.md`, §3) | At e297, construction had no source gate; this lane adds the zero-result gate when the reporting helper returns | Reporter effects remain UNKNOWN; no target or error behavior is asserted. |
| Secure delegate provenance | Constructor captures source and the helper selects it at invocation (`secrets/S4s7b-createsecuredelegate-emulated.md`, §§1–2) | At e297, source was captured at invocation and the target call always selected nil; this lane stores construction source and selects it for the inner call | Target security, inhibit flag and complete return publication remain UNKNOWN. |

**VERIFIED source, not a contradiction:** `issecretvalue` uses the existing VM registration (`src/lua_api/globals/security/mod.rs:66`, `value_access.rs:15–20`); its conditional fallback calls a shallow secret predicate (`value_access.rs:136–140`), not constant false. `canaccessvalue()` uses missing-input nil and returns true (`value_access.rs:145–158`), as S18 requires. Named taint already exists in slot and closure metadata (`value_access.rs:47–70`, `secret_values.rs:207–214`); S9 does not establish an event-source rule to replace current dispatch.

**UNKNOWN, not contradictions:** S5 public-boundary mode selection; S9 event inheritance; complete getter publication and clearing; missing restricted-helper singleton state. `securecallmethod` delegates to a protected `xpcall` bridge (`src/lua_api/script_helpers.rs:853–877`), which does not explicitly clear caller taint; a source-restoration difference is UNVERIFIED, not an established contradiction. Keep these unmodelled rather than selecting guessed native values.

## How it works

- [Lua API: taint](../wiki/systems/lua-api.md)
- [Secret values](retail-secret-values.md), [classic policy](classic-secret-policy.md), [secret display](aura-secret-display.md)

## Related narrow specs

[unit-aura-slot-secret-arguments](unit-aura-slot-secret-arguments.md), [unit-aura-index-secret-arguments](unit-aura-index-secret-arguments.md), [spell-aura-secrecy](spell-aura-secrecy.md), [secret-selector-aura-duration](secret-selector-aura-duration.md), [secret-selector-aura-caster](secret-selector-aura-caster.md), [tooltip-aura-instance-security](tooltip-aura-instance-security.md), [script-object-secret-aspects](script-object-secret-aspects.md), [widget-secret-aspects-12-0-7](widget-secret-aspects-12-0-7.md), [secret-number-ordering](secret-number-ordering.md), [secret-string-formatting](secret-string-formatting.md), [closure-taint-lifetime](closure-taint-lifetime.md), [taint-error-text](taint-error-text.md).

## Implementation inventory

- `src/c_api/c_secrets.rs`: publishes the Forever spell-casting predicate and its host GUID inputs.
- `src/c_api/secrets_queries.rs`: registers the broader retail predicate surface.
- `src/lua_api/globals/unit_misc.rs`: resolves built-in and host-supplied unit GUIDs.
- `src/c_api/unit_aura_access.rs`: restriction flag and the aura access guard.
- `src/lua_api/globals/auras.rs`: AuraData builder and table wrapping.
- `src/c_api/c_unit_auras.rs`, `c_unit_aura_*.rs`, `aura_duration.rs`: aura queries and per-function guards.
- `src/lua_api/globals/security/{secret_values,value_access}.rs`: helpers and the wrap/unwrap bridge.
- `src/lua_api/frame/methods/{secret_origin.rs,misc/secret.rs}`: widget sinks and aspects.
- `src/lua_api/frame/methods/widgets/texture/atlas.rs`: texture setters and atlas behavior.
- `src/lua_api/globals/admin.rs`, `admin_events.rs`: admin event dispatch taint.
- `rilua/src/{table_security.rs,stdlib/taint.rs}`: VM secret wrappers and taint.

## Tests asserting this spec

- `src/loader/tests/frame_state.rs::test_secret_and_protected_methods_reflect_frame_security_state` checks independent recorded-aspect queries against prevention/protection/forbidden state on Retail/PTR and preserves other profiles. Main approved retargeting the old conflation assertions against WM61 S6.
- `src/loader/tests/frame_state.rs::test_prevention_does_not_imply_anchoring_secret` checks prevention versus separately supplied anchoring state. These are registered-API calculation tests: no pinned Lua consumer calls these queries.
- `tests/secret_text_aspect_consumer.rs::compact_unit_name_marks_text_aspect_for_secret_identity` runs original `CompactUnitFrame_UpdateName` on Retail/PTR as a positive Text-aspect control. The native evidence is Retail-only; PTR shares the simulator model, not a claimed native proof.

- `tests/spell_casting.rs::forever_spell_cast_secrecy_uses_supplied_unit_and_control_guids_in_casting_bar` checks Forever's default, host-supplied GUID ownership, argument errors, and the original `CastingBarMixin:IsInterruptable` consumer.
- `tests/spell_casting.rs::casting_bar_original_interruptibility_uses_cast_secrecy_predicate` checks the original consumer on Retail and PTR.
- `tests/aura_secret_display.rs::tainted_secret_texture_string_reports_native_error` checks the VERIFIED tainted-string diagnostic.
- `tests/aura_secret_display.rs::aura_secret_texture_replaces_and_clears_without_plain_readout` checks secret texture input and getter behavior under `optional-tests`.
- `tests/aura_secret_display.rs::tainted_secret_texture_target_reports_prevention_error` checks the inferred prevention behavior under `optional-tests`.
- `tests/aura_container_util.rs::retail_aura_container_apply_icon_accepts_tainted_secret_file_id` exercises the original Retail `ApplyIcon` consumer under `optional-tests`.

## Casting and threat predicate surface

The pinned Retail and PTR `Blizzard_APIDocumentationGenerated/SecretPredicateAPIDocumentation.lua` declares `ShouldUnitSpellCastingBeSecret` at lines 372–385 and `ShouldUnitThreatValuesBeSecret` at lines 415–429. Both return one boolean. Casting requires one unit token; threat values requires two. `ShouldUnitThreatStateBeSecret` declares its second token nilable at lines 398–412.

**VERIFIED** in Forever 1.60.1.70707: `ShouldUnitSpellCastingBeSecret` is registered in both Game and Glue and takes one required unit input. **VERIFIED** in Forever, Retail 12.1.5.70688, and PTR 12.1.7.70668: its compiled secrecy feature is on, and PTR/Retail use the same decision shape. **VERIFIED** in Forever: the query returns false when the unit GUID equals the player's GUID or its owner/controller GUID is the player's GUID; an unresolved unit or any other resolved unit returns true. Strings and numbers are accepted; numbers become unit-token strings. Missing, nil, boolean, table, and function inputs raise the usage error; inaccessible secret inputs raise. Full PTR/Retail parser parity is UNKNOWN. **INFERRED**: the owner/controller match means a unit the player controls or owns.

The simulator resolves existing player, target, focus, and party GUIDs first. **INFERRED simulator inputs** add explicit unit-token-to-GUID overrides and a set of player-owned or controlled GUIDs; both are empty by default. The query does not infer ownership from token spelling. A default unresolved `pet` therefore returns true; a host-mapped pet GUID in the owned/controlled set returns false. The simulator does not model a missing current-player GUID input.

The original cached Forever `Blizzard_UIPanels_Game/Shared/CastingBarFrame.lua` reads the predicate at lines 2–6 and wraps outputs in `GetEffectiveBarType` and `IsInterruptable` at lines 354 and 605. The test above exercises the owned-pet and non-player paths through this consumer.

Threat secrecy still follows the supplied `threat_state_restricted` input. Its native trigger remains UNKNOWN. `CanCompareUnitTokens` uses the existing comparison-permission policy, not the inverse of comparison secrecy. Retail and PTR both enable the `retail-12-0-5` feature that gates the existing threat-state predicate.

## Native restriction surface (clean-room S10–S13, 2026-10-31)

Source: `native-evidence-70688/S13-restriction-surface-summary.md` (behavioural specs only; builds 12.1.5.70688, Forever 70707, 12.1.7 PTR).

- VERIFIED, all three builds: the restricted helper set is closed. It has 25 names: 24 at permission class 7 and `GetForbiddenObjectTable` at class 6. The set is `canaccessallvalues`, `canaccesssecrets`, `canaccesstable`, `canaccessvalue`, `hasanysecretvalues`, `issecrettable`, `issecretvalue`, `mapvalues`, `scrub`, `scrubsecretvalues`, `secretunwrap`, `secretwrap`, `securecopy`, `CreateFromMixins`, `CreateSecureDelegate`, `Mixin`, `RegisterEventCallback`, `RegisterUnitEventCallback`, `UnregisterEventCallback`, `UnregisterUnitEventCallback`, `dropsecretaccess`, `dumpobject`, `securecallmethod`, `settablesecurity` and `GetForbiddenObjectTable`. The simulator registers all 25. `securecopy` and `CreateSecureDelegate` are implemented in `src/lua_api/globals/real/secure_helpers.rs`, not temporary fallbacks. Equal registration names do not establish equal implementation semantics across builds; the VERIFIED delegate mismatches are listed in the audit above.
- VERIFIED: `GetAddOnRestrictionState(type)` returns one byte of a per-type state array, and `IsAddOnRestrictionActive(type)` is `state[type] == 2`. See [restricted-actions-state](restricted-actions-state.md).
- VERIFIED structure, two builds: `canaccessvalue` returns its fallback `1` when the argument is missing or fails its check. INFERRED meaning: fail-open (access allowed). The simulator matches this: `canaccessvalue()` returns `true`. Do not apply one fail-closed default to the whole surface.
- VERIFIED, three builds: `CreateSecureDelegate`, `secretunwrap` and `settablesecurity` refuse while a native singleton is set. They report an error and return `nil`; they do not raise. `secretwrap` has no guard. The singleton and its writer are native UNKNOWN, so the simulator does not model this refusal.
- Earlier S3's type-name-only inference is superseded by the bounded WM61 S3 converter evidence below. No complete accessor path is established.

## Known gaps (current cycle)

- [ ] Threat-values restriction trigger: native UNKNOWN; the shared simulated threat flag is not evidence of native `ShouldUnitThreatValuesBeSecret` behavior.
- [ ] Native S1 §7 describes a generic indexed state-byte predicate path comparing the byte to 2. The mapping of restriction indices and any pairing of a particular predicate with a getter remain unverified; existing `GetSpellCastSecrecy` and `ShouldUnitSpellCastBeSecret` are computed independently.

- [ ] Complete accessor-to-AuraData publication and the remaining secret-operation behavior are UNKNOWN; see the WM61 boundaries below.
- [ ] Restricted-helper refusal guard (the separate singleton-based `CreateSecureDelegate`, `secretunwrap` and `settablesecurity` path returns `nil`): not modelled; the singleton is UNKNOWN. See [secure helpers](secure-helpers.md) for the tainted-construction gate.
- [ ] Production restriction inputs and transitions (WM61 S1).
- [ ] Complete AuraData publication beyond the implemented WM61 S3 field marking; populated points and complete accessor linkage remain UNKNOWN.
- [ ] General operation rules for secrets in tainted code (after S4).
- [ ] Event-dispatch taint rule, which fixes the issue #13 `CompactArenaFrame` case (after S9).
- [ ] `RequiresUnitAuraAccess` predicate string: WM105 §6 shows it is only the fallback when the pending-error lookup finds no report, so it never appears on the aura-denial path. The simulator does not model a no-report denial arm; no native route to it is known.
- [x] Aura rejection condition (WM105): context, then a recorded taint source (a null source permits), then the generated-secrecy-policy flag, then any of restriction entries 0..3 equal to `Active`. `Map` and `Chat` are never read, `Activating` does not match, and the gate shares `GetAddOnRestrictionState`'s array. `forceinsecure()` installs `*** TaintForced ***`, so a forced caller is denied.
- [ ] Helper contracts left open by S7/S18: `canaccesssecrets` signature, `dropsecretaccess`, `mapvalues`, `settablesecurity`, `securecallmethod`, and live `secretunwrap` success outside the bounded S7a EMULATED fragments.

## Out of scope

- Classic profiles beyond [classic-secret-policy](classic-secret-policy.md).
- Non-aura `SecretWhen*` families (unit identity, power, stats, casts, cooldowns), except where they share the same machinery.
- Patching Blizzard Lua to avoid secrets.
