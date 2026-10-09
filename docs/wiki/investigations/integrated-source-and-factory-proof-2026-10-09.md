# Integrated source and factory proof — 2026-10-09

## Integration scope

Main-provided integration identifiers: p303 `4470912f5`, p302 `16b9658da`, p242 `0b3144090`, p240 `923b0c986`, p230 `48691e5a8`, p220 `c2481c4c5`, p210 `8877566d2`, p201 `88217d328`, p1601 `861e7ae5e`, p1159 `60524271c`, p1158 `dd710c6fc`. Integration alone gives no execution or native credit. Existing per-page source ledgers, immutable source/factory seals and counts remain authoritative; new source slices remain source-only unless explicit bounded proof says otherwise.

## Bare Retail factory: bounded PASS, mixed checkout

[Independent report](../../../data/patch-api/evidence/3.0.2-factory-2026-10-09/integrated/p30x-integrated-independent-report.md) records ten tests passing: p302 7/7, p303 3/3. Exact observations: p302 **373 rows / 185 matches / 188 gaps**; p303 **3 CVars / 1 match / 2 gaps**. Every observation equals retained GREEN JSON. `syncronizeConfig` and `synchronizeMacros` are nil/nil; `synchronizeBindings` is `"1"`/`"1"`. These are present factory observations, not historical defaults or synchronization semantics.

Requested revision `16b9658da`; observed start HEAD `2d83b9b8`, end HEAD `c2481c4c`, with staged integration and unrelated unresolved files. Relevant source tree, classifier, tests and Cargo.lock matched the requested revision; Cargo.toml additionally registered p242. Hash equality supports relevant-input scope, **not an immutable-commit execution claim**. [Epoch scope](../../../data/patch-api/evidence/3.0.2-factory-2026-10-09/integrated/p30x-integrated-independent-epoch-scope.json) and [comparison](../../../data/patch-api/evidence/3.0.2-factory-2026-10-09/integrated/p30x-integrated-independent-comparison.json) retain that distinction.

Original p302 36+1+2 seals, p303 16 seals, factory p302 112 seals and p303 16 seals validated; 197 protected files unchanged. The retained fabricated-global negative receipt has 373 rows / 189 gaps, exit 101, exactly AutoLootMailItem changed. Its older source tree differs from this run: sealed classifier/receipt evidence only, **not fresh current-runtime mutation rejection**. No loaded-UI, native, synchronization, model, global gate or final acceptance credit.

[Byte-identical retention manifest](../../../data/patch-api/evidence/3.0.2-factory-2026-10-09/integrated/p30x-retention.json) links original command, full streams, observations, comparisons and preservation receipts. Original seals were not rewritten.

## TBC SOURCE: retained bounded PASS, historical global FAIL

[Single retained independent report](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/integrated/p251-256-independent-report.md) is the count/status SSOT for 2.5.1–2.5.6: source 50/50, historical 6/6, relocated archive 6/6 at `fcdf1d151`. All source/native contracts remain UNPROVEN. Its global gate failed on three archived template snapshots; this is historical failure, not a fresh current gate. Later discovery repair proof is separately scoped in [narrow repairs](narrow-validator-discovery-and-forever-cfg-repairs.md).

[Aggregate raw receipts](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/integrated/p251-256-independent-raw-receipts.json) and [retention manifest](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/integrated/tbc-independent-retention.json) preserve original bytes and per-page receipts without duplicating facts across pages.

## Browser readiness handoff

[Main-thread readiness report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/browser-vnc-independent-report.md) retains loopback-only CDP 9222/VNC 5901 and a readable current GetSessionTime page with no visible challenge at observation. No remote access/authentication test, frozen-revision validation or native API proof. Earlier direct HTTP403 and current browser readability are distinct observations.

## 2.4.x shared-tool integration: source/historical PASS

[Independent report](../../../data/patch-api/evidence/2.4.0-source-2026-10-09/integrated/p24x-integrated-independent-report.md) covers selected content at `dd710c6fc`: **18/18** targeted fixtures; **170/170** default outcomes equal archived baseline. One whole replay, with a scoped harness correction for the CLI-only `--text-only` flag, not a repeated whole gate. Own 2.4.0/2.4.2/3.0.2/2.1.0 opt-ins match; both parser function returns retained.

Recorded saved outputs: **83/87 registers**, **78/87 extracts**. The report enumerates inherited differences, absent current provenance flags and the exact 12.1.0 template error; baseline equality is not universal saved-byte reproduction. Fresh copied no-Git validators pass; 23+5 and 432+3 seals unchanged. [Full retention](../../../data/patch-api/evidence/2.4.0-source-2026-10-09/integrated/main-retention.json) preserves receipts/output/hash scopes. No native/model acceptance.

## Older source integration: bounded PASS

