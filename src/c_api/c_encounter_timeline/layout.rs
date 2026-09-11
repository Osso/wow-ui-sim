//! Deterministic simulator track policy, not native track thresholds/capacities.
use super::model::{Event, EventState, Timeline};

pub(super) const HIGHLIGHT_TIME: f64 = 5.0;
pub(super) const SORTED_CAPACITY: usize = 3;
pub(super) const INDETERMINATE: u8 = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Position {
    pub track: u8,
    pub index: Option<usize>,
}

pub(super) struct LayoutChanges {
    pub tracks: Vec<(u32, Position)>,
    pub highlights: Vec<u32>,
}

pub(super) fn remaining(event: &Event) -> f64 {
    event.info.duration - event.clock.borrow().elapsed()
}

pub(super) fn sorted_ids(timeline: &Timeline) -> Vec<u32> {
    let mut ids: Vec<_> = timeline.events.keys().copied().collect();
    ids.sort_by(|a, b| {
        remaining(&timeline.events[a])
            .total_cmp(&remaining(&timeline.events[b]))
            .then(a.cmp(b))
    });
    ids
}

fn candidate_track(event: &Event, view: u8) -> u8 {
    if view == 0 || event.state == EventState::Paused {
        return INDETERMINATE;
    }
    let time = remaining(event);
    if time == 0.0 {
        0
    } else if time <= 15.0 {
        1
    } else if time <= 60.0 {
        2
    } else {
        3
    }
}

fn assign_position(event: &Event, view: u8, counts: &mut [usize; 5]) -> Position {
    if event.state.terminal() {
        return Position {
            track: event.track,
            index: event.track_index,
        };
    }
    let mut track = candidate_track(event, view);
    let sorted = track == 0 || track == 3;
    if sorted && counts[track as usize] >= SORTED_CAPACITY {
        track = INDETERMINATE;
    }
    counts[track as usize] += 1;
    let index = matches!(track, 0 | 3).then_some(counts[track as usize]);
    Position { track, index }
}

pub(super) fn refresh(timeline: &mut Timeline) -> LayoutChanges {
    let mut changes = LayoutChanges {
        tracks: Vec::new(),
        highlights: Vec::new(),
    };
    let mut counts = [0; 5];
    for id in sorted_ids(timeline) {
        let event = timeline.events.get_mut(&id).expect("sorted existing event");
        let position = assign_position(event, timeline.view, &mut counts);
        if event.track != position.track || event.track_index != position.index {
            event.track = position.track;
            event.track_index = position.index;
            changes.tracks.push((id, position));
        }
        if should_highlight(event) {
            event.highlighted = true;
            changes.highlights.push(id);
        }
    }
    changes
}

fn should_highlight(event: &Event) -> bool {
    let visible_active = event.state == EventState::Active && event.track != INDETERMINATE;
    visible_active && !event.highlighted && remaining(event) <= HIGHLIGHT_TIME
}

pub(super) fn visible(event: &Event) -> bool {
    !event.state.terminal() && event.track != INDETERMINATE
}
