# Pet assist mode

Retail and PTR `C_PetInfo.PetAssistMode()` returns zero values on the enabled path; Retail `PetInfoDocumentation.lua:76-78` declares the function. WM52 verifies the build's feature byte is enabled, and emulates the zero-value continuation.

A simulated summoned pet uses the existing pet-presence indicator. On this path, the command writes reaction cache 3, then dispatches `PET_BAR_UPDATE`. The original pet-action button sees the assist token active when that cache is 3 and control mask bit 4 is clear. No summoned pet leaves the cache unchanged. The simulator assumes its summoned pet is player-controlled; this is an INFERRED mapping, not a native identity proof.

The native prerequisite flags 2097152, 262144, 4194304 and 8388608, the linked-unit alternative and the additional linked-identity requirement for control mask 4 are not modeled. Their public meanings are unknown. Callback-induced effects, server acceptance, and server reaction-packet injection remain unmodeled.
