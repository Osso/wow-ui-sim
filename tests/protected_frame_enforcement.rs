use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn test_retail_create_forbidden_frame_is_absent() {
    let env = env();
    let constructor_type: String = env
        .eval("return type(CreateForbiddenFrame)")
        .expect("CreateForbiddenFrame availability probe should succeed");

    assert_eq!(constructor_type, "nil");
}

#[test]
fn test_insecure_addon_created_normal_frame_set_forbidden_is_noop() {
    let env = env();
    let (before, after_true, after_false): (bool, bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "NormalSetForbiddenProbe", UIParent)
            local before = frame:IsForbidden()
            forceinsecure()
            frame:SetForbidden(true)
            local afterTrue = frame:IsForbidden()
            frame:SetForbidden(false)
            local afterFalse = frame:IsForbidden()
            return before, afterTrue, afterFalse
            "#,
        )
        .unwrap();

    assert!(!before);
    assert!(!after_true);
    assert!(!after_false);
}

#[test]
fn test_retail_plain_frame_protection_probe() {
    let env = env();
    let (
        protected_before,
        forbidden,
        protect_missing,
        set_protected_missing,
        protect_ok,
        set_true_ok,
        set_false_ok,
        protected_after,
    ): (bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", nil, UIParent)
            local protectedBefore = frame:IsProtected()
            local forbidden = frame:IsForbidden()
            local protectMissing = type(frame.Protect) == "nil"
            local setProtectedMissing = type(frame.SetProtected) == "nil"
            local protectOk = pcall(function() frame:Protect() end)
            local setTrueOk = pcall(function() frame:SetProtected(true) end)
            local setFalseOk = pcall(function() frame:SetProtected(false) end)
            return protectedBefore, forbidden, protectMissing, setProtectedMissing,
                protectOk, setTrueOk, setFalseOk, frame:IsProtected()
            "#,
        )
        .unwrap();

    assert!(!protected_before);
    assert!(!forbidden);
    assert!(protect_missing);
    assert!(set_protected_missing);
    assert!(!protect_ok);
    assert!(!set_true_ok);
    assert!(!set_false_ok);
    assert!(!protected_after);
}

#[test]
fn test_protected_state_does_not_propagate_to_descendants_or_anchors() {
    let env = env();
    let statuses: String = env
        .eval(
            r#"
            local protected = CreateFrame("Button", "ProtectionPropagationRoot", UIParent)
            A_Admin.SetFrameProtected("ProtectionPropagationRoot", true)

            local child = CreateFrame("Frame", nil, protected)
            local grandchild = CreateFrame("Frame", nil, child)

            local anchoredToProtected = CreateFrame("Frame", nil, UIParent)
            anchoredToProtected:SetPoint("CENTER", protected, "CENTER")

            local anchoredToChild = CreateFrame("Frame", nil, UIParent)
            anchoredToChild:SetPoint("CENTER", child, "CENTER")

            local childAnchoredToProtected = CreateFrame("Frame", nil, UIParent)
            childAnchoredToProtected:SetPoint("CENTER", protected, "CENTER")
            protected.childAnchoredToProtected = childAnchoredToProtected

            local function protectionStatus(frame)
                local isProtected, isProtectedExplicitly = frame:IsProtected()
                return tostring(isProtected) .. ":" .. tostring(isProtectedExplicitly)
            end

            return table.concat({
                protectionStatus(protected),
                protectionStatus(child),
                protectionStatus(grandchild),
                protectionStatus(anchoredToProtected),
                protectionStatus(anchoredToChild),
                protectionStatus(childAnchoredToProtected),
            }, ",")
            "#,
        )
        .unwrap();

    assert_eq!(
        statuses,
        "true:true,false:false,false:false,false:false,false:false,false:false"
    );
}

#[test]
fn test_secure_set_forbidden_marks_frame_forbidden() {
    let env = env();
    let (before, after_true, after_false): (bool, bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "SecureSetForbiddenProbe", UIParent)
            local before = frame:IsForbidden()
            frame:SetForbidden(true)
            local afterTrue = frame:IsForbidden()
            frame:SetForbidden(false)
            local afterFalse = frame:IsForbidden()
            return before, afterTrue, afterFalse
            "#,
        )
        .unwrap();

    assert!(!before);
    assert!(after_true);
    assert!(!after_false);
}

