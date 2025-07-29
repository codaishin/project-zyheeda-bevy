use crate::{
	components::{is_blocker::Blocker, persistent_entity::PersistentEntity},
	tools::Units,
};
use bevy::prelude::*;

pub trait HandlesInteractions {
	type TSystems: SystemSet;
	type TBlockable: Component + BlockableDefinition;

	const SYSTEMS: Self::TSystems;

	fn beam_from<T>(value: &T) -> impl Bundle
	where
		T: BeamParameters;
}

pub trait BlockableDefinition {
	fn new<T>(blockable_type: BlockableType, blocked_by: T) -> Self
	where
		T: IntoIterator<Item = Blocker>;
}

pub trait BeamParameters {
	fn source(&self) -> PersistentEntity;
	fn target(&self) -> PersistentEntity;
	fn range(&self) -> Units;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BlockableType {
	Beam,
	Fragile,
}
