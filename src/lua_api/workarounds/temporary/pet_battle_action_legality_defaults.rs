//! Temporary pet-battle action legality, not a health-based eligibility inference.
//! Retire with the battle turn/swap-lock/skip-availability rules model.

const PET_ACTION_LEGALITY: &str = r#"
-- Missing system: authoritative pet-battle turn/action legality and swap locks.
if rawget(C_PetBattles, "CanPetSwapIn") == nil then
    function C_PetBattles.CanPetSwapIn(_slot)
        return false
    end
end

-- Missing system: pet-battle skip-action availability for the current turn.
if rawget(C_PetBattles, "IsSkipAvailable") == nil then
    function C_PetBattles.IsSkipAvailable()
        return false
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    use crate::client_profile::{ACTIVE, ClientProfile};
    if matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::Mists) {
        lua.exec(PET_ACTION_LEGALITY)?;
    }
    Ok(())
}
