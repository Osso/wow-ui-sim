# API contract probe

`docs/addons/ApiContractProbe/` prepares native investigations of scalar curve point returns, `UnitSexBase` comparison, unit name/realm returns and finite numeric formatting. It shares one manual recorder; the existing [dispel probe](aura-dispel-curve-probe.md) remains separate. See [capture protocol](../addons/ApiContractProbe/README.md).

## Manual scalar curve state

- [x] Manual `curve-state <label>` is excluded from `all` and leaves existing `curves` capture unchanged.
- [x] Construct empty and duplicate/unsorted scalar curves with inputs `(30,4), (10,7), (20,2), (10,9)`. Record `GetType`, `GetPointCount`, `GetPoints`, `HasSecretValues` and `Evaluate` at `-1,0,10,15,20,30,31` before and after `SetToDefaults`.
- [x] Set type only through safely discovered `Enum.LuaCurveType.Linear`, recording missing prerequisites or setter failures without guessing enum values.
- [x] Copy a separate populated curve, capture the copy before mutation, then original and copy after `AddPoint(40,11)` and `ClearPoints`. Preserve observations even if copying aliases the original; do not classify native behavior.
- [x] Call only constructed/copied curve methods; scalar-only method results remain opaque except the existing bounded point/GetXY observer for `GetPoints`. Restricted copies and constructor results fail closed; errors are opaque.

Pinned `LuaCurveObjectAPIDocumentation.lua` and `LuaCurveObjectBaseAPIDocumentation.lua` describe these argument and return shapes, not observed native defaults or copy behavior. Four point entries, eight return positions, seven evaluation inputs and the shared ten-snapshot limit bound capture. Color curves, SetPoints, RemovePoint and secret semantics remain outside this experiment. Local fake-client tests retain differing defaults, duplicate replacement/preservation, copied/aliased state and evaluation outputs; native execution remains pending.

## Manual scalar resource scale capture

- Manual `resources <label>` is excluded from `all`; observe player/target/focus/pet/nonexistent without existence gating.
- Preserve raw health/max and power/max/type, exact arity and nil positions. Health percent curve argument 3 follows explicit false prediction; default, false/no-curve and false/nil-curve calls remain separate. Power uses default/nil power type with default calls and explicit false/true unmodified variants; curve is argument 4.
- Record scalar curve inputs `(0,0), (.5,10), (1,20), (25,30), (50,40), (100,50)` and explicitly set accessible `Enum.LuaCurveType.Linear`. Missing construction prerequisites prevent curved queries, not underlying observations. Never guess enum numbers or power IDs.
- Reuse accessibility-first scalar-only recording and existing return/storage bounds. Do not interpret normalized versus percentage scale.
- Native partial-resource fixtures, prediction, secondary powers, color curves and security remain pending; no events or gameplay mutation.

Pinned retail `UnitDocumentation.lua` declares `UnitHealthPercent(unit, usePredicted=true, curve)` and `UnitPowerPercent(unit, powerType, unmodified=false, curve)`. Declarations establish argument positions, not native scale. Simulator policy specs remain unchanged.

## What it must do

- [x] Record build provenance and manual scenario labels without automatic capture or gameplay changes.
- [x] Observe scalar curve empty/populated returns, explicit index queries, fields/GetXY, repeated identity and an isolated returned-point mutation attempt without assuming native outcomes.
- [x] Preserve raw legacy/base sex values and arity for existing units, alongside named enum values; never convert numbering.
- [x] Capture `UnitName` and `UnitNameUnmodified` with `/apicontract names <label>` and `all` for fixed tokens `player`, `party1`–`party4`, `target`, `nonexistent`, `invalid-unit-token` and the empty string. Never skip queries based on `UnitExists`.
- [x] Preserve name/realm positional returns and exact arity, including zero returns, nil slots and empty realm strings, under existing accessibility-first redaction and opaque-error rules. Same/cross-realm fixture identity comes from manual labels, not inferred return values.
- [x] Capture `/apicontract numbers <label>` and `all` numeric samples for `C_StringUtil.FloorToNearestString` and `RoundToNearestString`, with `GetLocale` provenance, build and label. Use exactly the 25 finite inputs listed in the capture protocol: signed ties and their neighbors, integers, fractions and large finite magnitudes.
- [x] Preserve numeric helper result bytes, positional nils and exact arity through the existing bounded observer without guessing rounding or grouping. Missing APIs and failed access checks fail closed; errors remain opaque.
- [x] Redact inaccessible/secret values before comparison or saving, and record opaque failures without stringifying errors.
- [x] Bound capture count, inspected results, arrays, depth and strings.
- [x] Exercise actual addon loading and slash commands through local behavioral fixtures.

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
