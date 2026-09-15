# API Contract Probe

## Manual StatusBar fill style

`/apicontract statusbar-fill <label>` stays outside `all`. It creates one unnamed `CreateFrame("StatusBar", nil, UIParent)` and immediately attempts `Hide`, before recording the fresh `GetFillStyle()` default. For each published `Enum.StatusBarFillStyle` name `Standard`, `StandardNoRangeFill`, `Center`, `Reverse`, an accessible finite scalar value permits one `SetFillStyle(value)` followed by two independent `GetFillStyle()` observations. No enum numbers are substituted. Missing or throwing setters do not suppress those getters. Missing, restricted or nonscalar enum entries are recorded but not passed to setters.

Object, method and result access is guarded before inspection. Exact arity/nil slots and up to sixteen scalar positions are retained; objects and errors remain opaque. Strings retain at most 256 bytes; ten snapshots cap frame creation. Constructor or Hide failure aborts fill operations and retains no frame reference. A failed Hide is `visibility-unconfirmed`: the addon cannot guarantee disposal or invisibility of a client-owned frame when hiding fails. No Show, sizing, layout, rendering, destruction or other setters are used.

Pinned `SimpleStatusBarAPIDocumentation.lua` declares the getter and enum setter, not native defaults or successful roundtrips. Addon execution is tainted; this recorder assumes no untainted setup and performs no restricted-context or invalid-input experiments. Native validation, coercion and default behavior remain pending. Seven separate local actual TOC/slash fixtures are recorder proof only:

```text
luajit docs/addons/ApiContractProbe/tests/statusbar_fill.lua docs/addons/ApiContractProbe
```

## Manual raid-marker observations

`/apicontract raid-markers <label>` stays outside `all`. It calls `CanBeRaidTarget(unit)` twice for each of `player`, `target`, `focus`, `pet`, `party1`, `party2`, `nonexistent`, `invalid-unit-token` and the empty token; `IsRaidMarkerActive(index)` twice for each index 1–8; and `IsRaidMarkerSystemEnabled()` twice with no arguments. All 36 calls are independent: missing APIs never gate other observations. Repeated results remain raw, not stability comparisons.

Pinned `RaidMarkersDocumentation.lua` supplies these argument shapes, not native results. `GetRaidTargetIndex` is excluded because it declares secret returns. No setter, clear, place or remove API runs; no permission or security conclusions are recorded. Capture naturally available states with manual labels; native output and populated world-marker fixtures remain pending.

Accessibility precedes inspection on every call. Exact arity, nil slots and up to sixteen scalar positions are retained; strings stop at 256 bytes, with truncation explicit. Objects and errors remain opaque. Shared ten-snapshot limit applies. Six separate local TOC/slash fixtures test raw repeated results, independent failures, redaction, arity and bounds—not native conformance:

```text
luajit docs/addons/ApiContractProbe/tests/raid_markers.lua docs/addons/ApiContractProbe
```

## Manual color curve state

`/apicontract color-curves <label>` is excluded from `all`. Own `CreateColorCurve` and `CreateColor` inputs, in insertion order, are `(x; r,g,b,a)`: `(0; 0,1,.25,1)`, `(32; 1,0,.25,.75)`, `(-16; 0,.25,1,.5)`, `(48; 1,.75,0,.25)`. No enum type is assigned or guessed.

Empty and populated snapshots record type/count/secret flag/points, indices `-1,0,1,2,3,4`, and both evaluations at `-17,-16,0,16,32,48,49`. Copy observations surround adding `(16; .5,.25,.75,.5)` to the copy and clearing it; both original and copy are captured after each operation. The original is then captured before/after `SetToDefaults`, even if copying aliased it. Results are observations, not assertions of defaults, order, interpolation or isolation.

Returned tables expose only raw `x/y/r/g/b/a` fields and four array entries, with three levels of table traversal; userdata stays opaque. No returned `GetRGBA`, `__index`, equality or string methods run. Guarded methods on owned curves are allowed. Exact arity and 16 positions are recorded; larger tuples are marked truncated. Ten snapshots maximum; errors stay opaque and missing constructors fail closed.

