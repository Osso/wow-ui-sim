# Proof ledger

Base: 7a292c9d0fc7458c648eceb6cf67a036b2e22ac8. Runtime/test scope: c9e249647; later receipts at b9b1a17bf are docs/evidence-only changes. No src, shared helper, tool, Cargo, vendor or existing sweep changes.

| Proof | Command / retained receipt | Result | Invalidation |
|---|---|---|---|
| Retail branch/master | cargo test --test prefork_full_ui -- publication_sweep; all-sweeps.proof.json and master/all-sweeps.proof.json | 58/58 each; exact observation comparison sealed separately | Retail runtime/shared harness/existing inventory changes |
| Retail client lines | cargo test --test integration publication_sweep_client_lines -- --nocapture | 3/3 | Runtime/classifier/controls changes |
| Mists pages | cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_5_5_ -- --nocapture | 6/6 page/line cases; five empty inventories | Mists runtime/harness/inventory changes |
| Mists compile | cargo check --no-default-features --features sound,gui,casc,client-mists --tests | Exit 0; zero non-vendor warnings | Rust/build configuration changes |
| Negative | Same Mists test command scoped to patch_5_5_0_publication_sweep with P550_SWEEP_REGISTER | Expected exit 101: injected row count 1 versus 0 | Classifier/source/register changes |
| Python fixtures | python3 -B -m unittest discover -s tools -p test_*.py | 89 pass | Tools changes |
| Format | cargo fmt --check | Pass | Rust changes |
| Reproduction | python3 -B reproduce_sources.py HEAD at c9e249647 | 62 registers byte-identical; 59/62 extracts, exact inherited 12.0.5/12.0.7/12.1.0 errors | Sources/extractor/generator changes |

Own evidence validator and clean/later-audit gate run after receipts commit. Docs-only finalization does not invalidate runtime or reproduction proof. Full integration/lib/addons-enabled startup conditional gate not triggered: no shared source changed. No repeated broad builds/checks.
