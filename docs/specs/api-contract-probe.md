# API contract probe

`docs/addons/ApiContractProbe/` prepares native investigations of scalar curve point returns, `UnitSexBase` comparison, unit name/realm returns and finite numeric formatting. It shares one manual recorder; the existing [dispel probe](aura-dispel-curve-probe.md) remains separate. See [capture protocol](../addons/ApiContractProbe/README.md).

## Manual unit target display

- Manual `unit-target-display <label>` is excluded from `all`. Independently call only `UnitShouldDisplaySpellTargetName(unit)` twice for each of `player`, `target`, `focus`, `party1`, `nonexistent`, `invalid-unit-token`, and the empty string: fourteen calls per snapshot.
- Guard the token and function before use; recheck token access after function guards, immediately before invocation. Missing/restricted inputs or functions and opaque errors remain independent observations.
- Preserve raw arity/nils and accessible scalar results within sixteen positions, 256-byte strings, 128-byte labels and ten shared snapshots. Do not infer boolean expectations, defaults, repeated-read stability or native target semantics.
- Never query secret-return `UnitSpellTargetClass`/`UnitSpellTargetName` or add casting queries. Existing `casts` mode supplies separate manual context. Native cast/target fixtures remain unverified; eight local actual TOC/slash fixtures establish recorder mechanics only.

## Manual selected-slot spell durations

- Manual `spell-duration <slot> <label>` reuses the integer-slot parser and guarded `GetActionInfo(slot)` producer; it is excluded from `all`. Only an accessible `spell` string and finite numeric ID authorize calls with the original ID.
- Independently call `C_Spell.GetSpellChargeDuration(ID)` and `C_Spell.GetSpellLossOfControlCooldownDuration(ID)`. Protect namespace/function lookup and recheck both producer values immediately before each call. Missing/restricted/errored producers skip queries, never imply zero; one unavailable API does not gate its peer.
- Preserve raw arity, nils and opaque errors. Reuse the existing ten read-only duration methods through `inspectDuration`; no mutators, guessed IDs, spellbook or ordinary cooldown queries, or casts.
- Observe current returned objects only. Do not retain objects between captures or touch the cast-duration retention list/counter. Preserve sixteen-position, 256-byte string, 128-byte label and ten-snapshot bounds; at most two producers and 320 duration-method calls per snapshot.
- Nine separate actual TOC/slash fixtures establish recorder behavior, including interleaved cast-retention isolation. Native charge and loss-of-control timing/lifecycle and restricted-context behavior remain pending.

## Manual current unit auras

- Manual `unit-auras-current <label>` is excluded from `all`. Make three independent `C_UnitAuras.GetUnitAuras` calls with exact arguments `("player")`, `("player", "HELPFUL", 8)` and `("player", "HARMFUL", 8)`. Label the first `omittedFilterNegative`: missing required filter is a negative experiment, not valid/default semantics. Omit sort arguments; no continuation exists in the pinned signature.
- Preserve raw return arity/nils and opaque errors, with at most sixteen scalar observations. Only the first returned value may receive numeric entries 1–8, and only if it is an accessible table. Never traverse with length/pairs or mutate returned objects.
- For each accessible table/userdata entry, observe only `auraInstanceID`, `spellId` and `applications` through guarded field reads. Guard parent table/entry before every lookup and returned fields before scalar inspection; peer entries/calls continue after missing, restricted or failed observations. No coercion, downstream forwarding or retention.
- Fixed keys are consumer-grounded, not claimed schema declarations: pinned retail `Blizzard_FrameXMLUtil/AuraUtil.lua:45–54,266–267`. Signature source: `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:452–469`, under the profile cache. No `AuraData` field declaration was located there.
- Bound output to three calls per snapshot, ten snapshots, sixteen return positions, eight entries per first table, 256-byte strings and 128-byte labels. Local tests prove recorder mechanics, not native ordering/completeness, identity, defaults, sorting, security semantics or historical contract equality.

## Manual current-only aura time

