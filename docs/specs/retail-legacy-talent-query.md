# Retail legacy talent query

Retail 12.1.5 declares `C_SpecializationInfo.GetTalentInfo(query)` with one required `TalentInfoQuery` and one nullable result in `Blizzard_APIDocumentationGenerated/SpecializationInfoDocumentation.lua:292–304`; its query fields are at lines 533–544. The original `Blizzard_DeprecatedSpecialization/Deprecated_Specialization_Standard.lua:36–51` wrapper constructs that query and returns nil when its result is absent.

The Retail query validates the table, numeric index fields and optional target; tier and column must be present. With no modeled Retail legacy tier/column talent grid, the result is one nil. Zero and out-of-grid indices do not manufacture talent rows. The Mists implementation is unchanged.

Forever 1.60.1.70707 also declares the one-table query in `SpecializationInfoDocumentation.lua:316`. The same temporary implementation validates the query and returns nil for its unpopulated grid. Native evidence identifies the populated result fields as `talentID`, `name`, `icon`, `tier`, `column`, `selected`, `available`, `spellID`, `isPVPTalentUnlocked`, `known`, `grantedByAura`, `rank`, `maxRank`, `meetsPrereq`, `previewRank`, `meetsPreviewPrereq`, `isExceptional`, and `hasGoldBorder`. No populated Forever talent records are modeled, so this does not implement or test a populated result.

**Known gaps:** The simulator does not model the Retail or Forever legacy player/inspect talent grids, spell-name and selection inputs needed to return a populated row. Numeric-string and non-finite coercion, restricted-value behavior and inspect-cache lifecycle remain unverified. The current index validation is an inferred bounded implementation, not a claim of complete native parser parity.
