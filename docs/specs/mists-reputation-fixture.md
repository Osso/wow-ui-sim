# Mists reputation fixture

Mists uses the existing seeded reputation fixture, with `isAccountWide = false` for every seeded faction. This is a scenario default, not a claim about native Mists behavior, which remains unknown. Other profiles retain their existing fixture values. Explicitly supplied reputation state is not rewritten.

The pinned Mists `Blizzard_APIDocumentationGenerated/ReputationInfoDocumentation.lua:49-59,133-153` declares `C_Reputation.IsAccountWideReputation` and the `FactionData.isAccountWide` field. The selected `Blizzard_ActionBar/Shared/ReputationBar.lua:146-148` uses that query to select the account-wide label, which is absent from the pinned Mists strings. The Classic legacy list globals have a separate [behavior contract](mists-legacy-reputation-list.md).

## Legacy-list comparator inputs

Mists R21 sorts the same row array that the legacy globals read. Scenario fixtures may set `isSpecialHeader` and numeric `headerSortValue` values to exercise the observed header ordering. These are simulator fixture fields; `headerSortValue` does not claim the native field name, and mapping that native numeric field to Expansion is INFERRED. Collapsed child rows remain addressable above the visible count, but their cross-group order is undefined.