Cached color-curve/CurveUtil/base documentation and `Blizzard_SharedXMLBase/Color.lua` guide these calls, not native expectations. Userdata field representation, native captures, security semantics, removal/replacement and alternate interpolation types remain gaps. Local fixture command:

```text
luajit docs/addons/ApiContractProbe/tests/color_curves.lua docs/addons/ApiContractProbe
```

## Manual scalar resource scale observations

`/apicontract resources <label>` captures player, target, focus, pet and nonexistent units; it is excluded from `all`. Records raw health/max, power/max/type, omitted-curve and explicit-nil percent calls. Health curve argument 3 follows explicit `usePredicted=false`; the default call is separate. Power uses default power type (`nil`), with separate explicit false/true `unmodified` calls and curve argument 4. No observed power ID is reused or guessed.

One scalar curve records inputs `(0,0), (.5,10), (1,20), (25,30), (50,40), (100,50)`. Construction explicitly selects accessible `Enum.LuaCurveType.Linear`; missing enum/factory/SetType fails closed for curved queries while underlying observations remain available. Results use scalar-only recording, exact arity and the existing 16-position bound, never scale classification or returned-object methods.

Capture naturally available partial-resource states with labels and matching client build. Prediction, secondary powers, color curves, secret contracts and actual native scale remain pending. No events or gameplay changes are triggered. Ten-snapshot limit applies. Local fixtures distinguish fake normalized/percentage results; they are not native evidence.

Manual native observations for scalar curve points, `UnitSexBase`, unit names/realms, finite numeric formatting, publication/CVars and event payloads. Complements [AuraDispelCurveProbe](../AuraDispelCurveProbe/README.md). Neither recorder infers native contracts from fixture outputs.

## Scalar curve state

`/apicontract curve-state <label>` is manual-only, excluded from `all`; existing `curves` is unchanged. It creates empty and unsorted duplicate-point scalar curves using `(30,4), (10,7), (20,2), (10,9)`, attempts the discovered `Enum.LuaCurveType.Linear`, and records type/count/points/secret-flag plus evaluation at `-1,0,10,15,20,30,31` before and after reset. Setter failures and missing enum values are explicit observations.

A separate populated curve is copied; both original and copy are observed after adding `(40,11)` and after clearing the original. No expected defaults, duplicate policy, interpolation or copy isolation is assumed. Method results are scalar-only except existing bounded point/GetXY inspection; errors remain opaque. Four returned points and eight result positions are retained. Ten snapshots maximum. Color curves and secret semantics remain pending; `SetPoints` and `RemovePoint` have the separate manual experiment below. No native execution is claimed.

## Scalar point removal and replacement

`/apicontract curve-edit <label>` is manual-only and excluded from `all`. Each of six `RemovePoint` inputs (`-1,0,1,2,3,4`) gets a fresh scalar curve populated with `(30,4), (10,7), (20,2)`. Two more fresh curves receive `SetPoints`: an empty array, and the unsorted duplicate sequence `(30,4), (10,7), (20,2), (10,9)`. Every replacement point comes from the actual `CreateVector2D(x,y)` constructor; missing, throwing, inaccessible or invalid constructor results prevent that replacement. No plain-table coercion is assumed.

Each case records definitions, `GetPointCount`, bounded `GetPoints` and `Evaluate(15)` before and after the attempt, plus mutation return arity and opaque errors. Existing bounds retain four points, eight return positions and ten snapshots. Numeric out-of-range controls are observations, not expected rejection rules; default curve interpolation is not inferred. Curves and replacement vectors are independently constructed for every case; no returned-point alias or copy semantics are assumed.

Pinned `LuaCurveObjectAPIDocumentation.lua` declares `RemovePoint(index: luaIndex)` and `SetPoints(point: table<vector2>)`; `Blizzard_SharedXML/Vector2D.lua` implements `CreateVector2D` with `x`, `y` and `GetXY`. These sources guide construction, not native indexing, duplicate/order or mutation results. Local fixtures retain distinct zero-/one-based removal and replacement ordering. Native execution, invalid-type coercion, color curves and security semantics remain pending.

## Target