- Manual `aura-time <label>` is excluded from `all`; reuse the first-page player HELPFUL eight-slot producer without changing `aura-display-count` output or behavior. Forward only original accessible finite slots and guarded original aura instance IDs.
- Independently call `DoesAuraHaveExpirationTime`, `GetAuraBaseDuration`, `GetRefreshExtendedDuration`, and `GetAuraDuration`, each with exactly `("player", id)`. Omit optional spell IDs entirely. Recheck original inputs after namespace/function lookup and access guards, before forwarding.
- Preserve query status, return arity/nils and scalar kind/status. Reuse the ten-method duration whitelist on accessible table/userdata returns of `GetAuraDuration`, at positions 1–16, without calling the producer twice. Guard object access before inspection/lookup and recheck the receiver after function guards immediately before invocation; shared spell/cast consumers receive the same guard. Preserve independent method return arity/nils, opaque errors and bounded scalar values.
- No evaluate/set/copy methods, field traversal, retention, continuation, mutations or changes to cast-duration retained state: current-only observations, not lifecycle or native-conformance proof.
- Missing fixtures/APIs, restricted/invalid inputs and errors produce independent observations. Cap each snapshot at one page, eight data calls, 32 queries and 1,280 method calls (8 IDs × 16 returned positions × 10 methods), 16 tuple positions per producer/method, 256-byte strings and a 128-byte label; cap captures at ten.

## Manual first-page aura display counts

- Manual `aura-display-count <label>` is excluded from `all`. Call `C_UnitAuras.GetAuraSlots("player", "HELPFUL", 8)` exactly once. Preserve continuation plus vararg slots as a bounded raw tuple, never traverse continuation or claim enumeration completeness.
- At most eight original accessible finite slot values feed `GetAuraDataBySlot("player", slot)`. Preserve producer arity without traversing AuraData; access only `auraInstanceID` through guarded field lookup on an accessible table/userdata. Require an accessible finite original numeric ID, without coercion or invented values.
- Independently call `GetAuraApplicationDisplayCount("player", id)` with optional arguments omitted, `(1)`, `(2)`, `(2, 5)` and `(1, 1)`. Guard namespace/function and produced inputs before inspection/forwarding; recheck inputs after lookup and function guards. Record missing, inaccessible, invalid and error cases independently.
- Preserve raw return arity/nils and opaque errors with 16-position, 256-byte string, 128-byte label and ten-snapshot limits. At most eight data calls and forty count queries per snapshot; no gameplay mutation or native execution. Local fixtures establish recorder behavior only, not native ordering, defaults, formatting, coercion or completeness.

## Manual selected-slot spell metadata

- Manual `spell-metadata <slot> <label>` reuses the actions integer-slot parser and is excluded from `all`. Call `GetActionInfo(slot)` with exactly one argument; preserve raw producer arity and nil positions.
- Only an accessible string first return exactly `spell` and accessible finite numeric second return authorize querying the original ID. No conversions, guessed IDs or inferred classification.
- Independently call `C_Spell.GetSpellDisplayCount`, `GetSpellMaxCumulativeAuraApplications`, `IsConsumableSpell`, `IsExternalDefensive`, `IsPriorityAura`, `IsSpellCrowdControl` and `IsSpellImportant`, passing only the ID. Omit display-count optional arguments.
- Query `GetVisibilityInfo(ID, value)` once for each fixed published `Enum.SpellAuraVisibilityType` name: RaidInCombat, RaidOutOfCombat, EnemyTarget. Accept only guarded finite numeric values, without numeric fallbacks or arbitrary iteration. Missing/restricted/invalid enum values record `unavailable-enum` without calls; base queries remain independent. Recheck original spell inputs and the enum value after lookup/function guards, before each call. Preserve zero returns as distinct from nil and opaque errors.
- Guard each namespace/function lookup and producer values before inspection; recheck input access after lookup/function checks before every downstream call. Missing APIs and opaque errors do not suppress peers. No action execution or casts.
- Independently record `auraQueries.AuraIsBigDefensive` using `C_UnitAuras.AuraIsBigDefensive(originalID)`, whose pinned declaration accepts one `SpellIdentifier` and returns a boolean. No aura instance or invented ID. Guard namespace/field/function and original kind/ID before inspection; recheck input access after potentially revoking lookups/function guards. Aura and spell-query failures must not suppress each other.
- Preserve exact arity, nils, sixteen scalar positions, 256-byte strings, 128-byte labels and ten snapshots. At most seventy base metadata, thirty visibility and ten aura-defensive calls across ten snapshots. Nineteen cumulative actual TOC/slash fixtures include five added aura-defensive cases and prove bounded recorder behavior only, with no native-conformance credit; native outputs, classifications, rank/override fixtures and security remain pending.

