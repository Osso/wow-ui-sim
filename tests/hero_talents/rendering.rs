use super::env_with_protection_ui;

#[test]
fn test_class_talent_edges_render_below_visible_talent_buttons() {
    let env = env_with_protection_ui();
    let result: String = env
        .eval(
            r#"
            local ok = C_Traits.SetSelection(1, 99838, 123361) -- Lightsmith
            assert(ok, "expected deterministic hero subtree selection")
            assert(C_ClassTalents.GetActiveHeroTalentSpec() == 49, "expected active subtree 49")

            assert(PlayerSpellsUtil and PlayerSpellsUtil.OpenToClassTalentsTab, "expected class talents UI helper")
            PlayerSpellsUtil.OpenToClassTalentsTab()

            local frame = PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame
            assert(frame and frame.edgePool, "expected class talents frame and edge pool")

            -- Force one explicit update pass so edge state/frame-level updates have run.
            if frame.OnUpdate then
                frame:OnUpdate()
            end

            local minHeroVisibleButtonLevel = nil
            local heroVisibleButtons = 0
            for button in frame:EnumerateAllTalentButtons() do
                local nodeInfo = button.GetNodeInfo and button:GetNodeInfo()
                if button:IsShown() and nodeInfo and nodeInfo.subTreeID then
                    heroVisibleButtons = heroVisibleButtons + 1
                    local level = button:GetFrameLevel()
                    if minHeroVisibleButtonLevel == nil or level < minHeroVisibleButtonLevel then
                        minHeroVisibleButtonLevel = level
                    end
                end
            end
            assert(heroVisibleButtons > 0, "expected at least one visible hero talent button")
            assert(minHeroVisibleButtonLevel ~= nil, "expected hero talent frame levels")

            local checked = 0
            local violations = {}
            local iconViolations = {}
            local function iconLevel(button)
                if not button then
                    return nil
                end
                local icon = nil
                if button.GetActiveIcon then
                    local okCall, activeIcon = pcall(button.GetActiveIcon, button)
                    if okCall and type(activeIcon) == "table" and activeIcon.GetFrameLevel then
                        icon = activeIcon
                    end
                end
                if not icon and type(button.Icon) == "table" and button.Icon.GetFrameLevel then
                    icon = button.Icon
                end
                if icon then
                    return icon:GetFrameLevel()
                end
                return nil
            end
            for edge in frame.edgePool:EnumerateActive() do
                local edgeLevel = edge:GetFrameLevel()
                local startLevel = edge:GetStartButton():GetFrameLevel()
                local endLevel = edge:GetEndButton():GetFrameLevel()
                local startIconLevel = iconLevel(edge:GetStartButton())
                local endIconLevel = iconLevel(edge:GetEndButton())
                checked = checked + 1
                if edgeLevel >= math.min(startLevel, endLevel) then
                    table.insert(
                        violations,
                        string.format(
                            "edge=%d expected<min(%d,%d)",
                            edgeLevel,
                            startLevel,
                            endLevel
                        )
                    )
                end
                if startIconLevel and edgeLevel >= startIconLevel then
                    table.insert(
                        iconViolations,
                        string.format(
                            "edge=%d expected<startIcon(%d) start=%d",
                            edgeLevel,
                            startIconLevel,
                            startLevel
                        )
                    )
                end
                if endIconLevel and edgeLevel >= endIconLevel then
                    table.insert(
                        iconViolations,
                        string.format(
                            "edge=%d expected<endIcon(%d) end=%d",
                            edgeLevel,
                            endIconLevel,
                            endLevel
                        )
                    )
                end
            end

            assert(checked > 0, "expected active talent edges after opening class talents")
            return table.concat({
                tostring(checked),
                tostring(heroVisibleButtons),
                tostring(#violations),
                tostring(#iconViolations),
                table.concat(violations, " | "),
                table.concat(iconViolations, " | "),
            }, "::")
            "#,
        )
        .unwrap();

    let mut parts = result.splitn(6, "::");
    let checked = parts
        .next()
        .expect("checked count missing")
        .parse::<u32>()
        .expect("checked count should parse");
    let hero_visible = parts
        .next()
        .expect("hero visible count missing")
        .parse::<u32>()
        .expect("hero visible count should parse");
    let violations = parts
        .next()
        .expect("violation count missing")
        .parse::<u32>()
        .expect("violation count should parse");
    let icon_violations = parts
        .next()
        .expect("icon violation count missing")
        .parse::<u32>()
        .expect("icon violation count should parse");
    let details = parts.next().unwrap_or_default();
    let icon_details = parts.next().unwrap_or_default();

    assert!(checked > 0, "expected at least one checked edge");
    assert!(hero_visible > 0, "expected visible hero talent buttons");
    assert_eq!(
        violations, 0,
        "class talent edges should render below their endpoint buttons; violations={details}"
    );
    assert_eq!(
        icon_violations, 0,
        "class talent edges should render below endpoint icons; violations={icon_details}"
    );
}

#[test]
fn test_button_frame_level_change_relevels_connected_edges_on_update() {
    let env = env_with_protection_ui();
    let result: String = env
        .eval(
            r#"
            local ok = C_Traits.SetSelection(1, 99838, 123361) -- Lightsmith
            assert(ok, "expected deterministic hero subtree selection")
            PlayerSpellsUtil.OpenToClassTalentsTab()

            local frame = PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame
            assert(frame and frame.edgePool, "expected class talents frame and edge pool")
            if frame.OnUpdate then
                frame:OnUpdate()
            end

            local edge = nil
            for candidate in frame.edgePool:EnumerateActive() do
                edge = candidate
                break
            end
            assert(edge ~= nil, "expected at least one active edge")

            local startButton = edge:GetStartButton()
            local endButton = edge:GetEndButton()
            assert(startButton and endButton, "expected edge endpoints")
            local startParent = startButton:GetParent()
            assert(startParent and startParent.GetFrameLevel, "expected start button parent")

            local oldEdgeLevel = edge:GetFrameLevel()
            frame:SetElementFrameLevel(startParent, startParent:GetFrameLevel() + 50)
            frame:UpdateButtonFrameLevel(startButton)
            local expectedEdgeLevel = frame:GetFrameLevelForEdge(startButton, endButton)
            if frame.OnUpdate then
                frame:OnUpdate()
            end
            local newEdgeLevel = edge:GetFrameLevel()
            return table.concat({
                tostring(oldEdgeLevel),
                tostring(newEdgeLevel),
                tostring(expectedEdgeLevel),
            }, "::")
            "#,
        )
        .unwrap();

    let mut parts = result.splitn(3, "::");
    let old_edge_level = parts
        .next()
        .expect("old edge level missing")
        .parse::<i32>()
        .expect("old edge level should parse");
    let new_edge_level = parts
        .next()
        .expect("new edge level missing")
        .parse::<i32>()
        .expect("new edge level should parse");
    let expected_edge_level = parts
        .next()
        .expect("expected edge level missing")
        .parse::<i32>()
        .expect("expected edge level should parse");

    assert_ne!(
        old_edge_level, expected_edge_level,
        "test setup did not change expected edge frame level"
    );
    assert_eq!(
        new_edge_level, expected_edge_level,
        "connected edge should be re-leveled after button frame-level updates"
    );
}

#[test]
fn test_hero_spec_content_spec_image_anchors_to_spec_name() {
    // Regression test for xml_layer_batch two-pass ordering bug.
    // Before fix (commit 1cae5342), xml_layer_batch collected all textures first
    // then appended fontstrings, so SpecImage's SetPoint ran before SpecName FontString
    // existed, causing parent["SpecName"] to be nil and anchoring to the parent frame.
    let env = env_with_protection_ui();
    let result: String = env
        .eval(
            r#"
            PlayerSpellsUtil.OpenToClassTalentsTab()

            local talentFrame = PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame
            assert(talentFrame, "expected TalentsFrame")
            local heroContainer = talentFrame.HeroTalentsContainer
            assert(heroContainer, "expected HeroTalentsContainer")
            local dialog = heroContainer.specSelectionDialog
            assert(dialog, "expected specSelectionDialog")
            local pool = dialog.SpecContentFramePool
            assert(pool, "expected SpecContentFramePool")

            -- Acquire a frame; this instantiates HeroTalentSpecContentTemplate.
            -- The XML anchor on SpecImage uses relativeKey="$parent.SpecName".
            -- With the bug, SpecName FontString was created after SpecImage's SetPoint
            -- executed (two-pass batching), so the anchor fell back to the parent frame.
            local specFrame = pool:Acquire()
            assert(specFrame, "pool:Acquire() returned nil")
            assert(specFrame.SpecName, "expected SpecName FontString on spec content frame")
            assert(specFrame.SpecImage, "expected SpecImage Texture on spec content frame")
            assert(specFrame.Description, "expected Description FontString on spec content frame")
            assert(specFrame.NodesContainer, "expected NodesContainer frame on spec content frame")

            -- Check SpecImage's TOP anchor targets SpecName, not the spec frame itself.
            local point, relativeTo, relativePoint = specFrame.SpecImage:GetPoint(1)
            assert(point == "TOP",
                "expected SpecImage TOP anchor, got: " .. tostring(point))
            assert(relativeTo ~= nil,
                "SpecImage TOP anchor has no relativeTo — anchor was not set")
            assert(relativeTo ~= specFrame,
                "SpecImage is anchored to its parent frame instead of SpecName — two-pass ordering bug")
            assert(relativeTo == specFrame.SpecName,
                "SpecImage should anchor to SpecName, relativeTo=" .. tostring(relativeTo))

            -- Check Description's TOP anchor targets SpecImage.
            local descriptionPoint, descriptionRelativeTo, descriptionRelativePoint = specFrame.Description:GetPoint(1)
            assert(descriptionPoint == "TOP",
                "expected Description TOP anchor, got: " .. tostring(descriptionPoint))
            assert(descriptionRelativeTo ~= nil,
                "Description TOP anchor has no relativeTo — anchor was not set")
            assert(descriptionRelativeTo ~= specFrame,
                "Description is anchored to its parent frame instead of SpecImage")
            assert(descriptionRelativeTo == specFrame.SpecImage,
                "Description should anchor to SpecImage, relativeTo=" .. tostring(descriptionRelativeTo))

            -- Check NodesContainer's TOP anchor targets Description.
            local nodesPoint, nodesRelativeTo = specFrame.NodesContainer:GetPoint(1)
            assert(nodesPoint == "TOP",
                "expected NodesContainer TOP anchor, got: " .. tostring(nodesPoint))
            assert(nodesRelativeTo ~= nil,
                "NodesContainer TOP anchor has no relativeTo — anchor was not set")
            assert(nodesRelativeTo ~= specFrame,
                "NodesContainer is anchored to its parent frame instead of Description")
            assert(nodesRelativeTo == specFrame.Description,
                "NodesContainer should anchor to Description, relativeTo=" .. tostring(nodesRelativeTo))

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}
