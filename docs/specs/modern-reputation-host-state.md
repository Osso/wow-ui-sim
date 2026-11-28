# Retail and Forever reputation host state

The original Retail 12.1.5.70783 Mainline and Forever 1.60.1.70707 Camelot
`Blizzard_UIPanels_Game/*/ReputationFrame.lua` each rebuild the scroll data
provider from `C_Reputation.GetNumFactions` and `GetFactionDataByIndex`, then
display each original row's supplied name and standing. The UI does not
supply faction data itself. The old simulator fixture's eleven ordered
rows (three category headers plus eight factions), watched ID 2590,
descriptions, thresholds, standings, and synthetic always-present guild
row 1168 were not established as a client startup state. They remain
compatibility data only on profiles outside this change.

**VERIFIED API shape:** both pins'
`Blizzard_APIDocumentationGenerated/ReputationInfoDocumentation.lua`, including
`FactionData` at line 341, define the native namespace and fields. Adjacent
Retail 12.1.5.70688 evidence R88f.A.58–A.74 supplies checked index/ID conversion,
typed row-or-nil boundaries, selected/watched ID routing, and the zero-index
expand case. R88f.A.80–A.81 supplies activity argument/index constraints. These
answers do not establish native Forever internals or every Retail producer.

**INFERRED simulator mapping:** the host supplies `SimState.reputation` rows
and optional guild state through `A_Admin.SetReputationFactions`
([admin input](../admin-api/reputation.md)); the vector starts empty with no selected or watched
ID. A row's header and child flags designate groups in supplied order. Collapse
and expansion change only this local projection. A retained numeric sort
preference and a legacy-visibility flag do not imply a native sorter/filter.
Activity writes remain local, require a supplied valid reputation index and
eligibility, and do not assert native packets or notifications. A missing
guild row stays absent rather than synthesizing a faction. Merchant
requirements read the same model; the old inferred reaction comparison is
unchanged and missing rows fail. Existing paragon/major faction backing is
outside this model.

**UNKNOWN:** authoritative faction acquisition, locale and Faction/Group DB2
relations, native nested/inactive/collapsed list ordering, player/account
sources, guild membership lifetime, and server events are not filled from
invented records. At-war/reward-preload native side effects are not modeled by
this migration.
