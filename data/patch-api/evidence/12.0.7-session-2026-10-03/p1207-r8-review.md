# Independent Retail 12.0.7 R8 source-read review

**Verdict: REJECT against the supplied strict acceptance standard.** Substantial partial development exists; rejection is not a claim that the recorded targeted tests failed. Two explicit boundaries remain unmet: trusted Rust duration production and declared FontString/NumericFormatter identity. Keep all 21 rows **partial-development-green**, not bounded-coverage or historical/native conformance.

Reviewed 2026-10-04 at master `21390f7be`. Read verification skill first; used artifact/source verification. No cargo, tests, builds, simulator, agents/model CLIs, repository edits, or operational changes performed. This report was written early and updated during inspection; it is the only intentional persistent write.

## Scope and evidence provenance

Read `git log --oneline -12`, then `git show` for:

| Commit | Subject |
|---|---|
| `72437c655` | Model Retail duration binding state and authenticated inputs |
| `2acb53f87` | Verify secret text at binding FontString handoff |
| `cc405459c` | Use real duration in retained binding lifetime fixture |
| `92aceac97` | Adapt retained binding controls to duration and numeric formatter contracts |
| `890bea7e3` | Configure formatter in secure binding option-copy fixture |
| `80fdb294a` | Record bounded Retail duration binding proof and exclusions |

Eight current files match every SHA256 in `data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r8-result.md:147–156`. This establishes correspondence with the reported source scope, not fresh execution. The existing result reports **84 passing cases across seven nonempty filters**, and a zero-selected tick filter earning no credit (`p1207-r8-result.md:162–182`). No runtime pass was independently reproduced here.

Citation conventions: repository-relative paths resolve under `/home/osso-test/Projects/wow/wow-ui-sim`. **D** means `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua`; **U** means `DurationUtilDocumentation.lua` in that directory; **O** means `LuaDurationObjectAPIDocumentation.lua` there. **B** means `src/c_api/duration_text_binding.rs`; **S** means `src/c_api/duration_text_binding/state.rs`; **T** means `tests/patch_12_0_7_duration_text_binding.rs`. These abbreviations preserve exact file:line citations below.

The source page says only “Added DurationTextBinding script object type” and lists names (`data/patch-api/sources/12.0.7-api-changes.txt:22,31,89,93–110`). It does **not** establish signatures, coercion, defaults, or secrecy. The later cache supplies declaration text; it does not authenticate historical build 68182. The spec explicitly acknowledges this (`docs/specs/duration-text-binding-12-0-7-audit.md:3`).

## Findings and merge risks

### 1. Rust sampling resolves a replaceable Lua method, not the trusted duration producer

**Acceptance-blocking under the requested proof standard.** S:119–125 reads the host modifier, then does `state.gettable(duration, Val::Str(key))` for `"GetRemainingDuration"` and `call_function_state(...)`. B:164–165,190–192 also calls replaceable `IsZero`/`HasExpired` methods for branch selection.

Source-derived counterexample, **not executed**: configure the real duration/clock fixture, replace `Duration.GetRemainingDuration` with `function() return 999 end`, then format. The Rust bridge selects that function rather than computing the remaining duration from timing/clock state. Host clock changes no longer control the sampled number. A constant method can therefore replace production even though the existing live-clock tests reject a constant implementation when methods are unmodified.

The final spec itself says “trusted dispatch under method replacement is not established by these tests” (`docs/specs/duration-text-binding-12-0-7-audit.md:80`) and leaves its Rust live-production checkbox open (:26). Excluding this case cannot satisfy the user's stronger producer requirement. This is a dispatch gap, **not demonstrated disclosure of secret timing**.

### 2. Declared object types are accepted by forgeable protocols

**Acceptance-blocking for exact type claims.** D:283–290 requires `SimpleFontString`, nonnil. B:303–306 checks only a callable `GetObjectType` reporting `"FontString"`; it never verifies frame identity.