## Manual public queries

- Manual `public-queries <label>` is excluded from `all`. Independently call `C_GameRules.IsPersonalResourceDisplayEnabled()` twice and `C_DelvesUI.GetLockedTextForCompanion()` twice, each with exactly zero arguments.
- Record only the omitted-companion observation. No invented companion/trait-tree IDs, trait-tree query, mutations, CVar changes or additional query APIs.
- Protect namespace lookup; check access before inspecting functions/results. Preserve independent missing/lookup/call errors opaquely and retain raw arity, nils and repeated observations without stability conclusions.
- Reuse sixteen-position, 256-byte and ten-snapshot bounds. Seven separate actual TOC/slash fixtures cover zero arguments, repeated results, independent failures, access guards and bounds. Native ruleset transitions, companion lock policy, trait-tree fixtures and security behavior remain unverified.

## Manual equipped-item binding

- Manual `item-binding <label>` is excluded from `all`. Query `GetInventoryItemLink("player", slot)` once for each fixed slot 1–19, retaining exact producer return arity and nil positions.
- Use only the first actual producer return when it is an accessible non-secret string. Query `C_Item.IsItemBindToAccount` once with the original link, never a truncated saved string, synthetic link or numeric item ID.
- Check accessibility before type inspection and namespace lookup; recheck link accessibility immediately before calling the binding API, after function lookup/access checks. Missing, nil, nonstring or failed producers explicitly skip binding with `unavailable-input`; restricted links use `restricted-input`. Neither means false.
- Keep producer and binding errors opaque and continue independent slots. Retain sixteen scalar positions, 256-byte strings and ten snapshots. No mutations, alternate producers or native execution.
- Eight separate actual TOC/slash fixtures cover argument flow, arity, unavailable/restricted inputs, namespace failures, access revocation, bounds and manual routing. Pinned ItemDocumentation and the Journeys item-link consumer guide arguments, not native binding classifications or security conclusions.

## Manual StatusBar fill style

- Manual `statusbar-fill <label>` is excluded from `all`. Create one unnamed StatusBar under UIParent, immediately attempt Hide, then capture the raw fresh GetFillStyle default before any fill setter.
- Read only published `Enum.StatusBarFillStyle` names Standard, StandardNoRangeFill, Center and Reverse. Pass accessible finite scalar values without numeric fallbacks. Record missing/restricted/nonscalar inputs without setter attempts.
- For each eligible value, record one SetFillStyle and two subsequent independent GetFillStyle calls, preserving setter failures, raw arity/nils and opaque results rather than equality conclusions.
- Guard object, method and result access before lookup or inspection. Retain sixteen scalar positions, 256-byte strings and ten snapshots. Abort fill operations after constructor/Hide failure; retain no object reference. Hide failure explicitly leaves client visibility unconfirmed; no destruction or alternate setter is attempted.
- No Show, sizing, layout/rendering, invalid inputs, restricted-context probes or other setters. Addon code is tainted; no untainted setup assumption. Seven separate actual TOC/slash fixtures test recorder behavior, not native conformance. Native initial-state, validation and coercion captures remain pending.

## Manual scalar curve state

- [x] Manual `curve-state <label>` is excluded from `all` and leaves existing `curves` capture unchanged.
- [x] Construct empty and duplicate/unsorted scalar curves with inputs `(30,4), (10,7), (20,2), (10,9)`. Record `GetType`, `GetPointCount`, `GetPoints`, `HasSecretValues` and `Evaluate` at `-1,0,10,15,20,30,31` before and after `SetToDefaults`.
- [x] Set type only through safely discovered `Enum.LuaCurveType.Linear`, recording missing prerequisites or setter failures without guessing enum values.
- [x] Copy a separate populated curve, capture the copy before mutation, then original and copy after `AddPoint(40,11)` and `ClearPoints`. Preserve observations even if copying aliases the original; do not classify native behavior.
- [x] Call only constructed/copied curve methods; scalar-only method results remain opaque except the existing bounded point/GetXY observer for `GetPoints`. Restricted copies and constructor results fail closed; errors are opaque.

