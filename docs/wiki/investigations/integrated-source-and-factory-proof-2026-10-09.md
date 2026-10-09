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

## See also

- [3.0.2 audit](patch-3-0-2-api-audit.md), [3.0.3 audit](patch-3-0-3-api-audit.md).
- [Forever failure/functions gates](forever-cfg-failure-and-function-gates.md).
