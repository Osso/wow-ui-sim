-- name, valid arguments, exact current modeled/placeholder return values
statFixtures = {
    {'GetAttackPowerForStat', {1, 37}, {37}},
    {'GetAvoidance', {}, {0}},
    {'GetBlockChance', {}, {0}},
    {'GetCombatRating', {9}, {360}},
    {'GetCombatRatingBonus', {9}, {2}},
    {'GetCritChance', {}, {7}},
    {'GetDodgeChance', {}, {7}},
    {'GetDodgeChanceFromAttribute', {}, {2}},
    {'GetExpertise', {}, {0, 0, 0}},
    {'GetExpertisePercent', {}, {0, 0, 0}},
    {'GetHaste', {}, {3}},
    {'GetHitModifier', {}, {0}},
    {'GetLifesteal', {}, {0}},
    {'GetManaRegen', {}, {2, 1}},
    {'GetMasteryEffect', {}, {12, 4}},
    {'GetMeleeHaste', {}, {3}},
    {'GetModResilienceDamageReduction', {}, {0}},
    {'GetParryChance', {}, {8}},
    {'GetParryChanceFromAttribute', {}, {3}},
    {'GetPetSpellBonusDamage', {}, {0}},
    {'GetPvpPowerDamage', {}, {0}},
    {'GetPvpPowerHealing', {}, {0}},
    {'GetRangedCritChance', {}, {7}},
    {'GetRangedHaste', {}, {3}},
    {'GetShieldBlock', {}, {0}},
    {'GetSpeed', {}, {0}},
    {'GetSpellBonusDamage', {2}, {500}},
    {'GetSpellBonusHealing', {}, {500}},
    {'GetSpellCritChance', {2}, {7}},
    {'GetSpellHitModifier', {}, {0}},
    {'GetVersatilityBonus', {14}, {5}},
    {'UnitArmor', {'player'}, {1234, 1234, 1234, 0, 0}},
    {'UnitAttackPower', {'player'}, {600, 0, 0}},
    {'UnitAttackSpeed', {'player'}, {2, 2}},
    {'UnitDamage', {'player'}, {160, 210, 0, 0, 0, 0, 1}},
    {'UnitRangedAttackPower', {'player'}, {600, 0, 0}},
    {'UnitRangedDamage', {'player'}, {2, 160, 210, 0, 0, 1}},
    {'UnitSpellHaste', {'player'}, {510/180}},
    {'UnitStat', {'player', 1}, {300, 300, 0, 0}},
}

local function pack(...) return {n = select('#', ...), ...} end
function assertStatOutputs(restricted)
    for _, fixture in ipairs(statFixtures) do
        local name, args, expected = unpack(fixture)
        local query = _G[name]
        assert(type(query) == 'function', name)
        local actual = pack(query(unpack(args)))
        assert(actual.n == #expected, name .. ' arity')
        for index, value in ipairs(expected) do
            local result = actual[index]
            assert(issecretvalue(result) == restricted, name .. ' secrecy ' .. index)
            assert(canaccessvalue(result) == not restricted, name .. ' access ' .. index)
            if restricted then result = secretunwrap(result) end
            assert(math.abs(result - value) < 1e-10, name .. ' value ' .. index)
        end
    end
end
