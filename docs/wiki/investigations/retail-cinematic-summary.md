# Retail cinematic summary lifecycle

Retail 12.1.5 exposes a current cinematic ID and the original MovieFrame close dialog consumes its summary. Other profiles retain their existing behavior.

## Evidence and boundaries

**VERIFIED (Retail 12.1.5.70688, `pr46-49-decomp-answers.md`, R88ap part A §1):** the ID begins at zero; `CinematicStarted` sets it for types 0–3, and `CinematicFinished` clears it. `GetCurrentCinematicSummary` returns exactly one value: nil for ID zero, missing Movie row, absent summary reference, or empty text, otherwise the summary string. Empty string and zero return values are not native results.

**INFERRED (simulator data source):** the empty-default `cinematic_summaries` map represents Movie rows and summary references. Missing key represents no row, `None` represents no summary reference, and `Some(text)` supplies text. These are host inputs, not fabricated DB2 contents. This mapping is isolated in its own commit.

**INFERRED (simulator admission):** a separate input adapter accepts numeric types and nonnegative representable integer movie IDs; malformed inputs are neutral. **UNKNOWN:** native invalid-argument validation, other start/finish effects, and emitted Lua events. Tests make no native claims about malformed inputs; no events or other effects are invented.

## Original consumer

**VERIFIED (selected Retail 12.1.5.70783 source):** `Blizzard_FrameXML.toc` selects `MovieFrame.lua` and XML without a file-level game-type filter. The game-screen dependency-closure fixture loads this original addon and checks `MovieFrame.ShowCloseDialog` source provenance. The real close dialog sets/shows its summary label for a truthy result and hides it for nil.

The regression covers two distinct texts, zero ID despite a supplied row, absent row, empty text, absent summary reference, and clearing after finish. The separate contract test covers initial zero, types 0–3, rejection of type 4, and return arity. A test-only callable-wrong summary overlay exercises identical original-consumer assertions without modifying vendor source. Runtime receipts remain outside the repository.

## Sources

- [Producer](../../../src/lua_api/globals/real/cinematic_summary.rs)
- [Regression tests](../../../tests/cinematic_summary.rs)
- `pr46-49-decomp-answers.md`, R88ap part A §1 (untracked native handoff, summarized in tracker lt-5d82)

## See also

- [[consumer-selection-and-red-proof]]
