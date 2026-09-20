# Forever finite UI constants

## Contract

Forever 1.60.1.69913 publishes these additions from its generated API documentation, without changing other client profiles:

- `MinimapTrackingFilter.TrainerClass = 8388608`, `VendorAmmo = 16777216`; metadata: 26 values, range 0–16777216.
- `PingResult.FailedSilent = 8`; metadata: nine values, range 0–8.
- `Constants.Transmog.NoTransmogID = 0`.
- `GamepadPossessBarOverride`: SpecialPageTopBar=1; Page1LeftBar/RightBar/BottomBar=2/3/4; Page2TopBar/LeftBar/RightBar/BottomBar=5/6/7/8; Page3TopBar/LeftBar/RightBar/BottomBar=9/10/11/12. Metadata: twelve values, range 1–12.

Sources: authenticated Forever `MinimapConstantsDocumentation.lua`, `PingConstantsDocumentation.lua`, `TransmogConstantsDocumentation.lua`, and `GamepadUIDocumentation.lua` under `Blizzard_APIDocumentationGenerated`.

## Verification

`tests/wowforever_finite_constants.rs` checks publication and actual Camelot minimap filter construction. These additions do not establish complete Transmog initialization or gamepad possession behavior; remaining consumer failures must be diagnosed independently.