Source-derived counterexample, **not executed**: `Binding:SetFontString({GetObjectType=function() return 'FontString' end, SetText=function() end})` passes the explicit validation. `GetFontString` then returns a Lua table, not the declared SimpleFontString. Empty-table rejection in T:151–154 is insufficient to exclude this impostor.

D:294–302 declares `NumericFormatter`, nonnil. B:307–308 accepts any userdata with a callable `FormatNumber`. T:167–181 deliberately supplies `newproxy(true)` with mock `FormatNumber` functions and expects SetFormatter acceptance. This proves error propagation through a protocol mock, **not NumericFormatter identity**. It contradicts a claim that these checks fully validate the declared handle type and the task's prohibition on presence-only proof.

The gap is honestly recorded in `docs/specs/duration-text-binding-12-0-7-audit.md:69`; disclosure makes the work partial, not exact-type complete.

### 3. Inference labeling is incomplete at the inherited HasExpired producer

The pre-R8 committed spec already states “Retail 12.0.5+ reports `HasExpired=true` regardless of start/clock” and calls the mapping “an explicit simulator inference” (`docs/specs/duration-core.md:50`; also confirmed in `git show 72437c655^:docs/specs/duration-core.md`, line 50). The inherited zero-span branch is just `cfg!(feature = "retail-12-0-5")`, with no `INFERRED` label in `src/lua_api/globals/lua_duration_object/core.rs:342–347`. Nil-as-default modifier acceptance at :318–321 likewise lacks a local inference label despite the cache declaring `Nilable = false` with a default (O:342).

This round did not introduce the zero-span formula. Nevertheless the requested “spec AND producer code” label standard is not fully met by its audited producer. Most **new** binding guesses are properly labeled: B:109,156,160–161,189,194,244,278–279; S:16,70; audit spec:22,28,31,37–41.

### 4. Shared implementation changes exceed the Retail-specific branch

Strict Retail formatting/defaults/setter wrappers and new HasExpired authentication are gated. Shared host-scalar storage, use of the real manual-clock factory, unconditional replacement of the binding factory, and elimination of the former fallback construction are not locally Retail-gated (B:30–39,125–138,318; S:58–66).

Runtime registration still excludes ordinary Classic profiles and pre-12.0.7 Retail: B:342–349 returns early unless Forever or Retail/PTR interface >=120007. Thus this does **not** newly expose the factory to Wrath/Mists/Era/Anniversary. Forever does receive shared changes: clock representation changes from a mutable Lua table to the host manual clock; intervals must now be finite even without `retail-12-0-7` (S:61–62). No earlier-profile execution evidence exists. PTR inherits the Retail feature through Cargo.toml:120–122,150.

### 5. Cached-consumer success is bounded, not complete

Cached `Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:158–183` calls Assign or defaults/formatter, then SetFontString and SetDuration, and optionally SetTextFormat. Its OnLoad creates and retains a binding (:312–314). The modified retained test exercises this actual mixin and concrete labels (`tests/duration_text_binding_copy.rs:407–453`); its recorded pass is meaningful.

The consumer can still use component formatting. Cached D:320 says “The format string may contain '{}' placeholders”; B:203 and :238–240 preserve percent formatting and stored components, not the complete cached component contract. This is a **retained/pre-existing limitation**, excluded by the task's B31 boundary, not a newly dropped method. No cached Lua was edited. Do not extrapolate the retained consumer case to every options/secret/collection/error path.

## 1. Per-row declaration and implementation matrix

All rows recommend **partial-development-green**: recorded targeted behavior and substantive implementation exist, but historical/native evidence, exact identity and/or the stronger production boundary remain open. None earns bounded-coverage in this review. The persisted page ledger remains audit-pending for these IDs (`data/patch-api/sources/12.0.7-page-coverage.json:427,475,895,925–1027`); this report does not change it.

