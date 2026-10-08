# Changed Rust readability audit

Reviewed `tests/patch_6_2_4_publication_sweep.rs` and `tests/patch_6_2_4_behavior.rs` against rust-readability: no violations. Register-driven declarations use the existing sweep helper; row counts derive from JSON. Behavior setup mutates explicit backing state, clones seeded fixtures and asserts public outcomes in both environments. One named helper shares the concrete case; no duplicated test body, warning suppression, opaque parameter plumbing or runtime source change.

The tests intentionally do not claim historical tuple/input parity from present-day successor data.
