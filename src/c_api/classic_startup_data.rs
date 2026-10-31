//! Finite Vanilla startup data, grounded in cached Blizzard API documentation.

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    // ItemQualitiesDocumentation.lua:6-21 (0..8, nine values);
    // QuestConstantsDocumentation.lua:154-158 (ClassicPreWrath limit five);
    // BagIndexConstantsDocumentation.lua:29 (Vanilla Bank container -1).
    lua.exec(
        "Enum.BagIndex.Bank = -1\n\
         LE_ITEM_QUALITY_POOR = Enum.ItemQuality.Poor\n\
         NUM_LE_ITEM_QUALITYS = Enum.ItemQualityMeta.NumValues\n\
         Constants.QuestWatchConsts_ClassicPreWrath = { MAX_QUEST_WATCHES = 5 }",
    )?;
    Ok(())
}
