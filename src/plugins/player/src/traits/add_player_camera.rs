use crate::{components::player::Player, systems::set_to_orbit::SetCameraToOrbit};
use bevy::prelude::*;

pub(crate) trait AddPlayerCameras {
	fn add_player_cameras(app: &mut App);
}

impl<T> AddPlayerCameras for (T,)
where
	T: Component,
{
	fn add_player_cameras(app: &mut App) {
		app.add_systems(Update, T::set_to_orbit::<Player>);
	}
}

impl<T1, T2> AddPlayerCameras for (T1, T2)
where
	T1: AddPlayerCameras,
	T2: Component,
{
	fn add_player_cameras(app: &mut App) {
		T1::add_player_cameras(app);
		app.add_systems(Update, T2::set_to_orbit::<Player>);
	}
}
