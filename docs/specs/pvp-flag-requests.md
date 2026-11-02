# PvP flag requests

Retail `C_PvP.SetPVP([enablePVP])` and `C_PvP.TogglePVP()` return zero values; Retail `PvpInfoDocumentation.lua:1205-1214,1260-1264` declares the functions. WM52 verifies reason-0 source refusal before argument parsing. WM59 verifies the zero-value tails and the transport gate.

A supplied transport state of exactly 5 records a simulator-observable submitted request; no transport or any other state discards it without retry. `TogglePVP` records a toggle command without deriving an enable value. `SetPVP` records the supplied Lua-truthy value; omission and nil select false. This argument-to-request connection is INFERRED by WM35/WM59. Neither changes the modeled PvP desired flag. Transport holding, server delivery and acceptance remain unmodeled and unknown.
