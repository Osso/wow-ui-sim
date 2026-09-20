# Forever graveyard-port availability

## Contract

- Forever publishes `CanPortGraveyard()` with one nonnil boolean result, as declared by build `1.60.1.69913`'s `PlayerScriptDocumentation.lua`.
- The result reads `PlayerState.can_port_graveyard`, independently in each simulator environment.
- Availability defaults to false for the seeded alive player. This is simulator configuration, not an inferred native eligibility rule.
- Changing availability changes subsequent queries. It does not itself show frames, teleport the player, or change death/ghost state.
- Other profiles retain their existing publication behavior.

## Consumer and tests

`tests/wowforever_graveyard.rs` executes the actual `Blizzard_FrameXML/GhostFrame.lua` consumer: OnLoad registers `ADDON_LOADED`, hides an unavailable frame, and leaves its initial visibility unchanged when available. Tests cover availability changes, isolated environments, result arity, and profile exclusion.

## Limits

No native execution or inferred graveyard eligibility rules. Automatic death/ghost transitions and a graveyard-port action are outside this predicate's scope.
