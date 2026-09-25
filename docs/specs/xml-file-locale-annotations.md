# XML file locale annotations

XML file references in addon XML must honor the same text-locale selection as TOC file entries. See [addon loading](../wiki/systems/addon-loading.md) for the loading path.

## What it must do

- [x] Strip trailing file annotations before resolving selected `<Script file>` and `<Include file>` paths.
- [x] Under the simulator's enUS text-locale policy, execute selected enUS script and included XML, and skip non-enUS references even when their files exist.
- [x] Preserve the declared order of executed XML file references.

## How it works

- [Addon loading](../wiki/systems/addon-loading.md)
- [Loader pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/toc/mod.rs` — shared annotation stripping and existing enUS selection.
- `src/loader/xml_file.rs` — XML Script and Include file-reference loading.

## Tests asserting this spec

- `src/loader/tests/xml_basics_extra.rs::xml_file_locale_annotations_load_only_selected_scripts_and_includes` — temporary addon with selected and unselected Lua/XML files.

## Known gaps (current cycle)

- [ ] Client-selected non-enUS text locale is not modeled; enUS remains the simulator's existing fixed policy.

## Out of scope

Other XML annotations, locale switching, and changes to addon files or filesystem fallback behavior.
