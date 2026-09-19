//! Finite generated event-name baseline shared by mainline and Forever.

#[path = "valid_events_a.rs"]
mod a;
#[path = "valid_events_a_tail.rs"]
mod a_tail;
#[path = "valid_events_b.rs"]
mod b;
#[path = "valid_events_c.rs"]
mod c;

pub(super) fn contains(name: &str) -> bool {
    let first = name.as_bytes().first().copied().unwrap_or(0);
    if first <= b'G' {
        return a::EVENTS_A.contains(&name) || a_tail::EVENTS_A_TAIL.contains(&name);
    }
    let chunk = if first <= b'P' {
        b::EVENTS_B
    } else {
        c::EVENTS_C
    };
    chunk.contains(&name)
}
