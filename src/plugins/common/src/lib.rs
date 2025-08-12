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
	components::{child_of_persistent::ChildOfPersistent, life::Life, lifetime::Lifetime},
	states::game_state::GameState,
	traits::{
		register_controlled_state::RegisterControlledState,
		register_persistent_entities::RegisterPersistentEntities,
	},
};
use bevy::prelude::*;
use components::{
	asset_model::AssetModel,
	collider_relationship::ColliderOfInteractionTarget,
	flip::FlipHorizontally,
	insert_asset::InsertAsset,
};

use systems::{
	collect_user_input::collect_user_input_systems::CollectUserInputSystems,
	ui_input_primer::{apply_input::ApplyInput, set_input_state::SetInputState},
};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
	fn build(&self, app: &mut App) {
		game_state(app);
		persistent_entities(app);
		asset_loading(app);
		colliders(app);
		life_cycles(app);
		user_input(app);
	}
}

fn game_state(app: &mut App) {
	app.register_controlled_state::<GameState>();
}

fn persistent_entities(app: &mut App) {
	app.register_persistent_entities();
	app.add_observer(ChildOfPersistent::insert_child_of);
}

fn colliders(app: &mut App) {
	app.add_observer(ColliderOfInteractionTarget::link);
}

fn asset_loading(app: &mut App) {
	app.add_systems(Update, FlipHorizontally::system);
	app.add_observer(AssetModel::load);
	app.add_observer(InsertAsset::<Mesh>::apply);
	app.add_observer(InsertAsset::<StandardMaterial>::apply);
}

fn life_cycles(app: &mut App) {
	app.add_systems(Update, Lifetime::update::<Virtual>);
	app.add_systems(Update, Life::despawn_dead);
}

fn user_input(app: &mut App) {
	app.collect_user_input();
}
