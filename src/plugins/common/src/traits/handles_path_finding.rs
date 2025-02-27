use bevy::prelude::*;

pub trait HandlesPathFinding {
	type TComputePath: Component + ComputePath;
}

pub trait ComputePath {
	type TError;

	fn compute_path(&self, start: Vec3, end: Vec3) -> Result<Vec<Vec3>, Self::TError>;
}