For every row the source-page name/addition matches. “No annotation” below means the cache supplies no SecretArguments policy; it does **not** mean NotAllowed/NeverSecret. Successful setters/Enable/Disable return zero values; getters/predicates return one. Native exact extra-argument rejection is not specified by the retained page; ignored extras are authenticated for the annotated setters.

| Source ID | Cached contract and citation | Implementation, mismatch/gap, status reason |
|---|---|---|
| `prose-undated-022` | D:3–6: `Type = "ScriptObject"`, `ObjectType = "Userdata"`, Environment All; U:21–27 describes automatic duration-derived text. | B:121–141 returns one distinct userdata; scheduler exists. Whole native ScriptObject/type/default/production contract not established. **partial-development-green**. |
| `global api-C_DurationUtil-CreateDurationTextBinding-031` | U:21–27: no declared arguments; one nonnil DurationTextBinding; no input secrecy annotation. | B:107–141,316 returns one independent handle; T:30–47 tests defaults/identity. Two optional constructor inputs remain undocumented simulator extensions, not declared arity parity or authenticated setter substitutes. **partial-development-green**. |
| `scriptobjects-DurationObject-HasExpired-089` | O:335–347: AllowedWhenUntainted; modifier enum, nonnil, default RealTime; one nonnil bool; “reached its end time.” | core.rs:316–356 authenticates first and reads live timing/clock; zero-span true inherited; nil accepted as omitted/default. Zero-span/nil/coercion policy inferred, producer labels incomplete. T:283–300. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-CanFormatText-093` | D:22–32: zero explicit args; one nonnil bool; “enough configuration to produce formatted text”; no policy annotation. | B:159–172 checks duration, zero/expired text, formatter. Replaces constant true; T:37–45 changes eligibility. Exact eligibility/precedence and replaceable query methods remain inferred/unproved. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-CanUpdateFontString-094` | D:36–46: zero args; one nonnil bool; enough configuration to update FontString; no policy annotation. | B:174–175 combines eligibility and SetText presence. Real label updated in T:108–130; forged FontString protocol remains accepted. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-Disable-095` | D:74–80: zero args, no returns; “Disables automatic updates”; no policy annotation. | B:177 delegates host enabled=false; T:46 and :119–122 demonstrate zero returns and unchanged label across ticks. Scheduler collection/error behavior unproved. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-Enable-096` | D:83–89: zero args, no returns; “Enables automatic updates”; no policy annotation. | B:178 sets host enabled=true; T:47,:121–124 shows resume. Immediate cadence invalidation is an explicit simulator inference. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-GetDuration-097` | D:92–103: zero args, one nilable LuaDurationObject; no policy annotation. | B:180 returns configured duration; T:34–35,:55–75 and retained-copy fixtures assert identity. Default nil inferred; unsupported constructor/direct-field paths can bypass setter validation. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-GetExpiredText-098` | D:106–117: zero args, one nilable string; no policy annotation. | B:181 returns configured VM value; T:58,67,200 tests text/opaque wrapper. Nil/default and getter secret provenance not historically established. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-GetFontString-099` | D:120–131: zero args, one nilable SimpleFontString; no policy annotation. | B:182 returns reference; T:34–35,:55,65 and copy GC control prove real-handle retention. Forged table from SetFontString produces an out-of-contract result. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-GetFormattedText-100` | D:134–145: zero args; one nonnil string, `ConditionalSecret = true`; no input policy annotation. | B:183–205 returns configured/computed string or explicit error; T:88–102 and :233–274 verify changing text and wrappers. Mutable query dispatch, protocol formatter identity, native defaults/rounding/secrecy remain open. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-GetTimeModifier-101` | D:179–190: zero args; one nonnil DurationTimeModifier; no policy annotation. | B:221 reads independent host setting via S:45–54; T:56,66,94–95,139–148 varies modifier and output. Default/coercion historically unverified. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-GetUpdateInterval-102` | D:193–204: zero args; one nonnil number; zero updates every tick; no policy annotation. | B:222, S:45–54 reads live host interval; T:57,66,125–130 proves zero cadence. Finite/nonnegative range and exact cadence boundaries inferred. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-GetZeroDurationText-103` | D:207–218: zero args, one nilable string; unconfigured or zero-span text; no policy annotation. | B:223 returns configured text; T:35,67,202 and :268–274 demonstrate nil/plain/opaque values. Native secret getter tagging/defaults unverified. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-IsEnabled-104` | D:236–247: zero args, one nonnil bool; automatic update status; no policy annotation. | B:228 returns host boolean; T:37–47,:70–77,:119–124 changes it. Not constant; lifecycle/native defaults remain inferred. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-SetDuration-105` | D:250–258: AllowedWhenUntainted; one nonnil LuaDurationObject, zero returns. | B:230–232,287–313; S:113–115 invokes require_duration; lua_duration_object.rs:258–273 checks the actual duration metatable. All arguments/extras authenticated before validation. T:194–219. Real reference and numeric rejection supported; exact native proxy identity/constructor extensions unproved. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-SetExpiredText-106` | D:272–280: AllowedWhenUntainted; one nilable string, zero returns. | B:235,287–295 authenticates first, retains original wrapped string, normalizes wrapped nil. T:160,165–166,194–219. Strict noncoercion and wrapper normalization inferred. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-SetFontString-107` | D:283–291: AllowedWhenUntainted; one nonnil SimpleFontString, zero returns. | B:236,287–290,303–306 authenticates first but accepts forged GetObjectType protocol. Concrete identity tests do not establish declared type enforcement. **partial-development-green**, exact type unresolved. |
| `scriptobjects-DurationTextBinding-SetTimeModifier-108` | D:329–337: AllowedWhenUntainted; one nonnil DurationTimeModifier, zero returns. | B:242,287–297; S:64–66 accepts 0/1. T:56,159,194–225 tests values, output, authentication and atomic rejection. Nil rejected unlike HasExpired's optional modifier; native coercion inferred. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-SetUpdateInterval-109` | D:349–357: AllowedWhenUntainted; one nonnil number, zero returns; zero every tick. | B:258,287–302; S:60–62 validates finite/nonnegative, writes live host state. T:156–158,194–225,125–130. Extra range restriction explicitly inferred, not proven native. **partial-development-green**. |
| `scriptobjects-DurationTextBinding-SetZeroDurationText-110` | D:360–368: AllowedWhenUntainted; one nilable string, zero returns. | B:259,287–295 authenticates all first, preserves wrapped string; T:161,165–166,202–219,268–274 tests behavior. Nil acceptance supported; secrecy normalization and precedence inferred. **partial-development-green**. |

