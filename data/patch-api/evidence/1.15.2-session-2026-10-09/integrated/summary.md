# Independent Era 1.15.2 SOURCE verification

PASS at bf0e8372057bfdcd3751c9cb62a371fb00ac4b7c. Canonical cwd /home/osso/Projects/wow/wow-ui-sim.

- Canonical SOURCE 9/9 exit 0; copied SOURCE 9/9 exit 0; portable 3/3 exit 0.
- Copied historical validator exit 0; no Git/target/current-tools in copies; empty PATH. Default historical generator byte-identical.
- Serialized ledger omission: exit 1, AssertionError: seal: ledger.json. Serialized GREEN fabrication: exit 1, AssertionError: seal: green.log. Both restored exact bytes, all 54 copied seals verified, validator exit 0 afterward.
- Original 54 seals and separate 6 receipt seals checked before/after unchanged; seal maps, archive and receipts unchanged.
- Four substantive contracts remain UNPROVEN; Dragonflight 10.2.6 remains unexpanded attribution, not imported contracts or parity.

## Successor reconciliation

| Era | Original frozen status | Actual canonical integration | Exact pin/raw/response bytes |
|---|---|---|---|
| 1.15.3 | queued-pending-main-integration | 0b64e636c16c59e90e3406e7ff557e94cd7732a8 | equal |
| 1.15.4 | integrated-canonical-input-not-applied | 7ad66791e64f01dd6dc61d29d3aa48faccdce3b8 | equal |
| 1.15.5 | queued-pending-main-integration | 65b94c05023dc5e1e8422de0cea99c0e7bda9c10 | equal |
| 1.15.6 | queued-pending-main-integration | f3275801488519ea64a2fa6a6750240cf799924d | equal |
| 1.15.7 | integrated-canonical-input-not-applied | d9a00a294ee35c414e0527ea5d34d1d6c09207e9 | equal |
| 1.15.8 | integrated-canonical-input-not-applied | dd710c6fc469a53e3b77b97aa6340993ae83e46e | equal |
| 1.15.9 | integrated-canonical-input-not-applied | 60524271c44d40568e60dc90ffea5bb258325105 | equal |

Original statuses were not rewritten. All seven source-only integrations are separately supported by tracked files and first-parent merge history. No supersession/native/model/runtime/inventory claim.

## Main retention paths

/tmp/era1152-independent-20261009-150332/report.json
/tmp/era1152-independent-20261009-150332/successor-git-evidence.json
/tmp/era1152-independent-20261009-150332/integration-history.json
/tmp/era1152-independent-20261009-150332/final-status.json
/tmp/era1152-independent-20261009-150332-commands.json
/tmp/era1152-independent-20261009-150332-command-0.json
/tmp/era1152-independent-20261009-150332-command-1.json
/tmp/era1152-independent-verify.py

Full argv/cwd/revision/time/stdout/stderr in command receipts; copied child environment exactly PATH="", PYTHONDONTWRITEBYTECODE="1", LANG="C.UTF-8". Initial canonical fixture launcher inherited environment (not fully captured); copied fixture proof has explicit environment. Hash scope includes all 54 original files, six receipt files, maps/archive, executable copied inputs and seven frozen identity/raw/response comparisons.

No repository edits, Cargo, full suite, shared parser/global duplicate checks, delegation, push or deploy. Parent 1.15.1 worktree/proof untouched. Only pre-existing untracked .code-index.db reported. Retain evidence above under main ownership; per-test disposable copies cleaned.
