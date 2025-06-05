pub mod attributes;
pub mod blocker;
pub mod components;
pub mod dto;
pub mod effects;
pub mod errors;
pub mod observers;
pub mod resources;
pub mod states;
pub mod systems;
pub mod test_tools;
pub mod tools;
pub mod traits;

use crate::{
	components::child_of_persistent::ChildOfPersistent,
	traits::register_persistent_entities::RegisterPersistentEntities,
};
use bevy::prelude::*;
use components::{
	AssetModel,
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
		app
			// Asset loading through `AssetModel` component
			.add_systems(Update, FlipHorizontally::system)
			.add_observer(AssetModel::load)
			.add_observer(InsertAsset::<Mesh>::apply)
			.add_observer(InsertAsset::<StandardMaterial>::apply)
			// Handle `PersistentEntity`
			.register_persistent_entities()
			// Point link colliders and interaction targets
			.add_observer(ColliderOfInteractionTarget::link)
			// Handle child of persistent entity
			.add_observer(ChildOfPersistent::insert_child_of)
			// Collect user inputs
			.collect_user_input();
	}
}
