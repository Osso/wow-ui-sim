# Mists Glue intro movie blocks AccountLogin

**VERIFIED (Mists Classic 5.5.4.70268 cached Lua):** the Glue startup checks `playIntroMovie` and can show a full-screen MovieFrame over AccountLogin; MovieFrame calls cursor globals before it tries playback. This investigation separates that cache behavior from verified 5.5.4.70631 native behavior, inferred transfer between the builds, and the unknown offline movie result.

## Content

### Selected consumers

**VERIFIED (Mists 5.5.4.70268 cache):** the GlueParent Classic TOC selects `Classic/GlueParent.lua` for Glue/classic. The GlueXML Mists TOC selects `Classic/AccountLogin.lua` and `MovieFrame.lua` for Glue/mists. The selected sources connect `FRAMES_LOADED` to `GlueParent_CheckCinematic`, mark the movie secondary screen full-screen, and hide the current primary screen while that screen is open.

### Native and cache boundaries

**VERIFIED (Mists Classic 5.5.4.70631, R88ao):** `HideCursor` and `ShowCursor` are no-argument, zero-return Glue globals. They update cached cursor state. `playIntroMovie` defaults to `0`, and `GetClientDisplayExpansionLevel()` returns `4`.

**INFERRED (5.5.4.70631 behavior applied to the 5.5.4.70268 cache):** the first cinematic check sets `playIntroMovie` to `5` and opens the full-screen movie screen. **VERIFIED (5.5.4.70268 cached Constants.lua):** movie sequence 5 maps to ID 115.

**VERIFIED (Mists Classic 5.5.4.70631, R88ao):** `StartMovie` returns `(success, code)`; code 0 is success, codes 1–4 identify the documented failure paths, and no-record search returns code 2. An error return does not itself dispatch `OnMovieFinished`. `StopMovie` queues `OnMovieFinished` for an active handle; natural end uses the same stop path. `CinematicFinished` records completion state but does not close MovieFrame.

**UNKNOWN (offline playback):** the result for a movie ID depends on available records, files, and playback state. The simulator must keep that result as an empty-default host input; it must not claim that an offline client returns a particular code.

### Simulator evidence

**VERIFIED (Mists integration RED at source SHA `537f369a949468051115b7c68bed02704f93eb88`):** the selected original `GlueParent_CheckCinematic` opened MovieFrame, whose `OnShow` failed at `MovieFrame.lua:37` because `HideCursor` was nil. The original login-boot assertion found AccountLogin hidden, and the original edit-box hit test found no target under the full-screen frame.

**IMPLEMENTED, UNVERIFIED (source SHA `b265305c2`):** Mists cursor visibility globals now follow Glue/Game screen registration. The post-change build and test are pending.

## Sources

- R88ao in the local decomp-answer handoff — native Mists 5.5.4.70631 behavior.
- Mists 5.5.4.70268 cached TOCs and Lua files: `Blizzard_GlueParent_Classic.toc`, `Blizzard_GlueXML_Mists.toc`, `GlueParent.lua`, `AccountLogin.lua`, `MovieFrame.lua`, and `Constants.lua`.
- [Glue login contract](../../specs/glue-login-flow.md) — simulator API and test scope.
- [Mists login tests](../../tests/glue_login.rs) — original-consumer RED/GREEN coverage.

## See Also

- [[glue-login-lifecycle]] — related Retail Glue lifecycle; not evidence for Mists native behavior.
- [MovieFrame subtitle contract](../../specs/movie-subtitles.md) — existing MovieFrame method model and its playback boundary.
