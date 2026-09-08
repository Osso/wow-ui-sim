use crate::iced_app::app::App;
use rustc_hash::FxHashSet;

impl App {
    /// Hit test to find frame under cursor (uses cached rects from render pass).
    ///
    /// After finding the topmost frame at the cursor position (highest strata/level),
    /// drills down through child frames to find the deepest mouse-enabled descendant.
    /// This matches WoW behavior where child frames always receive clicks over parents,
    /// regardless of the parent's frame level. Descent walks through any visible child
    /// whose visual rect contains the point — even non-mouse-enabled containers — so
    /// hittable descendants of a transparent panning/scroll container are still reachable.
    pub(crate) fn hit_test(&self, pos: iced::Point) -> Option<u64> {
        self.apply_hit_grid_changes();
        let cache = self.cached_hittable.borrow();
        let grid = cache.as_ref()?;

        let env = self.env.borrow();
        let state = env.state().borrow();
        let initial_id = grid.topmost_matching_at(pos, |_| true)?;
        deepest_hover_target_through_visible_children(&state.widgets, grid, initial_id, pos)
    }

    pub(crate) fn hit_test_mouse_button(
        &self,
        pos: iced::Point,
        button_name: &str,
        down: bool,
    ) -> Option<u64> {
        self.apply_hit_grid_changes();
        let cache = self.cached_hittable.borrow();
        let grid = cache.as_ref()?;

        let env = self.env.borrow();
        let state = env.state().borrow();
        let initial_id = grid.topmost_matching_at(pos, |id| {
            deepest_click_target_through_visible_children(
                &state.widgets,
                &env,
                grid,
                id,
                pos,
                button_name,
                down,
            )
            .is_some()
        })?;
        deepest_click_target_through_visible_children(
            &state.widgets,
            &env,
            grid,
            initial_id,
            pos,
            button_name,
            down,
        )
    }
}

fn deepest_hover_target_through_visible_children(
    widgets: &crate::widget::WidgetRegistry,
    grid: &crate::iced_app::hit_grid::HitGrid,
    frame_id: u64,
    pos: iced::Point,
) -> Option<u64> {
    deepest_target_through_visible_children(widgets, grid, frame_id, pos, |frame, _| {
        frame.mouse_enabled || matches!(frame.widget_type, crate::widget::WidgetType::EditBox)
    })
}

fn deepest_click_target_through_visible_children(
    widgets: &crate::widget::WidgetRegistry,
    env: &crate::lua_api::WowLuaEnv,
    grid: &crate::iced_app::hit_grid::HitGrid,
    frame_id: u64,
    pos: iced::Point,
    button_name: &str,
    down: bool,
) -> Option<u64> {
    deepest_target_through_visible_children(widgets, grid, frame_id, pos, |frame, id| {
        frame_has_mouse_button_action(frame, env, id, button_name, down)
    })
}

fn deepest_target_through_visible_children(
    widgets: &crate::widget::WidgetRegistry,
    _grid: &crate::iced_app::hit_grid::HitGrid,
    frame_id: u64,
    pos: iced::Point,
    accepts_frame: impl Fn(&crate::widget::Frame, u64) -> bool,
) -> Option<u64> {
    let mut stack = vec![HitVisit::Enter(frame_id)];
    let mut seen = FxHashSet::default();

    while let Some(visit) = stack.pop() {
        match visit {
            HitVisit::Enter(current_id) => {
                if !seen.insert(current_id) {
                    continue;
                }
                let Some(frame) = widgets.get(current_id) else {
                    continue;
                };
                stack.push(HitVisit::Check(current_id));
                let mut child_ids = visible_descendants_at_point_by_z_order(widgets, frame, pos);
                while let Some(child_id) = child_ids.pop() {
                    stack.push(HitVisit::Enter(child_id));
                }
            }
            HitVisit::Check(current_id) => {
                let Some(frame) = widgets.get(current_id) else {
                    continue;
                };
                if target_contains_point(widgets, current_id, pos)
                    && accepts_frame(frame, current_id)
                {
                    return Some(current_id);
                }
            }
        }
    }

    None
}

enum HitVisit {
    Enter(u64),
    Check(u64),
}

fn target_contains_point(
    widgets: &crate::widget::WidgetRegistry,
    frame_id: u64,
    pos: iced::Point,
) -> bool {
    widgets
        .get(frame_id)
        .is_some_and(|frame| frame_visually_contains(widgets, frame_id, frame, pos))
        && ancestor_clips_contain(widgets, frame_id, pos)
}

