-- Prediction methods operate on the same values populated by UnitGetDetailedHealPrediction.
local methods = healPredictionMethods

function methods:GetMissingHealth()
    return math.max(0, self:GetMaximumHealth() - self:GetCurrentHealth())
end

-- INFERRED: zero maximum health yields zero rather than NaN/inf.
function methods:GetCurrentHealthPercent()
    local maximum = self:GetMaximumHealth()
    return maximum > 0 and self:GetCurrentHealth() / maximum or 0
end

function methods:GetMissingHealthPercent()
    local maximum = self:GetMaximumHealth()
    return maximum > 0 and self:GetMissingHealth() / maximum or 0
end

function methods:EvaluateCurrentHealthPercent(curve)
    return curve:Evaluate(self:GetCurrentHealthPercent())
end

function methods:EvaluateMissingHealthPercent(curve)
    return curve:Evaluate(self:GetMissingHealthPercent())
end

function methods:GetTotalDamageAbsorbs()
    return self._predictedValues.totalDamageAbsorbs
end
function methods:GetTotalHealAbsorbs()
    return self._predictedValues.totalHealAbsorbs
end
function methods:GetTotalIncomingHeals()
    return self._predictedValues.totalIncomingHeals
end
function methods:GetTotalIncomingHealsFromHealer()
    return self._predictedValues.totalIncomingHealsFromHealer
end

function methods:GetMaximumHealAbsorbs()
    return self:GetHealAbsorbClampMode() == 1 and self:GetMaximumHealth() or self:GetCurrentHealth()
end

function methods:GetMaximumIncomingHeals()
    if self:GetIncomingHealClampMode() == 1 then
        return self:GetMaximumHealth()
    end
    -- INFERRED: overflow factor expands the maximum-health boundary.
    return math.max(0, self:GetMaximumHealth() * self:GetIncomingHealOverflowPercent() - self:GetCurrentHealth())
end

function methods:GetMaximumDamageAbsorbs()
    local mode = self:GetDamageAbsorbClampMode()
    if mode == 2 then return self:GetMaximumHealth() end
    local missing = self:GetMissingHealth()
    if mode == 1 then return missing end
    local incoming = self:GetTotalIncomingHeals()
    if self:GetHealAbsorbMode() == 0 then
        incoming = math.max(0, incoming - self:GetTotalHealAbsorbs())
    end
    return math.max(0, missing - math.min(incoming, self:GetMaximumIncomingHeals()))
end
