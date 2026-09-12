local _, Probe = ...

local function report_error(message)
    local summary = Probe.describe(message, true)
    print("Ptr125RemainingProbe: " .. (summary.value or "error diagnostic redacted"))
end

local function command(message)
    local action = string.lower(string.match(message or "", "^%s*(%S+)") or "status")
    if action == "structures" or action == "secrets" then
        local ok, errorValue = pcall(Probe.run, action)
        if not ok then report_error(errorValue) end
    elseif action == "status" then
        local ok, errorValue = pcall(Probe.status)
        if not ok then report_error(errorValue) end
    else
        print("/ptr125probe structures | secrets | status")
    end
end

SLASH_PTR125REMAININGPROBE1 = "/ptr125probe"
SlashCmdList.PTR125REMAININGPROBE = command
