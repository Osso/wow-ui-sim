# PTR social scenario defaults

PTR's `C_BattleNet.IsFriendTagSupportedForCurrentGameType` and
`C_RecentAllies.IsInteractionCategoryFilterSupportedForCurrentGameType` return
true for any input as user-decided scenario defaults, not native observations.
PTR declarations are in `BattleNetDocumentation.lua:272` and
`RecentAlliesDocumentation.lua:69` under the pinned
`Blizzard_APIDocumentationGenerated` cache. The original SocialUIUtil builds
its supported lists during file load; RecentAlliesUtil builds its list on query.
Retail and other profiles retain their existing behavior.
