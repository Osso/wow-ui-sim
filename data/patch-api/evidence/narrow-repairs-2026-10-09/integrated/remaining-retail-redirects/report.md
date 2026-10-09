# Integrated independent SOURCE verification

**PASS — bounded integrated SOURCE gate only.** 7/7 copied audits exit 0; 38/38 SOURCE tests; 21/21 portable tests. All 21 proof invocations exit 0. 231 original seals and 70 separately sealed receipt files match; 238 archive members match integrated originals exactly.

## Actual epochs and scope

Requested baseline: `0d687a77b8d241fc616a9e85321195842cc04f7c`. Actual proof revision before/after every invocation: `e20c472caa43b2a422cef615579d03b22bf51866`. Initial/final recorded HEAD equal. [Requested-to-actual bounded diff](requested-to-actual-scope.json): exit 0, empty stdout/stderr; these seven evidence directories did not change between the requested and actual revisions.

Capture window: `2026-10-09T19:42:12.327145+00:00` through `2026-10-09T19:42:36.345257+00:00` UTC; final byte/scope comparison at `2026-10-09T19:43:30.727168+00:00`. Explicit CLI cwd throughout: `/home/osso/Projects/wow/wow-ui-sim`. Pre-existing `?? .code-index.db` unchanged in initial/final status. All seven evidence directory file maps unchanged before/after each invocation and at final scope check.

Read each page’s own HANDOFF/REPLAY and inspected its active scripts/pin rules. Fresh copies came from that page’s immutable archive, not sibling proofs. Every archive member was validated as a unique regular relative path, checked against its original map, and byte-compared to the integrated original before extraction. Receipt maps checked separately; no overlap with original maps. Original docs remain their own archived snapshots. Child execution epochs are not substituted for this fresh integrated proof.

## Coverage matrix

| Retail page | Original / receipt seals | Archive members | Copied audit | SOURCE | Portable |
|---|---:|---:|---|---|---|
| [1.10.1](1.10.1/page.json) | 29 / 6 | 30 | exit 0 | 5/5 | 3/3 |
| [1.10.0](1.10.0/page.json) | 25 / 7 | 26 | exit 0 | 5/5 | 3/3 |
| [1.9.0](1.9.0/page.json) | 30 / 10 | 31 | exit 0 | 6/6 | 3/3 |
| [1.8.0](1.8.0/page.json) | 40 / 16 | 41 | exit 0 | 5/5 | 3/3 |
| [1.7.0](1.7.0/page.json) | 46 / 17 | 47 | exit 0 | 6/6 | 3/3 |
| [1.6.0](1.6.0/page.json) | 30 / 7 | 31 | exit 0 | 5/5 | 3/3 |
| [1.5.0](1.5.0/page.json) | 31 / 7 | 32 | exit 0 | 6/6 | 3/3 |

All copied audits report exactly: physical/nonblank/metadata rows 1/1/1, references 1, UNPROVEN contracts 1; inventory/signatures/defaults/prose/headers/templates 0 each. Every page retains an unexpanded redirect. Own archived generator default bytes, extractor function default bytes and malformed-input error types/messages replayed; no general shared-tool or historical behavior acceptance claimed.

## Tamper/restoration proof

Each fresh portable run rejected both a serialized ledger-reference omission and fabricated green-log claim by the original seal, restored exact original bytes/hashes, preserved the original map, then replayed successfully. 14/14 rejection controls and 14/14 exact restorations. New receipts live outside immutable copies; originals were never resealed. Each page’s `fresh-portable-proof.json` contains all five receipt entries and actual rejected/restored hashes. The 1.10.1/1.10.0 script’s fabricated line mentions 1.11: retained literally as a rejected fabrication, not transplanted or accepted proof.

## Hash scope (pre = post = final)

Scope digest is SHA256 of sorted compact JSON mapping relative file paths to their SHA256. Includes each integrated evidence directory’s files, maps, archive, later receipts and docs. Per-invocation explicit maps cover both integrated evidence and archive copies.

