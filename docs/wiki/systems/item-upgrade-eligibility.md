# Item-upgrade eligibility

The [contract](../../specs/item-upgrade-eligibility.md) separates supplied observations from unknown native upgrade policy. This is not an upgrade-cost, currency or upgrade-window model.

## State and lifecycle

Each environment owns a location-keyed observation map. An observation records the resolved item ID and a supplied boolean, not a catalog-wide classification. The cursor carries an observation separately, guarded by the current item ID and origin. Existing bag/cursor/equipment verbs move both observations with their items. Admin replacement/removal discards the old observation. SetReagentCount and its shared reagent-seeding helper invalidate each removed record and every newly inserted record. Updating the quantity of an existing retained record does not invent a new eligibility policy. Clearing all inputs also clears the cursor observation. New split cursor stacks have no inferred observation.

`SetItemUpgradeEligibility` resolves the same ItemLocation fields as `C_Item`; it cannot seed an absent item. `ClearItemUpgradeEligibility` clears one location or the environment's inputs. Neither method changes items or invents an upgrade event. The query checks the actual item before projecting supplied eligibility; unsupplied and empty locations return the best-guess `false` default (2026-11-01), and unresolvable arguments raise a simulator argument error.

Non-admin crafting consumption/output and server-snapshot bag/equipment rewrites are excluded; their observation lifecycle is not implemented or claimed. Equipment-set transfers are excluded. `apply_equipment_set` rebuilds equipped records from saved item IDs, not item-instance identity. This lane does not modify that path or add identity tokens.

## Consumer and proof boundary

The fixture loads the selected FrameXMLUtil and ObjectAPI dependency closure. It authenticates the original generic ItemUtil function and supplies a plain shown `ItemUpgradeFrame` as context input. The fixture does not replace a Blizzard function or claim that an original upgrade window loaded. This permits bounded generic-consumer proof on Era/Anniversary without force-loading the unselected Mists window. Forever's small Camelot utility extension is not the target function.

The restored selector observes original Match/Mismatch for two distinct items/locations, changes, bag swaps, equipment transfers, removal, clear and a second environment. The separate default selector covers unsupplied/unresolved inputs and replacement. An additional admin regression reproduces stale true after SetReagentCount removes and recreates the same item ID at the same location; the corrected path falls back to the default until a new observation is supplied. The expanded final proof below closes the admin boundary; the earlier query-only proof does not.

## Final proof — 2026-10-26

Compiled GREEN is `bb5bb9ba4`; constant-false RED is `a88e072b7`. All 2,676 tracked test files are byte-identical between them. Only the query body in `src/c_api/item_upgrade_eligibility.rs` differs. Every runtime case has exact discovery of one test, then one execution with a successful source-bound build receipt, executable hash, recorded argv and a fresh byte-exact current CVar copy. Original CVar stat/hash guards remain unchanged.

| Exact selector / boundary | Retail, PTR, Forever | Mists, Era, Anniversary |
|---|---|---|
| `api_shape_item_upgrade::original_item_util_tracks_supplied_eligibility_movement_removal_and_isolation` | RED original supplied-match failure; GREEN PASS | RED original supplied-match failure; GREEN PASS |
| `api_shape_item_upgrade::item_upgrade_unresolved_and_unsupplied_inputs_report_native_unknown` | RED rejects constant false; GREEN PASS | RED rejects constant false; GREEN PASS |
| `item_upgrade_admin_reagent_count::admin_reagent_count_recreation_discards_item_upgrade_observation` | RED rejects constant false; GREEN PASS | RED rejects constant false; GREEN PASS |
| `c_item_api::c_container::test_c_item_upgrade_supplied_false_preserves_boolean_shape` | Both branches PASS; preservation only | Both branches PASS; preservation only |
| `bags::test_bag_ui_c_container_helpers_have_safe_default_shapes` | Both branches PASS; preservation only | Both branches BLOCKED before the seed/query |

The bag-helper block is the existing `tests/common/mod.rs:204` requirement for `Blizzard_POIButton`, whose cache has no compatible TOC on Mists/Era/Anniversary. Earlier `red2` receipts reach the same boundary. Tests and profile gates are unchanged. GREEN has 27 passing executions and three blocked preservation executions, not 30 passes. The separate generic-consumer lifecycle still passes on all six profiles.

The admin regression also failed against the real supplied query at `2d9d51bae`: SetReagentCount removes and recreates item 6948 at the same location, but the old observation still returned true. This is the mutation-specific RED. The final constant-false sibling fails earlier at the supplied-true precondition and is not a substitute for that regression proof.

All seven separate `cargo check --all-targets` receipts at GREEN succeed with zero warnings in Cargo JSON and plain output. Formatting passes at that same source; changed-line readability is a manual audit because metric executables were unavailable. No broad suite or new Wrath runtime acceptance is claimed.

Frozen-base stock controls use tests-only `b0d1a2b09` over `702c60c38`; producer bytes are unchanged. Final GREEN raw message/count maps match exactly: Retail 8/10, PTR 214/280, Forever 26/26, Mists 6/12, Era 0/0 and Anniversary 0/0. There are no added or removed keys and no count changes, so no newly exposed emitter or masking predecessor needs attribution. Era/Anniversary zero maps cover only their selected startup inputs, not complete native UI health.

The lane handoff retains `final-evidence-audit.json`, `final-stock-per-key.json`, `test-byte-proof-final.json`, `pinned-evidence.json`, `final-*-case*-{list,run}/receipt.json`, stock receipts, all-target build certificates, JSON warning summaries, plain journals and CPU accounting. Documentation-only commits carry this proof through unchanged source/test hashes; they do not trigger duplicate runtime or compiler commands.

## Sources

- [Producer](../../../src/c_api/item_upgrade_eligibility.rs).
- [Existing inventory transfers](../../../src/c_api/container_inventory.rs).
- [Behavioral fixtures](../../../tests/api_shape_item_upgrade.rs).
- [Admin recreation regression](../../../tests/item_upgrade_admin_reagent_count.rs).
- [Contract and pinned declaration/consumer lines](../../specs/item-upgrade-eligibility.md).

## See Also

- [[consumer-selection-and-red-proof]] — selection, preservation and producer acceptance.
- [[cf2-tts-integration]] — historical deferred shape coverage.