Interface `120100` follows the locally pinned retail `12.1.0.69497` source, not a newly observed desktop build. Install only on a matching client. No installation or native execution has been performed by this task.

## Capture

1. Copy this directory to the matching client's `Interface/AddOns/ApiContractProbe`, then enable it.
2. Run `/apicontract all normal` to capture curves, sex, names, numbers, casts and publication. Use `/apicontract curves`, `/apicontract sex <label>` or `/apicontract names <label>` to capture separately.
3. For sex comparisons, select real targets or repeat during an independently obtained transformation, labeling the scenario. The addon does not alter units or gameplay.
4. Run `/reload` or log out to flush SavedVariables.
5. Retain `WTF/Account/<ACCOUNT>/SavedVariables/ApiContractProbe.lua` with scenario notes. Keep the raw artifact unchanged.

Ten snapshots maximum; further calls increment `dropped`. No automatic captures. `all` includes publication but does not start event listeners.

Additional commands:

- `/apicontract publication after-login` records configured symbol and CVar observations. Repeat with labeled phases after manually loading relevant addons; no addon is loaded by this recorder.
- `/apicontract events-start before-cast` registers the configured events and records real subsequent deliveries.
- `/apicontract events-stop` unregisters listeners. Event recording is capped at 256 entries; excess deliveries increment `droppedEvents`.

`AuditTargets.lua` contains 199 publication/CVar targets and 88 event-registration targets tied to the blocker snapshot hash. Registration errors, missing parents and missing members are distinct observations. Event payloads retain arity and nil slots, with at most eight summarized values; tables remain bounded observations, not full structure captures. No native actions, purchases, transitions or account mutations are triggered. A symbol missing at one phase does not prove removal; a listener with no deliveries does not prove event absence. These shared recorders do **not** complete all associated behavioral plans. See the [full preparation inventory](../../baselines/native-probe-preparation.json). Start a new capture batch by moving the saved file aside while logged out.

## Manual abbreviations

`/apicontract abbreviations <label>` is excluded from `all`. It reuses the same 25 finite inputs as `numbers`, calling `AbbreviateLargeNumbers(number)` and `AbbreviateNumbers(number)` with options omitted, plus one `GetLocale()` observation. No locale changes or formatting assumptions are made.

Scalar-only capture preserves exact return arity, nil positions and accessible string bytes, with sixteen retained positions and 256 bytes per string; larger results are marked truncated. Returned objects and errors stay opaque. Access checks precede inspection; missing functions or failed guards fail closed. Each snapshot allows at most fifty abbreviation calls and one locale call; the shared ten-snapshot cap applies.

Pinned retail `LocalizationDocumentation.lua` declares each number argument and optional `NumberAbbrevOptions`. This slice does not create options/configuration tables or formatters. Native output, breakpoint/options/configuration behavior and table producers remain separate gaps.

Separate local fixtures (not native evidence):

```text
luajit docs/addons/ApiContractProbe/tests/abbreviations.lua docs/addons/ApiContractProbe
```

## Manual heal calculator

`/apicontract heal-calculator <label>` is manual-only, excluded from `all`. Each snapshot attempts two fresh `CreateUnitHealPredictionCalculator()` calls with no arguments. On each accessible object, query `GetHealAbsorbMode`, `GetHealAbsorbClampMode`, `GetDamageAbsorbClampMode`, `GetHealAbsorbs`, and `GetDamageAbsorbs` twice, in that order. Repeated observations remain separate; no equality or stability inference is performed.

Record constructor and getter exact arity, nil positions and at most sixteen scalar/opaque positions, marking truncation. Object access precedes method lookup; function and result access precede inspection. Returned objects and errors remain opaque. Missing, restricted or invalid constructors prevent getter calls. At most two constructor and twenty getter calls per snapshot; the shared ten-snapshot limit applies.

Pinned retail `UnitDocumentation.lua` declares the no-argument constructor; `UnitHealPredictionCalculatorAPIDocumentation.lua` declares three mode getters and two `(amount, clamped)` getters. These signatures guide calls, not expected values. No unit variants, population, setters, reset/default transitions, security experiments or native execution. Fresh default values and populated behavior remain unverified.

Separate local behavioral fixtures:

