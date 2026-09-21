local function fractions(expected, ...)
    local values = UnitEmpoweredStagePercentages('player', ...)
    assert(type(values) == 'table' and #values == #expected)
    for i, value in ipairs(expected) do
        assert(math.abs(values[i] - value) < 0.0000001,
            'stage '..i..': '..tostring(values[i])..' expected '..value)
    end
    return values
end

function CheckEmpoweredStageFractions()
    assert(type(UnitEmpoweredStagePercentages) == 'function', 'missing UnitEmpoweredStagePercentages')
    assert(select('#', UnitEmpoweredStagePercentages('player')) == 0)
    A_Admin.StartChannel(15407, 'Ordinary channel', '', 30)
    assert(select('#', UnitEmpoweredStagePercentages('player')) == 0)
    A_Admin.StartEmpower(357208, 'Uneven stages', '', {1, 2}, 3)
    fractions({1/6, 2/6, 3/6})
    fractions({1/6, 2/6, 3/6}, true)
    fractions({1/3, 2/3}, false)
    for _, unit in ipairs({'target', 'focus', 'party1', 'unknown', ''}) do
        assert(select('#', UnitEmpoweredStagePercentages(unit)) == 0)
    end
    local snapshot = UnitEmpoweredStagePercentages('player')
    snapshot[1] = 99
    fractions({1/6, 2/6, 3/6})
    A_Admin.UpdateEmpower({2, 4}, 2)
    fractions({1/4, 1/2, 1/4})
    fractions({1/3, 2/3}, false)
    A_Admin.UpdateEmpower({2, 4}, 0)
    fractions({1/3, 2/3, 0})
    fractions({1/3, 2/3}, false)
    assert(A_Admin.StopChannel(true))
    assert(select('#', UnitEmpoweredStagePercentages('player')) == 0)
end

function CheckEllesmereEmpowerPips()
    local ns = assert(EllesmereUI._ModuleNS.EllesmereUIUnitFrames)
    local previous = ns.db.profile.player.showPlayerCastbar
    ns.db.profile.player.showPlayerCastbar = true
    ns.ReloadFrames()
    local bar = assert(ns.frames.player.Castbar)
    local width = bar:GetWidth()
    assert(width > 0 and bar:GetOrientation() == 'HORIZONTAL')
    A_Admin.StartEmpower(357208, 'Uneven stages', '', {1, 2}, 3)
    assert(bar:IsShown() and bar.empowering, 'real empower consumer did not start')
    local function positions(expected)
        for index, offset in ipairs(expected) do
            local pip = assert(bar.Pips[index])
            local point, relative, relativePoint, x, y = pip:GetPoint(1)
            assert(pip:IsShown() and point == 'CENTER' and relative == bar)
            assert(relativePoint == 'LEFT' and math.abs(x-offset) < 0.0001 and y == 0)
        end
    end
    positions({width/6, width/2, width})
    bar:UpdatePips(UnitEmpoweredStagePercentages('player', false))
    positions({width/3, width})
    assert(not bar.Pips[3]:IsShown(), 'hold pip was not hidden')
    assert(A_Admin.StopChannel(true))
    ns.db.profile.player.showPlayerCastbar = previous
    ns.ReloadFrames()
end
