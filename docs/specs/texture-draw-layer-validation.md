# Texture creation draw-layer validation

Ordinary explicit draw-layer names passed to `Frame:CreateTexture` must be validated on Retail/PTR. The current entry point lives in `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs`; [widget architecture](../wiki/systems/widget-system.md) describes region storage. This bounded contract does not establish native WoW parity.

## What it must do

- [x] Accept `BACKGROUND`, `BORDER`, `ARTWORK`, `OVERLAY` and `HIGHLIGHT`; each created Texture returns that name from `GetDrawLayer` and adds one region.
- [x] Reject an ordinary invalid name such as `INVALID` through the real Lua API without increasing the parent's `GetNumRegions` count.
- [x] Apply this new rejection contract only to Retail/PTR; leave earlier-profile behavior unchanged. Profile containment is source-inspected; earlier-profile fresh runtime coverage remains open.

## How it works

- [Widget architecture](../wiki/systems/widget-system.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs`: ordinary Retail/PTR `CreateTexture` validates supplied names before constructing/registering a region; other creation entry points and earlier profiles are unchanged.
- `tests/texture_draw_layer_validation.rs`: profile-gated integration cases using the region-alpha test pattern.
- `tests/texture_draw_layer_validation/ordinary_names.lua`: shared real-API valid and invalid modes.

## Tests asserting this spec

- `create_texture_accepts_all_five_draw_layers`: five explicit creation/getter roundtrips and region count.
- `create_texture_rejects_invalid_draw_layer_without_adding_region`: five valid baseline creations, then invalid-name rejection and unchanged region count.

Source producer `6e085fce9` and test formatting `6bdb02b06` follow tests/spec `ab0af8126`. Pre-producer shared normal-Lua fixtures completed valid mode on frozen Retail/PTR binaries, then accepted `INVALID` and changed regions 5 → 6. First rejection assertion failed (CLI1); unchanged-count assertion and invalid completion marker were unreached. Initial independent Retail RED retains its separate failure boundary.

## Current bounded runtime status

Independent runtime acceptance: **five unique Rust cases/profile, 10 Retail/PTR outcomes**, comprising two name-validation cases and three creation-preservation cases. Invalid creation now returns `pcall=false`, regions remain 5 → 5 and both rejection assertions complete. Five layer iterations are not five tests. Two separate normal startups complete both fixture modes with `[]` and CLEAN diagnostics; CASC, sound, third-party addons and SavedVariables are disabled.

Preservation covers explicit `OVERLAY`/sublevel 7, explicit `BACKGROUND`/sublevel 0 supplied by the helper, and XML anonymous-wrapper Texture parentKey publication/object type. Neither omitted/nil-layer behavior nor an XML draw-layer getter is asserted by those controls.

Separate OLD480 Mists observation accepts the invalid name, adds a region (5 → 6), returns `ARTWORK`/sublevel 0 and completes with CLEAN diagnostics. Not current-Mists, all-Classic or native-history proof. The proposed BACKGROUND description was contradicted by observed output; no BACKGROUND assertion failed.

Retained full format and default Retail library/six-bin check pass: 733 JSON records, zero compiler-message records, six manifest deprecations. Format peak 567672832 bytes exceeds soft high 536870912 by **30801920 bytes**, below the 1 GiB hard maximum. Initial copy-authentication schema `KeyError('compiler_executable')` remains preserved; corrected metadata lookup did not rehash integration. Copy integrity is not source/archive/library authenticity.

## Known gaps (current cycle)

- [x] [15-text public evidence packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/texture-draw-layer-validation-2026-10-13/README.md): independent copy/privacy gate passed at d86c01f98, 15/15 whole texts/17 public files, 28 ordered literals and zero raw/decoded privacy findings. Prior rejected normalization/regex staging is not this packet. Bounded retention only; source/runtime/native/parent proof levels remain separate.
- [ ] Fresh earlier-profile coverage and native/render/full-parent acceptance remain open.

## Out of scope

Omitted/nil layers, coercion, secret-value semantics, `CreateFontString`, other creation methods, `SetDrawLayer`, error wording and renderer/native parity. Earlier-profile behavior is preserved without claiming native history.
