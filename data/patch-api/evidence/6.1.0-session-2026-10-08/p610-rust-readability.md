# Rust readability

Scope: new `tests/patch_6_1_0_publication_sweep.rs`; shared sweep and runtime Rust unchanged.

Manual changed-line audit: single linear prefork test; explicit named SweepSpec fields, parsed row count, ordered register inputs and three one-line pending integration placeholders. No new branching, nesting, state accumulation, suppressed warnings or duplicated runtime behavior. No violations.
