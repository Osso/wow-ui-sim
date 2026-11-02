# Battleground enlistment bonus

Retail `PvpInfoDocumentation.lua:932–939` declares `C_PvP.IsBattlegroundEnlistmentBonusActive` with two boolean results. The pinned Retail PVPUI consumer is `Blizzard_PVPUI/Mainline/Blizzard_PVPUI.lua:1019`. This contract models the normal local publication recovered from Retail 12.1.5 build 70688; it does not assert complete native return reachability through opaque guards.

The simulator accepts absent-by-default player level, prerequisite and flag gates, configuration key, cache-enabled state, and cached integers. Missing input or a rejected gate returns `false, false`. At level 90 or greater with enabled gates and cache, key 1 selects IDs 10542 and 13646; key 0 selects IDs 10537 and 13647. Each nonzero value produces true in its corresponding result; missing values produce false. The second result does not consult special-event brawl information. No arguments are consumed on the recovered path.

Known gaps: key-to-faction meaning, the underlying prerequisite and flag producers, full native return reachability, and server/cache lifecycle are unknown. The simulator's explicit inputs are scenarios, not claims about native defaults. Changes to native cached identifiers may conditionally dispatch `PVP_WORLDSTATE_UPDATE`; this input model does not implement the writer or promise that event.
