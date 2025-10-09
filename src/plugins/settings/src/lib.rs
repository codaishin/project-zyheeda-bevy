pub mod resources;

mod systems;
mod traits;

use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	systems::log::OnError,
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		handles_asset_resource_loading::HandlesAssetResourceLoading,
		handles_settings::{HandlesSettings, InvalidUserInput},
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
pub struct SettingsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading> SettingsPlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesAssetResourceLoading,
{
	pub fn from_plugin(_: &TLoading) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading> Plugin for SettingsPlugin<TLoading>
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

impl<TDependencies> HandlesSettings for SettingsPlugin<TDependencies> {
	type TKeyMap<TAction>
		= KeyMap
	where
		TAction: Copy + InvalidUserInput + TryFrom<ActionKey> + Into<ActionKey> + Into<UserInput>;
}
