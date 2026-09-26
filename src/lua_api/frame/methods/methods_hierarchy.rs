//! Pure hierarchy helpers shared by the rilua frame method modules.

use crate::widget::{FrameStrata, WidgetRegistry};

/// A proposed parent creates a cycle when its ancestry already contains the child.
/// The existing hierarchy is assumed acyclic; this checks the new edge before mutation.
pub fn would_create_parent_cycle(
    widgets: &WidgetRegistry,
    child_id: u64,
    new_parent_id: Option<u64>,
) -> bool {
    let mut ancestor_id = new_parent_id;
    while let Some(id) = ancestor_id {
        if id == child_id {
            return true;
        }
        ancestor_id = widgets.get(id).and_then(|frame| frame.parent_id);
    }
    false
}

pub fn reparent_widget(widgets: &mut WidgetRegistry, child_id: u64, new_parent_id: Option<u64>) {
    let old_parent_id = widgets.get(child_id).and_then(|frame| frame.parent_id);
    let same_parent = old_parent_id.is_some() && old_parent_id == new_parent_id;

    if !same_parent {
        detach_from_old_parent(widgets, child_id, old_parent_id);
    }

    let parent_props = read_parent_props(widgets, new_parent_id);
    update_child_parent_link(widgets, child_id, new_parent_id, same_parent, parent_props);
    if !same_parent {
        propagate_strata_level(widgets, child_id);
    }

    let parent_eff_alpha = parent_props.map(|(_, _, alpha, _)| alpha).unwrap_or(1.0);
    let parent_eff_scale = parent_props.map(|(_, _, _, scale)| scale).unwrap_or(1.0);
    widgets.propagate_effective_alpha(child_id, parent_eff_alpha);
    widgets.propagate_effective_scale(child_id, parent_eff_scale);

    if !same_parent {
        attach_to_new_parent(widgets, child_id, new_parent_id);
    }
}

fn detach_from_old_parent(widgets: &mut WidgetRegistry, child_id: u64, old_parent_id: Option<u64>) {
    if let Some(old_pid) = old_parent_id
        && let Some(old_parent) = widgets.get_mut_visual(old_pid)
    {
        old_parent.children.retain(|&id| id != child_id);
        old_parent
            .children_keys
            .retain(|_, value| *value != child_id);
    }
    if let Some(child) = widgets.get_mut_visual(child_id) {
        child.parent_key = None;
    }
}

fn read_parent_props(
    widgets: &WidgetRegistry,
    new_parent_id: Option<u64>,
) -> Option<(FrameStrata, i32, f32, f32)> {
    new_parent_id.and_then(|pid| {
        widgets.get(pid).map(|parent| {
            (
                parent.frame_strata,
                parent.frame_level,
                parent.effective_alpha,
                parent.effective_scale,
            )
        })
    })
}

fn update_child_parent_link(
    widgets: &mut WidgetRegistry,
    child_id: u64,
    new_parent_id: Option<u64>,
    same_parent: bool,
    parent_props: Option<(FrameStrata, i32, f32, f32)>,
) {
    let Some(frame) = widgets.get_mut_visual(child_id) else {
        return;
    };
    frame.parent_id = new_parent_id;
    if same_parent {
        return;
    }
    if let Some((parent_strata, parent_level, _, _)) = parent_props {
        if !frame.has_fixed_frame_strata {
            frame.frame_strata = parent_strata;
        }
        if !frame.has_fixed_frame_level {
            let offset = frame.frame_level_offset.unwrap_or(1);
            frame.frame_level = parent_level + offset;
        }
    }
}

fn attach_to_new_parent(widgets: &mut WidgetRegistry, child_id: u64, new_parent_id: Option<u64>) {
    if let Some(new_pid) = new_parent_id
        && let Some(new_parent) = widgets.get_mut_visual(new_pid)
    {
        new_parent.children.push(child_id);
    }
}

pub fn propagate_strata_level_pub(widgets: &mut WidgetRegistry, root_id: u64) {
    propagate_strata_level(widgets, root_id);
}

fn propagate_strata_level(widgets: &mut WidgetRegistry, root_id: u64) {
    let Some(root) = widgets.get(root_id) else {
        return;
    };
    let root_strata = root.frame_strata;
    let root_level = root.frame_level;
    let mut queue: Vec<(u64, FrameStrata, i32)> = root
        .children
        .iter()
        .map(|&id| (id, root_strata, root_level))
        .collect();

    while let Some((child_id, parent_strata, parent_level)) = queue.pop() {
        let Some(child) = widgets.get_mut_visual(child_id) else {
            continue;
        };
        if !child.has_fixed_frame_strata {
            child.frame_strata = parent_strata;
        }
        if !child.has_fixed_frame_level {
            let offset = child.frame_level_offset.unwrap_or(1);
            child.frame_level = parent_level + offset;
        }
        let child_strata = child.frame_strata;
        let child_level = child.frame_level;
        let children = child.children.clone();
        for &grandchild_id in &children {
            queue.push((grandchild_id, child_strata, child_level));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::reparent_widget;
    use crate::widget::{Frame, WidgetRegistry, WidgetType};

    #[test]
    fn reparent_widget_does_not_duplicate_same_parent_child() {
        let mut widgets = WidgetRegistry::new();
        let parent_id = register_frame(&mut widgets, "Parent", None);
        let child_id = register_frame(&mut widgets, "Child", Some(parent_id));

        widgets
            .get_mut_visual(parent_id)
            .expect("parent should exist")
            .children
            .push(child_id);

        reparent_widget(&mut widgets, child_id, Some(parent_id));

        let parent = widgets.get(parent_id).expect("parent should exist");
        assert_eq!(parent.children, vec![child_id]);
    }

    #[test]
    fn reparent_widget_moves_child_between_parent_lists() {
        let mut widgets = WidgetRegistry::new();
        let old_parent_id = register_frame(&mut widgets, "OldParent", None);
        let new_parent_id = register_frame(&mut widgets, "NewParent", None);
        let child_id = register_frame(&mut widgets, "Child", Some(old_parent_id));

        widgets
            .get_mut_visual(old_parent_id)
            .expect("old parent should exist")
            .children
            .push(child_id);

        reparent_widget(&mut widgets, child_id, Some(new_parent_id));

        assert!(
            widgets
                .get(old_parent_id)
                .expect("old parent should exist")
                .children
                .is_empty()
        );
        assert_eq!(
            widgets
                .get(new_parent_id)
                .expect("new parent should exist")
                .children,
            vec![child_id]
        );
    }

    fn register_frame(widgets: &mut WidgetRegistry, name: &str, parent_id: Option<u64>) -> u64 {
        let frame = Frame::new(WidgetType::Frame, Some(name.to_string()), parent_id);
        let id = frame.id;
        widgets.register(frame);
        id
    }
}
