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
	AssetModel,
	collider_relationship::ColliderOfInteractionTarget,
	flip::FlipHorizontally,
	insert_asset::InsertAsset,
	object_id::ObjectId,
};
use systems::{
	collect_user_input::collect_user_input_systems::CollectUserInputSystems,
	ui_input_primer::{apply_input::ApplyInput, set_input_state::SetInputState},
};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
	fn build(&self, app: &mut App) {
		app
			// Asset loading through `AssetModel` component
			.add_systems(Update, FlipHorizontally::system)
			.add_observer(AssetModel::load)
			.add_observer(InsertAsset::<Mesh>::apply)
			.add_observer(InsertAsset::<StandardMaterial>::apply)
			// Handling `ObjectId`s (mapping `Entity`s for persistent object references)
			.add_observer(ObjectId::update)
			// Point link colliders and interaction targets
			.add_observer(ColliderOfInteractionTarget::link)
			// Collect user inputs
			.collect_user_input();
	}
}
