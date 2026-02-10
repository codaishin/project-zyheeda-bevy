use crate::{tools::Units, traits::accessors::get::GetProperty};
use bevy::prelude::*;

pub trait HandlesPathFinding {
	type TComputePath: Component + ComputePath;
	type TSystemSet: SystemSet;
	type TComputerRef: Component + GetProperty<Entity>;

	const SYSTEMS: Self::TSystemSet;
}

pub trait ComputePath {
	type TIter<'a>: Iterator<Item = Vec3>
	where
		Self: 'a;

	fn compute_path(&self, start: Vec3, end: Vec3, agent_radius: Units) -> Option<Self::TIter<'_>>;
}
