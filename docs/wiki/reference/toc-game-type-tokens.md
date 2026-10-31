# Observed TOC game-type tokens

verified: 2026-10-09

Bounded source inventory: 12 literal `AllowLoadGameType` tokens observed in cached Blizzard TOCs, not an exhaustive recognized vocabulary or a native-client acceptance result.

## Native game-type masks

Retail build 70688 recognizes `mainline`, `standard`, `plunderstorm`, `classic`, `vanilla`, `tbc`, `wrath`, `cata`, `mists`, and `wowhack` as bits 0x001 through 0x200. Forever build 70707 and PTR 12.1.7 add `camelot` and `forever`, both 0x400. Matching is case-insensitive; unknown tokens contribute no bits. Space and comma separate tokens. A zero mask imposes no restriction; a nonzero allow mask requires overlap and an exclude mask rejects overlap.

Retail 70688 takes game-type header values verbatim, including brackets. The simulator applies this rule to all profiles. On Forever 70707 the bracket game-type list splits and accumulates the same way; its Camelot client mask is 0x401. The simulator uses `mainline` and `camelot` for Forever, and retains `mainline` and `standard` for Retail and PTR.

### Known gaps

Retail and PTR live client masks are unknown. The Forever and PTR header-value paths were not independently traced; the PTR bracket splitter and gate were not independently confirmed. Classic profile token tables and masks remain unverified. These are simulator policies, not native observations.

## Observed list

`camelot`, `cata`, `classic`, `mainline`, `mists`, `plunderstorm`, `standard`, `tbc`, `vanilla`, `wowhack`, `wowlabs`, `wrath`.

The retained `/tmp/allowload-game-types-list.md` research inventories 1,539 TOCs across `retail`, `ptr`, `mists`, and `wowforever`, combining headers and inline annotations split on commas/whitespace. That extraction describes the inventory procedure, not native annotation grammar. The selected source lines below were re-read on the verification date; the inventory was not rerun. No Era/Anniversary native cache was present in the original inventory.

## Literal source evidence

Paths below are relative to `/home/osso/.cache/wow-ui-sim/blizzard-ui/`. Quotes retain source spelling, separators, and whitespace. The [tiny tracked capture](../../../data/toc-evidence/allowload-game-types-2026-10-09.json) retains these lines, full-source SHA-256 identities, and cache metadata.

| Local TOC and line | Exact source line |
|---|---|
| `retail/AddOns/Blizzard_TimerunningUtil/Blizzard_TimerunningUtil.toc:5` | `## AllowLoadGameType: mainline` |
| `retail/AddOns/Blizzard_HousingMarketCart/Blizzard_HousingMarketCart.toc:4` | `## AllowLoadGameType: standard` |
| `retail/AddOns/Blizzard_CombatLogBase/Blizzard_CombatLogBase.toc:7` | `Vanilla\CombatLogColors.lua             [AllowLoadGameType vanilla, tbc]` |
| `retail/AddOns/Blizzard_CombatLogBase/Blizzard_CombatLogBase.toc:8` | `Wrath\CombatLogColors.lua               [AllowLoadGameType wrath, cata, mists]` |
| `retail/AddOns/Blizzard_EndOfMatchUI/Blizzard_EndMatchUI.toc:4` | `## AllowLoadGameType: plunderstorm` |
| `retail/AddOns/Blizzard_HUDInventoryTemplates/Blizzard_HUDInventoryTemplates.toc:4` | `## AllowLoadGameType: plunderstorm, wowhack` |
| `retail/AddOns/Blizzard_BuffFrame/Blizzard_BuffFrame.toc:8` | `[Family]/BuffFrame.lua [AllowLoadGameType classic]` |
| `wowforever/AddOns/Blizzard_TokenUI/Blizzard_TokenUI.toc:6` | `Camelot/Blizzard_TokenUI.lua [AllowLoadGameType camelot]` |
| `wowforever/AddOns/Blizzard_AccountStore/Blizzard_AccountStore.toc:7` | `## AllowLoadGameType: standard, wowlabs` |

## Provenance limits

Each inspected `AddOns/.wow-ui-sim-blizzard-ui-complete` contains `ok`. Corresponding `.wow-ui-sim-blizzard-ui-provenance` records retail `12.1.0.69933` / product `wow`, PTR `12.1.5.69594` / `wowxptr`, and Forever `1.60.1.69977` / `wow_classic_beta`. These are cache-recorded identities, not independent native runtime validation. Mists records `source=casc-primary` and `fallback=wow-ui-source` without a version/build; no build is inferred. Exact metadata and hashes are in the capture.

## What this does not establish

Filename suffixes such as `_Mainline`, `-Camelot`, or `_Vanilla` are filename-selection strings, not evidence of accepted `AllowLoadGameType` filter strings. Header syntax and inline syntax are distinct observations; the source inventory alone does not establish their native grammar. This inventory alone does not establish a taxonomy, synonym mapping, or mode assignment.

`BCC` is not observed in this inventory. Current native Anniversary acceptance of legacy `_BCC` / `-BCC` filenames remains unverified; contradictory online documentation is not resolved by these local TOCs. No native BCC claim is made.

The simulator's active Classic profile labels include `classic_era`, `classic_anniversary`, `wrath_classic`, and `mists_classic`; their native recognition is unverified. Simulator selection/filter policy belongs in [[client-profiles]], separately from this observed list.

## Sources

- Native behavior handoffs: Retail 70688 `TOC_AllowLoadGameType.md`; Forever 70707 F1 `TOC_AllowLoadGameType.md`; PTR 12.1.7 F5 `TOC_AllowLoadGameType.md` — supplied clean-room behavioral evidence.
- Local TOC paths/line numbers quoted above — observed literals only.
- [Tracked source capture](../../../data/toc-evidence/allowload-game-types-2026-10-09.json) — exact selected snippets, hashes, and cache metadata.
- `/tmp/allowload-game-types-list.md` — original bounded inventory; temporary research path, not a durable acceptance authority.
- [Client profile spec](../../specs/client-profiles.md) — simulator contract, not a native vocabulary specification.

## See Also

- [[client-profiles]] — simulator filename precedence and filter allow-lists.
