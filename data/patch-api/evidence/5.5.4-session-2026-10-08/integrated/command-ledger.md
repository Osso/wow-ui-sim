# Integrated proof ledger

Runtime: `7158cb3fbee1a617a7df2f090a6aef4cb64c2f23`. Master: `bebcc5830dcb21e768a2e3c365105351a400fccb`.

| Scope | Recorded revision | Command | Result | Later changes invalidate proof |
|---|---|---|---|---|
| all-sweeps | `7158cb3fbee1a617a7df2f090a6aef4cb64c2f23` | `cargo test --test prefork_full_ui -- publication_sweep` | 0 (PASS) | No; only own evidence and documentation changed |
| master-all-sweeps | `bebcc5830dcb21e768a2e3c365105351a400fccb` | `cargo test --manifest-path /home/osso/.worktrees/wow-ui-sim-p554-master-ref/Cargo.toml --test prefork_full_ui -- publication_sweep` | 0 (PASS) | No; only own evidence and documentation changed |
| mists-page | `7158cb3fbee1a617a7df2f090a6aef4cb64c2f23` | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_5_5_4 -- --nocapture` | 0 (PASS) | No; only own evidence and documentation changed |
| client-lines | `9cd4a2ce871ea171dba373f5f83c75831c9c8fbc` | `cargo test --test integration publication_sweep_client_lines -- --nocapture` | 0 (PASS) | No; only own evidence and documentation changed |
| mists-check | `9cd4a2ce871ea171dba373f5f83c75831c9c8fbc` | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 (PASS) | No; only own evidence and documentation changed |
| negative | `9cd4a2ce871ea171dba373f5f83c75831c9c8fbc` | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_5_5_4_publication_sweep -- --nocapture` | 101 (expected negative) | No; only own evidence and documentation changed |
| tools-tests | `7158cb3fbee1a617a7df2f090a6aef4cb64c2f23` | `python3 -B -m unittest discover -s tools -p test_*.py` | 0 (PASS) | No; only own evidence and documentation changed |
| format | `7158cb3fbee1a617a7df2f090a6aef4cb64c2f23` | `cargo fmt --check` | 0 (PASS) | No; only own evidence and documentation changed |

Reproduction: 56 registers, 53 extracts; inherited 12.0.5/12.0.7/12.1.0 errors exactly unchanged.
Retail: every row on 55 pages (9,740 observations) equals pinned master.
Prior validators: 38/38, selected by git ls-tree at pinned master.
