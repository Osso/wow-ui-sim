# Independent behavioral RED receipt audit

Date: 2026-10-10. Stage: `3c36b24546c0901837d4987b86ad33c634a8f671`.
Receipt epoch: `20261010T164918Z`.

## Verdict

**PASS — requested behavioral RED reached. Runtime complete, not pending.** This is a failing-test receipt, not a producer fix, GREEN acceptance, or native-client proof.

Saved `compile-result.json`: `exit: 0`, `source_equal: true`.
Saved `prose-result.json`: exact selected sealed integration test, `exit: 101`, `artifact_unchanged: true`.
Saved `prose.stdout`:

```text
running 1 test
test patch_12_0_0_prose::prose_spell_cast_count_reads_live_explicit_inputs ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7819 filtered out; finished in 0.23s
```

Saved `prose.stderr`, `tests/patch_12_0_0_prose.rs:22:10`:

```text
one ordinary scalar count from the real public getter: Lua(Runtime(RuntimeError { message: "spell 19750: expected 7, got 0", level: 0, traceback: [] }))
```

This is assertion-triggered test exit 101, not compiler exit 101, a resource block, timeout, or zero selected tests. Both bare environments completed initialization before the assertion failure. `outcome.json` records compile exit 0 and selected execution performed.

## Stage fields and feature/type gates

- Stage diff changes only the `spell_cast_counts` field gate, its empty-state initializer gate, and the new fixture. Field remains `HashMap<u32, u32>` (`src/lua_api/state/sim_state.rs:409-410`); initializer remains an empty map (`src/lua_api/state.rs:342-343`). Both gates now require `retail-12-0-0`. This is preparatory input exposure only.
- `Cargo.toml:118-119`: 12.0.5 includes 12.0.0. Thus later epochs retain this map. Artifact features are exactly `aura-instance-enumeration`, `profile-retail`, `retail-12-0-0`; no 12.0.5 producer feature.
- Producer registration remains gated on 12.0.5 (`src/c_api/c_spell.rs:98-102`); producer module remains gated on 12.0.5 (`src/c_api/mod.rs`). Existing modeled getter reads the map (`src/c_api/c_spell_counts.rs:17-33`), but is not enabled in this receipt.
- Unmodified earlier-epoch count shim remains gated by `not(retail-12-0-5)` and returns literal zero (`src/lua_api/workarounds/temporary/spell_metadata_defaults.rs:27-37`). It ignores explicit map input. Stage did not change producer or shim.

## Exact fixture semantics and proof boundary

`tests/patch_12_0_0_prose.rs:5-66` enables this test only for 12.0.0 without 12.0.5. Two `WowLuaEnv::new()` environments; no addon/Blizzard loading calls or public getter replacement. Inputs seed spell 19750 with count 7 and spell 642 with count 2; charge count 1 for 19750 deliberately differs from explicit count 7.

The first public-getter assertion checks one return, numeric type, non-secret output, then expected count. Observed `expected 7, got 0` establishes those preceding Lua checks passed and the first value assertion failed. Rust field initialization and mutation were reached. Test is behavioral, not an assertion about source shape.

Remaining written assertions cover the second spell, separate-environment default/isolation, charge mutation/removal independence, live explicit-count updates, entry removal, and clearing. **These later assertions were not reached and have no runtime proof from this RED receipt.** No inventory/casting derivation, secret-input policy, native parity, broad acceptance, or original 21-row acceptance is proven.

## Receipt integrity and limits

Independently compared saved before/after maps: equal, 23,670 entries. Compared stage Git contents against both saved SHA-256 maps for state initializer, SimState, exact test file, producer registration/module/implementation, shim, Cargo.toml, and integration entrypoint: all match. Independently hashed sealed executable: matches `artifact.json`, SHA-256 `a9ef8311bf772753b48a1f03558030d1bc026b6c11f0930d0df9c0087dbc2168`.

Worker exclusions remain: GUI-gated source, untracked files/index, external dependency provenance, inherited environment, and unused runtime assets. Hash checks support stage association within this recorded scope, not hermetic source-to-binary provenance. Saved compile stderr contains six iced manifest deprecation warnings; compilation nonetheless completed successfully in 1m43s.

## Rust readability

Read verify and rust-readability skills. Manual audit of the changed gates and new fixture: no readability violations identified. Fixture has explicit input/output checks, bounded mutation scopes, concrete values, and no warning suppressions. Test body remains below the skill's 200-line test threshold; no deeply nested logic. No readability tools, builds, or tests executed.

## Audit actions

Read saved artifacts and relevant source; used read-only Git inspection and hash comparison. No build/test repeated, delegation, service/runtime operation, source edit, or commit. Only this requested report was written. Main retains completion steering.
