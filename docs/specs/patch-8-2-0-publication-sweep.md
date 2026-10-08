# Patch 8.2.0 publication sweep

Audit Warcraft Wiki page 533235 revision 5142162 against current retail. Source: `data/patch-api/sources/8.2.0-*`. [Audit](../wiki/investigations/patch-8-2-0-api-audit.md).

## Requirements

- [x] Pin fetched current revision and retain every caption-table inventory and non-inventory occurrence with opt-in recipes.
- [x] Discover publication gaps through the prefork full-UI harness; apply later master registers chronologically, reserving 8.2.5 for integration.
- [x] Model `C_VoiceChat.SetMasterVolumeScale` through existing voice-chat state for normalized settings-slider inputs; getter reflects writes, unrelated output volume remains unchanged. Reject invalid input without mutation to preserve the simulator state invariant; native out-of-range/security parity is not claimed.
- [x] Retire unused `C_UIWidgetManager.GetTextureWithStateVisualizationInfo` on retail only after whole-word cached/source/test consumer scans. Preserve Mists' existing lookup.
- [x] Account for every unresolved inventory/prose gap with exact reasons, without placeholder publication credit.
- [x] Prove all publication sweeps, targeted affected behavior, own source reproduction, all register reproductions, Python fixtures, format and warning-clean non-vendor Mists check. Preserve and report inherited saved-extract failures rather than rewriting prior sources.

## Remaining contracts

- [ ] Fifty-nine exact inventory gaps and ten substantive prose contracts remain recorded-problematic; exact producer, lifecycle or permission-policy reasons live in the audit's per-ID review.
- [ ] Main thread inserts the real 8.2.5 register and refreshes supersessions/fixtures/accounting at integration.
- [ ] Inherited 12.0.5, 12.0.7 and 12.1.0 saved-extract reproduction failures remain unchanged, as in the completed 8.3.0 audit.

## Tests

`tests/patch_8_2_0_publication_sweep.rs`, `patch_8_2_0_publication_fixes.rs`, `patch_8_2_0_cached_surfaces.rs`, `patch_8_2_0_classic_surfaces.rs`; extractor/generator behavioral fixtures. Evidence and dynamic validator: `data/patch-api/evidence/8.2.0-session-2026-10-08/`.

## Exclusions

Publication alone is not signature, security, populated-output, historical or payload parity. No vendor/cache/Wowless edits, shims, Blizzard monkey-patches, full integration suite, push, merge or agents. Inherited saved-extract non-reproducibility remains recorded rather than silently rewritten.
