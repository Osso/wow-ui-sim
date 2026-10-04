# Retail 12.0.7 rounds 9–10 independent source re-review

**Verdict: ACCEPT WITH QUALIFICATIONS.** Both specific round-8 regressions are fixed on the Retail path. The stronger claim that duration production contains no addon-replaceable table reads is false. Recommend bounded credit for 20 individually scoped DurationTextBinding-method/factory rows and row 137; retain broad prose row 022 as partial. None establishes historical/native 12.0.7 conformance.

## Review scope and evidence

Date: 2026-10-04. Source: master `d58e3c2c96b8d343438b8dfb0a8b0d132fef1151`; initial git status empty. Inspected `git show` for `c1ded7c63`, `cbe938d38`, `6bf996e5b`, `77e0104bd`, `ed4de74ae`, `3582b545f`, current implementation/tests/specs, cached Blizzard Lua and saved proof. No cargo, tests, builds, simulator, agents or model CLIs executed; no repository/cache-runtime edits. Only this report written.

Verification skill Mode B: substantive source and external wiring inspected; execution evidence is attributed, not independently rerun. Paths below are repo-relative unless absolute. Abbreviations: **B** = `src/c_api/duration_text_binding.rs`; **S** = `src/c_api/duration_text_binding/state.rs`; **C** = `src/lua_api/globals/lua_duration_object/core.rs`; **T** = `tests/patch_12_0_7_duration_text_binding.rs`; **G** = `src/lua_api/frame/methods/widgets/model/model_unit_guid.rs`. Each abbreviation denotes that exact file in citations.

Current master saved output says “test result: ok. 72 passed; 0 failed” (`data/patch-api/evidence/12.0.7-session-2026-10-03/rounds-9-10-master-green.log.txt:80`), including both new binding regressions (:73,75), six GUID cases (:53–58), retained cached consumer (:36), and duration/clock controls. This is integrator evidence, not fresh execution by this reviewer. Worktree proof revisions in R9/R10 results differ from cherry-picked master SHAs; master confirmation prevents treating worktree-only evidence as master proof. Default Retail includes 12.1.0; this is not an executed strict-12.0.7-only profile (`Cargo.toml:108,120–122,149–150`).

## 1. Replaceable duration methods: closed narrowly, not an immutable producer

**The original three-method dispatch defect is closed on Retail.** B:165–166 and :191–193 call captured `durationIsZero`, `durationHasExpired`, `sampleRemaining`, not methods on the duration table. S:132–154 validates the duration and calls shared Rust `read_query_value`, including “`Query::Remaining, modifier`” (:151). Host modifier comes from `BindingSettings`, not an addon method (:148–149). Callbacks are passed during bootstrap (B:381–395) and captured as locals (:20).

**Same timing/clock formula:** real duration queries call `read_query_value` (C:330–345); bindings call that same function. It reads timing once (C:348–379), uses the same `clock_time`, `Timing::span/end` and `duration_value`; remaining calculation is “`span - elapsed`” with common rate scaling (C:278–315). No second binding-side formula was introduced. `git show c1ded7c63` confirms extraction of the existing implementation rather than alternate arithmetic.

**ANY remaining replaceable Lua/table lookup? Yes.** Distinguish three boundaries:

- Retail producer still reads mutable duration-table numeric slots -1/-2/-3 (C:10–12,31–72). `require_duration` checks shared metatable, not immutable host timing (`src/lua_api/globals/lua_duration_object.rs:258–273`). Existing duration allocation creates a Lua table (:333–353). A source-derived, unexecuted counterexample is `rawset(Duration,-2,32)`; timing changes without using the authenticated timing setter. This is inherited core storage, not the replaced-method defect.
- Shared clock resolution reads table field `clock`, then “`state.gettable(clock, Val::Str(key))`” for `time` (C:152–167). SetClock authenticates but stores the clock without native-clock type validation (`src/lua_api/globals/lua_duration_object.rs:298–305`). An accepted protocol clock `{time=12}` can be mutated, or supply a replaceable `__index`; a later read will observe it. Actual manual-clock userdata reads host `ManualClock.time` (`src/c_api/duration_clock.rs:8–12,93–105`), but the producer does not require that userdata. Public timing mutation via genuine methods/clock setters is expected live behavior; arbitrary raw/protocol replacement is a separate, unproved trust boundary.
- B itself retains legacy method lookup in `duration_value_to_text` and `secret_duration_text` (B:40–60), reached by the non-12.0.7 formatting branch (:206 onward). Retail also deliberately resolves the configured formatter's `FormatNumber`, binding fields and FontString `SetText` (B:167,175,196,260–266). These are not native-duration arithmetic, but defeat a literal blanket “no replaceable Lua lookup anywhere” claim.

