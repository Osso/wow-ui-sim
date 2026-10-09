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

## Pending p24x report

Main reports 18/18 own fixtures, 170/170 default outcomes matching archived baseline at `dd710c6fc`; recorded 87 registers, 83 saved-byte matches, 78/87 extracts with inherited/no-flags limits and matching own opt-ins. Final independent report has not arrived in this retention slice. These are main-reported results, not independently retained proof here; no broad/native/model credit.

## See also

- [3.0.2 audit](patch-3-0-2-api-audit.md), [3.0.3 audit](patch-3-0-3-api-audit.md).
- [Forever failure/functions gates](forever-cfg-failure-and-function-gates.md).
