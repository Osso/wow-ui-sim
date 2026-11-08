# Existing compile / sealed-copy verification: PASS

Source: `6c38a82739dbf27fc58c3c801c32a0f0fc98d21c`. One authentication epoch: `2026-10-14T01:49:46.174407+00:00`.

| Evidence | Result | Proof boundary |
| --- | --- | --- |
| Saved full compile.stdout | PASS: 741 JSON records; final build-finished success=true; 0 compiler diagnostics | No-run compilation only |
| Saved compile.stderr | Six manifest deprecations; Cargo 5m 09s | Not warning-free |
| Saved compile-terminal.txt | Result=success, ExecMainStatus=0; October 13, 2026 20:42:11–20:47:21 CDT; peak 7959556096 bytes | Saved unit evidence, no service query |
| New sealed executable | PASS: exactly one sequential SHA-256 read; pre/open/post stat matches seal | Copied integration executable only |
| Passive source gate | Historical PASS (assignment gate 4181); not runtime | Five positive cases + one original consumer candidate remain UNEXECUTED |

Executable: `<PRIVATE_HOME>/.local/state/wow-ui-sim/verification/widget-discrete-progress-current/crafted-quality-owned-retail-red-20261014T014209924708Z/artifacts/deps/integration-9f07e4a55f2bbbf7`.
Digest: `958e01e4acb63b9525f0829bf23357d899803c045f3ebc48829e02e7c18f24f1`.
Receipt: `<PRIVATE_HOME>/.local/state/wow-ui-sim/verification/widget-discrete-progress-current/crafted-quality-owned-retail-red-20261014T014209924708Z/receipt-integration.json` — absent before this epoch; newly created PASS, sha256_matches_seal=true, hash_reads=1, exact path/stat_after/digest recorded; mode 0600, fsync and exact readback verified.

Compile artifact filename: `integration-9f07e4a55f2bbbf7`; fresh=false means this Cargo record was freshly compiled, not a reused artifact. Seal features equal the integration compiler record. Source revision matches seal and saved compile metadata; no source tree/archive authentication performed.

Current HEAD prefix c652 is later publication only per assignment; no current-binary claim. Feature acceptance NOT ACCEPTED. Consumer fixture baseline unvalidated; no positive RED, positive PASS, or original-consumer runtime result claimed.

No old binary/library/archive/source-tree authentication, normal build binary copy/authentication, build/test/check/fmt rerun, service operation, delegation, Bash, or cwd tool.

Outputs: `<PRIVATE_HOME>/.local/state/wow-ui-sim/verification/crafted-quality-owned-retail-red-auth-current/report.md` and `<PRIVATE_HOME>/.local/state/wow-ui-sim/verification/crafted-quality-owned-retail-red-auth-current/proof.json` — exclusive creation, mode 0600, file/directory fsync and exact readback.
