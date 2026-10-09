# Three fixture preconditions — development proof

Date: 2026-10-09. Canonical checkout: /home/osso/Projects/wow/wow-ui-sim. No cwd change.

## Scope
Only tests/edit_mode_api/enums.rs, tests/patch_12_0_7_b23_b28.rs and tests/toplevel_render_groups.rs changed. No runtime/renderer/vendor, shared SSOT/log/docs, services, deployment, push or merge changes.

## Retained RED and comparison
Existing RED revision: 96e88494a6844833b79a574d37a2115dc03b44eb. Before editing, git show for each exact fixture was compared byte-for-byte with the canonical working file. All three equal; hashes and retained copies accompany pre-edit-comparison.json. Existing fullsuite report/failure diagnostics remain at /home/osso/.local/state/wow-ui-sim/verification/fullsuite-96-comparison/. No RED rerun.

## Source grounding
- Cargo.toml: client-retail selects retail-12-1-0, cumulatively retail-12-0-5. src/lua_api/env_init/enums.rs registers patch_12_0_5_enums after base publication. That publisher CURRENT_RETAIL_VALUES has BigDefensiveIconSize=21 and BuffIconSize=22 and calculates actual-member metadata. Cached EditModeManagerConstantsDocumentation.lua:684-714 has min0/max22/count23. AddOns provenance identifies retail 12.1.0.69933, not authenticated native execution. Preserve Big21/min0; add Buff22 and correct max/count. Revision96 line44 fails max21, not Big21. No move to 12.0.0.
- docs/specs/patch-12-0-7-vehicle-sound-assets.md and src/c_api/c_ui_file_asset.rs query_asset require explicit catalog membership without IO discovery. Replace the missing external tempdir_in parent with an owned tempfile::tempdir(); preserve all file creation/removal and catalog assertions. Existing RED stops before API assertions.
- docs/specs/toplevel-render-groups.md requires visible controls. src/lua_api/frame/methods/widgets/tooltip/owner.rs record_tooltip_owner hides via set_frame_visible(false). Explicit case5 Show after SetOwner restores the model-visible-fixture precondition. Before order-position lookup assert red/blue ancestor visibility; include case IDs and texture IDs in missing-order diagnostics. Preserve all ordering/strata/raise/parent/root assertions. Existing RED line63 is red order-position unwrap, not name registration. Original archived native probe absent; this is not a new native capture or independently reproduced native Show ordering.

## Proof ledger
- Pre-edit git-show comparisons: byte equal to revision96 for all three fixtures; proof scope is pre-edit only.
- Source/spec inspection: supports bounded fixture changes, no contradictions found.
- Standalone rustfmt --edition 2024 --config skip_children=true on the three absolute paths: exit0, formatting only, no compilation.
- Targeted GREEN: NOT RUN; main owns one combined build and targeted execution. No Cargo builds/tests/checks performed here. Post-change behavior remains unproven until that gate.

## Commit
21b513f3ef9eed7a5cea07fb11acf5708ce0327a — Correct enum and visible render fixture preconditions. Exactly the three authorized fixture files committed. Pre-existing untracked .code-index.db left untouched. No post-change execution proof.

## Bounded static inspection
Changed-line readability inspection found no new suppressions, fallback behavior, nested branching, or opaque diagnostics. B28 catalog/file assertions and render order/strata/raise/parent/root assertions remain intact. Committed diff and post-edit hashes saved separately. This is static inspection, not behavioral GREEN or native proof.
