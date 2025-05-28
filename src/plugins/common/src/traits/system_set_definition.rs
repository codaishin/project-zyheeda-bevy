use bevy::prelude::*;

pub trait SystemSetDefinition {
	type TSystemSet: SystemSet;

	const SYSTEMS: Self::TSystemSet;
}
