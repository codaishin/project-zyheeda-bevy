mod systems;

use bevy::prelude::*;
use common::{
	labels::Labels,
	resources::Shared,
	systems::log::log_many,
	traits::prefab::{Prefab, RegisterPrefab, RegisterPrefabWithDependency},
};
use std::{any::TypeId, marker::PhantomData};
use systems::instantiate::instantiate;

pub struct PrefabsPlugin;

impl Plugin for PrefabsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<Shared<TypeId, Handle<Mesh>>>()
			.init_resource::<Shared<TypeId, Handle<StandardMaterial>>>();
	}
}

pub struct PrefabsManager<TDependency>(PhantomData<TDependency>);

impl<TDependency> PrefabsManager<TDependency> {
	fn register_prefab<TPrefab>(app: &mut App)
	where
		TDependency: 'static,
		TPrefab: Prefab<TDependency> + Component,
	{
		let instantiate_system = instantiate::<TPrefab, TDependency>;
		app.add_systems(
			Labels::PREFAB_INSTANTIATION.label(),
			instantiate_system.pipe(log_many),
		);
	}
}

impl<TDependency> RegisterPrefabWithDependency<TDependency> for PrefabsManager<TDependency>
where
	TDependency: 'static,
{
	fn register_prefab<TPrefab: Prefab<TDependency> + Component>(self, app: &mut App) -> Self {
		PrefabsManager::<TDependency>::register_prefab::<TPrefab>(app);

		self
	}
}

impl RegisterPrefab for PrefabsPlugin {
	fn register_prefab<TPrefab: Prefab<()> + Component>(app: &mut App) {
		PrefabsManager::<()>::register_prefab::<TPrefab>(app);
	}

	fn with_dependency<TDependency>() -> impl RegisterPrefabWithDependency<TDependency>
	where
		TDependency: 'static,
	{
		PrefabsManager(PhantomData)
	}
}