```text
luajit docs/addons/ApiContractProbe/tests/heal_calculator.lua docs/addons/ApiContractProbe
```

## Producer cast durations

`/apicontract cast-durations <label>` is manual-only, excluded from `all`. Queries `UnitCastingDuration`, `UnitChannelDuration`, and `UnitEmpoweredChannelDuration` with hold omitted/false/true for player, target, focus, party1, nonexistent, invalid-unit-token and empty tokens. Record exact arity, nils and at most 16 positions; larger tuples are marked truncated.

For accessible returned duration objects, this controlled experiment calls only `GetTotalDuration`, `GetElapsedDuration`, `GetRemainingDuration`, `GetElapsedPercent`, `GetRemainingPercent`, `GetStartTime`, `GetEndTime`, `GetClockTime`, `GetModRate`, and `HasExpired`. Object and method access are checked before lookup/call; method results stay scalar/opaque, errors are not stringified. No setters, Assign, Reset, object equality or native identity inference.

Capture `active`, then a later `completed` or `interrupted` observation using actual externally produced states. Each capture re-queries up to 28 objects from the previous duration capture and labels them with that capture's `observationRef`; these are observation labels, not native object IDs. Only the new snapshot's first 28 accessible object occurrences are retained, with excess marked `retention = "limit"` and counted. Seven units × five calls can exceed this limit. Runtime references never enter SavedVariables, and reload clears retention. Ten total snapshots per batch; no timers or synthetic casts. Native tracking versus frozen behavior remains unknown until these observations are collected on the matching client.

## Plain-function callbacks

Run `/apicontract callbacks-start <label>`, produce a real `UNIT_HEALTH` transition externally, then `/apicontract callbacks-stop`. This manual mode is excluded from `all`; it never fires events or changes gameplay. It registers only its own global callback and its own `player` unit callback, preserving their identities for removal.

Each of at most ten sessions records build/time/label, separate registration/removal results and up to 128 deliveries shared across the two callbacks. Payloads retain exact arity and up to 16 scalar-only positions (including nils); objects remain opaque, inaccessible values are redacted, strings retain the existing 256-byte bound. Excess deliveries increment `dropped`; stop still attempts cleanup. Callbacks themselves never enter SavedVariables.

`registration-incomplete` preserves independent failures/refusals. A throwing registration may already have installed its callback, so stop still attempts removal. Missing, throwing, refused or inaccessible cleanup results retain the identity for another stop attempt and report `cleanup-incomplete`, not success. A successful stop disables recording even if an old callback is invoked later. Repeated start while a session needs cleanup is rejected; stop then start creates one new session. Missing access APIs prevent registration entirely. A nonthrowing zero-return call is recorded as accepted; this is not independent evidence that native registration took effect. Inspect real deliveries and raw results.

Native execution remains pending. FunctionContainer wrappers, duplicate/order/alias behavior, callback-time mutation, security and broader producer coverage are outside this bounded experiment. This is partial preparation, not native conformance proof.

## Observations

**Curves:** creates an empty curve and another with insertion order `(30,4), (10,7), (20,2)`. Records `GetPoints`, `GetPoint` at `0,1,2,3,-1,4`, return arity, accessible `x`/`y` and `GetXY`, and repeated-point equality. On a separate curve, attempts `GetPoint(1).x = 91` and records another lookup. Index 1 may be invalid; rejection is evidence, not failure of the experiment. Array inspection is bounded to four entries, return inspection to eight values, and object nesting to two levels.

**Names:** `/apicontract names same-realm-party1-cross-realm-party2` records `UnitName` (`name`) and `UnitNameUnmodified` (`unmodified`) under `names.units` for `player`, `party1`–`party4`, `target`, `nonexistent`, `invalid-unit-token` and the empty string. Every token is queried without an existence check. Exact return counts and positional nils distinguish zero returns, nil realms, empty realms and explicit realm text; results are not normalized. Establish actual same/cross-realm membership independently, then label each manual snapshot with the relevant slots and scenario. Unknown/invalid token spellings are fixed test inputs, not assumptions about native results. No party, target or gameplay changes are made. Name/realm captures contain character identities; retain them privately.

