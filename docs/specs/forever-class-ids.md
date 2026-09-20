# Forever class enumeration

For `client-wowforever`, `C_SpecializationInfo.GetAllClassIDs()` returns one fresh, contiguous Lua array of class IDs from the simulator's class catalogue. Membership does not depend on specialization availability. Mutating a returned array cannot change subsequent results.

Other profiles retain their existing publication behavior.

Source contract: Forever `Blizzard_APIDocumentationGenerated/SpecializationInfoDocumentation.lua` declares a non-nil numeric array. `Blizzard_Communities/ClubFinder.lua` enumerates it during file initialization to count specializations. Catalogue contents follow the existing simulator class model; this does not establish native Forever roster conformance.

Behavioral coverage: `tests/wowforever_class_ids.rs` checks catalogue IDs, return independence and actual vendor ClubFinder file initialization.
