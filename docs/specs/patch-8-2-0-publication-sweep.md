# Patch 8.2.0 publication sweep

Audit Warcraft Wiki page 533235 revision 5142162 against current retail. Source: `data/patch-api/sources/8.2.0-*`. [Audit](../wiki/investigations/patch-8-2-0-api-audit.md).

## Requirements

- [x] Pin fetched current revision and retain every caption-table inventory and non-inventory occurrence with opt-in recipes.
- [x] Discover publication gaps through the prefork full-UI harness; apply later master registers chronologically, reserving 8.2.5 for integration.
- [x] Model `C_VoiceChat.SetMasterVolumeScale` through existing voice-chat state for normalized settings-slider inputs; getter reflects writes, unrelated output volume remains unchanged. Reject invalid input without mutation to preserve the simulator state invariant; native out-of-range/security parity is not claimed.
- [x] Retire unused `C_UIWidgetManager.GetTextureWithStateVisualizationInfo` on retail only after whole-word cached/source/test consumer scans. Preserve Mists' existing lookup.
- [ ] Account for every unresolved inventory/prose gap with exact reasons, without placeholder publication credit.
- [ ] Prove all publication sweeps, targeted affected behavior, source reproduction, Python fixtures, format and warning-clean non-vendor Mists check.

## Tests

`tests/patch_8_2_0_publication_sweep.rs`, `patch_8_2_0_publication_fixes.rs`, `patch_8_2_0_cached_surfaces.rs`, `patch_8_2_0_classic_surfaces.rs`; extractor/generator behavioral fixtures. Evidence and dynamic validator: `data/patch-api/evidence/8.2.0-session-2026-10-08/`.

## Exclusions

Publication alone is not signature, security, populated-output, historical or payload parity. No vendor/cache/Wowless edits, shims, Blizzard monkey-patches, full integration suite, push, merge or agents. Inherited saved-extract non-reproducibility remains recorded rather than silently rewritten.