**Numbers:** `/apicontract numbers <label>` records `GetLocale` under `numbers.locale` and 25 ordered `numbers.samples` with `input`, `floor` (`C_StringUtil.FloorToNearestString`) and `round` (`RoundToNearestString`). Corpus: `-2.5,-1.5,-0.5,0,0.5,1.5,2.5`; `-0.500001,-0.499999,0.499999,0.500001`; `-1,1,-100,100`; `-0.1,0.1,-0.9,0.9`; `-1234.5678,1234.5678,-1e6,1e6,-1e12,1e12`. Retains accessible result bytes and exact arity using the existing bounded observer, including nil slots and zero returns; never derives expected output from helper names. Repeat manually on each matching build/locale, retaining labels and build provenance. Missing APIs/access checks fail closed and errors remain opaque. Native output corpora remain pending; nonfinite inputs, coercion, security behavior and values outside this corpus are not probed.

**Casts:** `/apicontract casts <label>` records `UnitCastingInfo` (`casting`) and `UnitChannelInfo` (`channel`) under `casts.units` for exactly `player`, `target`, `focus`, `party1`, `nonexistent`, `invalid-unit-token` and the empty string. No existence gating, casting or gameplay actions occur. A dedicated accessibility-first scalar-only observer retains exact arity and up to 16 positions, including interior/trailing nils; larger tuples set `truncated = true`. Accessible objects retain only status/kind, never fields, methods, equality or string conversion. IDs are recorded without coercion or assumptions that castID and castBarID match. Existing string bounds apply.

The pinned declarations in `data/patch-api/sources/12.0.0-register.json` list ten casting results (castID at 7, castBarID at 10) and eleven channel results (empowered flag at 9, stage count at 10, castBarID at 11). These declarations guide observation, **not native proof**; the current client may differ. Matching native builds, controllable player casts, non-player channels and empowered/non-empowered fixtures remain pending.

Use separately labeled manual snapshots for idle, active, repeated same cast, consecutive/replaced casts, completed and cancelled states. Capture channels and empowerment scenarios in separate batches: the shared ten-snapshot cap also counts `all` and other modes. Retain each flushed artifact with scenario notes, then move the saved file aside while logged out to begin another batch. Do not interpret timing or identity stability from unrelated captures or fixture values.

**Sex:** independently records `UnitExists`, `UnitSex` and `UnitSexBase` for player, target, focus, pet, party1, party2, nonexistent, invalid-unit-token and the empty token, plus named `Enum.UnitSex` values. Absent, restricted or failed existence queries never suppress either sex query. Each unit now stores separate `exists`, `legacy` and `base` observations instead of an aggregate `status`; their individual statuses and arities preserve errors, nils and redaction. No numbering conversion or output comparisons. Native before/during/after-disguise captures with independently established stable unit identity remain pending; the recorder generates no fixtures.

Restricted values are redacted before comparison or serialization. API errors are opaque status labels, not stringified error objects. Missing access APIs fail closed. Rejected addon-tainted calls are inconclusive; do not bypass restrictions. String observations are truncated to 256 bytes with `truncated = true` when shortened.

## Residual hyperlink observations

Run `/apicontract hyperlinks <label>` explicitly; `all` excludes this mode. Each snapshot makes 144 calls: 16 literal strings × nine variants. Inputs cover ASCII, `é漢字🙂`, empty text, literal `|n`, an actual newline, balanced item/color/atlas/texture markup and their combination, escaped `||`, unclosed item/color/atlas/texture markup and orphan `|h`.

Variants are omitted optional arguments; five false flags; each single true flag in order `maintainColor`, `maintainBrackets`, `stripNewlines`, `maintainAtlases`, `maintainTextures`; consumer `(false,true,false,true,true)`; and all true. Each row saves exact `input`, `variant`, `flags`, `argumentCount` and raw `result`. Omission remains one argument, not six false arguments. No expected stripping or malformed-text normalization is applied.