#[test]
fn test_insecure_combat_blocks_protected_parent_and_anchored_frame_mutations() {
    let env = env();
    let (protected_width, protected_height, anchored_width, parent_height, parent_kept, blocked): (
        f32,
        f32,
        f32,
        f32,
        bool,
        String,
    ) = env
        .eval(
            r#"
            local blocked = {}
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("ADDON_ACTION_BLOCKED")
            listener:SetScript("OnEvent", function(_, _, _, func)
                blocked[#blocked + 1] = func
            end)

            local parent = CreateFrame("Frame", "ProtectedParentFrame", UIParent)
            local protected = CreateFrame("Frame", "ProtectedActionFrame", parent)
            local anchored = CreateFrame("Frame", "AnchoredToProtectedFrame", UIParent)
            local otherParent = CreateFrame("Frame", "OtherParentFrame", UIParent)

            parent:SetSize(200, 100)
            protected:SetSize(100, 50)
            anchored:SetSize(80, 20)
            anchored:SetPoint("TOPLEFT", protected, "BOTTOMLEFT", 0, -4)

            A_Admin.SetFrameProtected("ProtectedActionFrame", true)
            A_Admin.SetInCombat(true)
            forceinsecure()

            protected:SetSize(120, 60)
            protected:SetParent(otherParent)
            anchored:SetWidth(123)
            parent:SetHeight(222)

            return protected:GetWidth(true),
                   protected:GetHeight(true),
                   anchored:GetWidth(true),
                   parent:GetHeight(true),
                   protected:GetParent() == parent,
                   table.concat(blocked, "|")
            "#,
        )
        .unwrap();

    assert_eq!(protected_width, 100.0);
    assert_eq!(protected_height, 50.0);
    assert_eq!(anchored_width, 80.0);
    assert_eq!(parent_height, 100.0);
    assert!(
        parent_kept,
        "SetParent should stay blocked on the protected frame"
    );
    assert_eq!(
        blocked,
        "ProtectedActionFrame:SetSize()|ProtectedActionFrame:SetParent()|AnchoredToProtectedFrame:SetWidth()|ProtectedParentFrame:SetHeight()"
    );
}

#[test]
fn test_secure_caller_can_mutate_protected_frame_during_combat() {
    let env = env();
    let (width, height, parent_changed, blocked_count): (f32, f32, bool, i32) = env
        .eval(
            r#"
            local blocked = 0
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("ADDON_ACTION_BLOCKED")
            listener:SetScript("OnEvent", function()
                blocked = blocked + 1
            end)

            local originalParent = CreateFrame("Frame", "SecureProtectedOriginalParent", UIParent)
            local newParent = CreateFrame("Frame", "SecureProtectedNewParent", UIParent)
            local protected = CreateFrame("Frame", "SecureProtectedFrame", originalParent)

            protected:SetSize(40, 20)
            A_Admin.SetFrameProtected("SecureProtectedFrame", true)
            A_Admin.SetInCombat(true)

            protected:SetSize(90, 45)
            protected:SetParent(newParent)

            return protected:GetWidth(true),
                   protected:GetHeight(true),
                   protected:GetParent() == newParent,
                   blocked
            "#,
        )
        .unwrap();

    assert_eq!(width, 90.0);
    assert_eq!(height, 45.0);
    assert!(parent_changed);
    assert_eq!(blocked_count, 0);
}

#[test]
fn test_insecure_out_of_combat_can_mutate_protected_frame() {
    let env = env();
    let (width, height, parent_changed, blocked_count): (f32, f32, bool, i32) = env
        .eval(
            r#"
            local blocked = 0
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("ADDON_ACTION_BLOCKED")
            listener:SetScript("OnEvent", function()
                blocked = blocked + 1
            end)

            local originalParent = CreateFrame("Frame", "OutOfCombatProtectedOriginalParent", UIParent)
            local newParent = CreateFrame("Frame", "OutOfCombatProtectedNewParent", UIParent)
            local protected = CreateFrame("Frame", "OutOfCombatProtectedFrame", originalParent)

            protected:SetSize(55, 25)
            A_Admin.SetFrameProtected("OutOfCombatProtectedFrame", true)
            A_Admin.SetInCombat(false)
            forceinsecure()

            protected:SetSize(95, 35)
            protected:SetParent(newParent)

            return protected:GetWidth(true),
                   protected:GetHeight(true),
                   protected:GetParent() == newParent,
                   blocked
            "#,
        )
        .unwrap();

    assert_eq!(width, 95.0);
    assert_eq!(height, 35.0);
    assert!(parent_changed);
    assert_eq!(blocked_count, 0);
}

#[test]
fn test_insecure_combat_blocks_protected_anchor_mutations() {
    let env = env();
    let (num_points, top_x, center_x, blocked): (i32, f32, f32, String) = env
        .eval(
            r#"
            local blocked = {}
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("ADDON_ACTION_BLOCKED")
            listener:SetScript("OnEvent", function(_, _, _, func)
                blocked[#blocked + 1] = func
            end)

            local frame = CreateFrame("Frame", "ProtectedAnchorFrame", UIParent)
            frame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 10, -10)
            frame:SetPoint("CENTER", UIParent, "CENTER", 20, 0)

            A_Admin.SetFrameProtected("ProtectedAnchorFrame", true)
            A_Admin.SetInCombat(true)
            forceinsecure()

            frame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 99, -99)
            frame:ClearPoint("CENTER")
            frame:ClearAllPoints()

            local _, _, _, topX = frame:GetPointByName("TOPLEFT")
            local _, _, _, centerX = frame:GetPointByName("CENTER")
            return frame:GetNumPoints(), topX, centerX, table.concat(blocked, "|")
            "#,
        )
        .unwrap();

    assert_eq!(num_points, 2);
    assert_eq!(top_x, 10.0);
    assert_eq!(center_x, 20.0);
    assert_eq!(
        blocked,
        "ProtectedAnchorFrame:SetPoint()|ProtectedAnchorFrame:ClearPoint()|ProtectedAnchorFrame:ClearAllPoints()"
    );
}

