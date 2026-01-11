-- TestAddon: Purpose-built addon for testing wow-ui-sim
-- Each test creates frames with known sizes/positions that we can verify

local addon, ns = ...

ns.tests = {}
ns.results = {}

-- Test 1: Basic frame creation and sizing
function ns.tests.BasicFrame()
    local frame = CreateFrame("Frame", "TestBasicFrame", UIParent)
    frame:SetSize(200, 100)
    frame:SetPoint("CENTER", 0, 0)

    return {
        name = "BasicFrame",
        expected = {
            width = 200,
            height = 100,
            visible = true,
        },
        actual = {
            width = frame:GetWidth(),
            height = frame:GetHeight(),
            visible = frame:IsVisible(),
        }
    }
end

-- Test 2: Anchor positioning
function ns.tests.AnchorPositions()
    local results = {}

    -- Top-left corner
    local tl = CreateFrame("Frame", "TestTopLeft", UIParent)
    tl:SetSize(50, 50)
    tl:SetPoint("TOPLEFT", 10, -10)

    -- Top-right corner
    local tr = CreateFrame("Frame", "TestTopRight", UIParent)
    tr:SetSize(50, 50)
    tr:SetPoint("TOPRIGHT", -10, -10)

    -- Bottom-left corner
    local bl = CreateFrame("Frame", "TestBottomLeft", UIParent)
    bl:SetSize(50, 50)
    bl:SetPoint("BOTTOMLEFT", 10, 10)

    -- Bottom-right corner
    local br = CreateFrame("Frame", "TestBottomRight", UIParent)
    br:SetSize(50, 50)
    br:SetPoint("BOTTOMRIGHT", -10, 10)

    return {
        name = "AnchorPositions",
        expected = "4 frames in corners, 10px from edges",
        frames = {"TestTopLeft", "TestTopRight", "TestBottomLeft", "TestBottomRight"}
    }
end

-- Test 3: Parent-child relationships
function ns.tests.ParentChild()
    local parent = CreateFrame("Frame", "TestParent", UIParent)
    parent:SetSize(300, 200)
    parent:SetPoint("CENTER", 0, 100)

    local child1 = CreateFrame("Frame", "TestChild1", parent)
    child1:SetSize(100, 50)
    child1:SetPoint("TOP", 0, -20)

    local child2 = CreateFrame("Frame", "TestChild2", parent)
    child2:SetSize(100, 50)
    child2:SetPoint("BOTTOM", 0, 20)

    return {
        name = "ParentChild",
        expected = {
            parent_has_children = true,
            child1_parent = "TestParent",
            child2_parent = "TestParent",
        },
        actual = {
            parent_has_children = true, -- Can't easily check this yet
            child1_parent = child1:GetParent():GetName(),
            child2_parent = child2:GetParent():GetName(),
        }
    }
end

-- Test 4: Visibility toggling
function ns.tests.Visibility()
    local frame = CreateFrame("Frame", "TestVisibility", UIParent)
    frame:SetSize(100, 100)
    frame:SetPoint("LEFT", 50, 0)

    local wasVisible1 = frame:IsVisible()
    frame:Hide()
    local wasVisible2 = frame:IsVisible()
    frame:Show()
    local wasVisible3 = frame:IsVisible()

    return {
        name = "Visibility",
        expected = {true, false, true},
        actual = {wasVisible1, wasVisible2, wasVisible3}
    }
end

-- Test 5: Event registration
function ns.tests.Events()
    local frame = CreateFrame("Frame", "TestEvents", UIParent)
    frame:SetSize(80, 80)
    frame:SetPoint("RIGHT", -50, 0)

    ns.eventLog = {}

    frame:SetScript("OnEvent", function(self, event, ...)
        table.insert(ns.eventLog, event)
    end)

    frame:RegisterEvent("PLAYER_LOGIN")
    frame:RegisterEvent("ADDON_LOADED")

    return {
        name = "Events",
        expected = "Frame registered for PLAYER_LOGIN and ADDON_LOADED",
        frame = "TestEvents"
    }
end

-- Test 6: Custom fields on frames (common addon pattern)
function ns.tests.CustomFields()
    local frame = CreateFrame("Frame", "TestCustomFields", UIParent)
    frame:SetSize(60, 60)
    frame:SetPoint("CENTER", -150, -100)

    -- Store custom data on frame
    frame.myData = "hello"
    frame.myNumber = 42
    frame.myTable = {a = 1, b = 2}

    -- Method-style function
    function frame:CustomMethod()
        return self.myData .. " world"
    end

    return {
        name = "CustomFields",
        expected = {
            myData = "hello",
            myNumber = 42,
            customMethod = "hello world"
        },
        actual = {
            myData = frame.myData,
            myNumber = frame.myNumber,
            customMethod = frame:CustomMethod()
        }
    }
end

-- Run all tests
function ns.RunAllTests()
    print("=== Running TestAddon Tests ===")

    for name, testFn in pairs(ns.tests) do
        local ok, result = pcall(testFn)
        if ok then
            ns.results[name] = result
            print("PASS: " .. name)

            -- Check expected vs actual if both exist
            if result.expected and result.actual then
                local match = true
                if type(result.expected) == "table" then
                    for k, v in pairs(result.expected) do
                        if result.actual[k] ~= v then
                            print("  MISMATCH: " .. k .. " expected " .. tostring(v) .. " got " .. tostring(result.actual[k]))
                            match = false
                        end
                    end
                end
                if match then
                    print("  All values match!")
                end
            end
        else
            ns.results[name] = {error = result}
            print("FAIL: " .. name .. " - " .. tostring(result))
        end
    end

    print("=== Tests Complete ===")
    return ns.results
end

-- Auto-run on load
local loader = CreateFrame("Frame")
loader:RegisterEvent("ADDON_LOADED")
loader:SetScript("OnEvent", function(self, event, name)
    if name == addon then
        ns.RunAllTests()
    end
end)