[Independent report](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/integrated/older-source/report.md) at `dd710c6fc` records current fixtures: 2.3.0 **6/6**, 2.2.0 **5/5**, 2.1.0 **4/4**, 2.0.1 **6/6**, WowForever1.60.1 **8/8**, Era1.15.9 **8/8**, 1.15.8 **8/8**. Seven copied no-Git historical validators pass; 137 original seals preserved, 14 serialized tamper rejections and 14 exact restorations. One verifier-only wrong path retained and corrected. [Receipts](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/integrated/older-source/main-retention.json) preserve selected hash scopes. Source-only proof; the separate Forever configured-name test is linked below, not credited to these fixtures.

## Era1.15.7–1.15.5 integration: metadata/link-only PASS

[Independent report](../../../data/patch-api/evidence/1.15.7-session-2026-10-09/integrated/era1157-1155/REPORT.md) at `65b94c050`: each current/copy SOURCE **8/8**, portable **3/3**. Original/separate seals 28+8, 34+6, 38+6 unchanged; disk ledger/log rejections and exact restorations pass. Each page has four nonblank rows, two UNPROVEN links and zero local inventory. Default empty-register byte equality and nine successor identity comparisons are not supersession, runtime/model or native proof. [Receipts](../../../data/patch-api/evidence/1.15.7-session-2026-10-09/integrated/era1157-1155/main-retention.json) retain exact commands, streams and environments. Literal TOCs, absent client names and configured Era11507 remain distinct.

## Era1.15.4–1.15.0 integration: bounded SOURCE PASS

Verified: 2026-10-09. Counts/status below are the current SSOT; original queued/in-flight ledger notes remain historical, not current integration status. Actual canonical successor identity/context reconciliation does not expand linked contracts or establish semantic supersession.

| Page | SOURCE / copied SOURCE | Portable | Original + separate receipt seals | Retained proof epoch |
|---|---|---|---|---|
| 1.15.4 | 9/9 | 3/3 | 44 + 6 | `0b64e636c` |
| 1.15.3 | 10/10 | 3/3 | 50 + 6 | `0b64e636c` |
| 1.15.2 | 9/9 | 3/3 | 54 + 6 | `bf0e83720` |
| 1.15.1 | 8/8 | 3/3 | 57 + 9 | `17cc9b0f2` |
| 1.15.0 | 10/10 | 3/3 | 64 + 5 | `17cc9b0f2` |

[1.15.4/3 report](../../../data/patch-api/evidence/1.15.3-session-2026-10-09/integrated/era1154-1153/report.md), [1.15.2 report](../../../data/patch-api/evidence/1.15.2-session-2026-10-09/integrated/report.json), and [1.15.1/0 report](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/integrated/era1151-1150/REPORT.md) retain exact scopes and commands. Canonical-cwd copied controls reproduce default bytes, reject serialized ledger/log changes and restore exact bytes; original seals remain unchanged. For 1.15.4/3 the scratch wrapper changes only historical harness cwd; initial SyntaxError attempts earn no credit. For 1.15.0 independent canonical-cwd controls are separate from the original worktree-bound harness. No runtime/model/native credit follows from SOURCE counts.

Byte-identical retention manifests: [1.15.4/3](../../../data/patch-api/evidence/1.15.3-session-2026-10-09/integrated/era1154-1153/main-retention.json), [1.15.2](../../../data/patch-api/evidence/1.15.2-session-2026-10-09/integrated/main-retention.json), [1.15.1/0](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/integrated/era1151-1150/main-retention.json). Main retains the 144 selected files separately; this documentation update does not alter or reseal them.

## Official Lua alias: retained 1/1 REUSED