#[test]
fn test_insecure_combat_blocks_protected_anchor_offset_mutations() {
    let env = env();
    let (offsets, blocked): (String, String) = env
        .eval(
            r#"
            local blocked = {}
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("ADDON_ACTION_BLOCKED")
            listener:SetScript("OnEvent", function(_, _, _, func)
                blocked[#blocked + 1] = func
            end)

            local protected = CreateFrame("Frame", "ProtectedOffsetFrame", UIParent)
            protected:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 10, -10)
            protected:SetPoint("CENTER", UIParent, "CENTER", 20, 5)
            local anchored = CreateFrame("Frame", "AnchoredOffsetFrame", UIParent)
            anchored:SetPoint("CENTER", protected, "CENTER", 3, 4)
            local plain = CreateFrame("Frame", "PlainOffsetFrame", UIParent)
            plain:SetPoint("CENTER", UIParent, "CENTER", 7, 8)

            A_Admin.SetFrameProtected("ProtectedOffsetFrame", true)
            A_Admin.SetInCombat(true)
            forceinsecure()
            protected:AdjustPointsOffset(2, 3)
            protected:SetPointsOffset(30, 40)
            anchored:AdjustPointsOffset(2, 3)
            anchored:SetPointsOffset(30, 40)
            plain:AdjustPointsOffset(2, 3)
            plain:SetPointsOffset(30, 40)

            local function offset(frame, point)
                local _, _, _, x, y = frame:GetPointByName(point)
                return x .. "," .. y
            end
            return table.concat({offset(protected, "TOPLEFT"), offset(protected, "CENTER"),
                offset(anchored, "CENTER"), offset(plain, "CENTER")}, "|"),
                table.concat(blocked, "|")
            "#,
        )
        .unwrap();
    assert_eq!(offsets, "10,-10|20,5|3,4|30,40");
    assert_eq!(blocked, "ProtectedOffsetFrame:AdjustPointsOffset()|ProtectedOffsetFrame:SetPointsOffset()|AnchoredOffsetFrame:AdjustPointsOffset()|AnchoredOffsetFrame:SetPointsOffset()");
}

