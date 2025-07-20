use crate::{tools::Units, traits::accessors::get::Getter};
use bevy::prelude::*;

pub trait HandlesPathFinding {
	type TComputePath: Component + ComputePath;
	type TSystemSet: SystemSet;
	type TComputerRef: Component + Getter<Entity>;

	const SYSTEMS: Self::TSystemSet;
}

pub trait ComputePath {
	fn compute_path(&self, start: Vec3, end: Vec3, agent_radius: Units) -> Option<Vec<Vec3>>;
}
