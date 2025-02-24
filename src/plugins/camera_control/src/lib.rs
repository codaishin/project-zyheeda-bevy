mod components;
mod systems;
mod traits;

use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	traits::{
		handles_graphics::{FirstPassCamera, WorldCameras},
		handles_player::{HandlesPlayer, PlayerMainCamera},
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

impl<TPlayers, TGraphics> CameraControlPlugin<(TPlayers, TGraphics)>
where
	TPlayers: ThreadSafe + HandlesPlayer + PlayerMainCamera,
	TGraphics: ThreadSafe + WorldCameras + FirstPassCamera,
{
	pub fn depends_on(_: &TPlayers, _: &TGraphics) -> Self {
		Self(PhantomData)
	}
}

impl<TPlayers, TGraphics> Plugin for CameraControlPlugin<(TPlayers, TGraphics)>
where
	TPlayers: ThreadSafe + HandlesPlayer + PlayerMainCamera,
	TGraphics: ThreadSafe + WorldCameras + FirstPassCamera,
{
	fn build(&self, app: &mut App) {
		app.register_required_components::<TGraphics::TFirstPassCamera, TPlayers::TPlayerMainCamera>();
		app.add_systems(
			Update,
			TGraphics::TWorldCameras::set_to_orbit::<TPlayers::TPlayer>,
		)
		.add_systems(
			Update,
			(
				move_on_orbit::<OrbitPlayer>,
				move_with_target::<OrbitPlayer>,
			)
				.run_if(in_state(GameState::Play)),
		);
	}
}
