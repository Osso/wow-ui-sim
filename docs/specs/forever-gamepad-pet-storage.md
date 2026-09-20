# Forever Gamepad pet storage

Forever `C_GamepadUI.GetFirstGamepadPetActionStorageSlotIndex()` exposes the simulator's first logical pet slot, `1`. This is an explicitly authorized simulator policy; equivalence to native storage numbering is unverified.

The eight Gamepad possess buttons address existing `pet_actions` slots 1–8 through pet queries, cooldowns, pickup and casting. They do not address generic player actions. Player action slot 1 remains independent of pet slot 1. No extra storage namespace or copied pet state is introduced.

The API is published only for Forever. Legacy pet APIs retain their existing one-based indexing and ten-slot capacity.

Evidence: matching vendor `ActionBarButton.lua` uses `GetPetActionInfo(self:GetID())` in `GamepadActionBarPetButtonMixin:HasAction`, pet pickup/casting in click handlers, and pet drag handling. `PossessBar.lua` assigns eight consecutive IDs and queries `GetPetActionCooldown`.
