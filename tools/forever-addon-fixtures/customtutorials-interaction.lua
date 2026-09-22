-- Read-only proposal: run with exact cached CustomTutorials-2.1 plus LibStub staged.
-- This exercises the library's public dot-syntax API and observable tutorial frame state.
local Lib = LibStub:GetLibrary("CustomTutorials-2.1")
assert(Lib, "CustomTutorials-2.1 unavailable")

local key = "__CustomTutorialsAudit_20260922"
_G[key] = nil
local addon = {}
local shown = {}
local callbackCount = 0

local data = {
  savedvariable = key,
  title = "Audit tutorial",
  onShow = function(_, index)
    callbackCount = callbackCount + 1
    shown[#shown + 1] = index
  end,
  { title = "Step one", text = "first" },
  { title = "Step two", text = "second" },
  { title = "Step three", text = "third" },
}

-- README documents dot syntax for these four methods; Embed is the only colon API.
Lib.RegisterTutorials(addon, data)
assert(Lib.GetTutorials(addon) == data, "GetTutorials did not return registered data")
assert(_G[key] == nil, "registration unexpectedly advanced saved progress")

local frame = Lib.frames[addon]
assert(frame, "tutorial frame was not created")
-- Registration creates a frame; establish a hidden baseline explicitly.
frame:Hide()
assert(not frame:IsShown(), "explicit hide failed")

-- With savedvariable and default maxAdvance, Trigger(2) displays first unseen step.
Lib.TriggerTutorial(addon, 2)
assert(frame:IsShown(), "TriggerTutorial did not show tutorial frame")
assert(frame.i == 1, "default saved-progress advance did not display step 1")
assert(_G[key] == 1, "displayed step was not saved")
assert(callbackCount == 1 and shown[1] == 1, "onShow did not observe step 1")

-- A second trigger advances one step, while the requested index unlocks progression.
Lib.TriggerTutorial(addon, 3)
assert(frame.i == 2, "second trigger did not display step 2")
assert(_G[key] == 2, "step 2 was not saved")
assert(callbackCount == 2 and shown[2] == 2, "onShow did not observe step 2")

-- maxAdvance=true jumps directly to the requested unlocked step.
Lib.TriggerTutorial(addon, 3, true)
assert(frame.i == 3, "maxAdvance=true did not display requested step")
assert(_G[key] == 3, "step 3 was not saved")
assert(callbackCount == 3 and shown[3] == 3, "onShow did not observe step 3")

-- Triggering an already-seen index is a no-op and leaves the frame/state intact.
Lib.TriggerTutorial(addon, 2)
assert(frame.i == 3 and _G[key] == 3, "already-seen trigger changed progress")
assert(callbackCount == 3, "already-seen trigger invoked onShow")

Lib.ResetTutorials(addon)
assert(not frame:IsShown(), "ResetTutorials did not hide tutorial frame")
assert(_G[key] == false, "ResetTutorials did not clear saved progress")

-- Reset is observable cleanup; re-trigger must start at step 1 again.
Lib.TriggerTutorial(addon, 2)
assert(frame:IsShown() and frame.i == 1, "reset progress was not honored on retrigger")
assert(_G[key] == 1 and callbackCount == 4 and shown[4] == 1, "retrigger did not restart tutorial")

print("CUSTOMTUTORIALS_INTERACTION_PASS", frame:GetName(), table.concat(shown, ","), callbackCount)
