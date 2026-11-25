# Chat windows

`A_Admin.SetChatWindowState(index, name, shown, docked)` supplies one chat-window record for Retail and WoW Forever. It writes simulator state; it does not emulate account SavedVariables.

The default record has no name. Windows 1 (General) and 2 (Combat Log) start shown and docked; later windows start hidden and undocked. This default is **INFERRED R88g4 (native evidence pending)** from `FCF_ResetChatWindows`, which resets ChatFrame1 and ChatFrame2 and docks both frames.

The chat-window APIs store names, visibility, docking, and saved dimensions and position in the same Rust state used by the other `SetChatWindow*` verbs. The simulator does not import a chat-cache SavedVariables record.

```lua
A_Admin.SetChatWindowState(1, "General", true, true)
A_Admin.SetChatWindowState(2, "Combat Log", true, true)
```
