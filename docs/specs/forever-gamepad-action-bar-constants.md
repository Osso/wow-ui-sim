# Forever gamepad action bar constants

## Contract

Forever 1.60.1.69913 publishes the eleven numeric members of
`Constants.GamepadActionBarConstants` documented by its authenticated
`Blizzard_APIDocumentationGenerated/GamepadUIDocumentation.lua`.
Other profiles do not gain this table.

The values describe four slots per group, two groups per bar, eight slots per
bar, four pageable bars, 32 slots per standard page, four reserved slots,
four pages (three standard), special page index four, 96 pageable slots,
and five stance bars.

## Proof

`tests/wowforever_gamepad_constants.rs` executes the unmodified vendor
`ActionBar.lua` initialization against eight real Button children and checks
ordering, indices, parent references, grid attributes, visibility and icon sizes.
A separate assertion checks raw table publication by profile.

## Limits

Constant publication does not implement hardware input, paging, stance changes,
or prove complete gamepad action bar startup or native conformance.
