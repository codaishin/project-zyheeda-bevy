mod behavior_schedule;
mod mouse_event;
mod move_event;

pub trait New {
	fn new() -> Self;
}

pub trait New1<T> {
	fn new(value: T) -> Self;
}