Pinned `LuaCurveObjectAPIDocumentation.lua` and `LuaCurveObjectBaseAPIDocumentation.lua` describe these argument and return shapes, not observed native defaults or copy behavior. Four point entries, eight return positions, seven evaluation inputs and the shared ten-snapshot limit bound capture. Color curves and secret semantics remain outside this experiment; point removal/replacement uses the separate experiment below. Local fake-client tests retain differing defaults, duplicate replacement/preservation, copied/aliased state and evaluation outputs; native execution remains pending.

## Manual scalar point mutation

- [x] Manual `curve-edit <label>` is excluded from `all`. Each `RemovePoint` index `-1,0,1,2,3,4` receives its own curve populated with `(30,4), (10,7), (20,2)`.
- [x] Two independent populated curves receive empty and unsorted duplicate `SetPoints` inputs `(30,4), (10,7), (20,2), (10,9)`. Construct every vector through actual accessible `CreateVector2D`; missing/throwing/restricted/invalid constructors prevent replacement, without a table-coercion fallback.
- [x] Record input definitions, count, bounded points and `Evaluate(15)` before and after each attempt; preserve mutation arity/nils and opaque errors. Retain four point entries, eight return positions and ten snapshots using existing accessibility-first observers.
- [x] Behavioral fixtures preserve differing zero-/one-based removal and SetPoints ordering, constructor failures, redaction, independent curves, manual routing and storage bounds. Do not infer index base, duplicate policy, interpolation or alias behavior from fixtures.

Pinned scalar documentation specifies `RemovePoint(index: luaIndex)` and `SetPoints(point: table<vector2>)`; pinned `Vector2D.lua` constructs an `x`/`y` object with `GetXY`. Native captures remain pending. Only numeric index controls are included; invalid-type coercion, color curves, vendor mutation and security behavior are excluded.

## Manual scalar resource scale capture

- Manual `resources <label>` is excluded from `all`; observe player/target/focus/pet/nonexistent without existence gating.
- Preserve raw health/max and power/max/type, exact arity and nil positions. Health percent curve argument 3 follows explicit false prediction; default, false/no-curve and false/nil-curve calls remain separate. Power uses default/nil power type with default calls and explicit false/true unmodified variants; curve is argument 4.
- Record scalar curve inputs `(0,0), (.5,10), (1,20), (25,30), (50,40), (100,50)` and explicitly set accessible `Enum.LuaCurveType.Linear`. Missing construction prerequisites prevent curved queries, not underlying observations. Never guess enum numbers or power IDs.
- Reuse accessibility-first scalar-only recording and existing return/storage bounds. Do not interpret normalized versus percentage scale.
- Native partial-resource fixtures, prediction, secondary powers, color curves and security remain pending; no events or gameplay mutation.

Pinned retail `UnitDocumentation.lua` declares `UnitHealthPercent(unit, usePredicted=true, curve)` and `UnitPowerPercent(unit, powerType, unmodified=false, curve)`. Declarations establish argument positions, not native scale. Simulator policy specs remain unchanged.

## Manual color-curve state

- [x] Manual `color-curves <label>` stays outside `all`; construct own curve/colors from four distinct recorded RGBA definitions at x `0,32,-16,48`, without guessed enum values.
- [x] Capture empty/populated `GetType`, `GetPointCount`, `HasSecretValues`, `GetPoints`, six indices `-1,0,1,2,3,4`, and `Evaluate`/`EvaluateUnpacked` at `-17,-16,0,16,32,48,49`.
- [x] Record Copy return arity/opaque curve shape; capture original and copy before/after adding the defined fifth point and clearing the copy. Record original before/after reset, preserving copy-alias effects rather than assuming isolation or defaults.
- [x] Inspect returned tables with accessibility-first raw `x/y/r/g/b/a` fields, four array entries and three table levels; userdata stays opaque. Never invoke returned color/point methods or metamethods. Only guarded owned-curve calls execute.
- [x] Preserve exact return arity, sixteen positions, nils, secret-channel redaction and opaque errors. Missing/invalid/restricted construction fails closed; shared ten-snapshot limit applies.
- [x] Local `tests/color_curves.lua` exercises different order/default/copy behaviors, packed/unpacked values, inaccessible channels, opaque userdata, hostile lookup, constructor failures, tuple/storage bounds and manual-only routing.

