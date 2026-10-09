# wow-ui-sim linker failure attribution (2026-10-09)

## Scope and evidence

Read-only investigation of retry epoch `20261009T223142Z` after orderly host reboot. No Cargo build/test/check, cleaning, deletion, edit to repository, or service mutation was performed. The earlier `20261009T222607Z` epoch was SIGTERM-interrupted and is not treated as completed build proof. Retry submission records source revision `77e552bae7210b3b6a1a9f604911a7dd9a1943b7`, local canonical checkout, `cargo test --offline --locked --lib --test integration --no-run --message-format=json`, `CARGO_BUILD_JOBS=4`, and rustc/Cargo 1.99.0.

## Observed failure

Retry `compile.stdout` contains two Cargo `compiler-message` linker errors and ends with `build-finished success:false`; `compile.stderr` confirms both targets failed. The `wow-sim` link fails with **462** mold undefined-symbol reports, predominantly anonymous LLVM/internal and ordinary standard-library/project symbols. First reported missing `anon.fbaa...llvm...` is referenced by an object member inside `libwow_ui_sim-3282a118e07cdd18.rlib`. The lib-test link separately reports **344** missing symbols; its references include newly built `wow_ui_sim-bcad23723a7b3b0d.*.rcgu.o`. These are linker failures, not Rust name-resolution/compiler diagnostics pointing to absent source definitions.

Cargo JSON records `wow_ui_sim` library artifact `libwow_ui_sim-3282a118e07cdd18.rlib` and matching `.rmeta`, `fresh:false`, before the aggregate failure. The rlib exists (429,344,436 bytes); rmeta exists (135,442,075 bytes). Thus the crate compilation produced a library artifact in this attempt, but the dependent executable did not link. The lib-test target also did not link, so no test executable or tests are runnable/proven by this epoch. Do not treat the rlib as a validated executable-ready artifact.

The linker diagnostics show different object/codegen-unit naming epochs within the same library linkage: references from the rlib use suffix `.1oref9o`, while the library test's newly generated object files use `.0z1c6qp`; multiple prior `wow_ui_sim-*` incremental directories exist, and two current directories have recent epoch writes. This is evidence of mixed/retained incremental/object state as a plausible artifact inconsistency, not proof of corruption or proof that every mismatched reference comes from stale state. No named missing project function is evidence of an absent definition by itself; many anonymous LLVM and std symbols fail too. Source revision has no captured source diagnostic implicating missing definitions.

## Configuration/toolchain

Active compiler and Cargo observed locally are Arch Linux rustc 1.99.0 (`b940084d`, LLVM 23.1.1) and Cargo 1.99.0, while repository `AGENTS.md` documents local installed rustc 1.98.1. The run metadata independently records 1.99.0. `.cargo/config.toml` selects `clang`, mold via `-fuse-ld=mold`, native CPU, four mold threads; Cargo jobs are four. `Cargo.toml` enables dev incremental compilation. No `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `CARGO_INCREMENTAL`, `CARGO_TARGET_DIR`, `RUSTC_WRAPPER`, or custom `LD` override was present in the inspected process environment. No user Cargo config file was found in the checked paths. Therefore toolchain drift is established, but this evidence does not establish a 1.99 regression; linker config is explicit and no evidence singles out mold as root cause.

## Attribution and smallest justified next invalidation

**Cause remains unproven.** Evidence favors investigating the exact `wow-ui-sim` crate's incremental/codegen artifacts before source changes: broad undefined symbols across anonymous LLVM, std, and simulator code are consistent with object/archive inconsistency, and the same-attempt objects show different CGU suffixes. Rust 1.99 versus documented 1.98.1 is a concurrent environmental difference, not a proven cause. Source-missing-definition hypothesis is not supported by the error shape, but cannot be conclusively excluded without a controlled subsequent compile.

No invalidation was performed. If authorized later, the narrowest justified diagnostic invalidation is the **current Retail dev-profile `wow-ui-sim` incremental crate directory only**, after verifying its fingerprint/ownership against this invocation; do not clean `target/`, dependencies, or unrelated profile artifacts. The exact directory-to-hash mapping is not established by these logs alone, so select it from the active Cargo fingerprint/incremental metadata before removing anything. Preserve all dependency artifacts. Then compile the exact affected crate/target under a controlled, documented toolchain; do not change toolchain or linker as a speculative workaround. Current failed epoch has no runnable `wow-sim` or lib-test executable.

## Durable inputs

- `.../verification/fixture-and-callstack-proof/20261009T222607Z/` — interrupted prior epoch and durable streams.
- `.../verification/fixture-and-callstack-proof/20261009T223142Z/` — retry submission and complete JSON compiler diagnostics.
- `Cargo.toml`, `.cargo/config.toml`, project `AGENTS.md` — profile, linker, documented toolchain.
