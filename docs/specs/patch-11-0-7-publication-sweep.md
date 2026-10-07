# Patch 11.0.7 publication sweep

## Contract

Probe all 98 inventory occurrences from Warcraft Wiki page 609319, revision 6726777 against unmodified cached Game UI. Default retail carries 12.1.0, not historical 11.0.7. Apply eleven later registers, 11.1.0 through 12.1.0, chronologically. Latest add/remove wins; changed rows preserve publication. Retain original direction and supersession IDs. Require the exact reviewed gap-ID set; persist every observation before checking the fixture. P1107_SWEEP_OUT selects results; P1107_SWEEP_REGISTER selects a same-sized negative-control register.

Publication/absence only; no signature, output, security, behavior or native parity claim. Generic namespace autostubs are not explicit publication. Cached Blizzard deprecation wrappers remain unchanged. Before simulator retirements, search all current cached retail Lua and retain active consumers with file:line evidence.

## Bounded behavior

Four removed namespace members stay absent after repeated ordinary/raw lookup: the three WorldLootObject callouts in C_ArrowCalloutManager and C_WorldLootObject.GetCurrentWorldLootObjectSwapInventoryType. Full cached retail Lua searches find zero consumers; existing AcknowledgeCallout stays callable. Cached LFG deprecation wrappers remain published and untouched.

RemoveRaidTargets clears the existing GUID-keyed marker map, returns no values, then emits RAID_TARGET_UPDATE without arguments. The behavioral fixture marks player, two party members and a hostile target, observes all markers cleared inside the callback, repeats on empty state, and reassigns a marker. Event timing follows the existing simulator SetRaidTarget policy, not native timing proof. Cached RaidMarkersDocumentation.lua:89–92 documents no arguments/results and removal of all markers; protected-action authorization remains unmodeled. Publication is enabled for supported 12.x retail epochs, not classic profiles.

## Acceptance

- [ ] Exact reviewed gap fixture and exhaustive inventory/non-inventory row ledger.
- [ ] Bounded modeled gap fixes with behavioral RED/GREEN evidence where practical.
- [ ] Twelve isolated sweeps, negative control and relevant isolated prefork tests.
- [ ] Formatting, Mists test check with zero non-vendor warnings, retail build and startup [].
