mod components;
mod resources;
mod system_params;
mod systems;
mod traits;

use crate::{
	components::action_key_interaction::ActionKeyInteraction,
	resources::mouse_override::MouseOverride,
	system_params::input::Input,
};
use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	systems::log::OnError,
	tools::action_key::ActionKey,
	traits::{
		handles_asset_resource_loading::HandlesAssetResourceLoading,
		handles_input::{HandlesActionKeyButton, HandlesInput, HandlesInputMut},
		load_asset::Path,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use resources::{
	asset_writer::AssetWriter,
	key_map::{KeyMap, dto::KeyMapDto as KeyMapDtoGeneric},
};
use std::marker::PhantomData;
use systems::save_changes::SaveChanges;

type KeyMapDto = KeyMapDtoGeneric<ActionKey>;

#[derive(Debug, PartialEq)]
pub struct InputPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading> InputPlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesAssetResourceLoading,
{
	pub fn from_plugin(_: &TLoading) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading> Plugin for InputPlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesAssetResourceLoading,
{
	fn build(&self, app: &mut App) {
		let path = Path::from("settings/key_map.keys");
		TLoading::register_custom_resource_loading::<KeyMap, KeyMapDto, LoadingEssentialAssets>(
			app,
			path.clone(),
		);

		app.init_resource::<AssetWriter>()
			.init_resource::<MouseOverride>()
			.add_systems(
				Update,
				(
					KeyMap::save_changes::<KeyMapDto>(path)
						.pipe(OnError::log)
						.run_if(resource_exists::<KeyMap>),
					ActionKeyInteraction::set_mouse_override_from_ui,
					ActionKeyInteraction::set_mouse_override_from_mouse,
					ActionKeyInteraction::set_override_status,
				)
					.in_set(InputSystems),
			);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct InputSystems;

impl<TDependencies> SystemSetDefinition for InputPlugin<TDependencies> {
	type TSystemSet = InputSystems;

	const SYSTEMS: Self::TSystemSet = InputSystems;
}

impl<TDependencies> HandlesActionKeyButton for InputPlugin<TDependencies> {
	type TActionKeyButton = ActionKeyInteraction;
}

impl<TDependencies> HandlesInput for InputPlugin<TDependencies> {
	type TInput<'world, 'state> = Input<'world, 'state, Res<'static, KeyMap>>;
}

impl<TDependencies> HandlesInputMut for InputPlugin<TDependencies> {
	type TInputMut<'world, 'state> = Input<'world, 'state, ResMut<'static, KeyMap>>;
}
