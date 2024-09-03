use bevy::{
	app::{FixedPostUpdate, FixedPreUpdate, FixedUpdate, Last},
	ecs::schedule::ScheduleLabel,
	prelude::Res,
	time::{Fixed, Time},
};
use std::time::Duration;

macro_rules! label {
	($name:ident, $label:ident) => {
		pub const $name: Label<$label> = Label($label);
	};
}

pub struct Labels;

impl Labels {
	label!(INSTANTIATION, FixedPreUpdate);
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

fn delta<TTime: Default + Sync + Send + 'static>(time: Res<Time<TTime>>) -> Duration {
	time.delta()
}
