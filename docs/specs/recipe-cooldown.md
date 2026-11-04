# Recipe cooldown

Retail 12.1.5 `C_TradeSkillUI.GetRecipeCooldown(recipeID)` returns no values for a recipe absent from the simulator recipe catalog. For a present recipe it returns four values: remaining seconds or nil, day-cooldown flag, available charges, and maximum charges. Supplied spell cooldown and charge inputs are keyed by recipe ID. A recovering charge replaces the ordinary cooldown result; full charges leave the ordinary result intact. The simulator does not advance charge state or emit cooldown events automatically.

Inputs accept numbers and unmarked numeric strings, truncate fractions toward zero, and reject nonnumeric or inaccessible secret values with the usage error. Out-of-range numbers raise integer overflow. Secret numeric access follows the VM's context policy.

The absence of category-flag state produces **false as an emulated default**, not a native day-classification claim. Native category flag 8 determines the day flag; duration does not. The conversion of wrapped 32-bit remaining milliseconds to seconds is an **unknown signedness seam** pending WM85b evidence. Expired or stale records are not established by this contract. Native evidence: handoff `native-evidence-70688/WM85-recipe-cooldown.md`.
