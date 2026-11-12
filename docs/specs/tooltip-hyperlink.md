# Hyperlink tooltip requests

The hyperlink dispatcher follows R82/R82b, Retail 12.1.5.70688. Source lives in `src/c_api/c_tooltip_info_hyperlink.rs`; see the [tooltip investigation](../wiki/investigations/att-unit-hyperlink-tooltip.md). PTR/Forever reuse is INFERRED, not separate native proof.

## What it must do

- [ ] Search the NUL-terminated input for all 22 R82 markers in native priority order, independent of text position. Accept raw tags and colored links; parse decimal and `0x`/`0X` numbers.
- [ ] Preserve parsed item descriptor, enchant, three gems, fourth-gem scalar, link level, contexts, upgrade mask, ordered bonus lists/modifiers, socket-bonus groups, crafter GUID and final scalar. Discard suffix/unique ID; never derive an owned GUID from them. Reject bonus count 16 or higher.
- [ ] Keep the two wrapper integers separate. Preserve the item's existing second field when its wrapper value is absent. The native downstream meanings are UNKNOWN.
- [ ] Spell links use mode 0, link semantics, disabled ordinary override resolution, and difficulty `-1` when absent, not a rank field. Wrapper integers do not replace the spell ID.
- [ ] Currency links pass quantity unchanged, default zero, to the existing producer. The title is type 0, without divide-by-100 scaling.
- [ ] `urlIndex`, `storecategory`, `perksactivity`, and `initiativetask` call no producer. Trade returns false; server response state stays unmodelled. Neither battle-pet spelling is a branch.
- [ ] R82b artifact rank text uses total after the flag check, including total zero despite lookup rank one. The second operand uses explicit normal-maximum/rank-limit/player-setting inputs. Adjusted maximum uses wrapping addition of `total-base` followed by a signed cap comparison. Flag 16 decrements the bonus only when signed-positive; nonzero adjusted bonus uses increase color, zero uses highlight.
- [ ] Essence mode 2 uses explicit rank/main/passive selection and supplied localized headings, emits identity type 8, and orders main before passive without next-rank/source/requirement annotations.
- [ ] Conduit mode 1 clamps signed rank below one, emits descriptor rows before description, preserves item-level metadata and suppresses soulbound.
- [ ] Quest uses a supplied quest-log record as its record-provider seam and emits only identity type 23 plus one type-0 title. Retail PvP talent uses primary SpellID and suppresses annotations.

## How it works

- [Tooltip investigation and removal boundary](../wiki/investigations/att-unit-hyperlink-tooltip.md)
- [Tooltip system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation)

## Implementation inventory

- `src/c_api/c_tooltip_info_hyperlink.rs`: dispatch and existing-producer composition.
- `src/c_api/c_tooltip_info_hyperlink/parse.rs`: typed requests and R82b rank arithmetic.
- `src/c_api/c_tooltip_info_hyperlink/inputs.rs`: explicit native-UNKNOWN state/record inputs with consumer citations.
- `src/c_api/item_descriptor/link.rs`: parsed item descriptor fields.
- `src/lua_api/globals/missing_surface/tooltip_info/`: bounded item/spell/currency adapters; no new producer formulas.
- `src/lua_api/frame/methods/widgets/tooltip/content.rs`: frame-method request forwarding without item-ID bypass.

## Tests asserting this spec

- `tests/tooltip_hyperlink_r82.rs`: original `GameTooltip:SetHyperlink` and `C_TooltipInfo.GetHyperlink`, typed parsing, rank arithmetic and supplied-state composition.
- `tests/tooltip_item_sources.rs`: no fabricated unit rows; item/spell controls.

## Known gaps (current cycle)

- [ ] Difficulty, mode and unknown wrapper fields are retained but cannot affect the current spell producer, which has no corresponding inputs. R81 integration is separate.
- [ ] Artifact/conduit common spell rows remain the existing description-only composition while R81 changes the producer; no guessed cost/learn/footer rows are added.
- [ ] Catalog bindings are INFERRED 70391 values, isolated in a droppable commit. The existing standalone artifact getter's flag8/maximum policy is not reused as hyperlink evidence.

## Out of scope

Unit object lookup, quest objectives/state rows, achievement/instance-lock rows, exceptional malformed input and native miss arity are UNKNOWN. They must not produce guessed rows. Talent output is unmodelled because no selected Retail link producer was established; non-Retail PvP talent lacks a row source. Enchant validation, curio/mount discriminators, Maw-power record selection and keystone field mapping stay unmodelled where the backing lookup or request mapping is unknown. No unrelated producer substitutes for them.
