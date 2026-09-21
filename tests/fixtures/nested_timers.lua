NestedTimers = { outer = 0, after = 0, timer = 0, ticker = 0, old = 0, cancelled = 0 }
C_Timer.NewTicker(0, function() NestedTimers.old = NestedTimers.old + 1 end, 3)
C_Timer.After(0, function()
    NestedTimers.outer = NestedTimers.outer + 1
    C_Timer.After(0, function() NestedTimers.after = NestedTimers.after + 1 end)
    C_Timer.NewTimer(0, function() NestedTimers.timer = NestedTimers.timer + 1 end)
    C_Timer.NewTicker(0, function() NestedTimers.ticker = NestedTimers.ticker + 1 end, 2)
    local cancelled = C_Timer.NewTimer(0, function()
        NestedTimers.cancelled = NestedTimers.cancelled + 1
    end)
    cancelled:Cancel()
end)

function CheckNestedTimerPass(pass)
    local expected = {
        { outer = 1, after = 0, timer = 0, ticker = 0, old = 1, cancelled = 0 },
        { outer = 1, after = 1, timer = 1, ticker = 1, old = 2, cancelled = 0 },
        { outer = 1, after = 1, timer = 1, ticker = 2, old = 3, cancelled = 0 },
    }
    for key, value in pairs(expected[math.min(pass, 3)]) do
        assert(NestedTimers[key] == value,
            'nested timer pass ' .. pass .. ' ' .. key .. ': expected ' .. value .. ', got ' .. NestedTimers[key])
    end
end
