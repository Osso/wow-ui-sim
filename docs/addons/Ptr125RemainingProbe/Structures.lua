local _, Probe = ...

local function capture_method(run, label, object, method, args, includeValues)
    return Probe.capture(run, label, "direct", function()
        return object[method](object, unpack(args, 1, args.n))
    end, Probe.pack(), includeValues)
end

local function observe_region(run, label, region)
    for _, method in ipairs({ "GetName", "GetDrawLayer", "GetSize" }) do
        capture_method(run, label .. ":" .. method, region, method, Probe.pack(), true)
    end
end

local function create_region(run, owner, method, kind, tableForm, template)
    local form = tableForm and "table" or "positional"
    local variant = template and "template" or "plain"
    local label = "regions:" .. kind .. ":" .. form .. ":" .. variant
    local name = Probe.next_name(kind)
    local args
    if tableForm then
        args = Probe.pack({ name = name, drawLayer = "OVERLAY", subLevel = 3, templateName = template })
    elseif kind == "FontString" then
        args = Probe.pack(name, "OVERLAY", template)
    else
        args = Probe.pack(name, "OVERLAY", template, 3)
    end
    local created = Probe.capture(run, label .. ":create", "direct", method,
        Probe.pack(owner, unpack(args, 1, args.n)), false)
    if created[1] and created.n > 1 and created[2] ~= nil then
        observe_region(run, label, created[2])
    end
end

local function region_controls(run)
    local created = Probe.capture(run, "regions:owner:create", "direct", rawget(_G, "CreateFrame"),
        Probe.pack("Frame", Probe.next_name("StructuresOwner"), nil), false)
    if not created[1] or created.n < 2 or created[2] == nil then return end
    local owner = created[2]
    Probe.defer(run, "structures-owner", function() owner:Hide() end)
    local hidden = capture_method(run, "regions:owner:Hide", owner, "Hide", Probe.pack(), false)
    if not hidden[1] then return end

    local kinds = {
        { "Texture", "CreateTexture", "Ptr125RemainingProbeTextureTemplate" },
        { "FontString", "CreateFontString", "Ptr125RemainingProbeFontStringTemplate" },
        { "Line", "CreateLine" },
        { "MaskTexture", "CreateMaskTexture" },
    }
    for _, entry in ipairs(kinds) do
        local kind, methodName, template = unpack(entry)
        local method = Probe.capture(run, "regions:" .. kind .. ":method", "direct",
            function() return owner[methodName] end, Probe.pack(), false)
        if method[1] and type(method[2]) == "function" then
            create_region(run, owner, method[2], kind, false, nil)
            create_region(run, owner, method[2], kind, true, nil)
            if template then
                create_region(run, owner, method[2], kind, false, template)
                create_region(run, owner, method[2], kind, true, template)
            end
        end
    end
end

local function inspect_first_signal(run, result)
    if not result[1] or result.n < 2 then return end
    local first = result[2]
    local kind = Probe.capture(run, "signals:first-kind", "direct",
        function() return type(first) end, Probe.pack(), true)
    if not kind[1] or (kind[2] ~= "table" and kind[2] ~= "userdata") then return end
    for _, field in ipairs({ "key", "time" }) do
        Probe.capture(run, "signals:entry:" .. field, "direct",
            function() return first[field] end, Probe.pack(), true)
    end
end

local function signal_control(run)
    local factory = Probe.capture(run, "signals:factory", "direct", function()
        local namespace = rawget(_G, "C_Timer")
        if type(namespace) == "table" then return rawget(namespace, "NewTimedSignalMap") end
    end, Probe.pack(), false)
    local created = Probe.capture(run, "signals:create", "direct", factory[1] and factory[2] or nil,
        Probe.pack(function() end), false)
    if not created[1] or created.n < 2 or created[2] == nil then return end
    local map = created[2]
    Probe.defer(run, "structures-signals", function() map:CancelAllSignals() end)
    local deadline = Probe.capture(run, "signals:deadline", "direct",
        function() return GetTime() + 600 end, Probe.pack(), true)
    if deadline[1] then
        capture_method(run, "signals:SignalAt", map, "SignalAt", Probe.pack(17, deadline[2]), false)
    end
    local nextSignal = capture_method(run, "signals:GetNextSignal", map, "GetNextSignal", Probe.pack(), true)
    inspect_first_signal(run, nextSignal)
end

function Probe.actions.structures(run)
    for _, name in ipairs({ "CreateRegionParams", "TimedSignalMapEntry" }) do
        Probe.capture(run, "globals:" .. name, "direct",
            function() return rawget(_G, name) end, Probe.pack(), false)
    end
    -- Each section remains independent even if an unexpected experiment fails.
    Probe.capture(run, "structures:regions", "direct", region_controls, Probe.pack(run), false)
    Probe.capture(run, "structures:signals", "direct", signal_control, Probe.pack(run), false)
end
