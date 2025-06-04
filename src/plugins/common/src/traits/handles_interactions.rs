use crate::{blocker::Blocker, components::persistent_entity::PersistentEntity, tools::Units};
use bevy::prelude::*;
use std::time::Duration;

pub trait HandlesInteractions {
	fn is_fragile_when_colliding_with(blockers: &[Blocker]) -> impl Bundle;
	fn is_ray_interrupted_by(blockers: &[Blocker]) -> impl Bundle;
	fn beam_from<T>(value: &T) -> impl Bundle
	where
		T: BeamParameters;
}

pub trait BeamParameters {
	fn source(&self) -> PersistentEntity;
	fn target(&self) -> PersistentEntity;
	fn range(&self) -> Units;
	fn lifetime(&self) -> Duration;
}
