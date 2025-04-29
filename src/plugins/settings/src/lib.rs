pub mod resources;

use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	tools::keys::Key,
	traits::{
		handles_asset_resource_loading::HandlesAssetResourceLoading,
		handles_settings::HandlesSettings,
		load_asset::Path,
		thread_safe::ThreadSafe,
	},
};
use resources::key_map::{KeyMap, dto::KeyMapDto as KeyMapDtoGeneric};
use std::marker::PhantomData;

type KeyMapDto = KeyMapDtoGeneric<Key, KeyCode>;

#[derive(Debug, PartialEq)]
pub struct SettingsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading> SettingsPlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesAssetResourceLoading,
{
	pub fn depends_on(_: &TLoading) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading> Plugin for SettingsPlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesAssetResourceLoading,
{
	fn build(&self, app: &mut App) {
		TLoading::register_custom_resource_loading::<KeyMap, KeyMapDto, LoadingEssentialAssets>(
			app,
			Path::from("settings/key_map.keys"),
		);
	}
}

impl<TDependencies> HandlesSettings for SettingsPlugin<TDependencies> {
	type TKeyMap<TKey>
		= KeyMap
	where
		Key: From<TKey>,
		TKey: TryFrom<Key> + Copy,
		KeyCode: From<TKey> + PartialEq;
}
