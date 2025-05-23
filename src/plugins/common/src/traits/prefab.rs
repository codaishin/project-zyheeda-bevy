mod app;

use crate::errors::Error;
use bevy::{ecs::system::EntityCommands, prelude::*};

pub trait Prefab<TDependency>: Component {
	fn insert_prefab_components(&self, entity: &mut EntityCommands) -> Result<(), Error>;
}

pub trait AddPrefabObserver {
	fn add_prefab_observer<TPrefab, TDependencies>(&mut self) -> &mut Self
	where
		TPrefab: Prefab<TDependencies>,
		TDependencies: 'static;
}
