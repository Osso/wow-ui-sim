# XML self-relativeKey anchors

Approved simulator policy for the observed ProductChoice path: a Texture XML direct `$parent.<ownParentKey>` self-anchor targets that receiver's parent instead. The generator publishes that single-segment parentKey immediately before its anchors. Direct Lua self-anchoring remains invalid. Source owner: `src/loader/xml_texture.rs`; see [XML templates](../xml-template-system.md). Not a native WoW parity claim.

## What it must do

- [x] Instantiate unchanged cached Mists `ProductChoiceItemDisplayTemplate` through real `CreateFrame` inheritance, preserving layers, decorations, scripts and child-frame subtrees with actual cached ModelScene and ProductChoice Lua prerequisites. Mouse handlers must bind to their real cached functions, not missing-global substitutes.
- [x] Preserve Icon size 63 × 63 and its single CENTER anchor: target exactly parent card, relative point CENTER, offsets (0, 15). Resolving `$parent.Icon` to Icon itself must not install a self-target or abort production.
- [x] Create later distinct IconBorder with parent card and a single CENTER anchor targeting sibling Icon, relative point CENTER, offsets (0, -5); complete following child-frame production with the entire Lua error accumulator empty.
- [x] Direct Lua self-`SetPoint` fails under `pcall` without changing any existing point; a distinct-target call succeeds with exact identity, relative point and offsets, preserving the untouched point.

## How it works

- [XML template loading](../xml-template-system.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Anchor resolution](../anchor-resolution.md)

## Implementation inventory

- `src/xml/parse.rs`: public cached XML parsing.
- `src/xml/template.rs`: virtual frame and texture registration.
- `src/lua_api/globals/create_frame/template_chain/runtime_loader_effects.rs`: runtime inheritance enters production layer loading.
- `src/loader/xml_layer_batch.rs`: ordered layer production.
- `src/loader/xml_texture.rs`: normalize only the published direct self-key to `$parent`; retain all other named-key handling and borrow unchanged anchor data without cloning.
- `src/lua_api/frame/methods/button_anchor_hierarchy/anchors.rs`: direct Lua anchor validation and mutation.

## Tests asserting this spec

`tests/xml_self_relative_key.rs` (auto-discovered primary integration target, Mists-only):

- `xml_self_relative_key_product_choice_targets_parent_and_preserves_later_sibling`
- `xml_self_relative_key_direct_lua_self_setpoint_rejects_without_mutation`

## Known gaps (current cycle)

- [x] Execute focused pre-producer Mists RED at `a90f84c5d`: 1 PASS / 1 FAIL, actual exit101. Real prerequisites load; ProductChoice creation fails at layer-child SetPoint self-anchor. Direct Lua rejection/preservation/distinct-target control passes. Two prior admission denials are separate zero-launch records.
- [x] Implement bounded Texture XML direct self-key normalization after executed RED; runtime SetPoint validation and shared frame/font-string generation remain unchanged.
- [x] Independently accept focused Mists producer `480b9f934` GREEN: 2/2, actual exit0. Both real cached construction and direct Lua atomic rejection assertions reached.
- [x] Independently accept normal Mists CLI startup: 142 cached Blizzard addons, CLEAN, zero observed Lua errors, actual exit0. CASC/sound and third-party addons/SavedVariables disabled; existing WTF EditMode cache still read read-only. Not rendered/native/installed-CASC or broader normal-session acceptance.
- [x] Independently accept shared runtime anchor controls 33/33 and Frame inherited/inline child-anchor controls 3/3. Historical expected counts15/2 were incorrect; actual results and additive corrections preserved, no reruns.
- [x] Independently accept named Mists cast-bar exact prefork control: 1/1, actual exit0 after explicit 0.55s fade settlement. Source `2d4b7e56a` contains documentation changes over producer480; original empty summary-regex output and historical immediate-hidden FAIL remain unchanged.
- [x] Independently accept final default lib/binaries check: 733 records, build-finished success, terminal exit0, zero source compiler-message records; six manifest deprecations remain. Project `cargo fmt --check` exit0. These are compile/format gates, not runtime acceptance; Mists builds separately retain a UnitRole source warning.
- [x] Publish complete follow-up evidence: [current owner](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/xml-self-relative-key-2026-10-13/README.md) separates initial77 and Store1 accepted copies from follow-up42 technical copy/privacy PASS and corrected metadata-reuse4/4 PASS. Historical no-read process FAIL, stale-label metadata FAIL, prepublication failures, pending-at-selection labels and Store failures remain unchanged. Original-protocol compliance and exact mtime history are not claimed; these gates do not establish new runtime/native/full-parent acceptance.
- [x] Classify Store fixture applicability from active Mists TBC sources: declared RightInset offset24 conflicts with test2, and `ProductCards` conflicts with the queried pool. [Whole counterexample](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/xml-self-relative-key-2026-10-13/store-contract-counterexample.md) retains limits: actual sibling-target identity and card bounds were not observed; executed0/2FAIL is neither XML regression proof nor product-defect attribution.
- [ ] Profile-correct Store runtime preservation remains unproved; no fixture expectations weakened or vendor behavior changed.

## Out of scope

Native parity; synthetic baseline-frame attribution to ProductChoice; generic missing-key fallbacks; other self-reference spellings/aliases; frame/font-string anchor changes; direct Lua validation changes; editor behavior; vendor patches; new ModelScene behavior; startup-wide acceptance.
