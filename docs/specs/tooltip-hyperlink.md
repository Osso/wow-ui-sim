# Hyperlink tooltip requests

The hyperlink dispatcher follows R82/R82b, Retail 12.1.5.70688. Source lives in `src/c_api/c_tooltip_info_hyperlink.rs`; see the [tooltip investigation](../wiki/investigations/att-unit-hyperlink-tooltip.md). PTR/Forever reuse is INFERRED, not separate native proof.

## What it must do

- [x] Search the NUL-terminated input for all 22 R82 markers in native priority order, independent of text position. Accept raw tags and colored links; parse decimal and `0x`/`0X` numbers. VERIFIED — `tooltip_hyperlink_r82_marker_priority_precedes_text_position`, `tooltip_hyperlink_r82_raw_and_coloured_currency_omit_top_level_quantity`, and `tooltip_hyperlink_r82_rank_inputs_and_marker_inventory`.
- [x] Preserve parsed item descriptor, enchant, three gems, fourth-gem scalar, link level, contexts, upgrade mask, ordered bonus lists/modifiers, socket-bonus groups, crafter GUID and final scalar. Discard suffix/unique ID; never derive an owned GUID from them. Reject bonus count 16 or higher. VERIFIED — `tooltip_item_descriptor_original_hyperlink_preserves_bonus_lists` and `tooltip_hyperlink_r82_parses_item_request_fields_without_owned_identity`.
- [x] Keep the two wrapper integers separate. Preserve the item's existing second field when its wrapper value is absent. The native downstream meanings are UNKNOWN. VERIFIED — `tooltip_hyperlink_r82_parses_item_request_fields_without_owned_identity` and `tooltip_hyperlink_r82_spell_wrapper_values_do_not_replace_spell_identity`.
- [x] Spell links use mode 0, link semantics, disabled ordinary override resolution, and difficulty `-1` when absent, not a rank field. Wrapper integers do not replace the spell ID. VERIFIED — `tooltip_hyperlink_r82_spell_difficulty_is_not_rank`, `r81_spell_override_and_hyperlink_keep_distinct_identity_and_cost_range`, and `test_set_hyperlink_spell_link_populates_spell_tooltip`.
- [x] Currency links pass quantity unchanged, default zero, to the existing producer. The title is type 0, without divide-by-100 scaling. VERIFIED — `set_hyperlink_resolves_currency_links` and `tooltip_hyperlink_r82_raw_and_coloured_currency_omit_top_level_quantity`.
- [x] R84 adds the populated currency payload contract: the amount affects text/conditional rows but is not a top-level `quantity`; see the [currency contract](tooltip-currency.md). VERIFIED — `set_hyperlink_resolves_currency_links`.
- [x] `urlIndex`, `storecategory`, `perksactivity`, and `initiativetask` call no producer. Trade returns false; server response state stays unmodelled. Neither battle-pet spelling is a branch. VERIFIED — `tooltip_hyperlink_r82_no_producer_markers_do_not_fall_through`, `tooltip_hyperlink_r82_trade_returns_false`, and `tooltip_hyperlink_r82_rank_inputs_and_marker_inventory`.
- [x] R82b artifact rank text uses total after the flag check, including total zero despite lookup rank one. The second operand uses explicit normal-maximum/rank-limit/player-setting inputs. Adjusted maximum uses wrapping addition of `total-base` followed by a signed cap comparison. Flag 16 decrements the bonus only when signed-positive; nonzero adjusted bonus uses increase color, zero uses highlight. VERIFIED — `tooltip_hyperlink_r82_apower_rank_colour_uses_adjusted_bonus` and `tooltip_hyperlink_r82b_artifact_display_math_uses_explicit_context`.
- [x] Essence mode 2 uses explicit rank/main/passive selection and supplied localized headings, emits identity type 8, and orders main before passive without next-rank/source/requirement annotations. VERIFIED — `tooltip_hyperlink_r82_essence_uses_supplied_rank_spell_selection`.
- [x] Conduit mode 1 clamps signed rank below one, emits descriptor rows before description, preserves item-level metadata and suppresses soulbound. VERIFIED — `tooltip_hyperlink_r82_conduit_clamps_rank_and_suppresses_soulbound`.
- [x] Quest uses a supplied quest-log record as its record-provider seam and emits only identity type 23 plus one type-0 title. Retail PvP talent uses primary SpellID and suppresses annotations. VERIFIED — `tooltip_hyperlink_r82_quest_only_publishes_supplied_title` and `tooltip_hyperlink_r82_pvptal_uses_primary_spell_without_annotation`.
- [x] Retail keystone links parse eight scalars, route the inferred first scalar to the shared item producer with mode 0 and clear preview, and preserve two distinct item/SellPrice outputs through `GameTooltip:SetHyperlink` even when `hideVendorPrice` is true. VERIFIED route behavior per R88aw; first-scalar mapping INFERRED per R82.
- [ ] The remaining seven keystone scalar-to-request mappings are UNKNOWN and discarded.

