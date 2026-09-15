# API Contract Probe

Manual native observations for scalar curve points, `UnitSexBase`, unit names/realms, finite numeric formatting, publication/CVars and event payloads. Complements [AuraDispelCurveProbe](../AuraDispelCurveProbe/README.md). Neither recorder infers native contracts from fixture outputs.

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

## Observations

**Curves:** creates an empty curve and another with insertion order `(30,4), (10,7), (20,2)`. Records `GetPoints`, `GetPoint` at `0,1,2,3,-1,4`, return arity, accessible `x`/`y` and `GetXY`, and repeated-point equality. On a separate curve, attempts `GetPoint(1).x = 91` and records another lookup. Index 1 may be invalid; rejection is evidence, not failure of the experiment. Array inspection is bounded to four entries, return inspection to eight values, and object nesting to two levels.

**Names:** `/apicontract names same-realm-party1-cross-realm-party2` records `UnitName` (`name`) and `UnitNameUnmodified` (`unmodified`) under `names.units` for `player`, `party1`–`party4`, `target`, `nonexistent`, `invalid-unit-token` and the empty string. Every token is queried without an existence check. Exact return counts and positional nils distinguish zero returns, nil realms, empty realms and explicit realm text; results are not normalized. Establish actual same/cross-realm membership independently, then label each manual snapshot with the relevant slots and scenario. Unknown/invalid token spellings are fixed test inputs, not assumptions about native results. No party, target or gameplay changes are made. Name/realm captures contain character identities; retain them privately.

**Numbers:** `/apicontract numbers <label>` records `GetLocale` under `numbers.locale` and 25 ordered `numbers.samples` with `input`, `floor` (`C_StringUtil.FloorToNearestString`) and `round` (`RoundToNearestString`). Corpus: `-2.5,-1.5,-0.5,0,0.5,1.5,2.5`; `-0.500001,-0.499999,0.499999,0.500001`; `-1,1,-100,100`; `-0.1,0.1,-0.9,0.9`; `-1234.5678,1234.5678,-1e6,1e6,-1e12,1e12`. Retains accessible result bytes and exact arity using the existing bounded observer, including nil slots and zero returns; never derives expected output from helper names. Repeat manually on each matching build/locale, retaining labels and build provenance. Missing APIs/access checks fail closed and errors remain opaque. Native output corpora remain pending; nonfinite inputs, coercion, security behavior and values outside this corpus are not probed.

**Casts:** `/apicontract casts <label>` records `UnitCastingInfo` (`casting`) and `UnitChannelInfo` (`channel`) under `casts.units` for exactly `player`, `target`, `focus`, `party1`, `nonexistent`, `invalid-unit-token` and the empty string. No existence gating, casting or gameplay actions occur. A dedicated accessibility-first scalar-only observer retains exact arity and up to 16 positions, including interior/trailing nils; larger tuples set `truncated = true`. Accessible objects retain only status/kind, never fields, methods, equality or string conversion. IDs are recorded without coercion or assumptions that castID and castBarID match. Existing string bounds apply.

The pinned declarations in `data/patch-api/sources/12.0.0-register.json` list ten casting results (castID at 7, castBarID at 10) and eleven channel results (empowered flag at 9, stage count at 10, castBarID at 11). These declarations guide observation, **not native proof**; the current client may differ. Matching native builds, controllable player casts, non-player channels and empowered/non-empowered fixtures remain pending.

Use separately labeled manual snapshots for idle, active, repeated same cast, consecutive/replaced casts, completed and cancelled states. Capture channels and empowerment scenarios in separate batches: the shared ten-snapshot cap also counts `all` and other modes. Retain each flushed artifact with scenario notes, then move the saved file aside while logged out to begin another batch. Do not interpret timing or identity stability from unrelated captures or fixture values.

**Sex:** records `UnitExists`, then raw `UnitSex` and `UnitSexBase` returns for existing player/target/focus/pet units, plus named `Enum.UnitSex` values. No legacy-to-enum conversion or assumption about what “Base” means.

Restricted values are redacted before comparison or serialization. API errors are opaque status labels, not stringified error objects. Missing access APIs fail closed. Rejected addon-tainted calls are inconclusive; do not bypass restrictions. String observations are truncated to 256 characters.

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
