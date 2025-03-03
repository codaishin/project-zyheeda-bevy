use bevy::prelude::*;

pub trait HandlesPathFinding {
	type TComputePath: Component + ComputePath;
}

pub trait ComputePath {
	fn compute_path(&self, start: Vec3, end: Vec3) -> Option<Vec<Vec3>>;
}
