# Independent Era 1.15.4 / 1.15.3 verification

**PASS — bounded SOURCE identity/accounting/replay/preservation only.** No runtime, model, native, loaded-UI, security, factory, compatibility parity or final acceptance credit.

Canonical cwd for every executed validator/fixture and portable child: `/home/osso/Projects/wow/wow-ui-sim`. Checked HEAD remained `0b64e636c16c59e90e3406e7ff557e94cd7732a8`. Both requested integrations are ancestors, and their evidence paths have no committed changes since their respective merges (`7ad66791e` / `0b64e636c`); final scoped working-tree diff is empty. Initial status had pre-existing untracked `.code-index.db`; untouched.

## Exact identity and coverage

| Source | Page/revision | Timestamp | Bytes / TOC | Fresh own fixtures | Fresh copied fixtures | Portable |
|---|---|---|---|---|---|---|
| 1.15.4 | 600355 / 6172581 | 2024-11-13T21:59:45Z | 439 / 11504 | 9/9, exit 0 | 9/9, exit 0 | 3/3, exit 0 |
| 1.15.3 | 593663 / 6114194 | 2024-08-30T01:06:01Z | 452 / 11503 | 10/10, exit 0 | 10/10, exit 0 | 3/3, exit 0 |

Both own and copied validators exit 0. Frozen manifest/registry identities and hashes match; registry has 101 pages ending 1.0.0. Exact raw/response hashes are in `actual-successor-application.json` and original pins. Each source has 8 physical lines, 6 nonblank rows, 4 metadata rows, 2 UNPROVEN rows, 2 nonnumerical headers, 5 UNPROVEN contracts (1 prose + 4 unexpanded links), 1 unexpanded navigation transclusion. Zero local API/event/CVar/widget-method/command occurrences and zero signature declarations. SOURCE omission fixtures exercise 19 omissions for 1.15.4 and 21 for 1.15.3, plus identity/content/fabricated-credit/foreign-history controls; 1.15.3 also rejects collapsed comparison bases.

Both copied historical generators exit 0 and reproduce original default-register bytes. Both registers have `entries: []` and `header_counts: []`. This is literal default extraction, NOT behavioral compatibility/parity proof.

Configured frozen Era/Anniversary profiles use 11507, distinct from source11504/11503. Current `src/client_profile.rs` also has Era/Anniversary11507. No native correspondence follows.

## Actual successor inputs, separate from historical notes

`actual-successor-application.json` applies ordered actual canonical frozen identity/context inputs, preserving historical queued notes without changing their sealed ledgers. Each actual pin equals the corresponding historical successor pin and frozen manifest entry; raw and response bytes match, including exact response page/title/revision/timestamp/content. Current canonical introduction Git receipts and per-input SHA256 scope are recorded separately.

| Successor | Page/revision | TOC | Local API occurrences in successor ledger |
|---|---|---|---|
| 1.15.4 | 600355 / 6172581 | 11504 | 0 |
| 1.15.5 | 610284 / 6235411 | 11505 | 0 |
| 1.15.6 | 619994 / 6281951 | 11506 | 0 |
| 1.15.7 | 626071 / 6778069 | 11507 | 0 |
| 1.15.8 | 686952 / 6778071 | 11508 | 0 |
| 1.15.9 | 685352 / 6780591 | 11509 | 4 |

For 1.15.4 apply 1.15.5→1.15.9; for 1.15.3 apply 1.15.4→1.15.9. Target inventory is zero in both, so superseded target inventory is empty. Successor1.15.9's four occurrences are not backfilled. Historical 1.15.4 notes label 1.15.5/6 queued and 1.15.7/8/9 integrated-not-applied; historical 1.15.3 additionally labels 1.15.4 in-flight. Those are preserved historical statements, not current integration status or proof credit.

1.15.4's unspecified War Within subset cross-attributes 11.0.0/11.0.2, without expanding/importing contracts or treating retail pages as Era successors. 1.15.3's unspecified Dragonflight10.2.7/Cataclysm4.4.0 subset stays unexpanded; Gethe comparison base4.4.0 and Ketho base1.15.2 remain distinct. Literal expansion/client labels do not establish a native Era client identity.

## Serialized rejection/restoration and preservation

Each copied portable suite executes seven fresh child commands with empty PATH and absolute Python: validator, SOURCE fixtures, historical generator, ledger rejection/restored validator, log rejection/restored validator. Archives contain no `.git`, `target`, or current `tools` directory. Four intentional serialized negative runs exit 1 with exact `AssertionError: seal: ledger.json` or `AssertionError: seal: green.log`; all four restored validators exit 0. Restored bytes exactly match originals, and all original seal checks pass after restoration.

| Source | Original + separate receipt seals | Archive members | Entire evidence directory unchanged |
|---|---|---|---|
| 1.15.4 | 44 + 6 | 45 | 51 files |
| 1.15.3 | 50 + 6 | 51 | 58 files |

Seal-map bytes and separate receipt-map bytes unchanged. Full before/after digest comparison and restored hashes are in `baseline.json` / `preservation.json`.

The own portable harness hardcodes its original worktree cwd. To honor this verification's canonical-cwd requirement, `portable-wrapper.py` loads the unchanged own harness, overrides only its CWD, and instruments receipts. Original fixture assertions and copied adapter/tools remain unchanged. This proves the same portable tests with explicit canonical cwd, not an unmodified invocation of the worktree-bound harness. First scratch wrapper accidentally had invalid trailing text; both initial attempts failed with SyntaxError, earned no credit, and remain recorded. `portable-wrapper-failed.py.txt` and initial streams preserve this verifier setup failure. Only the scratch wrapper was corrected; corrected runs pass as above.

## Evidence paths

All independent evidence lives under `/tmp/era1154-1153-independent-c2w7roqf/`:

- `commands.json`: raw argv/env/revision/cwd/start/end/exit/stream paths and hash scope for own fixtures/validators, outer portable commands and scoped Git observations.
- `1.15.4-portable-receipts.json`, `1.15.3-portable-receipts.json`: full child argv/env/revision/cwd/times/stdout/stderr and before/after SHA256 scope, exact restoration receipts. Individual child streams also retained as `1.15.[34]-copied-<index>.stdout/.stderr`.
- `actual-successor-application.json`: exact identities, current canonical receipt attribution, ordered successor application and unexpanded foreign contracts.
- `baseline.json`, `preservation.json`: immutable evidence hash scope and restoration proof.
- `1.15.[34]-original-portable-proof.json`, `1.15.[34]-original-portable-controls.json`, `1.15.[34]-original-proof-ledger.json`, `1.15.[34]-original-portable-controls.log`, `1.15.[34]-original-green.log`, `1.15.[34]-original-red.log`: byte-identical copies of original receipts/logs, not fresh proof attribution.

Original evidence paths preserved:
`/home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/1.15.4-session-2026-10-09/`
`/home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/1.15.3-session-2026-10-09/`

## Remaining gaps / exclusions

All five substantive contracts per page remain UNPROVEN: subset membership, linked contracts, exact APIs/signatures/defaults/state/security and native correspondence. Zero model/runtime/native measurements. No own model changes or work on in-flight main1.15.2/1.15.1 assumed or performed. No Cargo, shared-parser validation, full suites, global duplicate scans, repository edits, delegation, push/deploy or network. Successor identity/context application is not semantic supersession or successor behavioral proof. Original earlier RED/GREEN receipts retained without assigning their historical executions fresh credit.
