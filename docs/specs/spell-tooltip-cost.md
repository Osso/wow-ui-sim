# Shared spell-tooltip cost eligibility

`C_TooltipInfo.GetHyperlink` and `GetSpellByID` share resource-cost eligibility.
WM98 native control-flow evidence covers Forever 1.60.1.70707, Retail 12.1.5.70688
and PTR 12.1.7.70668. It rejects an unconditional hyperlink-cost exclusion.
[The historical audit](../wiki/investigations/patch-6-2-0-api-audit.md) explains
why the older frame-method note is not a modern data-getter oracle.

## What it must do

- [x] Use one cost producer for all profiles and both spell-data getters.
  `isLink` and `dontOverride` do not exclude cost text.
- [x] Exclude cost text when mode is 1, content suppression is set, or
  SpellMisc attribute group 9 mask 32768 is set. Other modes and masks alone
  must not suppress the existing eligible cost fixture.
- [x] Read supplied context first, otherwise normal mode 0 with content
  suppression clear. Normal getter defaults are source-backed by WM98.
- [x] Read source attribute group 9 from generated CSV build 12.1.5.70391 or
  the selected Forever spell cache. Preserve absent-row identity as `None`.
  **UNKNOWN:** `unknown_spell_attributes_9` treats an absent attribute row as
  clear bits; this is the sole missing-attribute seam, not native parity.
- [x] Keep only the existing Flash of Light 19750 formatted resource fixture.
  No fixture means no manufactured resource-cost string. Retain other tooltip
  lines and spell identity; their exact native layout is not newly claimed.

## How it works

- [Historical tooltip audit and WM98 correction](../wiki/investigations/patch-6-2-0-api-audit.md)
- [Lua API environment](../lua-api.md)

## Implementation inventory

- `src/c_api/spell_tooltip_cost.rs`: shared mode/content/attribute gates and
  the one named absent-attribute seam.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs`: shared DTO producer.
- `src/lua_api/state{,/sim_state}.rs`: supplied per-spell cost context.
- `src/bin/wow_cli/gen_spells.rs`: full and attribute-only generation paths.
- `crates/sim-data/data/spells.rs`: source-backed optional attribute group 9.
- `src/forever_spell_runtime.rs`: retains cached Forever Attributes_9.
- `src/spell_lookup.rs`: supplement rows explicitly lack attribute provenance.

## Tests asserting this spec

- `src/c_api/spell_tooltip_cost_tests.rs`: both public getters, positive and
  negative gate controls, and two spells without cost fixtures.
- `src/loader/tests/wow_api_tooltip.rs`: item/spell link payload, cost, cast text.
- `tests/patch_6_2_0_behavior.rs`: short/full links, cost retention, direct-query
  restoration and frame spell identity through the shared simulator producer.
- `src/bin/wow_cli/gen_spells.rs` tests: signed attribute masks, default-difficulty
  selection, attribute-only byte preservation and idempotence.

Bounded proof on runtime `d4fde71d1`: Forever targeted 18/18; Retail library
2206/0/1 plus cost gates, bare links and original prefork consumer; PTR library
2202/0/1 plus cost gates and bare links. Library counts include two new WM98
tests. These are simulator tests, not new native runtime observations.

## Known gaps (current cycle)

- [ ] PTR native evidence is build 70668, not pinned simulator UI 70493.
- [ ] Complete resource eligibility/formulas, required-aura conditions and
  formatting. No general resource calculation was added.
- [ ] WM98 is native local control-flow evidence, not observed client Lua output.

## Out of scope

Range/cast text rules, override selection, special effect-215 rewards, native
cost/range row placement and historical 6.2.0 frame-method conformance remain
separate contracts. The current frame presentation delegates to the shared DTO;
these tests do not claim independently observed native frame behavior.
