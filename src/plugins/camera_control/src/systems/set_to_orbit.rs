use crate::{
	components::orbit_player::{OrbitCenter, OrbitPlayer},
	traits::orbit::{Orbit, Vec2Radians},
};
use bevy::{ecs::query::QuerySingleError, prelude::*};
use common::{
	components::persistent_entity::PersistentEntity,
	errors::UniqueViolation,
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};
use std::f32::consts::PI;

impl<T> SetCameraToOrbit for T {}

pub(crate) trait SetCameraToOrbit {
	fn set_to_orbit<TPlayer>(
		mut commands: ZyheedaCommands,
		cameras: Query<Entity, (With<Self>, Without<OrbitPlayer>)>,
		players: Query<&PersistentEntity, With<TPlayer>>,
	) -> Result<(), UniqueViolation<(TPlayer, PersistentEntity)>>
	where
		Self: Component + Sized,
		TPlayer: Component,
	{
		if cameras.is_empty() {
			return Ok(());
		}

		let player = match players.single() {
			Ok(player) => player,
			Err(QuerySingleError::NoEntities(_)) => {
				return Err(UniqueViolation::none());
			}
			Err(QuerySingleError::MultipleEntities(_)) => {
				return Err(UniqueViolation::multiple());
			}
		};

		for entity in &cameras {
			let mut transform = Transform::from_translation(Vec3::X);
			let mut orbit = OrbitPlayer {
				center: OrbitCenter::from(Vec3::ZERO).with_entity(*player),
				distance: 15.,
				sensitivity: 1.,
			};

			orbit.orbit(&mut transform, Vec2Radians::new(-PI / 3., PI / 3.));
			orbit.sensitivity = 0.005;

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert((transform, orbit));
			});
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::components::persistent_entity::PersistentEntity;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	#[require(PersistentEntity)]
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
		let player = PersistentEntity::default();
		app.world_mut().spawn((_Player, player));

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
		let player = PersistentEntity::default();
		let preset_orbit = OrbitPlayer {
			center: OrbitCenter::from(Vec3::ZERO).with_entity(player),
			distance: 500.,
			sensitivity: 40.,
		};
		let cam = app.world_mut().spawn((_Cam, preset_orbit)).id();
		app.world_mut().spawn((_Player, player));

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

		assert_eq!(Err(UniqueViolation::none()), result);
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

		assert_eq!(Err(UniqueViolation::multiple()), result);
		Ok(())
	}

	#[test]
	fn do_nothing_if_no_cameras() -> Result<(), RunSystemError> {
		let mut app = setup();

		let result = app
			.world_mut()
			.run_system_once(_Cam::set_to_orbit::<_Player>)?;

		assert_eq!(Ok(()), result);
		Ok(())
	}
}
