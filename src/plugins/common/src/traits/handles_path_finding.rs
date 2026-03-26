use crate::{
	tools::Units,
	traits::{accessors::get::View, handles_map_generation::GroundPosition},
};
use bevy::prelude::*;

pub trait HandlesPathFinding {
	type TComputePath: Component + ComputePath;
	type TSystemSet: SystemSet;
	type TComputerRef: Component + View<Entity>;

	const SYSTEMS: Self::TSystemSet;
}

pub trait ComputePath {
	type TIter<'a>: Iterator<Item = GroundPosition>
	where
		Self: 'a;

	fn compute_path(
		&self,
		start: Vec3,
		end: Vec3,
		required_clearance: Units,
	) -> Option<Self::TIter<'_>>;
}
