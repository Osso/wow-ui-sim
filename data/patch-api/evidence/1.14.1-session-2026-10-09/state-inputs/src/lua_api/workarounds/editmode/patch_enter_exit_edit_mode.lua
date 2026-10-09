
    if not EditModeManagerFrame then return end
    local emm = EditModeManagerFrame

    function emm:EnterEditMode()
        self.editModeActive = true
        pcall(self.ClearActiveChangesFlags, self)
        pcall(self.UpdateDropdownOptions, self)
        pcall(self.ShowSystemSelections, self)
        if self.AccountSettings
            and self.AccountSettings.OnEditModeEnter then
            pcall(
                self.AccountSettings.OnEditModeEnter,
                self.AccountSettings
            )
        end
        pcall(EventRegistry.TriggerEvent,
            EventRegistry, "EditMode.Enter")
    end

    function emm:ExitEditMode()
        self.editModeActive = false
        pcall(self.ClearSelectedSystem, self)
        pcall(function()
            secureexecuterange(
                self.registeredSystemFrames,
                function(_, f)
                    if f.OnEditModeExit then
                        pcall(f.OnEditModeExit, f)
                    end
                end
            )
        end)
        if self.AccountSettings
            and self.AccountSettings.OnEditModeExit then
            pcall(
                self.AccountSettings.OnEditModeExit,
                self.AccountSettings
            )
        end
        pcall(EventRegistry.TriggerEvent,
            EventRegistry, "EditMode.Exit")
    end
