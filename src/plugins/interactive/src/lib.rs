mod assets;
mod components;

use crate::{assets::door_meta::DoorMeta, components::interactive::Interactive};
use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	systems::log::OnError,
	traits::{
		handles_custom_assets::HandlesCustomFolderAssets,
		handles_map_generation::HandlesMapGeneration,
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;

pub struct InteractivePlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TMaps> InteractivePlugin<(TLoading, TMaps)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TMaps: ThreadSafe + HandlesMapGeneration,
{
	pub fn from_plugin(_: &TLoading, _: &TMaps) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TMaps> Plugin for InteractivePlugin<(TLoading, TMaps)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TMaps: ThreadSafe + HandlesMapGeneration,
{
	fn build(&self, app: &mut App) {
		TLoading::register_custom_folder_assets::<DoorMeta, DoorMeta, LoadingEssentialAssets>(app);

		app.init_asset::<DoorMeta>().add_systems(
			Update,
			Interactive::configure_map_prefab::<TMaps::TMapPrefabs>.pipe(OnError::log),
		);
	}
}
