# ATT unit hyperlink tooltip

`AllTheThings/src/Classes/NPC.lua:58` reads `C_TooltipInfo.GetHyperlink("unit:Creature-…").lines[1].leftText`. This exposed missing hyperlink data, but it did not establish native GUID lookup behavior.

## R82 correction

VERIFIED: R82 (Retail 70688) establishes substring marker search, 22 ordered branches and producer-specific requests. UNKNOWN: accepted unit GUID prefixes, object lookup, unresolved-object results and server-backed rows. The previous simulator mapping from seeded player/target/focus/party GUIDs to unit tooltips was a guess, not native evidence.

The R82 dispatcher removes that mapping. Unit links remain parsed but do not produce invented rows. The simulator's unmodelled result policy is not a native nil/arity claim. Main approved three separately committed assertion retargets in `tests/tooltip_item_sources.rs`; they now preserve no-fabricated-data and item/spell controls rather than guessed names, levels or result shapes.

## Dispatcher boundary

Typed hyperlink requests live under `src/c_api/c_tooltip_info_hyperlink/`. The old frame-method item-ID bypass is removed; parsed item descriptors reach the existing item producer. Spell routing uses a narrow link adapter while R81 changes its producer. The [hyperlink contract](../../specs/tooltip-hyperlink.md) owns the exact requirements, supported rows and native-UNKNOWN exclusions. Catalog-backed artifact/conduit selection uses the existing INFERRED 70391 rows in an isolated commit, separate from request parsing and R82b rank arithmetic.

## Sources

- R82/R82b, `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md` — native routing and UNKNOWN boundaries.
- [Hyperlink contract](../../specs/tooltip-hyperlink.md) — requirements and proof inventory.
- [Tooltip identity spec](../../specs/tooltip-identity.md) — preserved direct/action identity contracts.

## See Also

- [[lua-api]] — tooltip data system.
- [[tooltip-money-line]] — separate tooltip investigation.
