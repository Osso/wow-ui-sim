# Journal instance by game map, NPEv2 eligibility

## Requirements

- `C_EncounterJournal.GetInstanceForGameMap(mapID)` returns the journal instance ID whose
  `JournalInstance.MapID` equals `mapID`, or `nil` when none matches.
  Consumer: `AdventureGuideUtil.GetCurrentJournalInstance` (Encounter Journal open) and
  `Blizzard_DelvesDifficultyPicker`.
- `C_PlayerInfo.IsPlayerEligibleForNPEv2()` returns `(isEligible, failureReason)` from the same
  `player.is_npe_eligible` state as `IsPlayerEligibleForNPE()`: `(true, "")` when eligible,
  `(false, "level")` otherwise. Consumer: `Blizzard_NewPlayerExperience/Blizzard_Tutorial.lua`.
- Both are registered on every profile whose generated docs declare them.

## Best-guess basis (best-guess, native UNKNOWN)

- Several journal instances share one game map (Dire Maul's three wings on map 429, Stratholme's
  two gates on 329). The lowest journal instance ID is returned.
- Journal rows with `MapID` 0 have no game map and never match.
- The simulator models one new-player gate, so v1 and v2 eligibility share it.

## Known gaps

- Native selection among instances that share a map (it may depend on the player's area or
  difficulty).
- Native v2 failure-reason strings.

Tests: `tests/c_encounter_journal_probes.rs`, `tests/c_player_info_probes.rs`.
