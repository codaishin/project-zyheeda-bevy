mod app;
mod entity_commands;

use crate::{errors::Error, traits::load_asset::LoadAsset};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

pub trait Prefab<TDependency>: Component {
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: &mut impl LoadAsset,
	) -> Result<(), Error>;
}

pub trait AddPrefabObserver {
	fn add_prefab_observer<TPrefab, TDependencies>(&mut self) -> &mut Self
	where
		TPrefab: Prefab<TDependencies>,
		TDependencies: 'static;
}

pub trait TryInsertIfNew {
	fn try_insert_if_new<TBundle>(&mut self, bundle: TBundle) -> &mut Self
	where
		TBundle: Bundle;
}

pub trait WithChild {
	fn with_child<TBundle>(&mut self, bundle: TBundle) -> &mut Self
	where
		TBundle: Bundle;
}

pub trait WithChildren {
	fn with_children<TFunc>(&mut self, func: TFunc) -> &mut Self
	where
		TFunc: FnOnce(&mut RelatedSpawnerCommands<ChildOf>);
}

pub trait PrefabEntityCommands: TryInsertIfNew + WithChild + WithChildren {}

impl<T> PrefabEntityCommands for T where T: TryInsertIfNew + WithChild + WithChildren {}
