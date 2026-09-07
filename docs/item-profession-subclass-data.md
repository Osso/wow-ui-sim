# Profession item subclass names

`C_Item.GetItemSubClassInfo(19, subclassID)` uses ItemSubClass DB2 display names, not the generic `Other` label. Syndicator's English search initialization rejects that incorrect label before registering profession-tool keywords.

## Source

Wago's ItemSubClass CSV for explicit client build **12.1.0.69497**, inspected September 7, 2026: 178 rows total. The acquired file is `/tmp/pi-ItemSubClass-12.1.0.69497.csv`, SHA-256 `05406731224b638b6a964d7e83c5cdbda18c3512f31404fe1e33e577d89f1fdc`.

ClassID `19` has DB2 row IDs `329..342`, SubClassID `0..13`. `DisplayName_lang` supplies these names in order:

```text
Blacksmithing, Leatherworking, Alchemy, Herbalism, Cooking, Mining,
Tailoring, Engineering, Enchanting, Fishing, Skinning, Jewelcrafting,
Inscription, Archaeology
```

All fourteen rows have empty `VerboseName_lang`. Names were taken from these rows, not inferred from addon comments. The API's existing second-result behavior remains `false` for this non-weapon/non-armor class; invalid subclass `14` still returns nil.

## Coverage

`tests/c_item_api/c_item.rs` asserts all fourteen exact names and boolean results, invalid subclass `14`, and Syndicator's locale-sensitive manual-keyword comparison on `enUS`. The consumer fixture covers this input validation boundary, not a complete Syndicator addon load. Full-addon startup verification remains a separate integration gate.