Cached `LuaColorCurveObjectAPIDocumentation.lua`, `LuaCurveObjectBaseAPIDocumentation.lua`, `CurveUtilDocumentation.lua` and `Blizzard_SharedXMLBase/Color.lua` guide construction/field names, not native proof. Actual native state/evaluation, userdata fields, security, removal/replacement and other interpolation types remain unverified.

## What it must do

- [x] Record build provenance and manual scenario labels without automatic capture or gameplay changes.
- [x] Observe scalar curve empty/populated returns, explicit index queries, fields/GetXY, repeated identity and an isolated returned-point mutation attempt without assuming native outcomes.
- [x] Independently preserve `UnitExists`, `UnitSex` and `UnitSexBase` observations for player, target, focus, pet, party1, party2, nonexistent, invalid-unit-token and empty tokens, alongside raw named enum values. Never gate sex queries on existence, convert numbering or compare outputs. Unit records contain `exists`, `legacy` and `base` rather than an aggregate status; each observation retains its own status and arity.
- [ ] Obtain native before/during/after-disguise captures with independently established stable unit identity; no fixture generation is performed.
- [x] Capture `UnitName` and `UnitNameUnmodified` with `/apicontract names <label>` and `all` for fixed tokens `player`, `party1`–`party4`, `target`, `nonexistent`, `invalid-unit-token` and the empty string. Never skip queries based on `UnitExists`.
- [x] Preserve name/realm positional returns and exact arity, including zero returns, nil slots and empty realm strings, under existing accessibility-first redaction and opaque-error rules. Same/cross-realm fixture identity comes from manual labels, not inferred return values.
- [x] Capture `/apicontract numbers <label>` and `all` numeric samples for `C_StringUtil.FloorToNearestString` and `RoundToNearestString`, with `GetLocale` provenance, build and label. Use exactly the 25 finite inputs listed in the capture protocol: signed ties and their neighbors, integers, fractions and large finite magnitudes.
- [x] Preserve numeric helper result bytes, positional nils and exact arity through the existing bounded observer without guessing rounding or grouping. Missing APIs and failed access checks fail closed; errors remain opaque.
- [x] Redact inaccessible/secret values before comparison or saving, and record opaque failures without stringifying errors.
- [x] Bound capture count, inspected results, arrays, depth and strings.
- [x] Exercise actual addon loading and slash commands through local behavioral fixtures.

## Manual raid markers

- Manual `raid-markers <label>` is excluded from `all`; use the nine sex-control tokens: player, target, focus, pet, party1, party2, nonexistent, invalid-unit-token and empty string.
- Independently call `CanBeRaidTarget(unit)` twice per token, `IsRaidMarkerActive(index)` twice per index 1–8, and `IsRaidMarkerSystemEnabled()` twice with no arguments. Preserve repeated raw observations, not stability comparisons; missing functions never gate others.
- Guard access before every inspection; record exact arity/nils and sixteen scalar positions, bounded 256-byte strings, opaque objects/errors and explicit truncation. Shared ten-snapshot cap bounds capture; at most 36 API calls per snapshot.
- Exclude secret-return `GetRaidTargetIndex`, setters, clear/place/remove operations, permission and security claims. Native output and populated world-marker fixtures remain pending.
- Six separate actual TOC/slash fixtures prove argument lists, repeated observations, independent failure handling, guards and bounds. Pinned `RaidMarkersDocumentation.lua` establishes signatures only, not native conformance.

## Manual abbreviations

- Manual `abbreviations <label>` stays outside `all`; reuse the existing 25-number corpus, preserving `numbers` behavior.
- Call `AbbreviateLargeNumbers(number)` and `AbbreviateNumbers(number)` independently with options omitted; capture `GetLocale()` once without changing locale.
- Preserve exact scalar result arity, nil slots and accessible bytes, with sixteen positions, 256-byte strings and explicit truncation. Guard access before inspection; returned objects and errors remain opaque.
- Missing APIs and failed access checks fail closed. Bound each snapshot to fifty abbreviation calls and one locale call, under the shared ten-snapshot limit.
- Separate actual TOC/slash fixtures cover shared inputs, omitted options, localization, missing/restricted functions and results, errors, arity, opaque objects and bounds.

Pinned retail `LocalizationDocumentation.lua` supplies signatures only. No formatter, options/configuration tables, producer or mutator belongs to this slice. Native semantics and option/configuration behavior remain unverified separate gaps.

