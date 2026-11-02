# Recent Allies support query

## What it must do

- [ ] Retail `C_RecentAllies.IsSystemSupported()` returns one boolean: the supplied feature state must be enabled and the supplied support-exclusion result must be false.
- [ ] The original Social UI tab filter uses that query independently of the separate enabled input.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/RecentAlliesDocumentation.lua:136` declares the boolean query. `Blizzard_SocialUI/Mainline/SocialUI.lua:196` captures it in the Recent Allies tab definition and its original tab filter calls it. WM23-misc build 70688 verifies a feature byte and inverse exclusion predicate; boolean publication is conditionally emulated. The simulator accepts these two decisions as host inputs, initially false; their native lifecycle and the exclusion flags' meanings are unknown.

## Implementation inventory

- `src/c_api/c_recent_allies.rs`: Retail query and explicit support inputs.

## Tests asserting this spec

- `tests/recent_allies_supported_consumer.rs`: original Social UI tab filter across feature-off, feature-on and excluded states. Runtime selection and RED/GREEN are pending.

## Known gaps

- [ ] Native support-state derivation, lifecycle, event delivery, real-client publication and other profile behavior remain unverified.
