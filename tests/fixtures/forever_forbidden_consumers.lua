ForeverForbiddenConsumers = {}

function ForeverForbiddenConsumers.secure_handlers()
    local header = CreateFrame('Frame', nil, UIParent, 'SecureHandlerBaseTemplate')
    local target = CreateFrame('Frame')
    SecureHandlerSetFrameRef(header, 'accepted', target)
    assert(not SecureHandlersUpdateFrame.HasAnyForbiddenAspects(header))
    assert(header:GetAttribute('frameref-accepted') ~= nil)
    header:AddForbiddenAspects(Enum.ForbiddenAspect.ScriptBindings)
    assert(SecureHandlersUpdateFrame.HasAnyForbiddenAspects(header))
    local ok, err = pcall(SecureHandlerSetFrameRef, header, 'rejected', target)
    assert(not ok and err:find('Cannot use SecureHandlers API on forbidden frames', 1, true), tostring(err))
    assert(header:GetAttribute('frameref-rejected') == nil)
end

function ForeverForbiddenConsumers.aura_shell()
    local function create_from_addon()
        assert(not issecure(), 'fixture must execute as addon code')
        ForbiddenConsumerAura = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
        ForbiddenConsumerAura:SetPoint('CENTER', UIParent, 'CENTER')
        ForbiddenConsumerAura:SetSize(1, 1)
        ForbiddenConsumerAura:AddAuraGroup('probe', 'HELPFUL', {
            maxFrameCount = 1,
            initializeFrame = function(button)
                ForbiddenConsumerAuraButton = button
                local icon = button:CreateTexture(nil, 'BACKGROUND')
                icon:SetAllPoints(button)
            end,
        })
        ForbiddenConsumerAura:SetUnit('player')
    end
    debug.setobjecttaint(create_from_addon, 'ForbiddenConsumerProbe')
    create_from_addon()
    assert(ForbiddenConsumerAuraButton, 'native group must create an aura button')
    assert(ForbiddenConsumerAuraButton:HasAnyForbiddenAspects())
    local mask = ForbiddenConsumerAuraButton:GetForbiddenAspects()
    assert(bit.band(mask, Enum.ForbiddenAspect.UntrustedScriptExecution) ~= 0)
    local private = GetForbiddenObjectTable(ForbiddenConsumerAura)
    private:MarkClean(private.dirtyFlags:GetFlags())
    ForbiddenConsumerDirtyCalls = 0
    private:SetDirtyPhases({{flag=1, handler=function()
        assert(ForbiddenConsumerAura:GetOnUpdateMode() == Enum.OnUpdateMode.Disabled)
        ForbiddenConsumerDirtyCalls = ForbiddenConsumerDirtyCalls + 1
    end}})
    private:MarkDirty(1)
    assert(ForbiddenConsumerAura:GetOnUpdateMode() == Enum.OnUpdateMode.RunWhenVisibleOnce)
end

function ForeverForbiddenConsumers.enums_and_inheritance()
    local e = Enum.ForbiddenAspect
    assert(e.UntrustedScriptExecution == 4 and e.UntrustedLayoutScriptExecution == 8)
    assert(e.QueryAnimationProgress == 2048 and e.AddAnimations == 4096)
    local meta = Enum.ForbiddenAspectMeta
    assert(meta.MinValue == 1 and meta.MaxValue == 4096 and meta.NumValues == 13)
    local paths = Enum.ScriptObjectPropagationPath
    assert(paths.Hierarchy == 0 and paths.Layout == 1)
    local parent = CreateFrame('Frame')
    parent:AddForbiddenAspects(e.UntrustedLayoutScriptExecution)
    assert(parent:GetInheritableForbiddenAspects(paths.Hierarchy) == 8)
    assert(parent:GetInheritableForbiddenAspects(paths.Layout) == 8)
    local child = CreateFrame('Frame', nil, parent)
    assert(child:GetForbiddenAspects() == 9)
    local plain = CreateFrame('Frame')
    local ok, err = pcall(plain.SetParent, plain, parent)
    assert(not ok and err:find('Cannot implicitly gain forbidden aspects', 1, true))
    ok, err = pcall(plain.SetPoint, plain, 'CENTER', parent, 'CENTER')
    assert(not ok and err:find('Cannot implicitly gain forbidden aspects', 1, true))
    assert(plain:GetParent() ~= parent and plain:GetNumPoints() == 0)
end