No selected input declaration says NeverSecret/NotAllowed. `ReturnsNeverSecret` on other methods is an **output** annotation, not authorization to invent an input rejection policy.

## 2. Authentication order in every changed producer

1. **Public selected binding setters**, plus wrapped Assign/SetEnabled/SetFormatter/SetClock: B:287–290 counts arguments, calls the authentication callback, then validates the decoded binding receiver. S:102–109 first calls `duration_clock::authenticate_arguments`, whose :119–123 executes `unwrap_secret` for each original argument; only afterward does it push decoded copies. The original argument count preserves trailing nils for `unpackArguments`. Type/range checks B:291–310 precede original setter invocation and mutation :313. Text restores the original VM wrapper only after authentication/type validation :292–295. **PASS for the all-input-before-validation ordering on these public methods.**
2. **SetDuration validation callback**: S:113–115 itself performs no authentication; its public wrapper already authenticated the input. It uses actual duration identity, not a competing provider. No bypass claim is made for direct internal callback invocation.
3. **Host scalar read/write/create**: S:14–27 initializes scalar-only userdata; :45–54 reads state live. Write :78–98 reads/validates the internal key **before** unwrapping the value, validates the setting, then mutates. This is not independently compliant as a general AllowedWhenUntainted public API; it receives an internal settings handle/static key and decoded public input after the outer wrapper. Direct scalar property assignment reaches this narrower internal path without all-input authentication; it is an existing nondeclared field extension, not public-setter proof.
4. **Remaining-duration sampling**: S:119–125 validates host settings, looks up the duration method, then invokes it. No all-input declaration applies to this private callback; the duration reference was normalized at SetDuration. Secret timing access is guarded in B:163–165,187–188 and underlying core. The issue here is replaceable dispatch, not missing public setter authentication.
5. **HasExpired**: core.rs:325–332 enters `authenticate_activity_modifier` under retail-12-0-7 before read_timing. That helper (:316–321) authenticates receiver/modifier/extras, then unwraps and validates the modifier. No state mutation; clock/timing read follows. Decoded receiver is not substituted into the stack; secure wrapped receivers remain an untested compatibility case, not covered by the wrapped-modifier fixture.
6. **Factory / retained bootstrap paths**: no input SecretArguments declaration for the factory (U:21–28); retained constructor inputs are not setter equivalents. Enable/Disable carry no annotation but their internally called SetEnabled wrapper authenticates receiver/extras actually forwarded. Copy/default/getter methods have no added input-policy claim. Existing secret helpers B:421–444 were not changed; output wrapper preservation now avoids double wrapping at B:264.

