# Ellesmere API-boundary audit

**PASS — bounded saved-artifact diagnostic only.** Recorded exit: **0**.

| Selected evidence | stdout count | stderr count | stderr line |
|---|---:|---:|---:|
| `IsUsingGamepad=nil / C_GamePad=table / IsEnabled=function` | 0 | 1 | 320 |
| `[EllesmereBoundaryComplete]` | 0 | 1 | 321 |
| `EllesmereUI_UICore.lua:1073: attempt to call a nil value` | 0 | 1 | 317 |

Counts cover selected emitted records only, not all startup failures or runtime occurrences. Outcome's empty `boundary_lines` does not mean no diagnostic: its both-stream records and direct stderr inspection agree. The nil-call headline precedes the producer diagnostic; no cross-stream timing inference is needed.

Current candidate line 1073: `return IsUsingGamepad() and C_GamePad.IsEnabled() or false`. This is consistent with the nil global observed later. Lua evaluates the first call before the second; the observed `IsEnabled` function does not supply the missing global. This is not native-client semantics proof or failure-instant instrumentation.

## Hash proof

- Candidate recorded before/after and audit before/after: `f7c73112a06f01008c14beb3c608704d3c43e83caf4497a83e5da5fb7ad08ff1` (all equal).
- Binary recorded before/after, existing compiler receipt, and audit: `a5e7c397129ec917ea32eb081bf5631b3cdccb8ae69c5e8c997cac981885988d` (all equal).
- Recorded source receipt: `1fac15bd0e477cd0fdf1f03dd0219dbbedfd4111`; compiler-artifact record, `fresh=false`. No rebuild or current HEAD acceptance.
- stdout: `278a0d047bf95244fc0800908b227b500b633f98daff4f557a3638bf3ac0adc1` (matches outcome).
- stderr: `0bb2259e98d1ec5c5e7730e6906069bd9ea116d87a6f7aa2c030daccc883ef0d` (matches outcome).

## Limits

- Actual selected addon root and exact loaded addon bytes are not proven; candidate identity is not load provenance.
- Current candidate hashes do not retroactively prove addon bytes in the older startup capture.
- Producer types were observed after the selected nil call, not instrumented at the failure instant.
- Binary/source association rests on the existing recorded compiler-artifact receipt, not independent rebuild or pristine-tree attestation.
- Bounded saved-artifact diagnostic only: no native parity, runtime fix, broad startup recertification, or current HEAD acceptance.

## Privacy

Raw streams remain untouched in the private capture directory. Only selected safe records and aggregates are published here and in `proof.json`; `privacy.json` records exclusions. Capture and state ancestor permissions are 0700; raw files are 0644 behind that boundary. No raw account payload, profile contents, environment dump, or unrelated diagnostic is copied. No simulator/build/test/check/network/repository edit/operations/delegation performed.
