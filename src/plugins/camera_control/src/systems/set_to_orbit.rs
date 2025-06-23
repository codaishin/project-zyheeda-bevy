use crate::{
	components::orbit_player::{OrbitCenter, OrbitPlayer},
	traits::orbit::{Orbit, Vec2Radians},
};
use bevy::{ecs::query::QuerySingleError, prelude::*};
use common::{errors::UniqueViolation, traits::try_insert_on::TryInsertOn};
use std::f32::consts::PI;

impl<T> SetCameraToOrbit for T {}

pub(crate) trait SetCameraToOrbit {
	fn set_to_orbit<TPlayer>(
		mut commands: Commands,
		cameras: Query<Entity, (With<Self>, Without<OrbitPlayer>)>,
		players: Query<Entity, With<TPlayer>>,
	) -> Result<(), UniqueViolation<TPlayer>>
	where
		Self: Component + Sized,
		TPlayer: Component,
	{
		let player = match players.single() {
			Ok(player) => player,
			Err(QuerySingleError::NoEntities(_)) => {
				return Err(UniqueViolation::found_none_of::<TPlayer>());
			}
			Err(QuerySingleError::MultipleEntities(_)) => {
				return Err(UniqueViolation::found_multiple_of::<TPlayer>());
			}
		};

		for entity in &cameras {
			let mut transform = Transform::from_translation(Vec3::X);
			let mut orbit = OrbitPlayer {
				center: OrbitCenter::from(Vec3::ZERO).with_entity(player),
				distance: 15.,
				sensitivity: 1.,
			};

			orbit.orbit(&mut transform, Vec2Radians::new(-PI / 3., PI / 3.));
			orbit.sensitivity = 0.005;

			commands.try_insert_on(entity, (transform, orbit));
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Player;

	#[derive(Component)]
	struct _Cam;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn set_orbit() -> Result<(), RunSystemError> {
		let mut app = setup();
		let cam = app.world_mut().spawn(_Cam).id();
		let player = app.world_mut().spawn(_Player).id();

		_ = app
			.world_mut()
			.run_system_once(_Cam::set_to_orbit::<_Player>)?;

		let mut transform = Transform::from_translation(Vec3::X);
		let mut expected = OrbitPlayer {
			center: OrbitCenter::from(Vec3::ZERO).with_entity(player),
			distance: 15.,
			sensitivity: 1.,
		};
		expected.orbit(&mut transform, Vec2Radians::new(-PI / 3., PI / 3.));
		expected.sensitivity = 0.005;
		assert_eq!(
			(Some(&expected), Some(&transform)),
			(
				app.world().entity(cam).get::<OrbitPlayer>(),
				app.world().entity(cam).get::<Transform>(),
			)
		);
		Ok(())
	}

	#[test]
	fn do_not_override_existing_orbit() -> Result<(), RunSystemError> {
		let mut app = setup();
		let player = app.world_mut().spawn(_Player).id();
		let preset_orbit = OrbitPlayer {
			center: OrbitCenter::from(Vec3::ZERO).with_entity(player),
			distance: 500.,
			sensitivity: 40.,
		};
		let cam = app.world_mut().spawn((_Cam, preset_orbit)).id();

		_ = app
			.world_mut()
			.run_system_once(_Cam::set_to_orbit::<_Player>)?;

		assert_eq!(
			Some(&preset_orbit),
			app.world().entity(cam).get::<OrbitPlayer>(),
		);
		Ok(())
	}

	#[test]
	fn error_if_player_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Cam);

		let result = app
			.world_mut()
			.run_system_once(_Cam::set_to_orbit::<_Player>)?;

		assert_eq!(Err(UniqueViolation::found_none_of::<_Player>()), result);
		Ok(())
	}

	#[test]
	fn error_if_multiple_players() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Cam);
		app.world_mut().spawn(_Player);
		app.world_mut().spawn(_Player);

		let result = app
			.world_mut()
			.run_system_once(_Cam::set_to_orbit::<_Player>)?;

		assert_eq!(Err(UniqueViolation::found_multiple_of::<_Player>()), result);
		Ok(())
	}
}
