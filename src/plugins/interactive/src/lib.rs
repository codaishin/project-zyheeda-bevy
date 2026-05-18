mod assets;
mod components;

use crate::{
	assets::door_meta::DoorMeta,
	components::{
		door::{ApplyDoorAnimations, Door},
		interactive::Interactive,
	},
};
use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	systems::{log::OnError, register_animations::RegisterAnimationsSystem},
	traits::{
		handles_animations::HandlesAnimations,
		handles_custom_assets::HandlesCustomFolderAssets,
		handles_map_generation::HandlesMapGeneration,
		prefab::AddPrefabObserver,
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;

pub struct InteractivePlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TMaps, TAnimations> InteractivePlugin<(TLoading, TMaps, TAnimations)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TAnimations: ThreadSafe + HandlesAnimations,
{
	pub fn from_plugin(_: &TLoading, _: &TMaps, _: &TAnimations) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TMaps, TAnimations> Plugin for InteractivePlugin<(TLoading, TMaps, TAnimations)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TAnimations: ThreadSafe + HandlesAnimations,
{
	fn build(&self, app: &mut App) {
		TLoading::register_custom_folder_assets::<DoorMeta, DoorMeta, LoadingEssentialAssets>(app);

		app.init_asset::<DoorMeta>()
			.add_prefab_observer::<Door, ()>()
			.add_systems(
				Update,
				Interactive::configure_map_prefab::<TMaps::TMapPrefabs>.pipe(OnError::log),
			)
			.add_systems(
				Update,
				ApplyDoorAnimations::register_animations_system::<TAnimations::TAnimationsMut>
					.pipe(OnError::log),
			);
	}
}
