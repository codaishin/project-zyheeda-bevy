mod assets;
mod components;
mod observers;
mod system_params;
mod systems;

use crate::{
	assets::door_meta::DoorMeta,
	components::{
		door::{ApplyDoorAnimations, ApplyDoorFrame, Door},
		interactive::Interactive,
	},
	system_params::interactive_param::{InteractiveParam, InteractiveParamMut},
};
use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	systems::{log::OnError, register_animations::RegisterAnimationsSystem},
	traits::{
		after_plugin::AfterPlugin,
		handles_animations::HandlesAnimations,
		handles_custom_assets::HandlesCustomFolderAssets,
		handles_interactive::HandlesInteractive,
		handles_map_generation::HandlesMapGeneration,
		handles_physics::{HandlesInteractiveDetection, HandlesPhysicsConfig},
		prefab::AddPrefabObserver,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;

pub struct InteractivePlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TPhysics, TMaps, TAnimations>
	InteractivePlugin<(TLoading, TPhysics, TMaps, TAnimations)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TPhysics: ThreadSafe + HandlesPhysicsConfig + HandlesInteractiveDetection + SystemSetDefinition,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TAnimations: ThreadSafe + HandlesAnimations,
{
	pub fn from_plugin(_: &TLoading, _: &TPhysics, _: &TMaps, _: &TAnimations) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TPhysics, TMaps, TAnimations> Plugin
	for InteractivePlugin<(TLoading, TPhysics, TMaps, TAnimations)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TPhysics: ThreadSafe + HandlesPhysicsConfig + HandlesInteractiveDetection + SystemSetDefinition,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TAnimations: ThreadSafe + HandlesAnimations,
{
	fn build(&self, app: &mut App) {
		TLoading::register_custom_folder_assets::<DoorMeta, DoorMeta, LoadingEssentialAssets>(app);

		app.init_asset::<DoorMeta>()
			.add_prefab_observer::<Door, ()>()
			.add_observer(Door::animate_open::<TAnimations::TAnimationsMut>)
			.add_observer(Door::animate_close::<TAnimations::TAnimationsMut>)
			.add_systems(
				Startup,
				Interactive::configure_map_prefab::<TMaps::TMapPrefabs>.pipe(OnError::log),
			)
			.add_systems(
				Update,
				(
					ApplyDoorFrame::apply::<TPhysics::TConfigMut>,
					ApplyDoorAnimations::register_animations_system::<TAnimations::TAnimationsMut>
						.pipe(OnError::log),
				)
					.chain()
					.after_plugin(TPhysics::SYSTEMS),
			);
	}
}

impl<TDependencies> HandlesInteractive for InteractivePlugin<TDependencies> {
	type TInteractive = InteractiveParam<'static, 'static>;
	type TInteractiveMut = InteractiveParamMut<'static, 'static>;
}
