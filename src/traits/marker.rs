pub mod hand_gun;

use crate::markers::meta::MarkerMeta;

pub trait GetMarkerMeta {
	fn marker() -> MarkerMeta;
}

#[cfg(test)]
pub mod test_tools {
	use super::*;
	use crate::{components::SlotKey, errors::Error};
	use bevy::{
		ecs::{
			component::Component,
			system::{Commands, In},
		},
		prelude::Entity,
	};

	#[derive(Component)]
	pub struct FakeLog {
		pub error: Error,
	}

	pub fn insert_lazy(
		marker: MarkerMeta,
		agent: Entity,
		slot: SlotKey,
	) -> impl FnMut(Commands) -> Result<(), Error> {
		move |mut commands| {
			let mut agent = commands.entity(agent);
			(marker.insert_fn)(&mut agent, slot)
		}
	}

	pub fn remove_lazy(
		marker: MarkerMeta,
		agent: Entity,
		slot: SlotKey,
	) -> impl FnMut(Commands) -> Result<(), Error> {
		move |mut commands| {
			let mut agent = commands.entity(agent);
			(marker.remove_fn)(&mut agent, slot)
		}
	}

	pub fn fake_log(agent: Entity) -> impl FnMut(In<Result<(), Error>>, Commands) {
		move |result, mut commands| {
			let Err(error) = result.0 else {
				return;
			};

			let mut agent = commands.entity(agent);
			agent.insert(FakeLog { error });
		}
	}
}
