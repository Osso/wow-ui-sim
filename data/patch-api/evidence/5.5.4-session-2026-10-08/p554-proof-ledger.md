# Proof ledger

Acceptance code scope: `9ca9cd746`. No runtime `src/` changes. Receipts record launch HEAD; development commands queued before test commits are not final acceptance. RED source is retained in `74baa6172` (test and original classifier), GREEN/full-preload sources in `167c37176`; final scoped proofs use committed `9ca9cd746` directly. Formatting/scoped-loader changes invalidated earlier Rust acceptance; final commands ran after that source change. No broad suite rerun follows evidence-only changes.

| Label | Launch revision | Exit | Scope / invalidation | Command |
|---|---|---|---|---|
| p554-client-lines-final | `9ca9cd746` | 0 | Current Rust scope 9ca9cd746 | `cargo test --test integration publication_sweep_client_lines -- --nocapture` |
| p554-client-lines-green | `167c37176` | 0 | Historical/development only | `cargo test --test integration publication_sweep_client_lines -- --nocapture` |
| p554-client-lines-red | `f74c89cc0` | 101 | Historical/development only | `cargo test --test integration publication_sweep_client_lines -- --nocapture` |
| p554-format | `9ca9cd746` | 0 | Current Rust scope 9ca9cd746 | `cargo fmt --check` |
| p554-mists-check-scoped | `9ca9cd746` | 0 | Current Rust scope 9ca9cd746 | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` |
| p554-mists-check | `167c37176` | 0 | Historical/development only | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` |
| p554-mists-discovery | `f74c89cc0` | 101 | Historical/development only | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_5_5_4_publication_sweep -- --nocapture` |
| p554-mists-final | `167c37176` | 101 | Historical/development only | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_5_5_4 -- --nocapture` |
| p554-mists-negative | `64459e44e` | 101 | Current Rust scope 9ca9cd746 | `env P554_SWEEP_REGISTER=/home/osso/.worktrees/wow-ui-sim-p554-page/data/patch-api/evidence/5.5.4-session-2026-10-08/p554-negative-register.json cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_5_5_4_publication_sweep -- --nocapture` |
| p554-mists-prefork-probe | `44ebca873` | 101 | Historical/development only | `cargo test --no-default-features --features sound,gui,casc,client-mists --test prefork_full_ui -- publication_sweep` |
| p554-mists-scoped | `9ca9cd746` | 0 | Current Rust scope 9ca9cd746 | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_5_5_4 -- --nocapture` |
| p554-retail-after | `167c37176` | 0 | Historical/development only | `cargo test --test prefork_full_ui -- publication_sweep` |
| p554-retail-before | `44ebca873` | 0 | Baseline: runtime and all pre-existing retail sweep sources unchanged from 787b47591 | `cargo test --test prefork_full_ui -- publication_sweep` |
| p554-retail-final | `9ca9cd746` | 0 | Current Rust scope 9ca9cd746 | `cargo test --test prefork_full_ui -- publication_sweep` |
| p554-rustfmt | `9ca9cd746` | 0 | Current Rust scope 9ca9cd746 | `rustfmt --edition 2024 --check tests/common/publication_sweep.rs tests/publication_sweep_client_lines.rs tests/patch_5_5_4_publication_sweep.rs` |
| p554-tools-tests | `74baa6172` | 0 | Current Python tools scope (unchanged since f74c89cc0) | `python3 -B -m unittest discover -s tools -p test_*.py` |

Registers/extracts: `p554-reproduction.json`, complete fixed scope at `f74c89cc0`, with flags from the pinned base receipts. Retail equality: `p554-retail-comparison.json`, derived from all historical sweep files at `787b47591`. Scans: `/usr/bin/grep`, retained without truncation; no removal identities exist. Mists full-Game preload failure is retained, not suppressed and not counted as passing.
