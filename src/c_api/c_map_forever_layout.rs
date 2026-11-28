//! WoW Forever's map layout, from that build's own UiMap and UiMapAssignment
//! exports (its uiMap IDs, 1411 on, are not in the retail tables).
//!
//! `C_Map.GetMapInfoAtPosition(parent, x, y)` answers with the child map whose
//! area on the parent contains the point. A child's area on its parent comes
//! from both maps' UiMapAssignment rows on the same world map: the child's
//! whole UI rectangle (0..1) taken to world space through its row, then into
//! the parent's UI space through the parent's row.
//! INFERRED: where children overlap (a city inside its zone's rectangle) the
//! smallest one wins, as the client's map highlights pick the city there.

use sim_data::ui_maps_forever::{self, UiMapAssignment, UiMapInfo};

pub(super) fn ui_map(ui_map_id: i32) -> Option<&'static UiMapInfo> {
    u32::try_from(ui_map_id)
        .ok()
        .and_then(ui_maps_forever::get_ui_map)
}

/// The child map of `parent` at normalized (x, y) on it.
pub(super) fn child_at(parent: i32, x: f64, y: f64) -> Option<(i32, &'static UiMapInfo)> {
    let parent_id = u32::try_from(parent).ok()?;
    let parent_rows = ui_maps_forever::get_ui_map_assignments(parent_id)?;
    let mut best: Option<(f64, u32, &'static UiMapInfo)> = None;
    for (child_id, info) in ui_maps_forever::UI_MAP_DB.entries() {
        if info.parent_map_id != parent_id {
            continue;
        }
        let Some(rect) = rect_on_parent(*child_id, parent_rows) else {
            continue;
        };
        let [left, top, right, bottom] = rect;
        if x < left || x > right || y < top || y > bottom {
            continue;
        }
        let area = (right - left) * (bottom - top);
        let better = best.is_none_or(|(best_area, best_id, _)| {
            area < best_area || (area == best_area && *child_id < best_id)
        });
        if better {
            best = Some((area, *child_id, info));
        }
    }
    best.map(|(_, id, info)| (id as i32, info))
}

/// `[left, top, right, bottom]` of map `child` in its parent's normalized space.
fn rect_on_parent(child: u32, parent_rows: &[UiMapAssignment]) -> Option<[f64; 4]> {
    let child_rows = ui_maps_forever::get_ui_map_assignments(child)?;
    child_rows.iter().find_map(|row| {
        let parent_row = parent_rows.iter().find(|p| p.map_id == row.map_id)?;
        let (ax, ay) = ui_to_parent(row, parent_row, 0.0, 0.0)?;
        let (bx, by) = ui_to_parent(row, parent_row, 1.0, 1.0)?;
        Some([ax.min(bx), ay.min(by), ax.max(bx), ay.max(by)])
    })
}

/// A point of `row`'s map in its UI space, in `parent_row`'s map's UI space.
fn ui_to_parent(
    row: &UiMapAssignment,
    parent_row: &UiMapAssignment,
    u: f64,
    v: f64,
) -> Option<(f64, f64)> {
    let (wx, wy) = ui_to_world(row, u, v)?;
    world_to_ui(parent_row, wx, wy)
}

// UI x runs along world -Y, UI y along world -X, between the row's region
// corners (as `project_retail_assignment` in c_map.rs).
fn ui_to_world(row: &UiMapAssignment, u: f64, v: f64) -> Option<(f64, f64)> {
    let local_x = (u - f64::from(row.ui_min[0])) / span(row.ui_min[0], row.ui_max[0])?;
    let local_y = (v - f64::from(row.ui_min[1])) / span(row.ui_min[1], row.ui_max[1])?;
    let world_x = f64::from(row.region_max[0])
        + (f64::from(row.region_min[0]) - f64::from(row.region_max[0])) * local_y;
    let world_y = f64::from(row.region_max[1])
        + (f64::from(row.region_min[1]) - f64::from(row.region_max[1])) * local_x;
    Some((world_x, world_y))
}

fn world_to_ui(row: &UiMapAssignment, wx: f64, wy: f64) -> Option<(f64, f64)> {
    let local_y = (f64::from(row.region_max[0]) - wx) / span(row.region_min[0], row.region_max[0])?;
    let local_x = (f64::from(row.region_max[1]) - wy) / span(row.region_min[1], row.region_max[1])?;
    let u =
        f64::from(row.ui_min[0]) + local_x * (f64::from(row.ui_max[0]) - f64::from(row.ui_min[0]));
    let v =
        f64::from(row.ui_min[1]) + local_y * (f64::from(row.ui_max[1]) - f64::from(row.ui_min[1]));
    Some((u, v))
}

fn span(min: f32, max: f32) -> Option<f64> {
    let span = f64::from(max) - f64::from(min);
    (span.abs() > f64::from(f32::EPSILON)).then_some(span)
}

#[cfg(test)]
mod tests {
    use super::child_at;

    #[test]
    fn eastern_kingdoms_points_find_their_zone_or_city() {
        // Stormwind City's middle, inside Elwynn Forest's rectangle too: the city.
        let (city, _) = child_at(1415, 0.4303, 0.6836).expect("a map at Stormwind");
        assert_eq!(city, 1453);
        // Goldshire, in Elwynn Forest away from the city.
        let (zone, _) = child_at(1415, 0.4527, 0.7215).expect("a map at Goldshire");
        assert_eq!(zone, 1429);
    }
}