Consequently **fully closed for replacing GetRemainingDuration/IsZero/HasExpired in the tested Retail branch; not fully closed if Finding 1 is interpreted as immutable host-backed duration/clock storage or all-profile override immunity.** No new scope to repair inherited core storage is implied by this review.

**Regression really fails pre-fix:** T:110–140 replaces methods with constants, first expecting live “`'6s'`” rather than 999 (:119), then proves modifier, expiration, rewind and zero-state behavior. Pre-fix dispatch would format 999 through the fixture formatter, failing that first assertion. Saved RED confirms exact cause: “binding sampled replaced remaining method” (`/home/osso-test/.cache/wow-ui-sim-audit/r9-RED-p1207_binding_ignores_replaced_duration_methods.log:22`), “0 passed; 1 failed” (:29). Current master saved output records pass (:73). This is source reasoning plus inspected historical RED/GREEN, not a new pre-fix run.

## 2. FontString forgery: genuine identity, authentication first, atomic rejection

**Closed for SetFontString.** Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:283–290` declares “`SecretArguments = "AllowedWhenUntainted"`” and “`Type = "SimpleFontString", Nilable = false`”. Public wrapper calls “`authenticateArguments(self, ...)`” before receiver and value checks (B:286–290). S:100–108 authenticates every original argument/extras before pushing decoded values. FontString validation (B:304–305) precedes invoking the original setter (:309); original storage is B:236. Therefore failure cannot store the forged value.

S:119–129 uses `native_frame_id_from_val`, requires a live registry entry, then “`frame.widget_type != WidgetType::FontString`”. The helper reads VM table “`.backing()?`” rather than Lua slot 0 or GetObjectType (`src/lua_api/methods.rs:145–151`). A plain table with a copied token or spoofed method has no native backing; a native Frame/Button/Texture has the wrong registry type. A genuine FontString with a lying GetObjectType still passes. This is simulator frame identity, not a claim of native-client object provenance or forbidden-access parity.

T:179–209 covers forged table, unchanged existing label, unchanged nil on an empty binding, spoofed non-FontString rejection, genuine FontString acceptance despite replacement, real text update, and secret denial. Historical RED says “binding accepted forged FontString” (`/home/osso-test/.cache/wow-ui-sim-audit/r9-RED-p1207_binding_rejects_forged_fontstring_without_storing_it.log:22`) and 0/1 (:29). R9 result transparently records an initial GREEN fixture failure from expecting the wrong error word, then correction to “untainted” and pass (`data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r9-result.md:12–14,52`). That correction did not alter producer behavior.

**Cached consumer compatibility:** scanned cached Retail Lua for SetFontString/CreateDurationTextBinding and found one binding setter call: `Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:176` under `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns`. The method first validates `RequireObjectType("FontString")` (:158–160), then initializes the same object (:163). `Blizzard_AuraContainerUtil.lua:289–292` adds forbidden aspects and “`return object`”; it does not convert it into a plain table. No observed cached consumer supplies a newly rejected impostor. Retained integration test supplies a real created FontString through this actual mixin and checks concrete label '9' plus original label '3' (`tests/duration_text_binding_copy.rs:407–453`); master proof records pass (:36). This covers that path, not every possible forbidden-object combination or third-party addon.

Constructor arguments/direct binding properties/Assign remain ways to populate configuration without invoking this setter (B:108–120,129–146). The fix is not a universal invariant over every nondeclared field/constructor extension.

## 3. Older profiles and ordinary callers

No new availability was added for Classic/pre-120007 Retail: B:339–345 returns early for those profiles. FontString validation and new sampling calls are used only inside the 12.0.7 Lua branch (B:163–194,282–313). Forever retains legacy formatting; its override immunity is not established. The comments-only commit `6bf996e5b` changes inference labels in core, clock and ping, not behavior.

For valid ordinary callers using existing modeled duration and clock, shared computation/clock/rate/zero rules remain the same. Expected changes are ignoring the three replaced duration methods and rejecting forged/wrong-widget FontStrings; genuine FontStrings no longer depend on GetObjectType being truthful. Current default-Retail regression evidence supports this scope.

Do not claim literally no other observable change: `c1ded7c63` also changes shared query evaluation order. Modifier stack conversion now occurs before `read_query_value` reads timing (C:337–351); pre-fix query read timing before `duration_value` parsed the modifier. Invalid/malformed modifier plus inaccessible secret timing can therefore change which error is raised first. That is source-derived, not executed here. Shared extraction affects older profiles' duration core even where binding registration is absent. No alternate-profile execution evidence; existing R9 result explicitly acknowledges shared-core risk (:77). Native defaults/coercion and exhaustive invalid-input error parity remain excluded.

## 4. Duration binding: exact row recommendations

**Scope for every bounded row below:** default-Retail simulator behavior through the named public factory/method, with genuine fixture-created duration and manual-clock objects, modeled numeric formatter and genuine FontString, configuration established through public methods. Includes listed concrete live outputs and authentication behavior only. Excludes raw timing/clock/property tampering, undocumented constructor parameters, complete native handle taxonomy, formatter-handle identity, B31 component/color composition, exhaustive forbidden/GC/scheduler/error combinations, alternate profiles and historical build-68182 conformance. INFERRED defaults, zero/expired precedence, finite/noncoercing numeric policy and secret-output mapping are permitted only as explicit simulator scope, not native facts.

Ledger currently leaves all 21 partial (`data/patch-api/sources/12.0.7-page-coverage.json:459–465,509–515,931–937,963–1105`). This report recommends classification; it does not modify ledger. Factory behavior is narrow enough to earn bounded credit; broad script-object addition prose is not closed by linking those tests.

In table, **bounded** means `bounded-coverage`; **partial** means `partial-development-green`. Method IDs carry prefix `scriptobjects-DurationTextBinding-` unless shown fully.

| Exact row | Status | Bounded claim / remaining reason; source and behavioral evidence |
|---|---|---|
| `prose-undated-022` | **partial** | Added ScriptObject type as a whole remains broader than these selected behaviors: component/options/native identity/access/lifetime contract unproved. Factory userdata existence alone is not whole prose completion. B:108–145; audit spec `docs/specs/duration-text-binding-12-0-7-audit.md:64–80`. |
| `global api-C_DurationUtil-CreateDurationTextBinding-031` | **bounded** | No-argument creation returns distinct userdata with independent configuration; not constructor-extension/default-native parity. B:108–145; T:27–48. |
| `scriptobjects-DurationObject-HasExpired-089` | **bounded** | Live before/end-time and inherited zero-span=true; one bool; modifier/extras authentication. C:318–379; T:337–364. Not exhaustive timing-secret/receiver cases. |
| `CanFormatText-093` | **bounded** | Unconfigured false, configured zero/active/expired eligibility; ignores replaced duration methods. B:156–172; T:27–48,80–104,110–140. Not arbitrary raw-state or formatter protocol identity. |
| `CanUpdateFontString-094` | **bounded** | False without label; eligible real label can receive changing text. B:174–175; T:27–48,80–104,179–209. Not all forbidden states. |
| `Disable-095` | **bounded** | No result; host enabled=false; ticks preserve label despite clock advances. B:177; T:45–48,143–165. |
| `Enable-096` | **bounded** | No result; host enabled=true; eligible tick resumes concrete updates. B:178; T:45–48,143–165. First-update cadence is inferred. |
| `GetDuration-097` | **bounded** | One nil/reference result; public-set duration identity retained through Copy/Assign. B:180; T:27–76; retained GC case `tests/duration_text_binding_copy.rs:161–203`. No universal identity guarantee over field/constructor bypasses. |
| `GetExpiredText-098` | **bounded** | One nil/string/wrapped string readback; configured state copied/reset independently. B:181; T:51–77,249–288. Secret-native provenance excluded. |
| `GetFontString-099` | **bounded** | One nil/real FontString reference; setter failure leaves old reference unchanged. B:182; T:51–77,179–209. Field/constructor/Assign bypasses excluded. |
| `GetFormattedText-100` | **bounded** | RealTime 6s/BaseTime 12s, rewind/expiration/zero, method-override isolation; configured formatter errors propagate; selected secret handoff. B:183–204; T:80–140,217–245,292–328. NOT immutable timing/protocol-clock authority or exact NumericFormatter identity/native formatting. |
| `GetTimeModifier-101` | **bounded** | One host scalar; 0/1 readback, independent copies and live rate-scaled output. B:221; S:45–54; T:51–77,80–140. |
| `GetUpdateInterval-102` | **bounded** | One host scalar; roundtrip/isolation and concrete cadence, including zero. B:222; S:45–54; T:51–77,141–178. |
| `GetZeroDurationText-103` | **bounded** | One nil/plain/wrapped value; configured zero formatting and reset. B:223; T:27–77,291–334. Native zero/default secrecy excluded. |
| `IsEnabled-104` | **bounded** | One boolean; actual Disable/Enable/SetEnabled and independent host state. B:228; S:45–54; T:27–77,141–178. |
| `SetDuration-105` | **bounded** | Genuine duration accepted, wrapped reference authenticated, invalid inputs atomic, tainted args/extras denied first. B:231–233,286–309; S:110–112; T:213–288. Exact unforgeable duration taxonomy not claimed. |
| `SetExpiredText-106` | **bounded** | String/nil and secure wrapped text; zero returns; invalid input atomic, tainted denial first. B:235,286–295; T:51–77,213–288,291–334. |
| `SetFontString-107` | **bounded** | Native FontString identity after all-input authentication, spoof/wrong-widget rejection without storage, concrete label update. B:236,286–309; S:115–130; T:180–210,249–288. |
| `SetTimeModifier-108` | **bounded** | Authenticated 0/1, no results, atomic invalid/nil/table/1.5 rejection, output changes. B:242,286–297; S:58–98; T:51–77,80–140,213–288. |
| `SetUpdateInterval-109` | **bounded** | Authenticated finite nonnegative values; atomic rejection; concrete zero/per-interval cadence. B:258,286–301; S:58–98; T:143–176,213–288. Native range/coercion unproved. |
| `SetZeroDurationText-110` | **bounded** | Authenticated string/nil/wrapped text and wrapped-nil normalization, atomic false rejection, opaque selected zero text. B:259,286–295; T:213–288,291–334. |

Thus **20/21 rows eligible for bounded scopes; 1/21 stays partial**. Unproved broader behavior remains unproved; narrow credit does not silently promote it. If the integrator requires a single whole-capability scope guaranteeing all duration/clock state is host-owned and unreplaceable, retain that aggregate capability partial: these fixes do not meet that stronger standard.

## 5. ModelSceneActorBase:GetModelUnitGUID — row 137

**Recommend bounded-coverage for default-Retail non-rendering getter scope.** G:22–46 rejects secret receiver/all extras before native identity validation, looks up live registry actor metadata, reads Rust `player_model_state.last_unit`, resolves current host identity via `existing_guid_for_unit`, and returns exactly one public string. No Lua UnitGUID/GetObjectType dispatch, new identity storage, or rendering. Existing token assignment is Rust (`src/lua_api/frame/methods/widgets/model/model_unit.rs:38`); host lookup checks existence and reads target/focus/party identity (`src/lua_api/globals/unit_misc.rs:203–234`). Native receiver backing uses `src/lua_api/methods.rs:145–151`. G:32 requires “`Some("ModelSceneActor")`”.

Registration is feature-gated, not merely a test-only helper (`src/lua_api/frame/methods/widgets/model.rs:10–11,653–654`). The 3D permanent gap stays intact (:683–691). Six concrete tests exercise empty/nonnil arity, host mutation/missing target, actor/environment locality, public output even when UnitGUID is secret, forged receivers and every secret argument for secure/tainted callers (`tests/p1207_remaining_model_unit_guid.rs:18–184`). R10 RED was six behavioral failures and GREEN six passes (`data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r10-result.md:7,12,47–59`); current master saved output records six passes (:53–58).

Source row explicitly says “ModelSceneActorBase:GetModelUnitGUID - ret1.ConditionalSecret” (`data/patch-api/sources/12.0.7-api-changes.txt:137`); public output matches that narrow removal, not a new secrecy invention.

**“Per cached declaration” needs qualification.** Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/FrameAPIModelSceneFrameActorBaseDocumentation.lua:138–149` declares no explicit arguments and one nonnil WOWGUID: “`{ Name = "guid", Type = "WOWGUID", Nilable = false }`”. It supplies NO SecretArguments policy. G:9 honestly says “INFERRED NotAllowed: later cache omits the input annotation.” Therefore input-secret rejection is tested simulator policy, not proven compliance with a cached NotAllowed annotation. Empty default and live-token versus bind-time-snapshot are also inferred (G:34–35). Exclude ClearModel lifecycle, wider unit taxonomy, older-feature execution, strict-12.0.7-only execution and native parity. None independently blocks narrow credit.