No taint clearing, parallel provider, fallback-on-formatter-failure, or wrapped constant numeric producer was added on the Retail branch. Configured zero/expired strings are documented output choices, not a shim alternative provider. Legacy non-Retail formatting alternatives remain B:206–218; they were not removed wholesale.

## 3. New-test strength: constants, old code and implementation shape

All nine new tests invoke public Lua APIs through WowLuaEnv (T:20–23), not source-substring checks. The recorded RED was 1 pass / 8 behavioral failures; cadence was already supported (`p1207-r8-result.md:27–32,179,186`). Tests are not nine independent proofs of new behavior.

| New test, T lines | Constant / pre-change assessment |
|---|---|
| Defaults/exact arity, :27–48 | Individual default assertions can pass constant nil/true getters. Whole case additionally changes zero text and enabled state and compares two bindings; unconditional getters cannot pass. Pre-change defaults/eligibility fail. |
| State roundtrip/Copy/Assign/reset, :51–80 | Distinct handles, scalar mutations and resetting only one object reject constant results/shared configuration. Pre-change reset/default and modifier-format result fail. Does not prove all retained methods merely by its name “preserve_surface.” |
| Live clock/rate/zero/expiration, :83–109 | Multiple clock values, rewind, changed duration/rate and distinct text reject a constant numeric/string producer. Pre-change ignores binding modifier in duration formatting and zero/expired branch selection. No method-replacement case. |
| Automatic cadence, :112–133 | Concrete label changes/holds/disable/resume/zero interval reject a fixed label. **Could and did pass the pre-change scheduler** according to retained RED evidence; regression proof, not new producer proof. |
| Environment locality, :136–149 | Different environments/configuration produce 12s versus 7s; constants/shared state fail. Old modifier behavior fails this setup. |
| Invalid setters/atomicity, :152–185 | Always-error methods could satisfy negative subassertions, but valid setup/text writes and unchanged configured state constrain the whole case. Old permissive setters fail. Protocol proxies prove formatter-error propagation, not declared formatter identity. |
| Authentication/extras, :188–227 | Secure wrapped acceptance plus tainted rejection, later-extra diagnostic and successful plain tainted mutation reject always-accept/always-error stubs. Old setters do not authenticate all extras. Error-message substring checks assert observable failure category, **not source shape**. No wrapped binding receiver success case; positive secret checks do not cover every wrapper/setter permutation. |
| Secret timing/text/handoff, :230–273 | Tests secret result, tainted inability to read it, changing expiration/zero output, and concrete secure label text; one wrapped constant cannot satisfy all transitions. Old binding's plain formatted result fails. Public SetText interception records actual value and then calls the real setter; this is boundary/side-effect proof, not an internal-call-count assertion. |
| HasExpired, :276–304 | Before/end-time false/true rejects constant bool. Zero-span true alone passes old core; invalid modifier and secret-extra-before-invalid-receiver checks distinguish added authentication. |

