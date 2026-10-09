# Bounded 4.3.4 evidence

No final acceptance gate: coordinator owns integration, broad checks, smoke, CI and prior validators. No dedicated validator added. Historical test results are not current/future-head results.

## Proof ledger

| Command/scope | Revision | Result | Invalidation |
|---|---|---|---|
| Own prefork `patch_4_3_4_publication_sweep`, empty fixture | `5c3bccdf7` | Expected RED, exit 1: seven discovered mismatches, 11 observations | Reviewed fixture replaces empty expectation; discovery retained |
| Same own prefork, reviewed fixture | `04f6f5ce1` | GREEN, exit 0: 1 passed, 0 failed | No subsequent sweep/runtime/fixture edits |
| Same own prefork, scratch register replacing ComplainChat with fabricated addition | `04f6f5ce1` | Expected exit 1: exactly one new mismatch, seven → eight gaps | Scratch only; production source unchanged |
| Own default generator plus own extractor `--text-only --check` | `04f6f5ce1` | Both exit 0, register/extract byte-identical | No subsequent raw/register/extract/tool edits |

Each `.proof.json` records argv, absolute cwd/output paths, explicit own target directory, exact revision, exit and log SHA-256. Logs preserve stdout followed by labeled stderr, not chronological stream interleaving. Results persist before expected mismatch failures. No broad suite, check/lint/type/readability/coverage, startup smoke, full suite or final gate run. No runtime or parser source changes; no new behavior needing runtime TDD.

## Accounting and historical pins

`accounting.json` derives inventory/directions/headers from the register, statuses from the ledger, gaps from fixture/results, supersessions from recorded observations, and successor set from the own sweep's literal includes. Zero substantive non-inventory rows are independently visible in the default extract (`Patch 4.3.4 API changes`, blank line); no signature is inferred from linked API pages.

`historical-inputs.json.gz` stores exact UTF-8 path-to-content bytes for own source/accounting inputs, tools, probe and all 64 included retail successors. `historical-input-pins.json` records SHA-256/Git blob identities, archive seal and runtime tree IDs. This avoids dependence on unreachable pre-rebase commit objects for the sparse inputs. Runtime tree IDs identify code only; they are NOT executable snapshots or a relocation/native-parity proof. Classic registers never enter this successor list.

`session-seals.json` seals an explicit historical file set; later coordinator proof may be appended without changing those bytes. All artifacts stay below 5 MB. Source, response and pin are locally cross-checked; no second HTTP request made.

## Coordinator replay

Run through Pyrun with every argv command `.cwd('/home/osso/.worktrees/wow-ui-sim-p434-source')`; keep `CARGO_TARGET_DIR` at that worktree's absolute `target` path. Use `cargo test --test prefork_full_ui -- patch_4_3_4_publication_sweep --nocapture`, selecting an absolute scratch output with `P434_SWEEP_OUT`. Negative replay also sets `P434_SWEEP_REGISTER` to the absolute `negative-register.json` path. Do not overwrite sealed recorded results/logs.

For source replay, invoke the two absolute tool paths and argv recorded in `reproduction.proof.json`, replacing the temporary register output with a new absolute scratch path. Default flags must reproduce the retained register and text; don't expand linked pages. Current broad/Classic checks and any future API additions are separate coordinator evidence, never retroactive credit to this page.
