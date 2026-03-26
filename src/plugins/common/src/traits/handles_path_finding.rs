use crate::{
	tools::Units,
	traits::{
		accessors::get::View,
		handles_map_generation::GroundPosition,
		system_set_definition::SystemSetDefinition,
	},
};
use bevy::prelude::*;

pub trait HandlesPathFinding: SystemSetDefinition {
	type TComputePath: Component + ComputePath;
	type TComputerRef: Component + View<Entity>;
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
