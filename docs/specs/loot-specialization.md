# Loot specialization

## Behavior

- `GetLootSpecialization()` returns the last applied player value, initially 0. Zero means follow the current specialization.
- `SetLootSpecialization(id)` accepts a number and returns no values. Missing or non-convertible arguments raise `Usage: SetLootSpecialization(specializationID)`.
- The setter sends any integer without client-side class or catalog validation. At most five requests are sent per 10-second window; excess requests are silently dropped.
- The simulator models a server response on the next OnUpdate tick. An accepted response stores the value and fires `PLAYER_LOOT_SPEC_UPDATED` with no arguments. No state change or event occurs synchronously in the setter.
- The simulated server accepts 0 and IDs in the player's class catalog and rejects other IDs. This is a best guess, not native evidence.
- `HasLootSpecializations()` uses the player's class catalog.

## Native basis (70688 behavioural spec)

The clean-room behavioral specs `GetLootSpecialization.md` and `SetLootSpecialization.md` for client build 70688 establish the getter's player field, setter's type check and lack of value validation, rate limit, lack of local write, and the event's update-path origin. The actual server validation and response message are unknown. The getter's field is unsigned 16-bit and returns zero without a player.

## Best-guess basis — native UNKNOWN

[Warcraft Wiki](https://warcraft.wiki.gg/wiki/API_SetLootSpecialization) reports silent failure for invalid or other-class IDs. [Blizzard's off-spec loot announcement](https://worldofwarcraft.blizzard.com/en-gb/news/10058095/patch-5-3-adds-off-spec-loot-and-rewards) describes choosing among class specializations. Neither establishes which side rejects an ID; the simulated server rejects it as a best guess. The exact rate-limit window reset semantics and repeated-value server responses are also unknown.

## Tests

`tests/loot_specialization.rs` covers the original popup, deferred application, rate limiting and usage error.

## Known gaps

- Server validation, response timing, persistence and whether the server sends a chat message are unknown.
- No actual server connection or loot generation is modeled.
