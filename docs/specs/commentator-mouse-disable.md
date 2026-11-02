# Commentator mouse disable

Retail 12.1.5 declares `C_Commentator.SetMouseDisabled(disabled)` with one required boolean and no documented return values in `Blizzard_APIDocumentationGenerated/CommentatorFrameDocumentation.lua:1426–1433`. The original consumer calls it from `Blizzard_Commentator/Mainline/Blizzard_Commentator.lua:269–272`.

The setter accepts a readable non-nil value using Lua truthiness, stores the shared mouse-disable flag, and returns zero values. It does not require commentator mode and does not block input. When modeled mouse availability changes, it updates the same cursor visibility state as `ShowCursor` and `HideCursor`.

Availability is `F || (!disabled && S && (!H || R))`, using supplied force, source, inhibitor and inhibitor-cancel inputs. These inputs default to false as an empty simulator scenario, not a native default. Unchanged availability does not refresh the cursor. Clearing availability requests a hidden cursor; when availability becomes true, cursor visibility changes only if the host has supplied the otherwise unknown requested visibility. The local truth table is verified for Retail 12.1.5 build 70688. **Known gaps:** the predicates that determine requested visibility, platform cursor transitions, mouse-look, input effects and persistence are not established.
