# Recent Allies enabled query

## What it must do

- [ ] Retail `C_RecentAllies.IsSystemEnabled()` returns the existing host-supplied Recent Allies enabled input as one boolean.
- [ ] The original Friends tab header shows or hides its Recent Allies tab as that input changes.
- [ ] Existing WoW Forever behavior is unchanged.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/RecentAlliesDocumentation.lua:127` declares the boolean query; `Blizzard_FriendsFrame/Mainline/FriendsFrame.lua:652` consumes it. The existing [Recent Allies input](recent-allies-state-data.md) models whether the service is enabled, but native eligibility, default and update transitions remain unverified. The simulator projects that explicit input without deriving it from the separate support query.

## Implementation inventory

- `src/c_api/c_recent_allies.rs`: Retail and Forever registration and input projection.
- `SimState.recent_allies.enabled`: existing host-supplied state.

## Tests asserting this spec

- `tests/recent_allies_enabled_consumer.rs`: selected Friends frame tab visibility across false/true/false input. Runtime selection and RED/GREEN are pending.

## Known gaps

- [ ] Native enablement policy, lifecycle, event delivery, and other profiles remain unverified.
