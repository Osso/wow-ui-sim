# Forever War Mode desired

Forever `Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua:1134` declares `C_PvP.IsWarModeDesired` with one non-null boolean result. `Blizzard_MicroMenu/Mainline/MainMenuBarMicroButtons.lua:811` uses that result to choose whether to query battlefield state.

The query returns the simulator's supplied War Mode desired state. Its initial false value is existing simulator scenario policy; the native Forever default is unknown. This does not publish the War Mode reward query on Forever.

Forever `Blizzard_APIDocumentationGenerated/SpecializationInfoDocumentation.lua:11` declares `CanPlayerUsePVPTalentUI` as a boolean and a non-null string, and line 152 declares `GetPvpTalentAlertStatus` as two booleans. The micro-menu calls these at lines 812 and 818; `Blizzard_FrameXML/PVPHonorSystem.lua:195` also calls the availability query. Retail declares the same return shapes at lines 11 and 128. The simulator returns independently supplied availability, reason, unspent-slot and new-talent state. Initial false booleans and empty reason are scenario defaults; native Forever defaults are unknown.

`tests/forever_war_mode_desired.rs` exercises the selected micro-menu consumer with both desired values after a specialization event.
