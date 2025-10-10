mod resources;
mod system_params;
mod systems;
mod traits;

use crate::system_params::input::{Input, InputMut};
use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	systems::log::OnError,
	tools::action_key::ActionKey,
	traits::{
		handles_asset_resource_loading::HandlesAssetResourceLoading,
		handles_input::{HandlesInput, HandlesInputMut},
		load_asset::Path,
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

		app.init_resource::<AssetWriter>().add_systems(
			Update,
			KeyMap::save_changes::<KeyMapDto>(path)
				.pipe(OnError::log)
				.run_if(resource_exists::<KeyMap>),
		);
	}
}

impl<TDependencies> HandlesInput for InputPlugin<TDependencies> {
	type TKeyMap = KeyMap;
	type TInput<'world, 'state> = Input<'world>;
}

impl<TDependencies> HandlesInputMut for InputPlugin<TDependencies> {
	type TInputMut<'world, 'state> = InputMut<'world>;
}
