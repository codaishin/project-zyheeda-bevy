use crate::traits::thread_safe::ThreadSafe;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use std::time::Duration;

macro_rules! label {
	($name:ident, $label:ident) => {
		pub const $name: Label<$label> = Label($label);
	};
}

pub struct Labels;

impl Labels {
	label!(UPDATE, Update);
	label!(PREFAB_INSTANTIATION, FixedPreUpdate);
	label!(PROCESSING, FixedUpdate);
	label!(PROPAGATION, FixedPostUpdate);
	label!(LAST, Last);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Label<T>(T);

impl<T: ScheduleLabel + Clone> Label<T> {
	pub fn label(self) -> impl ScheduleLabel + Clone {
		self.0
	}
}

impl Label<Update> {
	pub fn delta(&self) -> fn(Res<Time<Virtual>>) -> Duration {
		delta::<Virtual>
	}
}

impl Label<FixedUpdate> {
	pub fn delta(&self) -> fn(Res<Time<Fixed>>) -> Duration {
		delta::<Fixed>
	}
}

impl Label<FixedPostUpdate> {
	pub fn delta(&self) -> fn(Res<Time<Fixed>>) -> Duration {
		delta::<Fixed>
	}
}

fn delta<TTime: Default + ThreadSafe>(time: Res<Time<TTime>>) -> Duration {
	time.delta()
}
