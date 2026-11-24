# Admin API (A_Admin)

The `A_Admin` namespace provides administrative functions for controlling simulator state from Lua. These functions are unique to the simulator and do not exist in the real WoW client.

**Implementation:** `src/lua_api/globals/admin_api.rs`, `admin_api_world.rs`, `admin_combat.rs`, `admin_encounter.rs`

## Sections

- [Player Identity](player-identity.md) — SetPlayerName, SetPlayerClass, SetPlayerRace, SetPlayerLevel, SetPlayerSex
- [Battle.net Account State](battle-net.md) — SetPlayerBattleNetInfo, SetBattleNetAccountFeatures
- [Behavioral messaging](behavioral-messaging.md) — QueueBehavioralNotification, GetBehavioralNotificationReceipts
- [Beta script policy](beta-script-policy.md) — SetScriptsDisallowedForBeta
- [Combat](combat.md) — SetInCombat, SetCasting, StopCasting, SetGCD, SetSpellCooldown
- [Camera and tutorials](camera-tutorials.md) — SetCameraFOV, SetTutorialFlag, SetTutorialsEnabled, SetPlayerNPERestricted
- [Loot Journal](loot-journal.md) — SetLootJournalItemSets
- [Raid locks](raid-locks.md) — SetCompletedEncounters
- [Kiosk session](kiosk.md) — SetKioskSession
- [Health & Power](health-power.md) — SetPlayerHealth, SetPlayerPower, SetTargetHealth
- [Targeting](targeting.md) — SetTarget, ClearTarget, SetFocus, ClearFocus
- [Party](party.md) — SetPartySize, SetPartyMember, SetPartyMemberHealth, KillPartyMember, ResPartyMember, SetRotDamage
- [Movement](movement.md) — SetMoving, SetMounted, SetFlying, SetFalling, SetSwimming
- [Navigation target](navigation.md) — SetNavigationTarget
- [Talents](talents.md) — SetSpec, SetTalentRank, SetTalentSelection, ResetTalents
- [Profession specializations](profession-specs.md) — SetProfessionSpecSkillLines, SetProfessionSpecTabInfo
- Wrath spellbook: `A_Admin.SetWrathSpellBookSlot(book, slot, spellID, name, rank, icon[, eligibleTradeSkillLine, shapeshiftForm])` supplies a slot record. The optional trade-skill line represents an eligible ability with no trade-skill link; the optional form represents its first aura-36 effect. `A_Admin.SetWrathSpellBookSelection(tradeSkillLine, shapeshiftForm)` supplies current state; nil clears each input. `pet` selects the pet book, case-insensitively; other book strings select the player book. Slots are 1–1024. These setters do not alter known-spell membership or emit events. Empty rank is a scenario default, not a native claim.
- [Buffs](buffs.md) — AddBuff, RemoveBuff, ClearBuffs
- [Loss of Control](../specs/active-loss-of-control.md) — AddLossOfControl, ClearLossOfControl
- [Zone](zone.md) — SetZone, SetSubZone, SetInstanceInfo, SetInInstance
- [Housing](housing.md) — SetPlayerOwnedHouses
- [Equipment locks](equipment.md) — SetEquipmentItemLock, ClearEquipmentItemLock
- [Transmog outfits](transmog-outfits.md) — SetUnlockedOutfitSlots
- [Economy](economy.md) — SetMoney, SetItemLevel, AddBagItem, RemoveBagItem
- [Collections](collections.md) — AddTransmog, RemoveTransmog, CollectMount, UncollectMount, CollectPet, UncollectPet, CollectToy, UncollectToy, CollectCampsite, UncollectCampsite, SetAchievementEarned, EarnAchievement
- [PvP](pvp.md) — SetPvPEnabled, SetHonorLevel
- [Guild](guild.md) — SetGuildInfo, ClearGuild, JoinGuild, LeaveGuild
- [Mail](mail.md) — AddMail, ClearInbox, SetInboxCount
- [Interactions](interactions.md) — OpenGuildRename, OpenMailbox, CloseMailbox
- [Premade Groups](premade-groups.md) — AddPremadeListing, ClearPremadeListings, UpdatePremadeListing
- [Quests](quests.md) — simulator-only selected completed quest turn-in
- [Events](events.md) — FireEvent
- [Scripted animations](scripted-animations.md) — SetScriptedAnimationEffects
- [Performance metrics](performance-metrics.md) — SetFramerate, SetAddOnUsage
- [Examples](examples.md) — Usage examples (raid healing, combat testing, transmog)
