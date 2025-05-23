use crate::errors::Error;
use bevy::{ecs::system::EntityCommands, prelude::*};

pub trait Prefab<TDependency> {
	fn instantiate_on(&self, entity: &mut EntityCommands) -> Result<(), Error>;
}

pub trait RegisterPrefabWithDependency<TDependency>
where
	TDependency: 'static,
{
	fn register_prefab<TPrefab: Prefab<TDependency> + Component>(self, app: &mut App) -> Self;
}

pub trait RegisterPrefab {
	fn register_prefab<TPrefab: Prefab<()> + Component>(app: &mut App);
	fn with_dependency<TDependency>() -> impl RegisterPrefabWithDependency<TDependency>
	where
		TDependency: 'static;
}