fn visible_descendants_at_point_by_z_order(
    widgets: &crate::widget::WidgetRegistry,
    frame: &crate::widget::Frame,
    pos: iced::Point,
) -> Vec<u64> {
    let mut child_ids: Vec<_> = frame
        .children
        .iter()
        .copied()
        .filter(|&child_id| {
            widgets
                .get(child_id)
                .is_some_and(|child| child_visually_contains(child, pos))
                && direct_parent_clip_contains(frame, child_id, pos)
        })
        .collect();
    child_ids.sort_by_key(|&child_id| {
        widgets
            .get(child_id)
            .map(|child| hit_sort_key(child, child_id))
            .unwrap_or_default()
    });
    child_ids.reverse();
    child_ids
}

/// Whether the child's visual bounds (visible+layout_rect, scaled by UI_SCALE)
/// contain the screen-space point. Used for hit-test descent through any
/// visible frame, regardless of mouse-enabled status.
fn child_visually_contains(child: &crate::widget::Frame, pos: iced::Point) -> bool {
    if !child.visible {
        return false;
    }
    rect_contains_screen_point(child.layout_rect, pos)
}

fn frame_visually_contains(
    widgets: &crate::widget::WidgetRegistry,
    frame_id: u64,
    frame: &crate::widget::Frame,
    pos: iced::Point,
) -> bool {
    child_visually_contains(frame, pos) && ancestor_clips_contain(widgets, frame_id, pos)
}

fn direct_parent_clip_contains(
    parent: &crate::widget::Frame,
    child_id: u64,
    pos: iced::Point,
) -> bool {
    !parent_clips_child(parent, child_id) || rect_contains_screen_point(parent.layout_rect, pos)
}

fn ancestor_clips_contain(
    widgets: &crate::widget::WidgetRegistry,
    frame_id: u64,
    pos: iced::Point,
) -> bool {
    let mut current_id = frame_id;
    let mut seen = FxHashSet::default();
    while let Some(parent_id) = widgets.get(current_id).and_then(|frame| frame.parent_id) {
        if !seen.insert(current_id) || seen.contains(&parent_id) {
            return false;
        }
        let Some(parent) = widgets.get(parent_id) else {
            break;
        };
        if parent_clips_child(parent, current_id)
            && !rect_contains_screen_point(parent.layout_rect, pos)
        {
            return false;
        }
        current_id = parent_id;
    }
    true
}

fn parent_clips_child(parent: &crate::widget::Frame, child_id: u64) -> bool {
    parent.clips_children
        || (matches!(parent.widget_type, crate::widget::WidgetType::ScrollFrame)
            && parent.scroll_child_id == Some(child_id))
}

fn rect_contains_screen_point(rect: Option<crate::LayoutRect>, pos: iced::Point) -> bool {
    let Some(rect) = rect else {
        return false;
    };
    let scale = crate::render::texture::UI_SCALE;
    let x = rect.x * scale;
    let y = rect.y * scale;
    let w = rect.width * scale;
    let h = rect.height * scale;
    pos.x >= x && pos.x < x + w && pos.y >= y && pos.y < y + h
}

fn hit_sort_key(
    frame: &crate::widget::Frame,
    id: u64,
) -> (crate::widget::FrameStrata, i32, i32, u64) {
    (frame.frame_strata, frame.frame_level, frame.raise_order, id)
}

fn frame_has_mouse_button_action(
    frame: &crate::widget::Frame,
    env: &crate::lua_api::WowLuaEnv,
    frame_id: u64,
    button_name: &str,
    down: bool,
) -> bool {
    if !crate::iced_app::frame_collect::frame_accepts_mouse_button(frame, button_name) {
        return false;
    }

    if crate::iced_app::mouse::frame_click_registration_accepts_button(frame, button_name) {
        return true;
    }

    if down {
        return true;
    }

    let script = if down { "OnMouseDown" } else { "OnMouseUp" };
    crate::iced_app::frame_collect::frame_mouse_registration_matches(frame, button_name, down)
        && env.has_script_handler(frame_id, script)
}

#[cfg(test)]
mod tests {
    use super::{
        deepest_click_target_through_visible_children,
        deepest_hover_target_through_visible_children, frame_visually_contains,
    };
    use crate::iced_app::hit_grid::HitGrid;
    use crate::widget::{Frame, WidgetRegistry, WidgetType};
    use iced::{Point, Rectangle, Size};

