# Retail LFG reward writer

## Verified contract

Native R88n.4/.4a/.4b (Retail 12.1.5.70688), reproduced in lt-3146 comment 1213, proves an incoming decoded enclosing-message callback. The writer clears readiness, resets groups, creates/finds dungeon groups, resets shortage tiers, fills ordinary/shortage reward IDs, quantities and item/currency discriminators, replaces ordinary entries/count, sets readiness, then requests `LFG_LOCK_INFO_RECEIVED` before `LFG_UPDATE_RANDOM_INFO` for installed callbacks.

`DecodedLfgRewardMessage` is an explicit host-input envelope. It is not a wire decoder, public packet name, or a renamed `JamLFGLockListSlot` array element. Native internal value 5898267 is not claimed as a public wire opcode. All other cache-clear routes and transport acquisition remain UNKNOWN and are not implemented.

`c_api::lfg_reward_writer::apply_decoded_message` is admitted only by `client_profile::ACTIVE == Retail`; non-Retail calls leave state/events untouched. The host module is available independently of the `client-retail` convenience bundle, and the optional consumer test admits `profile-retail` with the required 12.1.5 getter epoch. It rebuilds the existing ready-gated state before dispatching events through normal registered Lua listeners. The existing represented dungeon key uses dungeon ID and type ID, matching current getter selection. Vec lengths supply authoritative entry counts. Supplied IDs/quantities/discriminators are retained without catalog-derived reward invention. Absent new groups and shortage tiers disappear; an empty update yields a ready empty cache and the same ordered notifications. Default cache remains empty/not ready until an explicit update.

## Original consumers

Pinned Retail `Blizzard_GroupFinder_Mainline.toc:5` lists dependencies, `:6–7` selects Game/mainline and `:12` selects original `Shared/LFGFrame.lua`. `LFGFrame.lua:948` defines the hover; `:961,963` calls the ordinary/shortage TooltipDataHandler accessors. The existing GroupFinder closure fixture verifies actual discovery/loading and original Lua source ownership.

Tests use two distinguishable updates and an empty update through the host writer, assert ready replacement state and ordered Lua events, and invoke original hover to prove tooltip content changes. Existing tooltip_lfg_rewards selectors remain preservation gates. No vendor functions are replaced.

## Sources

- Tracker lt-3146 comment 1213, R88n.4/.4a/.4b.
- `src/c_api/lfg_reward_writer.rs` and `src/lua_api/state_types/lfg_rewards.rs`.
- `tests/tooltip_lfg_rewards.rs` and original pinned Retail UI files above.

## Proof boundary

Coordinator comment 1266 approves a test-only, uncommitted missing-route RED adapter on the base with no writer. GREEN replaces only the adapter with the production import; fixture, input, listener and assertion body hashes are identical. This is a missing-writer proof, not evidence that an existing producer failed. Per-invocation compiler executable and actual loaded dylib hashes are checked before and after both RED and GREEN. Original GroupFinder closure diagnostics are retained; the test proves event/data/hover outcomes, not that every unrelated GroupFinder feature is error-free. Stock Retail startup is checked independently.
