# Spell tooltip composition

The shared spell tooltip producer uses R81 Retail 12.1.5.70688 evidence for its outer composition. Its source is `src/lua_api/globals/missing_surface/tooltip_info/spell.rs`. [Tooltip data](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) describes the host-input boundary. This is not a claim of native parity for other builds.

## What it must do

- [ ] VERIFIED R81 Q1: GetSpellByID resolves an effective spell unless dontOverride is set. A spell hyperlink keeps the parsed ID and passes a link context without suppressing cost or range.
- [ ] VERIFIED R81 Q1: ordinary detail rows follow replacement, Azerite annotation, cost/range, cast/recast order. Replacement requires a nonzero override, a resolved name and no suppression; it is left-only, type 0, normal color, without wrapText.
- [ ] VERIFIED R81 Q1/Q2: cost/range rows are highlighted type 0; first range may share cost's right side, remaining ranges go left. Empty buffers add nothing.
- [ ] VERIFIED R81 Q3: apply the supplied mode, native SpellMisc attribute-group masks, alternate-header and first-effect skip conditions. Existing attributes_9 catalog provenance remains separate from R81's verified native mask.
- [ ] VERIFIED R81 Q2: positive final cast milliseconds select SPELL_CAST_TIME_SEC below 60000 and SPELL_CAST_TIME_MIN otherwise, formatted through the Blizzard global. Nonpositive or absent results do not imply Instant.
- [ ] UNKNOWN helpers use neutral host inputs: final cost, range, alternate/recast/cooldown text and final cast milliseconds. An absent input omits its row. Aura remaining text is also already formatted and host-supplied by aura instance ID.
- [ ] The unsupported fixed slot-to-key binding line is absent.

## How it works

- [Tooltip data](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation)

## Implementation inventory

- `src/c_api/spell_tooltip.rs`: explicit host helper results and verified native masks.
- `src/c_api/spell_tooltip_cost.rs`: resource display eligibility and host text.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs`: shared rows and aura presentation.
- `src/c_api/c_tooltip_info_spell_mount.rs`: direct spell options and unchanged secret-input policy.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs`: generic option routing and action lookup.

## Tests asserting this spec

- `tests/tooltip_spell_r81.rs`: original Blizzard setters and direct data consumers.
- `tests/tooltip_item_spell.rs`: host-supplied presentation, colors and omission.
- `src/c_api/spell_tooltip_cost_tests.rs`: cost eligibility.

## Known gaps (current cycle)

- [ ] Record RED/GREEN and profile test results in the lane report.

## Out of scope

UNKNOWN resource formulas, cast/live modifiers, aura remaining-time calculation, charge/cooldown predicates, requirements, reward-path gates, exact later-row order and evaluated description templates. No 70448 DB2 mappings are introduced. Existing description behavior is unchanged by instruction. Full Azerite helper gates remain host-supplied.

Evidence: `/home/osso/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md`, section R81, Q1–Q4. The selected Retail runtime cache stays at 70575; evidence for 70688 does not certify its serialized output.
