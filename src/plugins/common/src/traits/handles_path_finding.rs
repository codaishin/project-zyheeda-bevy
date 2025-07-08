use crate::{
	tools::Units,
	traits::{
		accessors::get::{GetField, Getter},
		handles_map_generation::Map,
	},
};
use bevy::prelude::*;

pub trait HandlesPathFinding {
	type TComputePath: Component + ComputePath;
	type TPathAgent: Component + Default + Getter<Computer>;
	type TSystemSet: SystemSet;

	const SYSTEMS: Self::TSystemSet;
}

pub trait ComputePath {
	fn compute_path(&self, start: Vec3, end: Vec3, agent_radius: Units) -> Option<Vec<Vec3>>;
}

/// Points to the entity with the [`HandlesPathFinding::TComputePath`] component.
///
/// A blanket implementation for [`Getter<Computer>`] exists, that allows a plugin to reuse
/// a [`MapAgent`](crate::traits::handles_map_generation::HandlesMapGeneration::TMapAgent) to
/// point to a [`HandlesPathFinding::TComputePath`] component.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Computer {
	None,
	Entity(Entity),
}

impl<T> Getter<Computer> for T
where
	T: Getter<Map>,
{
	fn get(&self) -> Computer {
		match Map::get_field(self) {
			Map::None => Computer::None,
			Map::Entity(entity) => Computer::Entity(entity),
		}
	}
}
