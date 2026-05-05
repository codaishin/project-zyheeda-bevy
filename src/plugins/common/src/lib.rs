pub mod attributes;
pub mod components;
pub mod dto;
pub mod effects;
pub mod errors;
pub mod observers;
pub mod resources;
pub mod states;
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
		load_model::LoadModel,
	},
	states::game_state::GameState,
	systems::log::OnError,
	traits::{
		prefab::AddPrefabObserver,
		register_controlled_state::RegisterControlledState,
		register_persistent_entities::RegisterPersistentEntities,
	},
};
use bevy::prelude::*;
use components::{asset_model::AssetModel, insert_asset::InsertAsset};

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
		game_state(app);
		persistent_entities(app);
		life_cycles(app);

		if self.with_asset_loading {
			asset_loading(app);
		}
	}
}

fn game_state(app: &mut App) {
	app.register_controlled_state::<GameState>();
}

fn persistent_entities(app: &mut App) {
	app.register_persistent_entities();
	app.add_observer(ChildOfPersistent::insert_child_of);
}

fn life_cycles(app: &mut App) {
	app.add_systems(Update, Lifetime::update::<Virtual>);
}

fn asset_loading(app: &mut App) {
	app.add_prefab_observer::<AssetModel, AssetServer>();
	app.add_observer(LoadModel::execute.pipe(OnError::log));
	app.add_observer(InsertAsset::<Mesh>::apply);
	app.add_observer(InsertAsset::<StandardMaterial>::apply);
	app.add_observer(AssetMeshName::insert);
	app.add_systems(Update, GltfLookup::trigger_model_load);
}
