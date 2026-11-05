# PvP reward and available-brawl caches

## Contract

- Retail `C_PvP.GetRandomBGRewards`, `GetArenaSkirmishRewards`, and `GetRandomEpicBGRewards` read three independent supplied caches through one serializer. A reset-empty cache returns exactly `0, 0, nil, nil, nil`; absent item and currency lists are nil. Item rows contain `id`, `name`, `texture`, `quantity`; currency rows contain `id`, `quantity`. The final role-shortage value is nil. The meanings honor and experience for the first two integers are inferred from FrameXML.
- Retail and Forever `C_PvP.GetAvailableBrawlInfo` returns one `PvpBrawlInfo` snapshot or nil. A supplied nonzero regular-brawl ID remains available while simulator time is no later than receipt time plus duration. An absent duration counts as zero; equality returns a record with zero time remaining. A false `canQueue` does not hide the record.
- The original `HonorFrameBonusFrame_Update` consumes these values. The regular-brawl cache is independent of the special-event brawl cache.

## Evidence and limits

Retail pinned `PvpInfoDocumentation.lua:227–237,255–262,583–593,605–615,1628–1645` declares the functions and reward fields. `Blizzard_PVPUI/Mainline/Blizzard_PVPUI.lua:1043–1110` consumes them. WM26 identifies the local decision and serialization paths in Retail 12.1.5 build 70688; its publication connections are conditionally emulated, not a whole-wrapper or live-client proof.

Cache inputs and empty initial values are simulator scenarios. Native cache reset timing, throttled refresh, server replies, event delivery, complete return reachability and other profiles remain unknown. No refresh response is synthesized.

## Proof

`tests/pvp_reward_consumers.rs` and `tests/pvp_available_brawl_consumer.rs` each passed their exact original HonorFrame selectors at `b94c49535`. Separate REDs reached the missing random, arena, epic and available-brawl members at `b945b5225`, `9355362c9`, `2503c5fe2` and `d721bb190`, respectively. The special-event consumer has a separate RED/GREEN record in [its spec](special-event-brawl.md).
