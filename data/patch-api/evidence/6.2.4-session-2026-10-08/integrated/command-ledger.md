# Integrated 6.2.4 proof ledger

Input scope: `context.json`; runtime `f260499ee`, exact master `846a30663`. Logs and receipts are immutable evidence, not milestone-triggered rerun requests. No code changes after these commands alter their recorded input scopes.

| Scope | Exact argv | Source revision | Exit | Observed counts |
|---|---|---|---|---|
| all-sweeps | `["cargo", "test", "--test", "prefork_full_ui", "--", "publication_sweep"]` | `f260499eea6b77f3e5fe218006c62441f1a75418` | 0 | [["50", "0"]] |
| bnet-model | `["cargo", "test", "--test", "integration", "c_battle_net_probes::"]` | `f260499eea6b77f3e5fe218006c62441f1a75418` | 0 | [["11", "0"]] |
| deprecated-bnet | `["cargo", "test", "--test", "prefork_full_ui", "--", "blizzard_deprecated_battle_net"]` | `f260499eea6b77f3e5fe218006c62441f1a75418` | 0 | [["3", "0"]] |
| extend-receipts | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p624-page/tools/extend_patch_audit_receipts.py", "/home/osso/.worktrees/wow-ui-sim-p624-page", "/home/osso/.worktrees/wow-ui-sim-p624-page/data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/extension", "p624", "Merged 7.0.1/7.0.3 flags from pinned provenance", "6.2.4"]` | `f260499eea6b77f3e5fe218006c62441f1a75418` | 0 | [] |
| extract_patch_non_inventory-fixtures | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p624-page/tools/test_extract_patch_non_inventory.py"]` | `c37c5e63587145249abec0e05cac1697232a5756` | 0 | ["36"] |
| format | `["cargo", "fmt", "--check"]` | `c37c5e63587145249abec0e05cac1697232a5756` | 0 | [] |
| gen_patch_wikitext_register-fixtures | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p624-page/tools/test_gen_patch_wikitext_register.py"]` | `c37c5e63587145249abec0e05cac1697232a5756` | 0 | ["32"] |
| integration-patch-6-2-4 | `["cargo", "test", "--test", "integration", "patch_6_2_4"]` | `f260499eea6b77f3e5fe218006c62441f1a75418` | 0 | [["1", "0"]] |
| master-all-sweeps | `["cargo", "test", "--manifest-path", "/tmp/p624-master-74rz5nxs/Cargo.toml", "--test", "prefork_full_ui", "--", "publication_sweep"]` | `846a30663a0f5d5fd0aaecaec46937cd858ae95a` | 0 | [["49", "0"]] |
| mists-check | `["cargo", "check", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--tests"]` | `c37c5e63587145249abec0e05cac1697232a5756` | 0 | [] |
| negative | `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_6_2_4_publication_sweep"]` | `f260499eea6b77f3e5fe218006c62441f1a75418` | 1 | [["0", "1"]] |
| own-sweep | `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_6_2_4_publication_sweep"]` | `f260499eea6b77f3e5fe218006c62441f1a75418` | 0 | [["1", "0"]] |
| patch_audit_validation-fixtures | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p624-page/tools/test_patch_audit_validation.py"]` | `c37c5e63587145249abec0e05cac1697232a5756` | 0 | ["8"] |
| prefork-patch-6-2-4 | `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_6_2_4"]` | `f260499eea6b77f3e5fe218006c62441f1a75418` | 0 | [["2", "0"]] |
| source-reproduction | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p624-page/data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/reproduce_sources.py"]` | `0e40aff7e805e4a635ccef9d93b68029f700fd08` | 0 | [] |

Negative exit 1 is expected. Mists has zero non-vendor warnings; inherited iced manifest warnings remain unsuppressed. The three inherited extract failures are unchanged, not passes.

Read-only validators: first matrix records 30/31; retained `p703-validator-red.txt` identifies exact tool hash invalidation. After the bounded fix, only the invalidated 7.0.3 gate was rerun. `prior-validator-matrix.json` records 31/31 with per-validator/log hashes. `tool-replacement-green.txt` records the exact-byte/tamper behavioral test. New integrated validator is the final gate; its result is appended separately after committing the sealed evidence.
