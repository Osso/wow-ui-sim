# Mists Glue intro movie blocks AccountLogin

**VERIFIED (Mists Classic 5.5.4.70268 cached Lua):** the Glue startup checks `playIntroMovie` and can show a full-screen MovieFrame over AccountLogin; MovieFrame calls cursor globals before it tries playback. This investigation separates that cache behavior from verified 5.5.4.70631 native behavior, inferred transfer between the builds, and the unknown offline movie result.

## Content

### Selected consumers

**VERIFIED (Mists 5.5.4.70268 cache):** the GlueParent Classic TOC selects `Classic/GlueParent.lua` for Glue/classic. The GlueXML Mists TOC selects `Classic/AccountLogin.lua` and `MovieFrame.lua` for Glue/mists. The selected sources connect `FRAMES_LOADED` to `GlueParent_CheckCinematic`, mark the movie secondary screen full-screen, and hide the current primary screen while that screen is open.

### Native and cache boundaries

**VERIFIED (Mists Classic 5.5.4.70631, R88ao):** `HideCursor` and `ShowCursor` are no-argument, zero-return Glue globals. They update cached cursor state. `playIntroMovie` defaults to `0`, and `GetClientDisplayExpansionLevel()` returns `4`.

**INFERRED (5.5.4.70631 behavior applied to the 5.5.4.70268 cache):** the first cinematic check sets `playIntroMovie` to `5` and opens the full-screen movie screen. **VERIFIED (5.5.4.70268 cached Constants.lua):** movie sequence 5 maps to ID 115.

**VERIFIED (Mists Classic 5.5.4.70631, R88ao):** `StartMovie` returns `(success, code)`; code 0 is success, codes 1–4 identify the documented failure paths, and no-record search returns code 2. An error return does not itself dispatch `OnMovieFinished`. `StopMovie` queues `OnMovieFinished` for an active handle; natural end uses the same stop path. `CinematicFinished` records completion state but does not close MovieFrame.

**INFERRED (simulator mapping):** the simulator has no Movie DB2 records, so an absent `start_results` entry maps to the native no-accepted-record result `(false, 2)` (`Missing Rec`). Explicit host outcomes remain available for success and codes 1–4. This does not claim that an offline client with available records or files always returns code 2.

**UNKNOWN (native offline playback):** outcomes for available records and files depend on the records, files, and playback state.

### Simulator evidence

**VERIFIED (Mists integration RED at source SHA `537f369a949468051115b7c68bed02704f93eb88`):** the selected original `GlueParent_CheckCinematic` opened MovieFrame, whose `OnShow` failed at `MovieFrame.lua:37` because `HideCursor` was nil. The original login-boot assertion found AccountLogin hidden, and the original edit-box hit test found no target under the full-screen frame.

**Historical note (source SHA `b265305c2`):** the cursor visibility globals were added for the original Mists consumer. Later build and test evidence appears below.

## Sources

- R88ao in the local decomp-answer handoff — native Mists 5.5.4.70631 behavior.
- Mists 5.5.4.70268 cached TOCs and Lua files: `Blizzard_GlueParent_Classic.toc`, `Blizzard_GlueXML_Mists.toc`, `GlueParent.lua`, `AccountLogin.lua`, `MovieFrame.lua`, and `Constants.lua`.
- [Glue login contract](../../specs/glue-login-flow.md) — simulator API and test scope.
- [Mists login tests](../../tests/glue_login.rs) — original-consumer RED/GREEN coverage.

## See Also

- [[glue-login-lifecycle]] — related Retail Glue lifecycle; not evidence for Mists native behavior.
- [MovieFrame subtitle contract](../../specs/movie-subtitles.md) — existing MovieFrame method model and its playback boundary.

## R88aq cinematic behavior

**VERIFIED (Mists Classic 5.5.4.70631, R88aq):** `CinematicStarted` requires numeric or numeric-string type and movie ID values, truncates toward zero, accepts type 0–3 and signed-int32 IDs, treats only Boolean false as `canCancel=false`, ignores extra arguments, and returns no values. ID 0 skips record updates. An active same-type record is finished before its replacement starts.

**VERIFIED (R88aq-b):** cinematic types are GlueMovie=0, GameMovie=1, GameClientScene=2, and GameCinematicSequence=3. `CinematicFinished` updates the record and shared state before its type-1 follow-up, then clears the manager movie ID. The follow-up requires a separate game-movie flag. **UNKNOWN:** the flag writer and observer/listener effects. The simulator keeps that flag unset and does not model unknown observer effects.

**VERIFIED (Mists 5.5.4.70268 cache):** `MovieFrame_PlayMovie` checks only the truth value of `playSuccess`; a false result closes the secondary screen. The failure path does not depend on the old `(nil, nil)` pair.

**INFERRED (R88ao, 5.5.4.70631 behavior applied to cached 5.5.4.70268):** the simulator's no-record pair `(false, 2)` follows that same close path. The cached close operation restores AccountLogin. A later `StopMovie` may dispatch `OnMovieFinished` for the installed handle, but the cached finish handler does not advance while MovieFrame is hidden.

## R88ar Mists Glue chat selection

**VERIFIED (5.5.4.70268 cached TOCs):** `Blizzard_ChatFrameBase_Classic.toc` declares `AllowLoad: game` and `AllowLoadGameType: classic`; `Blizzard_ChatFrame_Mists.toc` declares `AllowLoad: game` and `AllowLoadGameType: mists`. The Mists game-type tokens match the active profile, while the `AllowLoad` screen gate rejects both add-ons on Glue.

**VERIFIED (simulator source at `67d34fbc`):** `screen_allowed_toc` applies `TocFile::allows_screen` and the active game-type filter. **VERIFIED (Mists login runtime at that source):** `C_AddOns.IsAddOnLoaded` returned false for both chat add-ons, yet `ChatFrame1` existed and was shown with built-in ownership. The loader filter was already correct.

**VERIFIED (source and simulator runtime at `67d34fbc`):** `chat_window_defaults` created a synthetic `ChatFrame1` before startup applied the requested screen. **INFERRED (R88ar applied to the 70268 cache):** the original Mists Glue login does not receive this Game-only chat frame.

**VERIFIED (source at `e9ec1c669`):** bootstrap skips the synthetic placeholder only when the active profile is Mists and the requested screen is Glue. The change adds no hide call. Mists Game and all other profile paths still create the placeholder.

**VERIFIED (Mists integration build at `e9ec1c669`, zero warnings):** the original login boot test passes 1/1 with its existing `ChatFrame1`-not-shown assertion unchanged; an added assertion confirms `ChatFrame1` is absent and both Game-only chat add-ons are unloaded. The original edit-box test passes 1/1. The focused `glue_login::mists_` movie/API filter passes 16/16, including the Mists Game placeholder test.