## Manual heal calculator

- Manual `heal-calculator <label>` stays outside `all`; attempt two independent no-argument `CreateUnitHealPredictionCalculator()` constructions per snapshot.
- For each accessible object, observe `GetHealAbsorbMode`, `GetHealAbsorbClampMode`, `GetDamageAbsorbClampMode`, `GetHealAbsorbs` and `GetDamageAbsorbs` twice each. Preserve repeats without comparing values or inferring stability.
- Record constructor/getter exact arity, nil slots and sixteen scalar/opaque positions with explicit truncation. Guard object access before method lookup and method/result access before inspection. Do not serialize runtime objects or errors, invoke returned-object methods or assume zero/default values.
- Missing, throwing, restricted or non-object constructor results prevent getter work for that attempt. Bound each snapshot to two constructor and twenty getter calls; retain the shared ten-snapshot cap.
- Separate actual TOC/slash fixtures cover repeated raw values, constructor/method/result access failures, opaque errors/objects, exact larger tuple arity, manual-only dispatch and storage limits.

Pinned retail `UnitDocumentation.lua` and `UnitHealPredictionCalculatorAPIDocumentation.lua` guide the constructor and five getter signatures only. Native observations, populated state, reset/default semantics and security remain unresolved. No unit contexts, setters, population, native execution or conformance credit belong to this partial experiment.

## Manual producer durations

- Manual `cast-durations <label>` is excluded from `all`. Query casting/channel durations and empowered durations with hold omitted, false and true for player/target/focus/party1/nonexistent/invalid-unit-token/empty tokens.
- Record exact arity and at most 16 scalar/opaque positions. On accessible returned objects, explicitly query only `GetTotalDuration`, `GetElapsedDuration`, `GetRemainingDuration`, `GetElapsedPercent`, `GetRemainingPercent`, `GetStartTime`, `GetEndTime`, `GetClockTime`, `GetModRate`, `HasExpired`. Guard object access before method lookup and function access before invocation; errors stay opaque. Method results are scalar-only, with no recursive object inspection.
- Keep at most 28 returned-object references from one previous duration snapshot in memory. Re-observe them on the next duration capture, then replace retention. Saved records contain local observation references, never runtime objects/functions or native identity claims. Mark excess retention explicitly; no equality/string conversion or mutation calls.
- Local fixtures distinguish live versus frozen producer objects across manual captures. Native active/completed/interrupted/absent states and time transitions require user-provided fixtures; no casts, timers, clock mutation or native security conclusions.

Pinned retail `UnitDocumentation.lua` declares optional empowered hold default true and potentially absent results; `LuaDurationObjectAPIDocumentation.lua` declares the ten read methods. These declarations guide calls, not native results. Shared ten-snapshot limit applies; retention resets when the addon reloads.

## Manual cast identities

- [x] `/apicontract casts <label>` and `all` query `UnitCastingInfo` and `UnitChannelInfo` for fixed `player`, `target`, `focus`, `party1`, `nonexistent`, `invalid-unit-token` and empty-string tokens without existence gating or gameplay actions.
- [x] Dedicated accessibility-first scalar-only protected calls preserve exact arity and up to 16 positions, including interior/trailing nils, with explicit truncation beyond 16. Accessible objects retain status/kind only; never execute returned-object methods, indexing, equality or string conversion. Errors remain opaque.
- [x] Preserve raw scalar IDs, empowerment flags and stage counts without identity/coercion assumptions; use existing build/time/label infrastructure and shared ten-capture limit. Document separate batches for fixture sequences.

Pinned `12.0.0-register.json` declarations describe casting position 10 and channel position 11 as castBarID; they are not native evidence and the executing client may differ. Matching native client, controllable casts, non-player channels and empowered/non-empowered fixtures remain pending. No recorder fixture earns native audit credit.

## Selected action counts and charges

