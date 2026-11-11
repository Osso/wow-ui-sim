# Quest reward spell IDs

Retail, PTR, and Forever `C_QuestInfoSystem.GetQuestRewardSpells([questID])` reads supplied quest state. The cached Forever `QuestInfoSystemDocumentation.lua:90-104` declares an optional quest ID and a spell-ID table or no return; Forever `Blizzard_UIPanels_Game/Mainline/QuestInfo.lua:622` calls it in the original reward panel. WM53-B verifies the selection and ordering, and emulates the return connections.

An omitted ID selects the quest-log selection. A ready supplied cache entry wins even if empty; its builder skips zeros and duplicates and keeps at most three IDs in order. If that route fails, an explicitly supplied ID matching the supplied current detail quest selects its three slots, skipping zeros but retaining duplicates. Success returns one table; failure returns zero values.

The cache-request lifecycle, native eligibility and condition predicates, and hidden/learned filtering are not modeled. Supplied cache candidates are assumed already eligible. The separate detail source is an INFERRED interpretation of the native state; its spell IDs are supplied, not inferred from the Wrath name/texture reward.
