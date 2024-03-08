use crate::systems::remove_not_owned::remove_not_owned;
use bevy::{
	app::App,
	ecs::{component::Component, schedule::ScheduleLabel},
};

pub trait OwnershipRelation {
	fn manage_ownership<TOwner: Component>(&mut self, label: impl ScheduleLabel) -> &mut Self;
}

impl OwnershipRelation for App {
	fn manage_ownership<TOwner: Component>(&mut self, label: impl ScheduleLabel) -> &mut Self {
		self.add_systems(label, remove_not_owned::<TOwner>);
		self
	}
}
