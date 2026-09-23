# XML frame creation errors

XML frame creation reports failed nested-template initialization through the addon's normal load diagnostics. `src/loader/xml_frame/setup.rs` is responsible for preserving the original `CreateFrame` error even when a parentKey or parentArray frame was partially created.

## What it must do

- [x] A failed nested template creation is reported in addon load warnings with its original error, even when the partially created frame has parentKey and parentArray links.
- [x] A partially created frame does not turn an unsuccessful XML load into healthy loading.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/loader/xml_frame/setup.rs`: executes XML frame creation and propagates failures.
- `src/loader/xml_file.rs`: passes XML failures to addon loading.
- `src/loader/addon.rs`: exposes XML load failures as warnings.

## Tests asserting this spec

- `tests/xml_create_error_reporting.rs`: loads a real temporary addon whose nested template OnLoad fails under parentKey and parentArray; inspects public load diagnostics.

## Known gaps (current cycle)

- [ ] Independently verify the unchanged Datamine archive exposes its original nested texture creation error instead of a false-clean startup.

## Out of scope

- Repairing the failed nested template, rolling back partially created widgets, modifying addon/vendor files, or treating this diagnostics fix as a successful Datamine interaction.