No implementation-shape assertions found in these changes. `type(...)=userdata`, public handle equality, exact return counts and metatable-configured protocol doubles concern the external Lua contract. However **protocol doubles are insufficient positive type proof**. Coverage also omits trusted dispatch replacement, forged FontString rejection, exact native formatter handle rejection, Retail weak-scheduler collection and exhaustive error-handler behavior.

## 4. Changed existing assertions and removed behavior

### startup_globals.rs

`src/loader/tests/wow_api_globals/startup_globals.rs:111–139,167–169,200–208`:

| Changed expectation | Grounding and judgment |
|---|---|
| Zero-span HasExpired false → true (:111) | Existing **pre-R8 committed** duration-core spec:50 supports it; inherited producer core.rs:343–345 unchanged. Cached O:338 says “Returns true once the duration has reached its end time”; source page only lists addition. **Not weakened to excuse new code; simulator-spec correction, not native proof.** |
| Default GetDuration nonnil → nil (:116) | Cache getter is nilable (D:102) and reset clears duration (D:342), but neither proves constructor default nil. Newly recorded INFERRED policy, audit spec:22. **Expectation changed to the proposed contract; legitimate only as explicit inference.** |
| Default CanFormatText true → false (:117) | “Enough configuration” (D:24) supports state-dependent eligibility, but exact empty-state behavior remains inferred. Stronger configuration control, not historical default proof. |
| Default GetFormattedText "0" → explicit pcall failure (:121) | Native nonnil string return declaration does not itself require error on unconfigured binding. Audit spec:28 explicitly chooses this instead of fabricated output. **Contract change, not independently established native behavior.** A blanket-error implementation could satisfy this assertion alone; configured output assertions prevent that for the whole fixture. |
| Numeric duration 10 / "10" → real DurationObject / "10s" (:127–134) | D:257 explicitly requires LuaDurationObject and D:301 NumericFormatter. SecondsFormatter abbreviation/whitespace configured explicitly. Identity and concrete output checks remain; fixture becomes source-shaped rather than testing a pseudo-duration extension. |
| Modifier 1.5 → enum value 1 (:139–140) | D:336 requires DurationTimeModifier; core formula spec:46 defines RealTime=0/BaseTime=1. Positive assertion now uses a valid enum. Old arbitrary-number extension deliberately no longer accepted; T:159 separately asserts 1.5 rejects. |
| Numeric rebind 7 / label "7" → mutate real duration / label "7s" (:167–169) | Declared type plus explicitly configured formatter; still observes live reconfiguration and real label output. Preserved behavioral strength, different fixture format. |
| Retained GetDuration == 17 → == actual duration handle (:200–208) | Cached duration type and existing identity/lifetime intent support it. GC call and retained identity assertions survive. Stronger handle fidelity, not dropped GC proof. |

### duration_text_binding_copy.rs and lib availability control

