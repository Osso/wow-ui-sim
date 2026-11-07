# Supplied challenge run and manual clock

The challenge subsystem owns an optional active run, a supplied map catalogue, ordered criteria per step and explicit world elapsed timers. It does not derive dungeon metadata, penalties or weighted display text. The separate manual environment clock supports long forecast fixtures without changing real-time defaults.

## State and dispatch

`c_api::challenge_run` stores Rust snapshots on each SimState. Admin calls validate input before publishing it, then dispatch the declared events. Criteria and affix readers allocate detached Lua results. The existing scenario step model supplies step metadata and row counts. Run reset/completion removes the active run, criteria and elapsed timers before notifications. Map records remain available.

Publication follows pinned declarations: Retail/PTR/Forever have active map, key, deaths and active flag; Mists/Era/Anniversary have map and active flag. Wrath receives no new members or elapsed-backend routing. Its old `(id, 0, 0)` elapsed placeholder is preserved as an ungrounded gap for a separate lane. The key level, four affix slots and charged byte are stored apart from the active run: the run-start input applies them, `ResetActiveKeystoneInfo` zeroes them or skips them, and run clear/completion leaves them intact ([spec](../../specs/challenge-mode-run.md#keystone-globals)). Mists original criteria rendering is still blocked by the existing legacy C_Scenario info/step constants; this subsystem does not replace them.

## Clock boundary

`SimState::clock_elapsed` and `clock_now` route gameplay timestamps and timer deadlines. A manual clock stores both coordinates, so choosing a new GetTime epoch cannot expire pending timers. Host and admin steps share LuaState-level OnUpdate, animation and timer dispatch. This avoids reborrowing the active Lua VM cell. Timer dispatch restores the previous executing owner, including nested callbacks. The manual queue selects the earliest deadline and ID after each callback; repeat deadlines advance from their previous value. Zero-interval tickers run once per advance.

Duration-object manual clocks, explicit world elapsed timers and the delta-driven encounter timeline retain their own state. Real startup/render scheduling, watchdogs and performance diagnostics do not use manual time. Autonomous party damage still samples real SystemTime randomness; only its death/resurrection timestamp follows the environment clock.

## Evidence status

Verified 2026-10-26. Own executable feature RED precedes the producer commits. Byte-identical producer-subtracted tests pass original challenge timer transitions on Retail/PTR/Forever and original dungeon-icon SetUp on Retail/PTR. The icon proof is a standalone original template/method, not a complete ChallengesUI-panel claim; the unrelated missing-PVEFrame XML error remains recorded. Direct criteria snapshot tests pass on all six declared profiles and reject the removed query producer on the three modern profiles. They are query proof, not original-rendering proof.

All 28 manual-clock cases pass across seven profiles; the four selectors independently reject removed clock activation on Retail/PTR/Forever. Six existing timer/OnUpdate cases pass, including original AsyncRequest timeout/cancellation and callback provenance. All seven refreshed all-target checks pass with zero Cargo JSON/plain compiler warnings. The final clock-site audit classifies all 184 remaining Instant/origin sites; profiling, host scheduling and test deadlines stay real.

Raw default stock maps are exactly unchanged against master `17baa3e9a`: Retail 8/10, PTR 134/154, Forever 26/26, Mists 6/12, Era 0/0 and Anniversary 0/0 (keys/occurrences). No added, removed or count-changed key needs predecessor attribution. Every runtime has an unchanged original CVar stat/hash guard. No full-suite or globally clean-runtime claim is made.

The unchanged InTimeBL runner passes both discovery modes: 10 unit, 4 generator and 4 simulator assertions, exit 0. Its forecast fixture passes ahead→overtime→reset assertions in both modes: progress 22.5% at time 301 with +7:06, then 73.0% at time 2402 with -16:22. Forecast lua-errors commands still exit 1: exclusive mode has the stock errors plus two 480-count forms of the original missing C_VoiceChat.CanAccessSettings error (SettingsDefinitions_Shared/Audio.lua:565, five-second ticker at 577); default mode also has ambient addon errors. These errors are not hidden or repaired with a guessed voice-policy value.

### Blocked original criteria initialization

The criteria fixture needs original `ScenarioObjectiveTracker:InitModule` before `UpdateCriteria`. Its first failure was an uninitialized parentModule. Original InitModule then fails at line 74 on unguarded `C_Scenario.ShouldShowCriteria`, absent from all 622 Retail and 647 PTR generated documentation files, with no textual Lua definition found in either pinned addon tree. Original SetContainer invokes InitModule at 78; container AddModule reaches it at Retail 123/PTR 141. Original criteria rendering remains BLOCKED. No exception swallowing or vendor replacement is permitted. Native clarification and any missing producer belong to a separate lane. Mists remains behind its separately recorded legacy GetInfo/GetStepInfo defaults.

## Legacy probe fixture migration

`startup_api_stubs::startup_lfg_world_timer_and_bn_surfaces_return_empty_safe_shapes` no longer decodes an invented `(7, 0, 0)` elapsed tuple on six profiles. It requires zero results for an absent supplied timer, retains the old tuple explicitly on excluded Wrath, and retains every unrelated LFG/BN assertion. `c_mythic_plus_probes::challenge_mode_map_table_seeds_mists_challenge_map` now supplies its expected map through admin instead of assuming a seeded native catalogue. Its four old assertions remain; added assertions cover the supplied map ID, time limit and texture fields. These are fixture/contract corrections, not remaining failures or a native catalogue claim.

## Sources

- [Challenge contract](../../specs/challenge-mode-run.md) — pinned declarations, publication and selected consumers.
- [Clock contract](../../specs/manual-simulation-clock.md) — opt-in, validation and callback order.
- [Admin guide](../../admin-api/challenge-mode.md) — fixture inputs and examples.

## See Also

- [[lua-api]] — environment state and public API registration.
- [[consumer-selection-and-red-proof]] — runtime-selected consumer and paired proof requirements.
