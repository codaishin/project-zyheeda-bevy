use crate::{blocker::Blocker, components::persistent_entity::PersistentEntity, tools::Units};
use bevy::prelude::*;

pub trait HandlesInteractions {
	type TSystems: SystemSet;

	const SYSTEMS: Self::TSystems;

	fn is_fragile_when_colliding_with<TBlockers>(blockers: TBlockers) -> impl Bundle
	where
		TBlockers: IntoIterator<Item = Blocker>;

	fn is_ray_interrupted_by<TBlockers>(blockers: TBlockers) -> impl Bundle
	where
		TBlockers: IntoIterator<Item = Blocker>;

	fn beam_from<T>(value: &T) -> impl Bundle
	where
		T: BeamParameters;
}

pub trait BeamParameters {
	fn source(&self) -> PersistentEntity;
	fn target(&self) -> PersistentEntity;
	fn range(&self) -> Units;
}
