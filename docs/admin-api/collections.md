# Collections

### A_Admin.AddTransmog(appearanceId)

Marks a transmog appearance as collected by the player.

- **appearanceId** `number` -- Appearance ID from the transmog database
- **Affects:** `C_Transmog.GetAppearanceSourceInfo()` -- `isCollected` field returns `true`
- **Example:**
```lua
A_Admin.AddTransmog(12345)
```

### A_Admin.RemoveTransmog(appearanceId)

Marks a transmog appearance as uncollected.

- **appearanceId** `number` -- Appearance ID
- **Affects:** `C_Transmog.GetAppearanceSourceInfo()` -- `isCollected` field returns `false`
- **Example:**
```lua
A_Admin.RemoveTransmog(12345)
```

### A_Admin.SetMountCollected(mountId, collected)

Toggles whether a mount is collected.

- **mountId** `number` -- Mount ID from `C_MountJournal`
- **collected** `boolean` -- `true` = collected, `false` = not collected
- **Affects:** `C_MountJournal.GetMountInfoByID(mountId)` -- `isCollected` field
- **Example:**
```lua
A_Admin.SetMountCollected(232, true)    -- Invincible
```

### A_Admin.SetPetCollected(petId, collected)

Toggles whether a battle pet is collected.

- **petId** `number` -- Pet species ID
- **collected** `boolean`
- **Affects:** `C_PetJournal.GetNumCollectedInfo(petId)`
- **Example:**
```lua
A_Admin.SetPetCollected(39, true)       -- Mechanical Squirrel
```

### A_Admin.SetToyCollected(toyId, collected)

Toggles whether a toy is in the toy box.

- **toyId** `number` -- Item ID for the toy
- **collected** `boolean`
- **Affects:** `PlayerHasToy(toyId)`. Newly collecting a known catalog toy sets `C_ToyBoxInfo.NeedsFanfare(toyId)` and the fifth `C_ToyBox.GetToyInfo` result; clearing fanfare preserves ownership. Uncollecting clears fanfare. Repeating collection does not rewrap a toy.
- **Example:**
```lua
A_Admin.SetToyCollected(37710, true)    -- Paper Flying Machine Kit
```

### A_Admin.SetCampsiteCollected(campsiteId, collected)

Toggles whether a campsite (warband scene) is collected.

- **campsiteId** `number` -- Warband scene ID from `C_WarbandScene`
- **collected** `boolean`
- **Affects:** `C_WarbandScene.HasWarbandScene(campsiteId)`
- **Example:**
```lua
A_Admin.SetCampsiteCollected(3, true)
```

### A_Admin.CollectCampsite(campsiteId)

Marks a campsite as collected.

- **campsiteId** `number` -- Warband scene ID from `C_WarbandScene`
- **Affects:** `C_WarbandScene.HasWarbandScene(campsiteId)`
- **Example:**
```lua
A_Admin.CollectCampsite(3)
```

### A_Admin.UncollectCampsite(campsiteId)

Marks a campsite as uncollected.

- **campsiteId** `number` -- Warband scene ID from `C_WarbandScene`
- **Affects:** `C_WarbandScene.HasWarbandScene(campsiteId)`
- **Example:**
```lua
A_Admin.UncollectCampsite(3)
```

### A_Admin.SetAchievementEarned(achieveId, earned)

Toggles whether an achievement has been earned.

- **achieveId** `number` -- Achievement ID
- **earned** `boolean`
- **Affects:** `GetAchievementInfo(achieveId)` -- `completed` field
- **Example:**
```lua
A_Admin.SetAchievementEarned(2144, true)    -- "What a Long, Strange Trip It's Been"
```
