# PTR I4 first bounded runtime repair

## Scope

PTR 12.1.7.70493 publishes only two formerly missing endpoints in this packet. The initial packet preserved other profiles, including Forever's existing graveyard query. [Forever item-cooldown publication](forever-spell-item-cooldown.md) later reuses this bounded model under an explicit declared-profile gate. Profile selection uses `client_profile::ACTIVE`.

## Supplied-state contract

- `C_Spell.GetItemCooldown(itemID)` accepts a numeric item ID and returns one nullable `SpellCooldownInfo`. A missing `SimState.item_cooldowns` entry returns nil; a supplied record projects its exact start and duration. No equipment-slot, spell-ID or bag-slot alias is used.
- The record uses the existing simulator cooldown policy: enabled and unmodified playback (`isEnabled=true`, `modRate=1`), with `isActive` derived from nonzero start and duration. Optional category, recovery and GCD fields remain absent because the item model does not supply them. Each query returns a separate table.
- `CanPortGraveyard()` returns exactly one boolean from `player.can_port_graveyard`. State changes and independent environments must be visible without leaking. No death, map, capability or graveyard policy is inferred from other fields.

## Evidence and native-unknown limits

The authenticated PTR APIDocumentation declares `GetItemCooldown` at `SpellDocumentation.lua:139–153`, its record at `SpellSharedDocumentation.lua:19–34`, and the global at `PlayerScriptDocumentation.lua:87–93`. Original selected consumers are `Blizzard_CollectionTemplates.lua:37` and `GhostFrame.lua:4`.

The item documentation says "Returns nil if the item is not found or on cooldown". That sentence does not establish how supplied simulator cooldown records map to native eligibility. This packet's record projection and missing-record policy are an explicit shape-only simulator contract, not native cooldown correctness. Enabled/rate fields retain existing simulator policy, not measured native constants. Graveyard availability is an explicit supplied input; its seeded false value is not a native capability default. Secret argument handling and native restrictions remain unproved.

## Verification

`tests/ptr_i4_green1.rs` retains the lane's supplied-state tests for concrete records, changed/removed records, return shapes and environment isolation. PTR per-consumer tests authenticate the original source bytes and load the selected Game TOC closure. Collections must hide for an absent record, show the supplied cooldown, track changed times, and hide after removal. GhostFrame must track false/true/false supplied availability and explicit hiding. Each selector rejects constant/no-op producers through observable transitions.

A producer-only RED sibling keeps every landing test byte unchanged and removes only the two producers. Its per-consumer tests must fail at `Blizzard_CollectionTemplates.lua:37` and `GhostFrame.lua:4`; GREEN must pass those exact selectors.

The deliberately failing aggregate I4 discovery and preservation tests stay exclusively on `ptr-i4-runtime`, with all 173 required shapes unchanged. They are not part of this landing branch or its normal suite. The wider 167 missing shapes and three blocked consumers remain separate obligations; this packet does not claim aggregate I4 acceptance.

Stock PTR startup compares master against this repair with identical pinned sources and guarded CVar copies. Every added error key requires an original emitter line and the predecessor failure that masked it. PTR/Retail all-target checks and formatting are separate acceptance evidence.

## Accepted bounded evidence

On 2026-10-25, GREEN `3eb4ae215` and producer-only RED `719a15bbb` include master `7ed5a81f0`, including PTR-input. Each of the four selectors has an exact one-test list and a single execution per sibling: RED fails, GREEN passes. The original consumers fail RED at the two Blizzard call sites specified above. Tests and all non-producer source bytes match between siblings.

Stock comparison uses the PTR-input merged baseline `120379c39`: 298→296 full raw message keys, 412→410 occurrences, zero additions, and unchanged common-key counts. The removed keys are GhostFrame OnLoad's missing graveyard call and its error wrapper. The older 428-key baseline predates PTR-input and is not final acceptance evidence. Remaining Mists-only pet-display changes on master do not publish or read that model on PTR.

Retail and PTR all-target checks pass with zero and exactly three inherited late-bind dead-code warnings respectively; both Cargo JSON and plain stderr were inspected. Formatting passes. Documentation-only descendants do not change this code/test proof scope. Native eligibility, capability policy, restrictions, and wider I4 acceptance remain unknown.

## Related documentation

- [I4 source investigation](../wiki/investigations/patch-12-1-7-api-audit.md)
- [Existing item cooldown model](item-cooldowns.md)
- [Existing graveyard query](forever-graveyard-port.md)
