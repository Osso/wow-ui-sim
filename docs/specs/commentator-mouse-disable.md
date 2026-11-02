# Commentator mouse disable

Retail 12.1.5 declares `C_Commentator.SetMouseDisabled(disabled)` with one required boolean and no documented return values in `Blizzard_APIDocumentationGenerated/CommentatorFrameDocumentation.lua:1426–1433`. The original consumer calls it from `Blizzard_Commentator/Mainline/Blizzard_Commentator.lua:269–272`.

The setter accepts a readable non-nil value using Lua truthiness, stores the shared mouse-disable flag, and returns zero values. It does not require commentator mode and does not block input. When modeled mouse availability changes, it updates the same cursor visibility state as `ShowCursor` and `HideCursor`.

**EMULATED:** Availability is `not disabled`, unless a supplied override keeps it available. The stock override is false. Best-guess basis: Retail 12.1.5 build 70688 shows a shared disable flag, a higher-priority availability override, and a cursor refresh on availability changes. **Known gaps:** the complete native availability truth table, actual platform cursor transitions, mouse-look, input effects and persistence are not established.
