# Secret-string formatting

Bounded `string.format` secret `%s` contract from [retained 12.0.5 prose](../../data/patch-api/sources/12.0.5-api-changes.txt), line 40, source ID `prose-2026-03-12-040`. This chronological PTR proposal is not proof of shipped/native behavior. See [patch audit](../wiki/investigations/patch-12-0-5-api-audit.md).

## What it must do

- [ ] Secret `%s` ignores width, alignment and precision: `%.5s` and `%12s` preserve the full `abcdefgh` payload without truncation or padding, including positional wrapper selection.
- [ ] Public `%s` retains normal width and precision behavior.
- [ ] Formatting produces a full secret string, not a public payload. **Inference:** tainted addon callers may perform this opaque operation while input/result `secretunwrap` remains rejected and stack taint remains intact.
- [ ] Preserve secure host inspection guards and GC-rooted payloads; expose no payload-reading capability or arbitrary callback access to addon code.

Observed simulator integration GREEN is bounded; checkboxes do not imply independent final acceptance.

## How it works

- [Patch audit and proof boundaries](../wiki/investigations/patch-12-0-5-api-audit.md#Secret-string-formatting-bounded-runtime-proof)

## Implementation inventory

- `Cargo.toml`, `Cargo.lock` — simulator pin adopted in `c5ba89ae3`, rilua `host-secret-bool` revision `6044544b960cd68b4b0c58bb3373412757c2caee`; no other dependency changes.
- External rilua formatter — bounded opaque `%s` implementation; published by MAIN after explicit user approval and remote verification.
- `tests/secret_string_formatting.rs` — six grouped simulator cases, direct/positional public and secret formatting, tainted caller guards and GC.

## Tests asserting this spec

- `tests/secret_string_formatting.rs`: actual old-pin RED at `eac08bda3`, 1 public PASS / 5 secret FAIL (`/tmp/patch-12.0.5-batch6-secret-format-red.log`). New-pin batch7 integration PASS 6/6; independent final acceptance remains pending.
- External rilua grouped tests: independent report `/tmp/rilua-secret-format-independent-proof.md` at `6044544b960cd68b4b0c58bb3373412757c2caee`, 6 formatter PASS / 2 host-guard PASS; fmt/check pass with pre-existing `strlen` warning. Not warning-free or native proof.

## Known gaps (current cycle)

- [ ] Independent batch7 audit and final acceptance remain pending after observed 6/6 simulator GREEN.
- [ ] Future native probe: secure and addon-tainted callers format host-secret `abcdefgh` with `%.5s`, `%12s`, left alignment and positional selection; record success/error, secret result classification, guarded full-payload inspection, input/output unwrap rejection and taint restoration, alongside public controls. Resolve inferred opaque permission separately from payload behavior.

## Out of scope

- `SetFormattedText`, display provenance and other formatting domains: unproven, not covered by this `%s` contract.
- Arbitrary callbacks, general secret reads/declassification, whole-page completion and native parity: not authorized or established by runtime development proof.

## Batch7 observed proof — 2026-10-01

Observed batch7 default build snapshot `c5ba89ae3d35a951cd77ca8b773b4bfc56ad9ebd`, rilua `6044544b960cd68b4b0c58bb3373412757c2caee`, compiled successfully in 34m51s. Exact argv, artifact SHA256 and referenced outputs: `/tmp/patch-12.0.5-batch7-integration-runs.json` and `/tmp/patch-12.0.5-batch7-lib-runs.json`. Independent verifier 104 report `/tmp/patch-12.0.5-batch7-independent-proof.md` was not yet available when recording these logs; no independently validated final acceptance, native parity or whole-page completion is claimed.

`secret_string_formatting::` PASS 6/6 (`/tmp/patch-12.0.5-batch7-integration-7.log`), now simulator integration proof rather than external runtime-only proof. Inferred opaque permission and display limits remain.
