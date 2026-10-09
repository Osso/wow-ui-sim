# Main native-source falsification — 2026-10-09

Independent of agent479's incomplete transition report, main read complete relevant cached Mists native methods in Blizzard_UIParent/Shared/UIParent.lua:120–210 and Mainline/UIParent.xml:46–83.

Observed native behavior:
- UIParentManagedFrameMixin:OnShow calls self.layoutParent:AddManagedFrame(self).
- AddManagedFrame returns when ignoreFramePositionManager, when non-default, and when not frame:IsShown(); only after those checks does it populate showingFrames and call UpdateFrame.
- UpdateFrame clears points and SetParent(frame.layoutOnBottom and self.BottomManagedLayoutContainer or self).
- XML actual named container is UIParentBottomManagedFrameContainer. BottomManagedLayoutContainer is an unnamed child with parentKey, not the name asserted by the fixture.

Retained exact probe after EDIT_MODE_LAYOUTS_UPDATED and before original assertions: IsInitialized=true; IsInDefaultPosition=true; managed/bottom=true; IsShown=false; attached=false; parent=UIParent; layoutParent=UIParentBottomManagedFrameContainer; showingFrames[cast]=nil. This is consistent with native AddManagedFrame's hidden early return, whether or not that method executed during startup. It is not proof of missing registration or simulator reparent failure. Showing the cast bar should reach the OnShow path, but that transition has not been executed/observed in this diagnostic epoch.

The fixture's unconditional hidden-startup parent-name assertion conflicts with this native conditional behavior. Original fixture body/parent construction/assertions remain unchanged. No production workaround or assertion weakening authorized by this observation. Need reconcile the governing original-fixture constraint before changing test expectations/setup; alternatively retain the precise contract conflict and continue independent handoff work. Cached native behavior is source-grounded, not native-client execution proof.

Correction to agent479 report: before EDIT_MODE_LAYOUTS_UPDATED the retained probe says systemInfo=nil and initialized/default=false; only after the event is systemInfo default=true and initialized/default=true. Its narrative must not substitute post-event values for the earlier boundary. The exact diagnostic run started22:14:59Z, not epoch-directory creation22:13:27Z.
