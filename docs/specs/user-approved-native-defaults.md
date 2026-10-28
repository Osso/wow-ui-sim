# User-approved native-unknown defaults

These are simulator scenario defaults, approved 2026-10-28, not evidence of native behavior. Replace them when the named backing systems are modeled.

| Item | Value | Profiles | Consumer | Missing system |
|---|---|---|---|---|
| C_AccountServices.GetPreferredDatacenterLocalities | Two empty tables; user-approved, not native evidence | PTR | Blizzard_Communities/GuildPreferredPlaySettings.lua:154; Blizzard_Settings_PreferredPlaySettings.lua:75 | Account datacenter preferences |
| FORCE_DECLINE_CHARACTER | Register; do not emit | PTR | Blizzard_DeclensionFrame/Mainline/DeclensionFrameGlue.lua:12 | Declension flow |
| C_Login.GetState | Preserve modeled connection fields; waitingForRealmList, superDistrictChoicePending, keepingCharacterListOnRealmJoin, waitingForRuleSets false; hasSelectedExperiencePreset true | PTR | Blizzard_GlueParent/Mainline/GlueParent.lua:222–254 | Login policy |
| IsNameReservationOnly | false | PTR | Blizzard_GlueXML/Mainline/CharacterSelect.lua:1019 | Name-reservation realm policy |
| C_CharacterServices.HasQueuedUpgrade | false | Retail, PTR | Blizzard_GlueXML/Mainline/CharacterSelect.lua:337 (Retail), :342 (PTR) | Queued character upgrades |
| C_CharacterCreation.SaveSeenChoices | Record offered choice IDs as seen; user-approved, not native evidence | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:744 (Retail), Mainline/Blizzard_CharacterCreate.lua:787 (PTR) | Character customization seen-choice persistence |
| C_CharacterCreation.ClearPreviewChoices | Accept clearSavedChoices; no effect or returns | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:727 (Retail), Mainline/Blizzard_CharacterCreate.lua:770 (PTR) | Character customization preview and saved choices |
| C_CharacterCreation.AreRegionalUniqueNamesEnabled | false | PTR | Blizzard_CharacterCreate/Mainline/Blizzard_CharacterCreate.lua:2079 | Regional naming policy |