    fn register_frame(
        registry: &mut WidgetRegistry,
        widget_type: WidgetType,
        parent_id: Option<u64>,
        rect: crate::LayoutRect,
    ) -> u64 {
        let mut frame = Frame::new(widget_type, None, parent_id);
        frame.visible = true;
        frame.effective_alpha = 1.0;
        frame.layout_rect = Some(rect);
        let id = frame.id;
        registry.register(frame);
        if let Some(parent_id) = parent_id {
            registry.add_child(parent_id, id);
        }
        id
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> crate::LayoutRect {
        crate::LayoutRect {
            x,
            y,
            width,
            height,
        }
    }

    fn set_mouse_enabled(registry: &mut WidgetRegistry, frame_id: u64) {
        registry
            .get_mut_visual(frame_id)
            .expect("frame should exist")
            .mouse_enabled = true;
    }

    fn grid_for_all_frames(registry: &WidgetRegistry) -> HitGrid {
        let mut frames: Vec<_> = registry
            .iter_ids()
            .filter_map(|id| {
                let frame = registry.get(id)?;
                let rect = frame.layout_rect?;
                Some((
                    id,
                    Rectangle::new(
                        Point::new(rect.x, rect.y),
                        Size::new(rect.width, rect.height),
                    ),
                    (frame.frame_strata, frame.frame_level, frame.raise_order, id),
                ))
            })
            .collect();
        frames.sort_by_key(|entry| entry.2);
        let ordered = frames
            .into_iter()
            .enumerate()
            .map(|(rank, (id, rect, _))| (id, rect, rank))
            .collect();
        HitGrid::new(ordered, 200.0, 200.0)
    }

    #[test]
    fn scroll_child_descendant_outside_scroll_frame_is_not_visually_hittable() {
        let mut registry = WidgetRegistry::new();
        let scroll_frame = register_frame(
            &mut registry,
            WidgetType::ScrollFrame,
            None,
            rect(0.0, 0.0, 100.0, 100.0),
        );
        let scroll_child = register_frame(
            &mut registry,
            WidgetType::Frame,
            Some(scroll_frame),
            rect(0.0, 0.0, 100.0, 300.0),
        );
        let button = register_frame(
            &mut registry,
            WidgetType::Button,
            Some(scroll_child),
            rect(10.0, 150.0, 80.0, 20.0),
        );
        registry
            .get_mut_visual(scroll_frame)
            .expect("scroll frame should exist")
            .scroll_child_id = Some(scroll_child);

        let button_frame = registry.get(button).expect("button should exist");
        assert!(
            !frame_visually_contains(
                &registry,
                button,
                button_frame,
                iced::Point::new(20.0, 160.0)
            ),
            "scroll-frame clipped descendants below the viewport should not be hit-test candidates"
        );
    }

    #[test]
    fn scroll_child_descendant_inside_scroll_frame_stays_visually_hittable() {
        let mut registry = WidgetRegistry::new();
        let scroll_frame = register_frame(
            &mut registry,
            WidgetType::ScrollFrame,
            None,
            rect(0.0, 0.0, 100.0, 100.0),
        );
        let scroll_child = register_frame(
            &mut registry,
            WidgetType::Frame,
            Some(scroll_frame),
            rect(0.0, 0.0, 100.0, 300.0),
        );
        let button = register_frame(
            &mut registry,
            WidgetType::Button,
            Some(scroll_child),
            rect(10.0, 50.0, 80.0, 20.0),
        );
        registry
            .get_mut_visual(scroll_frame)
            .expect("scroll frame should exist")
            .scroll_child_id = Some(scroll_child);

        let button_frame = registry.get(button).expect("button should exist");
        assert!(
            frame_visually_contains(
                &registry,
                button,
                button_frame,
                iced::Point::new(20.0, 60.0)
            ),
            "scroll-frame clipped descendants inside the viewport should remain hittable"
        );
    }

    #[test]
    fn click_hit_testing_handles_deep_visible_child_chain_iteratively() {
        let env = crate::lua_api::WowLuaEnv::new().expect("Lua env should initialize");
        let mut registry = WidgetRegistry::new();
        let hit_rect = rect(0.0, 0.0, 100.0, 100.0);
        let root = register_frame(&mut registry, WidgetType::Frame, None, hit_rect);
        let mut parent = root;

        for _ in 0..20_000 {
            parent = register_frame(&mut registry, WidgetType::Frame, Some(parent), hit_rect);
        }
        let leaf = register_frame(&mut registry, WidgetType::Button, Some(parent), hit_rect);
        set_mouse_enabled(&mut registry, leaf);
        let grid = grid_for_all_frames(&registry);

        assert_eq!(
            deepest_click_target_through_visible_children(
                &registry,
                &env,
                &grid,
                root,
                Point::new(50.0, 50.0),
                "LeftButton",
                true
            ),
            Some(leaf)
        );
    }

    #[test]
    fn hover_hit_testing_handles_deep_visible_child_chain_iteratively() {
        let mut registry = WidgetRegistry::new();
        let hit_rect = rect(0.0, 0.0, 100.0, 100.0);
        let root = register_frame(&mut registry, WidgetType::Frame, None, hit_rect);
        set_mouse_enabled(&mut registry, root);
        let mut parent = root;

        for _ in 0..20_000 {
            parent = register_frame(&mut registry, WidgetType::Frame, Some(parent), hit_rect);
            set_mouse_enabled(&mut registry, parent);
        }
        let grid = grid_for_all_frames(&registry);

        assert_eq!(
            deepest_hover_target_through_visible_children(
                &registry,
                &grid,
                root,
                Point::new(50.0, 50.0)
            ),
            Some(parent)
        );
    }
}
