# Macro test inputs

The simulator-only `A_Admin.SetMacroRows(json)` setter replaces the shared `MacroInfo` rows. Each row has `name`, `icon`, and `body` string fields. The setter supports empty rows to model vacant slots. The data drives legacy macro getters and the `C_Macro` getters on Retail and WowForever.

```lua
A_Admin.SetMacroRows([[{"name":"RaidPlan","icon":"134400","body":"/say ready"}]])
```

`A_Admin.SetMacroIconCatalog(json)` replaces the four icon sources used by the legacy icon getters. The JSON object accepts `looseSpells`, `looseItems`, `spells`, and `items`; loose entries are strings or `null`, while catalog entries are signed integers. Missing fields are empty.

```lua
A_Admin.SetMacroIconCatalog([[{"looseSpells":["Heal"],"looseItems":[],"spells":[134331],"items":[]} ]])
```

These setters supply simulator test state. They do not model macro persistence or native macro creation.
