  local healPredictionMethods = {}

  local function healPredictionDefaultValues()
    return {
      health = 0,
      healthMax = 0,
      totalDamageAbsorbs = 0,
      totalHealAbsorbs = 0,
      totalIncomingHeals = 0,
      totalIncomingHealsFromHealer = 0,
    }
  end

  local function healPredictionCopyValues(values)
    values = values or {}
    return {
      health = values.health or 0,
      healthMax = values.healthMax or 0,
      totalDamageAbsorbs = values.totalDamageAbsorbs or 0,
      totalHealAbsorbs = values.totalHealAbsorbs or 0,
      totalIncomingHeals = values.totalIncomingHeals or 0,
      totalIncomingHealsFromHealer = values.totalIncomingHealsFromHealer or 0,
    }
  end

  function healPredictionMethods:Reset()
    self._damageAbsorbClampMode = 0
    self._healAbsorbClampMode = 0
    self._healAbsorbMode = 0
    self._incomingHealClampMode = 0
    self._incomingHealOverflowPercent = 1
    self._maximumHealthMode = 0
    self._predictedValues = healPredictionDefaultValues()
    self._hasSecretValues = false
  end

  function healPredictionMethods:GetIncomingHeals()
    local values = self._predictedValues or healPredictionDefaultValues()
    local total = values.totalIncomingHeals or 0
    local healer = values.totalIncomingHealsFromHealer or 0
    return total, healer, total - healer, false
  end

  function healPredictionMethods:GetDamageAbsorbs()
    local values = self._predictedValues or healPredictionDefaultValues()
    return values.totalDamageAbsorbs or 0, false
  end

  function healPredictionMethods:GetHealAbsorbs()
    local values = self._predictedValues or healPredictionDefaultValues()
    return values.totalHealAbsorbs or 0, false
  end

  function healPredictionMethods:GetDamageAbsorbClampMode()
    return self._damageAbsorbClampMode or 0
  end

  function healPredictionMethods:GetHealAbsorbClampMode()
    return self._healAbsorbClampMode or 0
  end

  function healPredictionMethods:GetHealAbsorbMode()
    return self._healAbsorbMode or 0
  end

  function healPredictionMethods:GetIncomingHealClampMode()
    return self._incomingHealClampMode or 0
  end

  function healPredictionMethods:GetIncomingHealOverflowPercent()
    return self._incomingHealOverflowPercent or 1
  end

  function healPredictionMethods:GetCurrentHealth()
    local values = self._predictedValues or healPredictionDefaultValues()
    return values.health or 0
  end

  function healPredictionMethods:GetMaximumHealth()
    local values = self._predictedValues or healPredictionDefaultValues()
    local maximum = values.healthMax or 0
    if self._maximumHealthMode == 1 then
      maximum = maximum + (values.totalDamageAbsorbs or 0)
    end
    return maximum
  end

  function healPredictionMethods:GetMaximumHealthMode()
    return self._maximumHealthMode or 0
  end

  function healPredictionMethods:GetPredictedValues()
    return healPredictionCopyValues(self._predictedValues)
  end

  function healPredictionMethods:HasSecretValues()
    return self._hasSecretValues == true
  end

  function healPredictionMethods:ResetPredictedValues()
    self._predictedValues = healPredictionDefaultValues()
  end

  function healPredictionMethods:SetDamageAbsorbClampMode(mode)
    self._damageAbsorbClampMode = mode or 0
  end

  function healPredictionMethods:SetHealAbsorbClampMode(mode)
    self._healAbsorbClampMode = mode or 0
  end

  function healPredictionMethods:SetHealAbsorbMode(mode)
    self._healAbsorbMode = mode or 0
  end

  function healPredictionMethods:SetIncomingHealClampMode(mode)
    self._incomingHealClampMode = mode or 0
  end

  function healPredictionMethods:SetIncomingHealOverflowPercent(percent)
    self._incomingHealOverflowPercent = percent or 1
  end

  function healPredictionMethods:SetMaximumHealthMode(mode)
    self._maximumHealthMode = mode or 0
  end

  function healPredictionMethods:SetPredictedValues(values)
    self._predictedValues = healPredictionCopyValues(values)
  end

  function healPredictionMethods:SetToDefaults()
    self:Reset()
  end

  local nextPredictionId = 0
  function CreateUnitHealPredictionCalculator()
    nextPredictionId = nextPredictionId + 1
    local label = "UnitHealPredictionCalculator:" .. tostring(nextPredictionId)
    local calculator = setmetatable({}, {
      __index = healPredictionMethods,
      __tostring = function() return label end,
      __newindex = function(object, key, value)
        if healPredictionMethods[key] ~= nil then
          error("read-only key: " .. tostring(key), 2)
        end
        rawset(object, key, value)
      end,
    })
    calculator:Reset()
    return calculator
  end
