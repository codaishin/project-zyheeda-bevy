use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::errors::Error;

pub trait AfterInstantiation {
	fn spawn(spawn_fn: impl Fn(&mut ChildBuilder) + Sync + Send + 'static) -> impl Bundle;
}

pub trait Prefab<TDependency> {
	fn instantiate_on<TAfterInstantiation>(&self, entity: &mut EntityCommands) -> Result<(), Error>
	where
		TAfterInstantiation: AfterInstantiation;
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

pub fn sphere(radius: f32) -> Mesh {
	Mesh::from(Sphere { radius })
}