The [1.15.1/0 report](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/integrated/era1151-1150/REPORT.md#retained-headless-runtime-proof-reuse) supports reuse of **1/1 at `714277147`**, not a fresh Cargo run at current HEAD. Its original relevant-input inventory is linked in [runtime hash evidence](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/integrated/era1151-1150/runtime-reuse-input-hashes.json); the independent verifier confirms current relevant-input hashes unchanged. No Cargo rerun was performed for this docs update.

Real WowLuaEnv, existing `IsPublicBuild=true`, raw SeasonOfDiscovery numeric2 and initially absent Placeholder; direct unchanged pinned official Lua yields both raw names numeric2/equal. No mocks, flag injection or pre-test enum write. Native numeric correspondence, pre-vendor timing, full Blizzard UI loading, C_Seasons state and non-public branch remain UNPROVEN. 1.15.0 receives no alias/runtime credit.

## Era1.14.4: integrated source, retained development proof only

[1.14.4 audit](patch-1-14-4-api-audit.md#copied-historical-proof) retains SOURCE11/11 at `183437dc5`, portable3/3 at `fb6e6ba53`, 67 original + four separate receipt seals. Integration is not a fresh independent replay: no such later report is included in the selected retention. All eleven linked/prose contracts remain UNPROVEN; no native/model/runtime or foreign-history supersession credit.

## Era narrow-helper integration: bounded PASS

[Independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era-helper/report.md) records Era **2/2 + 1/1** and Retail **78/78** at report HEAD `5b12dac256abc0ffaf541df2bc445e8381612d2d`; relevant source hash scopes unchanged across concurrent docs/data commits. All **412** factory observations equal retained inventory, including IDs/order/details/values/defaults/classifications: **286** direction matches / **126** reviewed mismatches unchanged. The original **18** target warnings are absent; baseline **7 simulator** (6 library + 1 binary) and **6 vendor manifest** warnings remain. No suppressions or visibility expansion.

Copied SOURCE **8/8 each**, corrected portable **3/3 each**; original/current seals **54+21** (1.14.3) and **72+17** (1.14.2) unchanged. Initial portable harness setup failures remain retained, not passing proof. First combined Era streams were transcribed verbatim from the tool result, not independently captured raw streams; exact start/end timestamps unavailable. Later receipts retain direct captures and timestamps. No native/model/event-producer credit: all observation native/model credit fields false; accepting a nonsense event name proves no producer.

[Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era-helper/main-retention.json) preserves 32 original files / 1,155,904 bytes byte-identically, including report, receipts, both hash scopes, comparison, observations, successful and failed streams, and reused consumer logs. Original seals untouched; scratch fixture trees and recording helper excluded. These are bounded proof epochs, not latest-HEAD blanket acceptance.

## Saved full suite: FAIL, broader goal open

[Full-suite report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-dd/fullsuite-dd-independent-report.md) and [exact comparison](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-dd/fullsuite-dd-independent-comparison.json) retain **FAIL at `dd710c6fc`**: 23 integration + one Garrison prefork + six library failures, same identities/boundaries as `614402d56`. 29/30 raw assertion payloads equal; the remaining payload differs only a random missing-directory suffix. No newly failing identity against that baseline; no root-cause repair inferred. This is saved-run proof, not execution at later HEAD.

Twelve new integration cases and two new prefork cases PASS. Exactly 75 executed publication sweeps plus three helpers PASS; selected-name inventories are not extra sweep credit. Excluded Era/Forever profile targets receive no credit. The runner's `new_failures` flag is not the direct `614` comparison; its precise baseline selection remains unproven.

Main's four partial-startup cases remain in ordinary integration. The new Cargo/shared-helper refactor was in flight and not proven by this retained suite; the later bounded Era-helper proof above is separate, not full-suite acceptance; ignored, untracked PLAN is not acceptance evidence. No fresh tests or broad gates run here. **Broader goal remains open**, including native/full-UI and final integration gaps.

## Era1.14.1/1.14.0: independent bounded epoch

[Exact report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1141-1140/report.md) requested `ce40cfb899bcc342457fc1cd03c2b23b26969406`; checkout later advanced to `045e396b0`. Copied SOURCE **8/8** (1.14.1) and **10/10** (1.14.0), portable **3/3 each**, and standalone 1.14.1 getter **1/1** pass. First FontString changes 1.5→2.25; second remains 0.75. Original/separate receipt seals **160+10 / 92+5** match; original seal-map hashes and all protected evidence bytes remain unchanged. The 1.14.0 default generator's expected `: Scripts` rejection remains a negative control, not a failure or new default inference.

[Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1141-1140/main-retention.json) binds 95 byte-identical files, including original receipts, streams, archives, preservation and runtime scope. [Concurrent scope note](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1141-1140/concurrent-scope-note.json) records Cargo drift; [literal successor comparison](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1141-1140/successor-literal-comparison.json) is identity/input inspection only, not semantic supersession. Frozen queued/in-flight statements retain their original epoch. No native/signature/default/alias, security, loaded-UI or broad acceptance credit.

## Forever standalone contracts: independent bounded epoch

[Exact report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/report.md) records **3/3** at actual `045e396b0c6f717c2d962b97cdf46b736a3328ce`, with [compiled-scope equivalence](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/equivalence.json) to requested `f611a6752a14b4d660902417c59e14073de9ac31`. Later SOURCE merges do not promote that epoch to latest prefork-runtime acceptance. [Function-gate SSOT](forever-cfg-failure-and-function-gates.md#independent-standalone-proof-epoch) details ordered records, environment isolation, two invalid-prefix results with one shared final snapshot, and exact getter absence. Formatting and manual changed-Rust readability pass; **14 warnings retained**, not warning-free proof.

[Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/main-retention.json) binds 14 byte-identical report/receipt/scope/comparison/stream files; original seals unchanged. No native/security/transport/inbound-delivery or fresh Retail restriction credit. Prefork migration remains outside this retention task; no migration-pass claim.

## Sources

- Retained independent reports, comparisons, hash evidence and retention manifests linked above — bounded execution epochs, not latest-HEAD blanket proof.

## See also

- [3.0.2 audit](patch-3-0-2-api-audit.md), [3.0.3 audit](patch-3-0-3-api-audit.md).
- [Forever failure/functions gates](forever-cfg-failure-and-function-gates.md).