- [x] `/apicontract actions <slot> <label>` queries one explicitly selected signed decimal integer slot; zero and negative controls are accepted. Reject invalid syntax and integers outside the exactly representable range before any query. This is command validation, not a claim about native coercion. `all` never queries actions.
- [x] Capture optional `GetActionInfo`, default `C_ActionBar.GetActionDisplayCount`, and explicit thresholds 0, 1 and 9999 with replacement `*`, preserving differing display/charge observations.
- [x] Capture `GetActionCharges` exact arity and the five accessible raw table fields `currentCharges`, `maxCharges`, `cooldownStartTime`, `cooldownDuration`, `chargeModRate`; never invoke metatable lookup. Capture `GetActionChargeDuration` only as accessible kind/status and arity, never its values or methods.
- [x] Bound each snapshot to seven selected-slot calls, 16 stored return positions per call, existing ten-capture storage and build/time/label provenance; redact before inspection and keep errors opaque.

Native ordinary-spell, charged-spell, consumable and empty-slot fixtures plus before/use/recharge/restoration transitions remain pending. Duration-method contracts remain uncaptured. These eight plan rows are only partially prepared; no `GetActionCooldown` query belongs to this slice.

## Residual StripHyperlinks

- [x] Manual-only `/apicontract hyperlinks <label>` records 16 literal inputs × nine flag variants (144 calls); `all` excludes it. Exact corpus and ordered flags are documented in the capture protocol.
- [x] Preserve omitted optional arguments separately from five explicit false flags; record exact input, variant, flags, argument count and raw accessible result arity without guessed stripping/coercion or malformed-text normalization.
- [x] Use existing eight-position observer and ten-capture bounds. Strings retain at most 256 bytes with explicit truncation; inaccessible results are redacted, missing APIs fail closed and errors are opaque.
- [x] Behavioral fixtures discriminate all argument positions/omission, UTF-8 and embedded NUL output bytes, malformed output, restrictions/errors, truncation, manual-only routing and overflow without extra calls.

Native execution, nonboolean truthiness, arbitrary-byte contracts beyond the fixed corpus, string-view lifetime and security remain unverified. This recorder does not change the [simulator StripHyperlinks contract](strip-hyperlinks.md) or earn native audit credit.

## Residual plain-function callbacks

- [x] Manual `callbacks-start <label>` and `callbacks-stop`, excluded from `all`, register only owned plain functions for global `UNIT_HEALTH` and unit `UNIT_HEALTH`/`player`; cleanup uses those exact identities.
- [x] Preserve independent registration/removal arity and scalar-only results with opaque errors. Refused or uncertain registration is not success; a throwing call retains possible registration ownership for cleanup. Failed/refused/uncertain cleanup retains identity and reports incomplete cleanup until a successful retry.
- [x] Capture real deliveries only: 128 entries per session, exact arity and 16 scalar-only positions, nil preservation, accessibility-first redaction, label/build/time provenance. Overflow does not prevent manual cleanup; stale callbacks cannot record after stop.
- [x] Reject repeated starts while cleanup is outstanding, create one new session after stop, and bound saved sessions to ten. Missing access APIs prevent registration. Callback function objects remain private, outside SavedVariables.

The four pinned global APIs take event name and callback, with a third unit argument for unit registration/removal; current retail declarations list no returns. Record observed arity instead of importing the simulator global registration boolean policy. Zero-return protected-call success is an accepted call, not proof of a native delivery. External `UNIT_HEALTH` fixtures remain pending. FunctionContainer wrappers, duplicates, ordering, aliases, mutation, recursion and security are excluded; this is not complete callback preparation.

Behavioral fixtures cover owned registrations, hostile/secret/nil payloads, identity removal, no after-stop records, partial registration errors, cleanup retries/refusals, repeated start, new sessions, payload/session limits and `all` exclusion.

## Pure mapvalues capture

- [x] Manual `/apicontract mapvalues <label>` is excluded from `all`; query the actual accessible global with a callback followed by a fixed vararg tuple, never a guessed table signature.
- [x] Nine cases cover zero/one/multiple inputs, interior/trailing nils, fixed one/multiple/nil/zero callback returns and an opaque callback throw. Preserve callback invocation sequence, each invocation's arity/positions and outer return arity/scalars without asserting native traversal or packing.
- [x] Check accessibility before inspecting callback arguments or outer results; objects stay opaque, errors are not stringified, and callback outputs are ordinary fixed literals independent of observed arguments. Missing/restricted APIs fail closed.
- [x] Retain at most 16 positions per tuple with explicit truncation and 32 callback invocations shared across a snapshot. Excess invocations throw opaquely, mark `invocation-limit` and prevent later cases even when the mapper swallows the error. Reject late callbacks after case completion; callbacks never recurse. Preserve existing string/storage bounds. No recorder can terminate an arbitrary mapper that loops while swallowing callback errors.
- [x] `tests/mapvalues.lua` loads the actual TOC/slash handler and contrasts individual-value, whole-tuple and reversed fake mappers, nil/multiple returns, hostile/restricted values, errors, tuple/invocation/storage limits and manual-only routing.