- 1.10.1: `6b8f48afcdad1e0ec144747fd09d658b3a41cf824bbb214cc454dc307ca0bbdf`; archive `93739459bb668016b4b56d8dc683e5e56add1bc81ee232ff2ee1b1d326f94438`; original map `6cdbf8ac0ec4111f01574718c30f2e0103283c6101a26691e2a6c4390d0d04c9`.
- 1.10.0: `7bd916fadd64ccc66cda32483f04ed1b2c5c4d7de309e952cc3f2699277592e2`; archive `06a4f7047bb1733be2b9e1cefc6bf56977d774e9be2c6973e4840ce13a8e5553`; original map `ca1dd001a840cd744346c9c0f39b22dfc1cc49023a4a6832a3b0e81941a5c9bc`.
- 1.9.0: `83ac03f317bc42addc0a3a2bb96f8a82d4b97977589ae5b5d06eeb5f529e0c6d`; archive `6706bbcd2bea4939572702cb39c241666521dbab3a31ebb39e0929f562b66afe`; original map `70f4d6a102ba93defa5fab63ff9bb81a99001501f5f4277a0824eaf19e741a28`.
- 1.8.0: `e6c375a1d8ada5d91262d8eb3ae171c1fc0eaf252b131db8c3bb7a79cef6c9cb`; archive `0764cb47f1436bffd42d826b437134cfdeeb8819744c63b556aa600d9cbbc6e1`; original map `421372f1a47738ccecd90e2d48b5ca99e5e7710848b208b6667ed48db4c75dc4`.
- 1.7.0: `1bad17c8f54cd28ccb59c57d1c106319058f758b07ec72694dbd87c928dc7be3`; archive `955255a99902a3076c3eebe50798b4fa78f399251e80afbdc3c4e246400822c7`; original map `38129730211271ef3223ec1b3a76ec92ebdd0d9eb2c8a444a81c2acc07e9b153`.
- 1.6.0: `baa85e10614c9c9f3f418776f4a0cb9951f78694ebcaca5378a61939d15038ef`; archive `f25bd16c4f07ff7abc04e73bd2547353ce7614c33cf9e2945fc9bbe9f1089bde`; original map `b193b6752c7fb6e2686f27aef28b7bd2321f7a8c18846c23198e27bf8aa7702f`.
- 1.5.0: `dffadc05c25658ebfed61076364995bb21ca28cc3090bd898c9dd4db75fbfdc1`; archive `a83268b3517583f477cc023ae7614449d448606e4c8feb9102806ec59c8525bf`; original map `63ff91b8064989a422d5aa62c195cb895ea4b98b6ef9a1e80d173a0ee88581db`.

## Exact command/stream artifacts

Each page retains `1-audit.receipt.json`, `2-test_source_accounting.receipt.json`, `3-test_portable.receipt.json`: exact argv, explicit cwd, actual pre/post revision, UTC start/end, real exit, environment assignments, pre/post scope digests and full stream paths. Corresponding `.stdout`/`.stderr` files contain full unaltered streams (including empty streams), read in full. Corresponding integrated/copy pre/post maps retain exact hashes. No stdout/stderr chronological interleaving claim.

Command shape used independently for each own page:

```text
/usr/bin/python3 -I -B /tmp/remaining-redirects-integrated-independent/<version>/archive-copy/audit.py
/usr/bin/python3 -I -B /tmp/remaining-redirects-integrated-independent/<version>/archive-copy/test_source_accounting.py
/usr/bin/python3 -I -B /tmp/remaining-redirects-integrated-independent/<version>/archive-copy/test_portable.py /tmp/remaining-redirects-integrated-independent/<version>/fresh-portable-proof.json
```

Explicit environment: `PATH=/nonexistent`, `PYTHONNOUSERSITE=1`, `PYTHONDONTWRITEBYTECODE=1`. Canonical cwd replaces obsolete child-worktree cwd only; page-relative scripts, own frozen inputs and pin semantics unchanged. No temporary writes in canonical: audit/test temporary writes resolve inside copied evidence.

Supporting artifacts: [summary](summary.json), [Git command epochs/full streams](commands.json), [final scope check](final-scope-check.json), [capture losses](capture-losses.json), [privacy inspection](privacy.json). Each page includes archive member identity, own script inspection and read-doc snapshots. Controller source retained as `verify_source.py`; it was corrected for the actual `seals_sha256` context field before any proof invocation.

## Losses, privacy, limits

No proof command capture loss. Two unavailable discovery tools and the initial controller’s blank-error wrapper have no captured numeric exit; none is fabricated or counted as proof. Exact setup timestamps absent where stated in capture-losses.json. No broad command rerun; each of 21 bounded proof commands ran once.

Sensitive environment key names inspected without reading/retaining values; only explicit replay environment assignments retained. Evidence and archive-member bytes inspected before copying; streams scanned before retention. No selected high-specificity credential patterns matched. Final artifact scan recorded separately. Heuristic scanning is not exhaustive privacy assurance.

**No historical/native/model/runtime/parent closure credit.** Redirect target revision/content and patch-specific behavior stay UNPROVEN. No Cargo/build/runtime/client/network, delegation, operational mutation, tracked-file/seal edit, push/merge or worktree deletion performed. Verification confirms SOURCE integrity/accounting and portable rejection/restoration only.
