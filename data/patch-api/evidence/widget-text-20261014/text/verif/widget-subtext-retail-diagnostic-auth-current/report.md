# No-run compile authentication: PASS

- Exact seal: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-update-diagnostic-20261014T174735853857Z/seal-integration.json`.
- Exact executable: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-update-diagnostic-20261014T174735853857Z/artifacts/deps/integration-9f07e4a55f2bbbf7`.
- SHA-256: `745112e8bae99a0ecaeea6f32a6944d31c06f5e518b42f35a1dbbb0fee2f21b6`; matches seal. One executable hash pass; flat before/after stats identical and match sealed identity/size/mtime. Executable mode 0700 preserved without chmod.
- Source: `e948624b0b3691b626c49369f5f93b7d2c2c1ae6`; metadata agrees across seal, revision, submission, source-scope, compile-result. Two snapshot assertion files match commit. Commit changes assertion diagnostics only; production source comparison against e37f5426a1da8d721b0ab3ff0341877f59100c02 shows no changes to src, crates, Cargo.toml, Cargo.lock, or build.rs.
- Cargo: 741 JSON records; exactly one integration artifact, fresh=false, exact seal cargo_artifact match. Actual features match seal: `aura-containers, aura-instance-enumeration, aura-xml-widgets, base-spell-relationships, casc, client-retail, default, forbidden-aspects, gui, native-duration-formatting, numeric-rule-formatters, on-update-modes, player-cast-durations, prefork-full-ui, profile-retail, retail-12-0-0, retail-12-0-5, retail-12-0-7, retail-12-1-0, rodio, sound`. Retail profile. build-finished.success=true; stderr Finished test profile in 25.82s.
- Six manifest deprecation warnings, not warning-free:

- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.large-enum-variant` is deprecated in favor of `lints.clippy.large_enum_variant` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.map-entry` is deprecated in favor of `lints.clippy.map_entry` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.match-wildcard-for-single-variants` is deprecated in favor of `lints.clippy.match_wildcard_for_single_variants` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.redundant-closure-for-method-calls` is deprecated in favor of `lints.clippy.redundant_closure_for_method_calls` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.trivially-copy-pass-by-ref` is deprecated in favor of `lints.clippy.trivially_copy_pass_by_ref` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.type-complexity` is deprecated in favor of `lints.clippy.type_complexity` and will not work in a future edition

Receipt: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-update-diagnostic-20261014T174735853857Z/receipt-integration.json`.

Report directory 0700; report/proof/receipt files 0600. Files and containing directories fsynced; content/modes read back. No executable created/copied; no build/test/runtime, archive hashing, delegation, or canonical edits. Source association is not cryptographic authentication of source/dependencies/cache/settings. No runtime success or native parity claim.
