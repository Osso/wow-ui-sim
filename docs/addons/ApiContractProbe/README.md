# API Contract Probe

Manual native observations for scalar curve points and `UnitSexBase`. Complements [AuraDispelCurveProbe](../AuraDispelCurveProbe/README.md). Neither recorder infers native contracts from fixture outputs.

## Target

Interface `120100` follows the locally pinned retail `12.1.0.69497` source, not a newly observed desktop build. Install only on a matching client. No installation or native execution has been performed by this task.

## Capture

1. Copy this directory to the matching client's `Interface/AddOns/ApiContractProbe`, then enable it.
2. Run `/apicontract all normal` to capture both experiments. Use `/apicontract curves` or `/apicontract sex <label>` to capture separately.
3. For sex comparisons, select real targets or repeat during an independently obtained transformation, labeling the scenario. The addon does not alter units or gameplay.
4. Run `/reload` or log out to flush SavedVariables.
5. Retain `WTF/Account/<ACCOUNT>/SavedVariables/ApiContractProbe.lua` with scenario notes. Keep the raw artifact unchanged.

Ten captures maximum; further calls increment `dropped`. No automatic captures. Start a new capture batch by moving the saved file aside while logged out.

## Observations

**Curves:** creates an empty curve and another with insertion order `(30,4), (10,7), (20,2)`. Records `GetPoints`, `GetPoint` at `0,1,2,3,-1,4`, return arity, accessible `x`/`y` and `GetXY`, and repeated-point equality. On a separate curve, attempts `GetPoint(1).x = 91` and records another lookup. Index 1 may be invalid; rejection is evidence, not failure of the experiment. Array inspection is bounded to four entries, return inspection to eight values, and object nesting to two levels.

**Sex:** records `UnitExists`, then raw `UnitSex` and `UnitSexBase` returns for existing player/target/focus/pet units, plus named `Enum.UnitSex` values. No legacy-to-enum conversion or assumption about what “Base” means.

Restricted values are redacted before comparison or serialization. API errors are opaque status labels, not stringified error objects. Missing access APIs fail closed. Rejected addon-tainted calls are inconclusive; do not bypass restrictions. String observations are truncated to 256 characters.

## Local fixture

```text
luajit docs/addons/ApiContractProbe/tests/harness.lua docs/addons/ApiContractProbe
```

Fixtures test the recorder under different fake client behaviors. They do not establish native indexing, ordering, copy/identity, sex numbering, transformation or security semantics. See [spec](../../specs/api-contract-probe.md).
