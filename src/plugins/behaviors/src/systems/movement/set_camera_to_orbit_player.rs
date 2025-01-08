use crate::{
	components::cam_orbit::{CamOrbit, CamOrbitCenter},
	traits::{Orbit, Vec2Radians},
};
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;
use std::f32::consts::PI;

impl<T> SetCameraToOrbit for T {}

pub(crate) trait SetCameraToOrbit {
	fn set_camera_to_orbit<TPlayer>(
		mut commands: Commands,
		cameras: Query<Entity, With<Self>>,
		players: Query<Entity, Added<TPlayer>>,
	) where
		Self: Component + Sized,
		TPlayer: Component,
	{
		let Ok(player) = players.get_single() else {
			return;
		};

		for entity in &cameras {
			let mut transform = Transform::from_translation(Vec3::X);
			let mut orbit = CamOrbit {
				center: CamOrbitCenter::from(Vec3::ZERO).with_entity(player),
				distance: 15.,
				sensitivity: 1.,
			};

			orbit.orbit(&mut transform, Vec2Radians::new(-PI / 3., PI / 3.));
			orbit.sensitivity = 0.005;

			commands.try_insert_on(entity, (transform, orbit));
		}
	}
}