`77e0104bd` also authors four timeline tests; `3582b545f` correctly leaves their proof open. `tests/p1207_remaining_timeline_color_event.rs:1` requires `retail-12-1-5`, absent from default Retail. R10 records “0 passed; 0 failed” for this filter (:15,48). **ZERO timeline lifecycle proof**, not four passing tests and not row-145 bounded credit.

## 6. Verification summary and merge risks

- **[EXIST] PASS:** inspected substantive B (491 lines), S (154), C (434), G (46), regressions, cached declarations/consumer and result artifacts. Git commit source and current files present.
- **[SUBSTANTIVE] PASS:** shared live duration formula, native backing/type validation, actual Rust host identity lookup. Not stubbed constants/name-only registration.
- **[WIRED] PASS:** binding bootstrap registered by `src/lua_api/env_init/mod.rs:66`; captured Rust callbacks used by Lua formatting; GUID in gated method table (`model.rs:653–654`). Saved master runner names execute the test modules; timeline module explicitly excluded.
- **[ANTI-PATTERN] PASS within inspected producers:** B/S/C/G each has zero TODO/FIXME/HACK/XXX markers; changes contain no empty error swallowing or warning suppression. Legacy formatting alternatives remain real existing code, not commented-out replacements.

**Merge risks today:**

1. Shared core extraction is default-Retail-tested only; alternate-profile error ordering/compatibility unexecuted. Malformed modifier precedence changed. No claim of an older-profile pass.
2. Binding factory/direct configuration, mutable duration timing slots and protocol-clock `time` lookup remain outside the native-setter/three-method fixes. Reject blanket immutable-producer or universally validated-handle claims; no demonstrated secret disclosure is established here.
3. Exact NumericFormatter identity, full component formatting, exhaustive forbidden/access/GC/scheduler error behavior and native historical defaults/secrecy remain unproved. Tested modeled-formatter/public-method scope is acceptable, whole native contract is not.
4. GUID empty default/live-token semantics and NotAllowed inputs remain explicit inferences. Timeline tests are disabled; do not count them as proof or claim strict 12.0.7 timeline availability.

**Overall ACCEPT WITH QUALIFICATIONS:** merge these bounded fixes/getter with the stated exclusions; permit 20 narrow duration rows plus row 137 as bounded recommendations. Broad DurationTextBinding prose and broader immutable-storage/native-conformance claims remain partial. Repository and ledger unchanged by this review.