#[test]
fn test_allowed_protected_anchor_offset_mutations() {
    let env = env();
    let (offsets, blocked): (String, i32) = env
        .eval(
            r#"
            local blocked = 0
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("ADDON_ACTION_BLOCKED")
            listener:SetScript("OnEvent", function() blocked = blocked + 1 end)
            local frame = CreateFrame("Frame", "AllowedOffsetFrame", UIParent)
            frame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 10, -10)
            frame:SetPoint("CENTER", UIParent, "CENTER", 20, 5)
            A_Admin.SetFrameProtected("AllowedOffsetFrame", true)
            A_Admin.SetInCombat(true)
            frame:AdjustPointsOffset(2, 3)
            frame:SetPointsOffset(30, 40)
            A_Admin.SetInCombat(false)
            forceinsecure()
            frame:AdjustPointsOffset(2, -3)
            local _, _, _, x1, y1 = frame:GetPointByName("TOPLEFT")
            local _, _, _, x2, y2 = frame:GetPointByName("CENTER")
            return x1 .. "," .. y1 .. "|" .. x2 .. "," .. y2, blocked
            "#,
        )
        .unwrap();
    assert_eq!(offsets, "32,37|32,37");
    assert_eq!(blocked, 0);
}

#[test]
fn test_insecure_combat_blocks_protected_visibility_and_scale_mutations() {
    let env = env();
    let (scale, shown, visible, show_frame_shown, blocked): (f32, bool, bool, bool, String) = env
        .eval(
            r#"
            local blocked = {}
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("ADDON_ACTION_BLOCKED")
            listener:SetScript("OnEvent", function(_, _, _, func)
                blocked[#blocked + 1] = func
            end)

            local frame = CreateFrame("Frame", "ProtectedVisibilityScaleFrame", UIParent)
            local showFrame = CreateFrame("Frame", "ProtectedShowFrame", UIParent)
            frame:SetScale(1.25)
            frame:Show()
            showFrame:Hide()

            A_Admin.SetFrameProtected("ProtectedVisibilityScaleFrame", true)
            A_Admin.SetFrameProtected("ProtectedShowFrame", true)
            A_Admin.SetInCombat(true)
            forceinsecure()

            frame:SetScale(2.0)
            showFrame:Show()
            frame:Hide()
            frame:SetShown(false)

            return frame:GetScale(),
                   frame:IsShown(),
                   frame:IsVisible(),
                   showFrame:IsShown(),
                   table.concat(blocked, "|")
            "#,
        )
        .unwrap();

    assert_eq!(scale, 1.25);
    assert!(shown);
    assert!(visible);
    assert!(!show_frame_shown);
    assert_eq!(
        blocked,
        "ProtectedVisibilityScaleFrame:SetScale()|ProtectedShowFrame:Show()|ProtectedVisibilityScaleFrame:Hide()|ProtectedVisibilityScaleFrame:SetShown()"
    );
}

#[test]
fn test_insecure_combat_blocks_protected_strata_level_and_toplevel_mutations() {
    let env = env();
    let (strata, level, fixed_level, fixed_strata, toplevel, blocked): (
        String,
        i32,
        bool,
        bool,
        bool,
        String,
    ) = env
        .eval(
            r#"
            local blocked = {}
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("ADDON_ACTION_BLOCKED")
            listener:SetScript("OnEvent", function(_, _, _, func)
                blocked[#blocked + 1] = func
            end)

            local frame = CreateFrame("Frame", "ProtectedStrataLevelFrame", UIParent)
            frame:SetFrameStrata("LOW")
            frame:SetFrameLevel(3)
            frame:SetFixedFrameLevel(false)
            frame:SetFixedFrameStrata(false)
            frame:SetToplevel(false)

            A_Admin.SetFrameProtected("ProtectedStrataLevelFrame", true)
            A_Admin.SetInCombat(true)
            forceinsecure()

            frame:SetFrameStrata("BLIZZARD")
            frame:SetFrameStrata("DIALOG")
            frame:SetFrameLevel(40)
            frame:SetFixedFrameLevel(true)
            frame:SetFixedFrameStrata(true)
            frame:SetToplevel(true)

            return frame:GetFrameStrata(),
                   frame:GetFrameLevel(),
                   frame:HasFixedFrameLevel(),
                   frame:HasFixedFrameStrata(),
                   frame:IsToplevel(),
                   table.concat(blocked, "|")
            "#,
        )
        .unwrap();

    assert_eq!(strata, "LOW");
    assert_eq!(level, 3);
    assert!(!fixed_level);
    assert!(!fixed_strata);
    assert!(!toplevel);
    assert_eq!(
        blocked,
        "ProtectedStrataLevelFrame:SetFrameStrata()|ProtectedStrataLevelFrame:SetFrameLevel()|ProtectedStrataLevelFrame:SetFixedFrameLevel()|ProtectedStrataLevelFrame:SetFixedFrameStrata()|ProtectedStrataLevelFrame:SetToplevel()"
    );
}
