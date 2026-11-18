-- Callable wrong producer: expose an out-of-range call-pet index.
local getCallPetSpellInfo = GetCallPetSpellInfo
function GetCallPetSpellInfo(spellID)
    if spellID == 901005 then
        return 5, nil
    end
    return getCallPetSpellInfo(spellID)
end
