pub mod attributes;
pub mod blocker;
pub mod components;
pub mod dto;
pub mod effects;
pub mod errors;
pub mod labels;
pub mod observers;
pub mod resources;
pub mod states;
pub mod systems;
pub mod test_tools;
pub mod tools;
pub mod traits;

use bevy::prelude::*;
use components::{
	collider_relationship::ColliderOfInteractionTarget,
	flip::FlipHorizontally,
	insert_asset::InsertAsset,
	object_id::ObjectId,
};
use labels::Labels;
use systems::{
	collect_user_input::collect_user_input_systems::CollectUserInputSystems,
	load_asset_model::load_asset_model,
	ui_input_primer::{apply_input::ApplyInput, set_input_state::SetInputState},
};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
	fn build(&self, app: &mut App) {
		let on_instantiate = || Labels::PREFAB_INSTANTIATION.label();

		app
			// Asset loading through `AssetModel` component
			.add_systems(First, load_asset_model::<AssetServer>)
			.add_systems(Update, FlipHorizontally::system)
			.add_systems(
				on_instantiate(),
				(
					InsertAsset::<Mesh>::system,
					InsertAsset::<StandardMaterial>::system,
				),
			)
			// Handling `ObjectId`s (mapping `Entity`s for persistent object references)
			.add_systems(on_instantiate(), ObjectId::update_entity)
			// Collect user inputs
			.collect_user_input()
			// Point link colliders and interaction targets
			.add_observer(ColliderOfInteractionTarget::link);
	}
}
