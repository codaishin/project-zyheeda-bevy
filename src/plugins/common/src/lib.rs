pub mod attributes;
pub mod components;
pub mod dto;
pub mod effects;
pub mod error_logger;
pub mod errors;
pub mod observers;
pub mod prelude;
pub mod resources;
pub mod system_params;
pub mod systems;
pub mod tools;
pub mod traits;
pub mod zyheeda_commands;

use crate::{
	components::{
		asset_mesh_name::AssetMeshName,
		child_of_persistent::ChildOfPersistent,
		gltf::GltfLookup,
		lifetime::Lifetime,
		load_world_asset::LoadWorldAsset,
		persistent_entity::PersistentEntity,
		unique_child_model::UniqueChildModel,
	},
	error_logger::GlobalErrorLogger,
	systems::log::OnError,
	traits::{prefab::AddPrefabObserver, register_persistent_entities::RegisterPersistentEntities},
};
use bevy::{prelude::*, time::TimePlugin};
use components::{insert_asset::InsertAsset, model::Model};

pub struct CommonPlugin {
	with_asset_loading: bool,
}

impl CommonPlugin {
	pub fn with_asset_loading(flag: bool) -> Self {
		Self {
			with_asset_loading: flag,
		}
	}
}

impl Plugin for CommonPlugin {
	fn build(&self, app: &mut App) {
		persistent_entities(app);
		life_cycles(app);

		if self.with_asset_loading {
			asset_loading(app);
		}
	}
}

fn persistent_entities(app: &mut App) {
	app.register_persistent_entities();
	app.add_observer(ChildOfPersistent::insert_child_of);
	app.add_systems(Update, PersistentEntity::has_parent::<GlobalErrorLogger>);
}

fn life_cycles(app: &mut App) {
	if !app.is_plugin_added::<TimePlugin>() {
		app.add_plugins(TimePlugin);
	}

	app.add_systems(Update, Lifetime::update::<Virtual>);
}

fn asset_loading(app: &mut App) {
	app.add_prefab_observer::<Model, AssetServer>();
	app.add_prefab_observer::<UniqueChildModel, ()>();
	app.add_observer(LoadWorldAsset::execute.pipe(OnError::log));
	app.add_observer(InsertAsset::<Mesh>::apply);
	app.add_observer(InsertAsset::<StandardMaterial>::apply);
	app.add_observer(AssetMeshName::insert);
	app.add_systems(Update, GltfLookup::trigger_model_load);
	app.add_systems(Update, GlobalErrorLogger::remove_elapsed);
}
