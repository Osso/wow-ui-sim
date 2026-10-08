# Integrated 6.2.4 goal

## Rebase reconciliation onto master 15b417367

Scope: retire the superseded live-tool allowlist dependency, preserve historical checks, record durable old/new revision evidence, and re-pin prior validators to the master tree. No runtime/vendor changes, push, merge, agents, bytecode or working-directory switch.

- [x] Record ten rebase mappings with patch IDs and blob/tree hashes.
- [x] Retain historical allowlist logs; remove obsolete fixture and match master's 7.0.3 gate.
- [x] Actually run the 30 validators selected from master 15b417367.
- [x] Rerun all/own sweeps, three Python fixture scripts and format; seal six receipts.
- [x] Update audit wiki without shrinking index/log.
- [x] Commit reconciliation and verify all 32 historical/integrated gates.

- [x] Reproduce all saved registers/extracts with recorded flags; retain inherited failures.
- [x] Run own sweep and apply only attributable later-audit supersessions if needed.
- [x] Compare every publication sweep against exact master 846a30663.
- [x] Extend merged-register/sweep receipts; refresh negative and all-sweeps proof separately from sealed historical artifacts.
- [x] Pass every historical/integrated validator and requested targeted, fixture, format and Mists checks.
- [x] Update audit wiki and commit. No push, merge, agents, vendor edits or working-directory changes.
