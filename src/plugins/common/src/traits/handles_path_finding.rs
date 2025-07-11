use crate::{tools::Units, traits::accessors::get::Getter};
use bevy::{ecs::query::QueryFilter, prelude::*};
use std::collections::HashMap;

pub trait HandlesPathFinding {
	type TComputePath: Component + ComputePath;
	type TSystemSet: SystemSet;

	const SYSTEMS: Self::TSystemSet;

	type TComputerRef: Getter<Entity>;

	fn computer_mapping_of<TFilter>() -> impl IntoSystem<(), HashMap<Entity, Self::TComputerRef>, ()>
	where
		TFilter: QueryFilter + 'static;
}

pub trait ComputePath {
	fn compute_path(&self, start: Vec3, end: Vec3, agent_radius: Units) -> Option<Vec<Vec3>>;
}
