# Currency ID getters

## What it must do

Each getter takes no arguments and returns one number. Every profile publishes the Azerite and War Resources getters; only Retail publishes the Dragon Isles Supplies getter.

| Getter | Value |
|---|---:|
| `C_CurrencyInfo.GetAzeriteCurrencyID()` | 1553 |
| `C_CurrencyInfo.GetWarResourcesCurrencyID()` | 1560 |
| `C_CurrencyInfo.GetDragonIslesSuppliesCurrencyID()` | 2003 |

VERIFIED (WM50, build 70688): each native handler writes its literal constant without DB2, game-rule, faction or expansion lookups, and publishes one number.

`WorldMap_GetWorldQuestRewardType` reads all three before it classifies reward currencies (`Blizzard_UIPanels_Game/Mainline/WorldMapFrame.lua:36-38`).

## Known gaps

- Native control flow between the constant and its publication is not fully proven (WM50 UNKNOWN). The simulator returns the constant unconditionally.
