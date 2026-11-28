# Texture source resolution cache

Environment-owned source lookup reduces repeated filesystem enumeration in `SetTexture` and rendering without changing texture selection. Implementation: `src/texture/source_index.rs`; [mechanics](../wiki/systems/texture-source-resolution-cache.md).

## What it must do

- [ ] Share one source resolver between an environment's setters and renderers; isolate environments and binary client profiles.
- [ ] Preserve normalized request spelling, ordered addon roots, exact-before-case-insensitive extension precedence, file/directory distinctions and existing return values.
- [ ] Discover file creation/deletion and root changes without retaining stale negative decisions; invalidate source listings and CPU negatives on root changes and UI reload. Unchanged repeated frame loads must not repeatedly enumerate missing-source directories.
- [ ] Repeated assignments retain field, secret, atlas, color and tiling effects, including nil/empty clears.
- [ ] Bound directory opens for repeated mixed-case requests, measured against a concrete filesystem fixture rather than internal calls.
- [ ] Publish shared CASC extractions and persistent PNG crops only after the complete asset is written. Readers that opened the previous complete file keep its contents.

## How it works

- [Source lookup ownership and invalidation](../wiki/systems/texture-source-resolution-cache.md).

## Implementation inventory

- `src/texture{.rs,/resolve.rs,/source_index.rs}` — source enumeration, metadata freshness and CPU load retries.
- `src/lua_api/state{.rs,/sim_state.rs}` — per-environment manager and root synchronization.
- `src/lua_api/frame/methods/widgets/texture/atlas.rs` — setter existence checks; assignment logic unchanged.
- `src/iced_app/` and `src/bin/wow_sim/gui_commands.rs` — environment-associated renderers share the manager.
- `src/lua_api/globals/state_backed_queries.rs` — Lua reload invalidation.
- `src/dump/display.rs` — standalone diagnostics retain no global source cache.

## Tests asserting this spec

- `tests/texture_source_cache.rs` — live fixtures, root/environment isolation, setter effects, CPU miss recovery, calibrated Linux directory opens and a child-process CASC failure guard (first/warm/root/reload diagnostics).
- `tests/texture_set_texture_result.rs`, `src/texture/tests.rs` — existing result and source-selection contracts.
- `tests/wowforever_texture_metatable.rs`, `tests/aura_secret_display.rs`, `tests/texture_file_dimensions.rs` — profile and secret controls.
- `tests/methods_texture.rs`, `tests/texture_atlas_regressions.rs`, `tests/methods_button_textures.rs` — atlas, tiling and dirty controls.
- `src/iced_app/app_startup_proof_tests.rs::filesystem::app_preload_setter_render_reuse_root_change_and_gui_reload_recover_sources` — actual App, valid filesystem BLP pixels, calibrated Linux directory opens for setter/queued-preload/draw reuse, root replacement and unchanged-directory GUI reload invalidation, followed by missing-source recovery. PNG extension probing can legitimately refresh missing higher-priority filenames; the zero-reopen fixture uses BLP, the first extension in the lookup order.
- `src/iced_app/app_startup_proof_tests.rs::startup_cleanup_layout_and_texture_warmup_precede_first_nonempty_quad` — visible filesystem PNG decoded before the first nonempty App primitive, with cleanup/layout and upload-data assertions. See [startup contract](gui-startup-yield.md). Neither fixture invokes GPU prepare/upload or proves presented pixels.

## Bounded App proof — 2026-10-18

The Linux filesystem App fixture passed once under the [Retail App proof revision and artifact](gui-startup-yield.md#bounded-app-proof--2026-10-18), alongside the separate first-quad fixture and eight startup controls. Real uncompressed 2×1 BLP files exercise queued decoding and rendering; kernel directory-open observation is calibrated against two real enumerations.

| Phase | Observed fixture-directory opens |
|---|---:|
| First mixed-case setter | 1 |
| Queued App preload, reusing the setter listing | 0 |
| 32 warm setter/update/draw cycles | 0 |
| Root-listing replacement | 1 |
| Actual GUI reload and root-change draw recovery | 1 |
| GUI reload with unchanged directory | 1 |
| Late-file creation after reload | 1 |

The test observes missing draw requests before recovery and checks concrete decoded/upload pixels, not resolver call counts. Root-change recovery deliberately crosses actual GUI ReloadUI; the pre-reload failed-handle boundary below is not made into a new automatic-retry requirement. Filesystem-only runtimes use CASC disabled; GPU prepare/upload/presentation and successful decoded-byte hot reload are not covered.

## Known gaps (current cycle)

Final acceptance requires the requested gates on the final implementation revision; source-only inspection and zero-selected filters are not passing receipts. Native/live-GPU/real-addon acceptance is not supplied by these bounded fixtures.

Observed App boundary: after a failed draw request, changing roots and repeating the identical `SetTexture` can return true while retaining that failed draw handle until GUI invalidation/reload. The contract requires source-listing and CPU-negative invalidation, not automatic pre-reload recovery of draw handles. App recovery fixtures therefore use `Message::ReloadUI` after changing roots, with separate root-listing and unchanged-directory reload oracles. This observation is not classified as a contract divergence.

## Out of scope

- Decoded pixel, dimensions, crop and GPU-handle hot reload: source lookup freshness is not rendered-byte freshness.
- Startup timer yielding or real Ellesmere first-draw/IPC acceptance: this fix only addresses measured texture resolution work.
- File-dimension getters retain their independent metadata-only lifetime; no persistent dimension-cache behavior change.