## How it works

- [Tooltip investigation and removal boundary](../wiki/investigations/att-unit-hyperlink-tooltip.md)
- [Tooltip system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation)

## Implementation inventory

- `src/c_api/c_tooltip_info_hyperlink.rs`: dispatch and existing-producer composition.
- `src/c_api/c_tooltip_info_hyperlink/parse.rs`: typed requests and R82b rank arithmetic.
- `src/c_api/c_tooltip_info_hyperlink/inputs.rs`: explicit native-UNKNOWN state/record inputs with consumer citations.
- `src/c_api/c_tooltip_info_hyperlink/catalog.rs`: isolated INFERRED 70391 artifact/conduit bindings and R82 prefix composition.
- `src/c_api/item_descriptor/link.rs`: parsed item descriptor fields.
- `src/lua_api/globals/missing_surface/tooltip_info/`: bounded item/spell/currency adapters; no new producer formulas.
- `src/lua_api/frame/methods/widgets/tooltip/content.rs`: frame-method request forwarding without item-ID bypass.

## Tests asserting this spec

- `tests/tooltip_hyperlink_r82.rs`: `tooltip_hyperlink_r82_raw_and_coloured_currency_omit_top_level_quantity`, `tooltip_hyperlink_r82_marker_priority_precedes_text_position`, `tooltip_hyperlink_r82_no_producer_markers_do_not_fall_through`, `tooltip_hyperlink_r82_trade_returns_false`, `tooltip_hyperlink_r82_apower_rank_colour_uses_adjusted_bonus`, `tooltip_hyperlink_r82_conduit_clamps_rank_and_suppresses_soulbound`, `tooltip_hyperlink_r82_essence_uses_supplied_rank_spell_selection`, `tooltip_hyperlink_r82_quest_only_publishes_supplied_title`, `tooltip_hyperlink_r82_pvptal_uses_primary_spell_without_annotation`, `tooltip_hyperlink_r82_parses_item_request_fields_without_owned_identity`, `tooltip_hyperlink_r82_spell_difficulty_is_not_rank`, `tooltip_hyperlink_r82_spell_wrapper_values_do_not_replace_spell_identity`, `tooltip_hyperlink_r82_rank_inputs_and_marker_inventory`, and `tooltip_hyperlink_r82b_artifact_display_math_uses_explicit_context`.
- `tests/tooltip_item_descriptor.rs`: `tooltip_item_descriptor_original_hyperlink_preserves_bonus_lists`.
- `tests/tooltip_currency_r84.rs`: `set_hyperlink_resolves_currency_links`.
- `tests/tooltip_residual_prefork.rs`: `tooltip_residual_keystone_links_keep_sell_price_preview_clear` drives two eight-scalar keystone-marker links through the original `SetHyperlink` consumer.

VERIFIED on code tree `195448e8b` after the m41 merge: Retail library 2264/2264 and tooltip-related integration 653/653 pass, with zero compiler warnings. VERIFIED on preceding production tree `0763fde84`: PTR and Forever each pass all 15 exact changed cases, with zero warnings. These are simulator proofs, not full native output equivalence; request-only and catalog evidence limits remain as stated above.

## Known gaps (current cycle)

- [ ] Difficulty, mode and unknown wrapper fields are retained but cannot affect the current spell producer, which has no corresponding inputs. R81 integration is separate.
  Reason: R82 verifies request fields, but does not establish the missing downstream producer effects; R81 integration remains separate.
- [ ] Artifact/conduit common spell rows remain the existing description-only composition while R81 changes the producer; no guessed cost/learn/footer rows are added.
  Reason: R82 leaves the complete serialized spell rows UNKNOWN, so adding them would invent native output.
- [ ] Catalog bindings are INFERRED 70391 values, isolated in a droppable commit. The existing standalone artifact getter's flag8/maximum policy is not reused as hyperlink evidence.
  Reason: The bindings have only 70391 inferred evidence, not direct R82 native confirmation.
- [ ] The remaining seven keystone scalar-to-request mappings are UNKNOWN and discarded.
  Reason: R82 proves eight scalars and an item request, but not the field mapping.

## Out of scope

Unit object lookup, quest objectives/state rows, achievement/instance-lock rows, exceptional malformed input and native miss arity are UNKNOWN. They must not produce guessed rows. Talent output is unmodelled because no selected Retail link producer was established; non-Retail PvP talent lacks a row source. Enchant validation, curio/mount discriminators, Maw-power record selection and the remaining seven keystone scalar mappings stay unmodelled where the backing lookup or request mapping is unknown. No unrelated producer substitutes for them.
