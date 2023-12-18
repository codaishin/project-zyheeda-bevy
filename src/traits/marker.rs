pub mod hand_gun;
pub mod sword;

use crate::markers::meta::MarkerMeta;

pub trait GetMarkerMeta {
	fn marker() -> MarkerMeta;
}

#[cfg(test)]
pub mod test_tools {
	use super::*;
	use crate::{components::SlotKey, errors::Error};
	use bevy::{ecs::system::Commands, prelude::Entity};

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
}
