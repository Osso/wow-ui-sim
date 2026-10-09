# Frozen Era Patch 1.13.6 API audit

Independent bounded literal audit of page461367/revision4435603, timestamp2021-05-06T13:20:50Z; retrieved2026-10-09T08:51:36.400174+00:00. Original827-byte body SHA256 `21a9bf3248ca5db0bee46c5fec4d0332744a6e089a52e17b38dde8dbebc4e4c9`; response1139bytes SHA256 `46e81af7accc100716c0fd72c06060ec94cc9a3b4e1eae3b27778d72595c5e9d`. Manifest/response/body read and matched before derivation. No network retrieval or linked expansion.

## Literal coverage

| Boundary | Exact coverage | Proof level |
|---|---|---|
| Source | Every nonblank physical line, unchanged | SOURCE GREEN7/7 |
| Inventory | `nameplateCommentatorMaxDistance`, `specular`, `textureErrorColors`; all added CVars | Publication names only; behavior UNPROVEN |
| Headers | Summary, CVars; literal `3 new cvars` matches three occurrences | Independent accounting; default generator header_counts empty retained |
| Prose | `Patch 1.13.6 [[Naxxramas (Classic)\|Naxxramas]] content patch.` | Attribution only; no modeled effect contract |
| References | Naxxramas link; exact wow-ui-source compare and BlizzardInterfaceResources commit | Three unexpanded boundaries, all UNPROVEN |
| Templates | Navigation prev1.13.5/next1.13.7; three api CVar references | Four unexpanded occurrences |
| Defaults/signatures/examples | Zero literal declarations | No invented values or imported callable contracts |

Seven contracts remain UNPROVEN: three CVar publication-only declarations, three linked boundaries, one content-patch attribution. Source TOC11306 is not current configured Era/Anniversary11507. No TBC/retail/Forever supersession, no current-client or native equivalence from publication.

## Current model assessment

Read-only `src/`/`tests/` exact-symbol scan found no modeled callsites for these three names. `src/cvars.yaml` has current `specular: '1'` and `textureErrorColors: '1'`; these are simulator configuration, not source defaults. Current shared CVar getters use `SimState.cvars`; generic storage alone cannot establish the three CVars' effects or native historical contracts. Source has no default/type/range/security/persistence/effect details sufficient for a meaningful production edit. No runtime proposal warranted; leave precise gaps. Current bare-Era getter observations, if taken, remain separate from sealed SOURCE measurements.

## Current bare Era observation matrix

| Source name | Current value/default | Publication | Model/native credit |
|---|---|---|---|
| nameplateCommentatorMaxDistance | nil/nil | Missing | None |
| specular | `1`/`1` | Published | None |
| textureErrorColors | `1`/`1` | Published | None |

Current getter raw types are both `function`; an unknown CVar returns nil/nil, not invented defaults. Source defaults remain null for all three. Compatibility GetBuildInfo tuple `12.0.7`/`68256`/`120007` is separately observed, neither activeEra11507 nor source11306 native proof. No production identity changes.

At `e44841ae3`, the initially empty exact-gap fixture fails with precisely `wt-cvars-nameplateCommentatorMaxDistance-14`; unknown-name control passes1/1. At `2d2659297`, revised exact one-ID gap fixture plus complete concrete observation equality passes1/1; unchanged negative proof remains valid, not rerun. All RED/GREEN observations byte-identical. Remaining source contracts7/7 UNPROVEN; current publication two matches/one mismatch does not close them. Seven inherited simulator and six vendor-manifest warnings retained unsuppressed; narrow helper adds no unused sweep machinery. Only offline/locked Cargo, own target/TMPDIR and existing package-home bookkeeping used. No CoW/cache/toolchain/dependency/network/vendor changes.

## Proof and replay

Own SOURCE RED7/7 retained at `fe64d92ca`; GREEN7/7 at `d69734ceb` covers55 omission controls and fabricated defaults/credit/history rejection. Portable RED3 retained; GREEN3/3 at `271ed3a03` runs fresh copied SOURCE7, default register/extract byte replay and both serialized seal rejections/exact restorations. Original88 seals and89-member/138222-byte archive remain immutable; later receipts separate. Shared parser/extractor defaults are unchanged: copied generator reproduces three inventory entries; copied default extractor retains six nonblank text lines. Original seals/archive are immutable after capture; current receipts stay separate. No broad/check/lint/type/coverage/final gates.

## Successor boundaries

Own immutable inputs retain 1.13.7 in flight; 1.14.0/1.14.1 queued; actual 1.14.2..1.14.4 and 1.15.0..1.15.9 source pins/statuses present but unapplied. `later_registers` stays empty. Main resolves ordered same-Era histories, integration, native proof and acceptance; no supersession imported here.

## Sources

- [Frozen SOURCE and ledger](../../../data/patch-api/evidence/1.13.6-session-2026-10-09/).
- [Specification](../../specs/patch-1-13-6-source-accounting.md).
- [Read-only 1.14.3 template](patch-1-14-3-api-audit.md) and [1.14.2 template](patch-1-14-2-api-audit.md): boundaries only, no borrowed proof.

## See Also

- [[client-profiles]] — configured profile differs from source identity.
