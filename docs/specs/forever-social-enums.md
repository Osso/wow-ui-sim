# Forever social enums

## Contract

Forever 1.60.1.69913 publishes the generated API documentation's social enum values and metadata:

- `ClubStreamType`: General=0, Guild=1, Officer=2, Discord=3, Other=4; metadata 0–4, five values. Other profiles retain their existing stream values.
- `BattleNetFriendTag`: Professions=0, PvP=1, Raiding=2, Dungeons=3, Delves=4, Questing=5, Roleplaying=6, DamagerRole=7, HealerRole=8, TankRole=9; metadata 0–9, ten values. Reuses the existing retail 12.1 definitions without enabling its API epoch.
- `RecentAlliesInteractionCategoryFilter`: Professions=0, PvP=1, Raiding=2, Dungeons=3, Delves=4, Questing=5; metadata 0–5, six values. Exposure is Forever-only.

## Evidence and tests

Sources: authenticated Forever `ClubDocumentation.lua`, `BattleNetSharedDocumentation.lua`, and `RecentAlliesConstantsDocumentation.lua` in `Blizzard_APIDocumentationGenerated`.

`tests/wowforever_social_enums.rs` compares all published fields and metadata against those sources and executes actual `CommunitiesUtil.SortStreams`, including Discord placement and creation-time ordering.

## Exclusions

No social service functions, support-query stubs, native conformance, or full SocialUI initialization claims. These enum definitions do not implement Battle.net or recent-allies backing services.
