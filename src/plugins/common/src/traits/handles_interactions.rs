use crate::{blocker::Blocker, tools::Units};
use bevy::prelude::*;
use std::time::Duration;

pub trait HandlesInteractions {
	fn is_fragile_when_colliding_with<const N: usize>(blockers: [Blocker; N]) -> impl Bundle;
	fn is_ray_interrupted_by<const N: usize>(blockers: [Blocker; N]) -> impl Bundle;
	fn beam_from<T>(value: &T) -> impl Bundle
	where
		T: BeamParameters;
}

pub trait BeamParameters {
	fn source(&self) -> Entity;
	fn target(&self) -> Entity;
	fn range(&self) -> Units;
	fn lifetime(&self) -> Duration;
}