Pinned retail `FrameScriptDocumentation.lua` declares `mapvalues(func, values...)` with strided input/output values. `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua` forwards mapped tooltip arguments, while `Blizzard_AuraContainer/Blizzard_AuraContainerUtil.lua` discards validation-callback results. Native execution, nil/order/packing/error contracts and security semantics remain pending; fixture behavior earns no native credit.

## Publication and event recording

- [x] Capture raw and ordinary lookup observations for configured enum, constant and API paths, distinguishing missing parents from missing members.
- [x] Capture configured CVars through `C_CVar.GetCVar` and `GetCVarDefault` without writes.
- [x] Register configured events only on explicit `events-start`; retain registration errors and bounded positional payloads with nil slots.
- [x] Stop listeners with `events-stop`; retain build/source provenance, scenario labels and overflow counts.

`AuditTargets.lua` contains 199 publication/CVar targets and 88 event-registration targets from the blocker snapshot. These are recording capabilities, not 287 complete behavioral probes. Native transitions, earlier builds, load phases, producers and restricted payloads remain unresolved. Configuration processing is bounded to 256 targets per category; events to 256 and payloads to eight values. Passive event payloads use raw table inspection without invoking their methods or `__index`.

Runtime `d959372a3` passes 10/10 fixtures (exact-hash reuse) plus 21/21 independent supplemental checks. Proof: `/tmp/verify-all-probes-recorders-corrected-ledger.json`. The initial passive-inspection bug is preserved by regression `5870cfa46`; no native API credits follow from recorder tests.

## How it works

- [Capture protocol](../addons/ApiContractProbe/README.md)

## Implementation inventory

- `docs/addons/ApiContractProbe/ApiContractProbe.lua`: shared observation and manual curve, sex, name, numeric, cast, publication and event captures.
- `docs/addons/ApiContractProbe/ApiContractProbe.toc`: manual addon and SavedVariables registration.
- `docs/addons/ApiContractProbe/AuditTargets.lua`: exact snapshot-derived capture targets and source hash.

## Tests asserting this spec

`docs/addons/ApiContractProbe/tests/harness.lua`. Initial RED at `e2adbb57d`: addon not yet present; exit 1. Logs `/tmp/api-contract-probe-red.stdout` and `/tmp/api-contract-probe-red.stderr`. Runtime `a055b98c9` passes 8/8 local fixtures. Independent `/tmp/verify-api-contract-probe-ledger.json` reuses exact-hash fixture proof and passes 14/14 supplemental checks, including invalid indices, read-only points, raw values, access filtering and real addon wiring. This proves the recorder, not native API semantics.

Name fixtures exercise differing modified/unmodified names, nil/empty/explicit realms, positional nils and zero returns, all fixed tokens without existence gating, redacted secrets/inaccessible values, opaque errors, missing APIs and `all` inclusion. These are recorder tests, not native name/realm expectations.

Numeric fixtures assert the literal 25-input corpus against deliberately different fake clients, locale bytes, multiple/nil/zero returns, redacted results, opaque errors and missing/failed access APIs. They do not establish native rounding rules.

Cast fixtures cover distinct tenth/eleventh IDs, interior/trailing nils, empowerment flags/stage counts, repeated and consecutive manual captures, zero-result idle states, restricted/opaque errors, hostile objects, missing access/APIs and 16-position bounds. Tests assert recording behavior only.

## Known gaps (current cycle)

- [ ] Native numeric formatting corpora on matching builds/locales; nonfinite/coercion/security behavior and values outside the finite corpus remain outside this bounded recorder.
- [ ] Native captures on a matching client, including independently identified same-realm and cross-realm party fixtures for names.

## Out of scope

Installation, native execution, inferred numeric mappings or vector contracts, secret access bypass, arbitrary object serialization and gameplay manipulation. No API audit credit from local recorder fixtures.
