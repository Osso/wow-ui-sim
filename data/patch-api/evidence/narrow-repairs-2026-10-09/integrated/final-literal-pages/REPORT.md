# Independent integrated SOURCE verification — 1.4.0 / 1.3.0

PASS within bounded SOURCE scope. 7 executed commands exit 0: two copied validators plus 24/24 tests. No tracked edits, builds, Cargo, runtime, network, model/native execution, delegation, operational mutation, or parent closure. 1.1.0 and 1.0.0 NOT verified; waiting for explicit mailbox authorization.

| Page | Original seals | Separate receipt seals | Archive members / bytes | SOURCE | Portable | Current extension |
|---|---:|---:|---|---|---|---|
| 1.4.0 | 44/44 | 25/25 | 45 / 84,207 | 7/7; 197 omission/count controls | 3/3 | 3/3; 55 category/omission controls; 3/3 extension seals, 4 archive members |
| 1.3.0 | 46/46 | 17/17 | 47 / 62,573 | 8/8 | 3/3 | Not applicable |

All archive hashes, map hashes, exact member sets, and copied file hashes verified. Separate extension hash/member set/seals verified before current SOURCE3. Tests reject serialized ledger omission and fabricated log in each page (4 total rejection cases), restore exact bytes/hash/original map (4 restoration cases), and replay without resealing. Canonical original evidence and seal/archive bytes unchanged before/after every executed command. Final copied original seals also match. Frozen inputs immutable.

1.4.0: validator reports 42 physical / 34 nonblank rows, inventory27, signatures25, prose25, headers6, templates28, links3, navigation2, defaults0, count claims0, contracts32. Current correction changes only TogglePVP group from Miscellaneous to null; contracts/counts/measurements preserved, historical ledger still sealed.

1.3.0: validator reports 44 physical / 38 nonblank rows, inventory31, signatures31 (23 explicit argument lists), prose3, headers7, links4, templates31, defaults0, count claims0, UNPROVEN contracts76. Historical default generator/extractor bytes and exact two error messages tested by both validators, SOURCE and portable controls. Extractor outputs 2,250 / 932 bytes respectively.

## Actual epochs and artifacts

All seven command HEAD-before and HEAD-after observations: e20c472caa43b2a422cef615579d03b22bf51866. These are observed execution epochs, NOT an assertion of an immutable checkout. Main may merge concurrently. Later status observation showed concurrent staged 1.0.0 integration and wiki index/log conflicts; verifier did not modify or verify those paths. Pre-existing untracked .code-index.db untouched.

Exact argv, absolute canonical cwd, explicit PATH=/nonexistent / PYTHONNOUSERSITE=1, UTC start/end timestamps, exit codes, before/after sourcehash maps and complete returned streams: executions.json. Per-page preflight.json and postflight.json retain archive identities/seal counts/hash maps; portable-controls.json retains concrete rejection/restoration hashes. Per-command *.combined.log preserves FULL merged process output, read in full through CommandResult. combined_to_file redirects stderr into the merged stdout result; *.returned-stdout and *.returned-stderr preserve API-returned streams (stderr empty), NOT independently captured child stdout/stderr channels. Separate original channel chronology is not proven. No rerun merely to relabel streams.

Proof scopes remain valid across unrelated docs merges; do not rerun without relevant hash changes or explicit request. Copied artifacts retained under each page's copy/ directory, separate from canonical evidence.

## Privacy and limits

Scoped high-specificity private-key/provider-token/bearer/credential-value pattern checks before artifact retention found 0 matches; not exhaustive secret detection. No environment values, secrets, or unrelated process streams inspected/copied. Only bounded proof-command streams retained. Archive members equal privacy-inspected canonical sealed files by hash.

Verifier preflight initially expected 1.4-style archive identity keys for 1.3 and stopped with KeyError('sha256') before executing any 1.3 command. Read own identity schema, resumed only 1.3; did not repeat valid 1.4 proof. No SOURCE failure.

No runtime/model/native correctness or equivalence proved. All32 /76 historical contracts remain UNPROVEN. No attribution, aliases, defaults, linked-target expansion, retirement, behavioral credit, main acceptance or parent closure inferred. Source-only PASS is not full-project completion.
