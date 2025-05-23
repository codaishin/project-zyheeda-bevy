mod app;

use crate::errors::Error;
use bevy::{ecs::system::EntityCommands, prelude::*};

pub trait Prefab<TDependency>: Component {
	fn insert_prefab_components(&self, entity: &mut EntityCommands) -> Result<(), Error>;
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

pub trait AddPrefabObserver {
	fn add_prefab_observer<TPrefab, TDependencies>(&mut self)
	where
		TPrefab: Prefab<TDependencies>,
		TDependencies: 'static;
}