- `tests/duration_text_binding_copy.rs:140–152` adds actual duration/manual-clock and NumericRuleFormatter fixtures. Seven ungated groups retain their handle equality, GC weak-resource retention, copy independence, component-copy and cached-consumer checks (:159–453). **The author handoff incorrectly called all copy controls Forever-gated** (`handoff-p1207-b08-b10.md:919`); the result corrects that (`p1207-r8-result.md:191`).
- GC group's formatter mutation changes table `.Format` replacement to shared NumericRuleFormatter breakpoint mutation (:170–198); exact output remains `retained:17` then `shared:17`. This stops proving arbitrary callback replacement but still proves shared formatter identity/state. Declared NumericFormatter justifies replacement of the incompatible fixture.
- Assign/receiver and independent-copy groups replace numeric 8.2/3.2 with actual durations, retaining `9`, `4`, `8.2` and independent-label expectations (:239–245,308–320). No existing output assertion relaxed.
- Secure option-copy adds a real duration and configured formatter (:260–274; commit `890bea7e3`); binding/option identity and actual `7` label assertion remain. Source-shaped setup, not a skipped failure.
- Atomic Assign group replaces numeric GetDuration==6 with reference equality and retains `kept 6` before/after rejected assignments (:369–384). Empty-config equality of formatted strings changes to both CanFormatText false and both GetFormattedText calls failing (:397–400). **This adopts the new inferred empty-format policy**, not a cache-required error. It no longer tests equality of fabricated default output; invalid-Assign atomicity remains covered.
- Cached-aura fixture changes numeric 2.2/8.2 to real durations (:419–449); actual mixin execution, option copies, `3`/`9` output, curve and source independence assertions remain. No consumer edit or test exclusion.
- The existing lib availability test also changes table formatter/numeric duration and `value:12` to real duration/SecondsFormatter and `12s` (B:461–472). Binding/modern-method profile availability assertions (:479–480) unchanged. Declared type and explicit formatter support the fixture adaptation; it does not establish older-profile execution.

**Method inventory:** no existing binding method removed. Copy, Assign, SetClock, SetFormatter, percent SetTextFormat, color methods, scheduler and same factory remain B:139–318. Behavior **was** removed/changed under Retail: numeric pseudo-durations, table/function formatters, arbitrary modifier values, implicit default duration/text, unconditional CanFormatText, and preserving FontString on SetToDefaults. These are explicit compatibility changes, not “no behavior changed.” Types/reset support most changes; exact defaults/error/range rules remain INFERRED. Legacy alternatives remain behind the non-Retail branch.

The new secret-test follow-up (`2acb53f87`, T:246–255) changes wrapper-at-GetText assertion to wrapper-at-public-SetText plus plain secure GetText. Existing `src/lua_api/frame/methods/text_attribute_event/text.rs:571–606` checks read permission then pushes a plain string. **Supported by existing implementation, not native secret-return proof**. Boundary scope narrowed transparently; tainted widget-readout parity is not established by this test.

## 5. Artifact verification and proof limits

- **[EXIST] PASS:** all eight claimed source/spec/test files present; SHA256s match the recorded final artifacts. Six requested commits present; initial git status --short empty.
- **[SUBSTANTIVE] PASS:** S has 125 lines, real scalar userdata and read/write/auth/sample functions; B has 483 lines with one provider and scheduler; T has 304 lines and nine concrete public-API tests. Not empty stubs. Exact-type and trusted-dispatch gaps above prevent full contract acceptance.
- **[WIRED] PASS for runtime:** `src/lua_api/env_init/mod.rs:66` registers provider; B:365–384 passes new callbacks into bootstrap; B:388–390 roots scheduler; `src/lua_api/on_update.rs:60` ticks it. `build.rs:54–60,83–96,463–470` discovers top-level test files and writes the module manifest; `tests/integration.rs:1` includes it. Existing result documents execution of the new filter. Test execution is attributed evidence, not newly verified here.
- **[ANTI-PATTERN] PASS for marker scan:** added lines across six diffs contain 0 TODO, 0 FIXME, 0 HACK, 0 XXX. No new empty catch/error suppression identified; B:331–335 routes update errors. Retained non-Retail fallback branches are not evidence of strict Retail parity.

**Overall strict verification: FAIL.** Public setter authentication order is credible, and most fixture adaptations preserve useful behavioral proof. Merge today would nevertheless retain forgeable declared types and replaceable numeric production, plus knowingly inferred defaults/secrecy and unexecuted shared-profile changes. The reported 84 passes justify partial-development-green only. Native historical claims, bounded row promotion and the stronger producer/type completion claim are rejected. No repository or ledger changed by this review.
