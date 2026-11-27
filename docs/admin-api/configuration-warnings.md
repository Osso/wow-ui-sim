# Configuration warnings

### `A_Admin.SetConfigurationWarnings(records)`

Supplies the configuration-warning records read by Retail and WoW Forever `C_ConfigurationWarnings` queries and by the original Glue character-select consumer.

- `records` must be a dense array of tables with `warning` (non-negative integer), `text` (string), and optional `seen` (boolean).
- Omitted `seen` is `false`. An empty array is the default state.
- `C_ConfigurationWarnings.GetConfigurationWarnings(includeSeenWarnings)` returns IDs in supplied order. The default `includeSeenWarnings` value is `false`.
- `GetConfigurationWarningString(id)` returns the supplied text or nil. `GetConfigurationWarningSeen(id)` reads the supplied seen value. `SetConfigurationWarningSeen(id)` marks a matching record as seen.

### `A_Admin.SetSystemRequirementsChecked(checked)`

Sets the session state read by the Glue consumer's `HasCheckedSystemRequirements()` global. `checked` must be a boolean. The default is `false`.

The configuration-warning record shape and filtering follow the pinned `Blizzard_APIDocumentationGenerated/ConfigurationWarningsDocumentation.lua`. The checked-state global is INFERRED (R88g9, native declaration not found); the selected Retail consumer reads it in `Blizzard_GlueXML/Mainline/CharacterSelect.lua` around line 711. No warning records are supplied by default.
