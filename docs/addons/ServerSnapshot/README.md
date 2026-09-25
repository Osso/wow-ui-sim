# ServerSnapshot

ServerSnapshot is a small World of Warcraft addon that records character UI state into `SavedVariables`. The main target is state that wow-ui-sim cannot reliably recover from static WTF files alone: action bar slot contents, carried bag contents, addon enable state as exposed by the live AddOn List APIs, and keybindings.

## Install

Copy this folder to the matching WoW client AddOns directory:

```text
World of Warcraft/_retail_/Interface/AddOns/ServerSnapshot/
```

The folder should contain:

```text
ServerSnapshot/ServerSnapshot.toc
ServerSnapshot/ServerSnapshot.lua
ServerSnapshot/README.md
```

## Use

Log into the character whose data you want to capture. The addon snapshots automatically on login, entering the world, coalesced bag updates, bag-container changes, and action bar, spell, talent, macro, keybinding, and specialization changes. It also snapshots on logout and when the AddOn List OK path is available.

Slash commands:

```text
/serversnapshot
/ssnap
```

Both commands take a fresh snapshot and print the character key plus action slot count. Use `/reload` or logout after a capture so WoW flushes `ServerSnapshotDB` to disk.

## Saved Data

WoW writes the database at logout or `/reload`:

```text
World of Warcraft/_retail_/WTF/Account/<ACCOUNT>/SavedVariables/ServerSnapshot.lua
```

The global table is:

```lua
ServerSnapshotDB
```

Snapshots are stored by character key:

```lua
ServerSnapshotDB.characters["Realm/Character"]
```

Each snapshot includes metadata, action bar slot contents, carried bags when ready, addon enable states, sampled keybinding state, spellbook data when available, macros when available, talent/loadout details where Blizzard exposes a public API, and the active EditMode layout.

### Carried bags

`snapshot.bags` captures backpack container 0 and equipped bag containers 1 through `NUM_TOTAL_EQUIPPED_BAG_SLOTS` (including reagent bag 5 on clients with that slot):

```lua
bags = {
    maxBagID = 5,
    containers = {
        [0] = { numSlots = 16, family = 0, items = { [1] = { itemID = 100, stackCount = 4, hyperlink = "item:100" } } },
        [1] = { numSlots = 2, family = 8, name = "Herb Bag", inventorySlot = 20, itemID = 500, hyperlink = "item:500", items = {} },
        -- Every carried container has a row, including empty/unequipped bags.
    },
}
```

`numSlots` and the complete container range identify empty bags and slots; missing `items[slot]` means an empty slot. `family` is the second result of `C_Container.GetContainerNumFreeSlots`. `inventorySlot` comes from `C_Container.ContainerIDToInventoryID` for equipped bags only; bag name, equipped item ID/link, and occupied slot hyperlink are optional. Bank containers are not captured. Capture omits the entire `bags` domain if required modern `C_Container` functions or container-to-inventory mapping are unavailable, any container/slot read fails, or backpack size is zero (not ready). It does not reuse previous bags: absent `snapshot.bags` means *not captured*, not an authoritative empty bag set. A populated domain replaces carried bag state on import.

### EditMode layout

`snapshot.editMode` records the live client's EditMode state:

```lua
editMode = {
    activeLayoutName = "Ultrawide",   -- EditModeManagerFrame:GetActiveLayoutInfo().layoutName
    activeLayout = 3,                 -- C_EditMode.GetLayouts().activeLayout index
    layoutNames = { "Modern", "Classic", "Ultrawide" },
}
```

The addon refreshes this on `EDIT_MODE_LAYOUTS_UPDATED` (i.e. when you switch
layouts in Edit Mode), so the captured name tracks whatever you last selected.

## wow-ui-sim Import

When wow-ui-sim starts with SavedVariables enabled, it looks for:

```text
WTF/Account/<ACCOUNT>/SavedVariables/ServerSnapshot.lua
```

If present, the simulator loads `ServerSnapshotDB`, picks `lastCharacterKey` when available, and falls back to the newest captured character snapshot.

Before third-party addon loading, wow-ui-sim uses the captured `addons.entries[*].enabled` values as an enable-state overlay. This is more reliable than trying to infer the AddOn List UI state from `AddOns.txt` alone.

Before Blizzard addons load, wow-ui-sim applies captured keybindings and clears/seeds spell action slots from the captured action bar data. Empty action slots and non-spell action entries such as macros are ignored today.

The simulator also reads `snapshot.editMode.activeLayoutName` and uses it as the
preferred EditMode layout when loading the WTF EditMode cache. This means the
simulator selects the same layout the live client was using, instead of relying
on the (sometimes stale) `edit-mode-cache-character.txt` active-layout index.
The `WOW_SIM_EDIT_MODE_LAYOUT` env var still takes precedence as an explicit
manual override; the layout selection priority is:

1. `WOW_SIM_EDIT_MODE_LAYOUT` env var (manual override)
2. `snapshot.editMode.activeLayoutName` (captured from the live client)
3. the active-layout index in `edit-mode-cache-character.txt`

## Notes

- Empty action slots are recorded as `{ empty = true }`.
- Missing APIs are skipped instead of breaking the addon; unavailable bag capture omits `snapshot.bags`.
- Keybinding capture stores `GetBinding()` rows plus a sampled key map for common/default keys so explicit unbinds can shadow simulator defaults.
- `## Interface` may need to be updated for the exact WoW client build. In game, run `/run print(select(4, GetBuildInfo()))`.
