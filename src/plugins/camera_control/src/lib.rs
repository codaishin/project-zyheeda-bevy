mod components;
mod systems;
mod traits;

use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	tools::action_key::camera_key::CameraKey,
	traits::{
		handles_graphics::{FirstPassCamera, WorldCameras},
		handles_player::{HandlesPlayer, PlayerMainCamera},
		handles_settings::HandlesSettings,
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

impl<TSettings, TPlayers, TGraphics> CameraControlPlugin<(TSettings, TPlayers, TGraphics)>
where
	TSettings: ThreadSafe + HandlesSettings,
	TPlayers: ThreadSafe + HandlesPlayer + PlayerMainCamera,
	TGraphics: ThreadSafe + WorldCameras + FirstPassCamera,
{
	pub fn depends_on(_: &TSettings, _: &TPlayers, _: &TGraphics) -> Self {
		Self(PhantomData)
	}
}

impl<TSettings, TPlayers, TGraphics> Plugin
	for CameraControlPlugin<(TSettings, TPlayers, TGraphics)>
where
	TSettings: ThreadSafe + HandlesSettings,
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
				move_on_orbit::<OrbitPlayer, TSettings::TKeyMap<CameraKey>>,
				move_with_target::<OrbitPlayer>,
			)
				.run_if(in_state(GameState::Play)),
		);
	}
}
