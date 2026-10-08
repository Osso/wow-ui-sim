# 5.5.3 proof ledger

Scope: current Mists Classic profile, resources-only inventory, no runtime changes. Complete argv, revisions, exits and log hashes live in `*.proof.json`; `context.json` seals acceptance inputs. Counts come from registers, source-defined cases and retained results.

| Proof | Scope | Invalidation policy |
|---|---|---|
| `all-sweeps` | Every retail prefork publication page plus animation factory case | Retail source/tool/build changes invalidate; the one import deletion in a wholly Mists-gated module does not compile under retail |
| `client-lines` | Source-defined retail client-line controls | Same retail boundary |
| `mists-pages-final` | 5.5.3 empty-inventory discovery and both 5.5.4 Mists cases | Any Mists source/build change invalidates |
| `mists-check` | Current Mists `cargo check --tests` | Any relevant Rust/build change invalidates; reject non-vendor warnings |
| `negative` | One injected row against zero-row page contract | Must exit 101 at the 1 → 0 row-count boundary |
| `tools-tests` | All `tools/test_*.py` fixtures | No tools source changed during this audit |
| `format-final` | Final Rust source | Import deletion invalidates original format receipt; final receipt supersedes it |
| `p553-reproduction.json` | Complete source/register/extract set at `d650107bf` | Source/tool bytes unchanged after that revision; three exact inherited extract failures retained |
| `master/` | Historical retail baseline pinned at master `4d046d99d` | Git diff proves recorded baseline runtime equals master's src/tests/tools/Cargo/build scope; no redundant baseline rebuild |
| Validator gate | Committed evidence in clean and unrelated-later-audit checkouts | Report records exact tested commit; later docs/evidence-only commits do not change sealed acceptance scope |

Initial `mists-pages` started before removing the unused `WowLuaEnv` import. Its receipt cannot establish final source-revision proof and is retained as historical build evidence only. `mists-pages-final` refreshes that targeted scope without rerunning the unchanged retail suite. No broad suite is rerun merely for a commit milestone.

`/usr/bin/grep -RInw` scans retain complete stdout/stderr in `mists-consumer-context.txt` and `src-tests-context.txt`; cache Documentation is excluded. These are context scans only: neither pinned page nor 5.5.4 enumerates a removal or re-addition candidate. No API retirement or positive coverage is inferred.

Manual Rust readability audit: new test follows the required 5.5.4 harness, one bounded closure, no deep nesting or warning suppressions; unused import removed. No production code changed. User forbids agents/model CLIs, so all verification is main-thread command/artifact proof, not independent model review.
