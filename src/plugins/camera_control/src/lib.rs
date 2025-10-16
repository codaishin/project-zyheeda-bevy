mod components;
mod systems;
mod traits;

use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	systems::log::OnError,
	traits::{
		handles_graphics::{FirstPassCamera, WorldCameras},
		handles_input::{HandlesInput, InputSystemParam},
		handles_player::{HandlesPlayer, PlayerMainCamera},
		handles_saving::HandlesSaving,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::orbit_player::OrbitPlayer;
use std::marker::PhantomData;
use systems::{
	move_on_orbit::move_on_orbit,
	move_with_target::move_with_target,
	set_to_orbit::SetCameraToOrbit,
};

pub struct CameraControlPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TInput, TSavegame, TPlayers, TGraphics>
	CameraControlPlugin<(TInput, TSavegame, TPlayers, TGraphics)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSavegame: ThreadSafe + HandlesSaving,
	TPlayers: ThreadSafe + HandlesPlayer + PlayerMainCamera,
	TGraphics: ThreadSafe + WorldCameras + FirstPassCamera,
{
	pub fn from_plugins(_: &TInput, _: &TSavegame, _: &TPlayers, _: &TGraphics) -> Self {
		Self(PhantomData)
	}
}

impl<TInput, TSavegame, TPlayers, TGraphics> Plugin
	for CameraControlPlugin<(TInput, TSavegame, TPlayers, TGraphics)>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSavegame: ThreadSafe + HandlesSaving,
	TPlayers: ThreadSafe + HandlesPlayer + PlayerMainCamera,
	TGraphics: ThreadSafe + WorldCameras + FirstPassCamera,
{
	fn build(&self, app: &mut App) {
		TSavegame::register_savable_component::<OrbitPlayer>(app);

		app.register_required_components::<TGraphics::TFirstPassCamera, TPlayers::TPlayerMainCamera>();
		app.add_systems(
			Update,
			(
				TGraphics::TWorldCameras::set_to_orbit::<TPlayers::TPlayer>.pipe(OnError::log),
				move_on_orbit::<OrbitPlayer, InputSystemParam<TInput>>,
				move_with_target::<OrbitPlayer>,
			)
				.chain()
				.after(TInput::SYSTEMS)
				.run_if(in_state(GameState::Play)),
		);
	}
}
