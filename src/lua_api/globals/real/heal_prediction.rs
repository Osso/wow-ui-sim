//! Health-prediction objects own input values and clamp configuration.
//! UnitGetDetailedHealPrediction populates the same value fields from unit vitals.

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    let source = include_str!("heal_prediction.lua");
    #[cfg(feature = "retail-12-0-5")]
    let source = format!("{source}{}", include_str!("heal_prediction_12_0_1.lua"));
    lua.exec(&source)?;
    Ok(())
}