Existing observer bounds retain exact return arity and at most eight positions. Strings retain up to 256 bytes and set `truncated = true` when shortened; this may split UTF-8. Missing APIs/access checks fail closed, secret values are redacted and errors remain opaque. Ten captures maximum; rejected captures make no calls. Native execution remains pending; nonboolean truthiness/coercion, arbitrary-byte behavior outside the corpus, string-view lifetime and security semantics remain unverified. Local fixtures prove recording only; see [StripHyperlinks contract](../../specs/strip-hyperlinks.md).

## Pure mapvalues observations

Run `/apicontract mapvalues <label>` manually; `all` excludes it. The pinned retail `Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua` declares `mapvalues(func, ...)`, not a table argument. Actual `TooltipDataHandler.lua` forwards `mapvalues(SanitizeTooltipDataArgument, ...)`; `Blizzard_AuraContainerUtil.lua` applies validation callbacks through it and discards the return. These sources guide the experiment, not native packing or traversal conclusions.

Each snapshot runs nine cases: zero inputs; `(17)`; `(17,"two",false)`; `(17,nil,"tail")`; `(17,nil,nil)`; then `(17,"two")` with multiple, nil and zero callback returns; finally `(17)` with an opaque thrown object. The first five callbacks return literal `"mapped"`; multiple returns are `"first",nil,"third"`. Callback results never depend on observed arguments. No callbacks recurse into `mapvalues`.

Each case stores input arity/positions, callback mode, ordered invocation observations and outer output. Exact arity is retained with at most 16 scalar positions per tuple; objects remain opaque and inaccessible values are redacted before inspection. Existing 256-byte string and ten-snapshot limits apply. Errors retain only `call-error`, never the thrown object. There is one shared budget of 32 recorded callback invocations per snapshot. An excess invocation throws an opaque object and sets `invocation-limit`; remaining cases are skipped even if the mapper catches that error. A callback invoked after its case returns is rejected without recording. This bounds recorder work/storage, not an arbitrary broken mapper that loops while swallowing errors.

Native order, nil handling, multi-return packing, callback failure propagation and security remain unverified. Contrasting fake mappers test raw recording, not native expectations. Local fixture:

```text
luajit docs/addons/ApiContractProbe/tests/mapvalues.lua docs/addons/ApiContractProbe
```

## Selected action slots

Run `/apicontract actions 7 before-use` with a slot you independently identify, then repeat with labels after manual use, during recharge and after restoration. No slot roles are inferred and no action is executed. Use separate labeled batches for ordinary spells, charged spells, consumables and empty slots; `0` and `-1` are explicit input controls. Syntax accepts signed decimal integers within ±9007199254740991 only, rejecting malformed input before queries. This does not establish native slot validation. `all` excludes this selected-slot mode.

Each snapshot records optional `GetActionInfo`, default display count and thresholds `0`, `1`, `9999` with replacement `*`, plus `GetActionCharges` arity and five raw table fields: `currentCharges`, `maxCharges`, `cooldownStartTime`, `cooldownDuration`, `chargeModRate`. Display counts are not assumed to equal charges. `GetActionChargeDuration` records only arity and accessible kind/status: no methods or value inspection. No `GetActionCooldown` call is made. Charge fields use `rawget`, never metatable lookup; restricted values and errors remain opaque.

At most seven selected-slot calls per snapshot, 16 saved return positions per call (exact arity and explicit truncation), and ten captures per batch. Build/time and bounded labels are retained. Native slot fixtures/transitions and duration-method contracts remain pending; recorder fixtures do not establish native parity or full preparation of this plan.

## Local fixture

```text
luajit docs/addons/ApiContractProbe/tests/harness.lua docs/addons/ApiContractProbe
```

Cast fixtures exercise distinct tenth/eleventh IDs, empowerment, nil/zero arity, repeated/consecutive captures, 16-position truncation, hostile objects and inaccessible/error returns without asserting native semantics.

Numeric fixtures deliberately return differing locale bytes, non-rounding strings, multiple/nil/zero returns and restricted/error results; these prove literal recording, not native rounding. Fixtures test the recorder under different fake client behaviors, including differing modified/unmodified names, realm forms, nil arity, unknown/invalid tokens and restricted/error returns. They do not establish native indexing, ordering, copy/identity, sex numbering, transformation or security semantics. See [spec](../../specs/api-contract-probe.md).
