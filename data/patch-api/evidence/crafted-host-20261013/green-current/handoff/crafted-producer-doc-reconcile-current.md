# Crafted producer documentation reconciliation

Revision: cd9f42756472b8f8d706fd3e592f1977b3d89868 (observed HEAD).
Scope: bounded read-only documentation reconciliation; not a new source or runtime gate.

## Disposition

Documentation reconciliation: PASS at this revision. Prior 05665ddad6aa083406777f75887504a8e54c5725 Source PASS / Spec FAIL / Overall FAIL remains preserved unchanged. No retroactive PASS.

Read full current docs/specs/crafted-item-quality-info.md and prior verification/crafted-quality-producer-source-current/report.md and proof.json. Existing source PASS is reused, not rerun; no manual metrics repeated.

## Accepted findings

- Prior spec line 38 was stale: five owned cases plus one whole-consumer case executed on each Retail/PTR profile, not merely authored/unexecuted. Current wording records five first-positive table-check failures and one original-consumer positive-flag failure per profile. Payloads remain unknown; later field, alias, lifecycle, security and visibility/overlay assertions remain unreached.
- Prior spec line 48 was stale: positive RED prerequisite already closed before producer projection. Current spec removes that task without converting RED into GREEN.
- Consumer inventory now records closure and empty-input baselines passing before positive RED, while retaining unproved later overlay assertions and host-created Frame/native-panel distinction.
- Existing-parser model forms are numeric/decimal-string/item-string/hyperlink selectors. Num/Str guard plus parser reuse is simulator policy, not native selector validation or catalog acceptance.
- Eight cases per profile remain the GREEN scope: two initial, five owned, one whole consumer. Full thirteen-field contract and required-input/security/lifecycle checkboxes remain unchecked; no GREEN claim.

## Rejected conclusions

- Reject retroactively marking prior Spec FAIL or Overall FAIL as PASS.
- Reject treating source projection, registration, clone/root ordering, or prior source PASS as executed GREEN, GC proof, alias/native validation, or consumer visibility acceptance.
- Reject inferring unknown return payloads or downstream field/security/lifecycle results from first-positive RED failures.
- Reject launching GREEN compilation/runtime: main compile admission denied for parent memory/PSI; no launch performed. All crafted GREEN claims remain unproved.

## Git-only continuity evidence

05665dd..cd9f427 exact Git name/stat diff contains only initial RED evidence README, publication-metadata.json, and crafted-item-quality-info.md: three files, 14 insertions, 7 deletions. Producer/source files unchanged in that commit range. 5baa28b03..cd9f427 diff contains only spec: seven changed lines (3 insertions, 4 deletions); full spec diff inspected. Bare 5baa was ambiguous and failed; resolved explicitly to commit 5baa28b03, not a tree/blob.

Prior literal source pins retained from source report: src/c_api/c_trade_skill_quality.rs:70 original unwrap before :71 Num/Str guard, :76 existing parser, :77-80 independent map clone, :81-85 one nil miss, :87 shared projection, :90-117 fresh rooted thirteen-field snapshot; sim_state.rs:152-157 separate maps and state.rs:310 empty default; item_spell/c_item.rs:19-42 existing parser. These are inherited source-only observations, not fresh runtime proof or binary/source-tree authentication.

## Proof ledger and limits

- Git rev-parse HEAD: exact requested revision, exit 0.
- Git status: untracked .code-index.db and verification/ observed; untouched.
- Full spec and prior report/proof reads; Git diff/stat/name-only inspections as above, exit 0 except explicitly recorded ambiguous-prefix probe.
- No canonical edits, build, tests, checks, fmt, binaries, runtime, vendor/services, delegation, Bash or cwd-changing tools. Only this new handoff artifact written.
- Historical RED evidence reused at its recorded scope; no execution logs reauthenticated. Required source gate already PASS; not rerun.
