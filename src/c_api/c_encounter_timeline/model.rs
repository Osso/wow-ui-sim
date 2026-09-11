//! Script events use the runtime OnUpdate clock. Tracks/queue holds are not modeled here.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum EventState {
    Active = 0,
    Paused = 1,
    Finished = 2,
    Canceled = 3,
}

impl EventState {
    pub(super) fn terminal(self) -> bool {
        matches!(self, Self::Finished | Self::Canceled)
    }
}

#[derive(Clone)]
pub(super) struct EventInfo {
    pub id: u32,
    pub spell_id: u32,
    pub icon: u32,
    pub name: Vec<u8>,
    pub duration: f64,
    pub max_queue_duration: f64,
    pub icons: u32,
    pub severity: u8,
}

pub(super) struct EventClock {
    pub now: Rc<Cell<f64>>,
    pub duration: f64,
    pub elapsed: f64,
    pub running_since: Option<f64>,
}

impl EventClock {
    pub fn elapsed(&self) -> f64 {
        let running = self
            .running_since
            .map_or(0.0, |start| self.now.get() - start);
        (self.elapsed + running).clamp(0.0, self.duration)
    }

    fn freeze(&mut self, finished: bool) {
        self.elapsed = if finished {
            self.duration
        } else {
            self.elapsed()
        };
        self.running_since = None;
    }
}

pub(super) struct Event {
    pub info: EventInfo,
    pub clock: Rc<RefCell<EventClock>>,
    pub state: EventState,
    pub terminal_tick: Option<u64>,
}

#[derive(Default)]
pub(crate) struct Timeline {
    pub(super) now: Rc<Cell<f64>>,
    pub(super) tick: u64,
    pub(super) events: BTreeMap<u32, Event>,
    next_id: u32,
}

impl Timeline {
    pub(super) fn add(&mut self, mut info: EventInfo, paused: bool) -> Option<u32> {
        let id = self.next_id.checked_add(1)?;
        self.next_id = id;
        info.id = id;
        let clock = EventClock {
            now: self.now.clone(),
            duration: info.duration,
            elapsed: 0.0,
            running_since: (!paused).then_some(self.now.get()),
        };
        self.events.insert(
            id,
            Event {
                info,
                clock: Rc::new(RefCell::new(clock)),
                state: if paused {
                    EventState::Paused
                } else {
                    EventState::Active
                },
                terminal_tick: None,
            },
        );
        Some(id)
    }

    pub(super) fn transition(&mut self, id: u32, next: EventState) -> bool {
        let Some(event) = self.events.get_mut(&id) else {
            return false;
        };
        if event.state.terminal() || event.state == next {
            return false;
        }
        event
            .clock
            .borrow_mut()
            .freeze(next == EventState::Finished);
        if next == EventState::Active {
            event.clock.borrow_mut().running_since = Some(self.now.get());
        }
        event.state = next;
        event.terminal_tick = next.terminal().then_some(self.tick);
        true
    }

    pub(super) fn begin_tick(&mut self, elapsed: f64) -> Vec<u32> {
        self.tick += 1;
        self.now.set(self.now.get() + elapsed);
        self.events
            .iter()
            .filter_map(|(&id, event)| {
                let due = event.state == EventState::Active
                    && event.clock.borrow().elapsed() >= event.info.duration;
                due.then_some(id)
            })
            .collect()
    }

    pub(super) fn expired_ids(&self) -> Vec<u32> {
        self.events
            .iter()
            .filter_map(|(&id, event)| {
                event
                    .terminal_tick
                    .filter(|&tick| tick < self.tick)
                    .map(|_| id)
            })
            .collect()
    }
}
