use crate::system_param::movement_param::MovementContextMut;
use bevy::prelude::*;
use common::traits::handles_movement::{MovementSpeed, SpeedToggle, ToggleSpeed};

impl<TMotion> ToggleSpeed for MovementContextMut<'_, TMotion>
where
	TMotion: Component,
{
	fn toggle_speed(&mut self) -> SpeedToggle {
		let toggle = match (self.config.speed, self.current_speed.0) {
			(MovementSpeed::Fixed(..), ..) | (.., SpeedToggle::Right) => SpeedToggle::Left,
			(.., SpeedToggle::Left) => SpeedToggle::Right,
		};

		self.current_speed.0 = toggle;

		toggle
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::config::{Config, SpeedIndex},
		system_param::movement_param::MovementParamMut,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::UnitsPerSecond,
		traits::{
			accessors::get::GetContextMut,
			handles_movement::{ConfiguredMovement as MovementMarker, MovementSpeed},
		},
	};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn toggle_speed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Config {
				speed: MovementSpeed::Variable([
					UnitsPerSecond::from_u8(1),
					UnitsPerSecond::from_u8(2),
				]),
				..default()
			})
			.id();

		let toggled =
			app.world_mut()
				.run_system_once(move |mut p: MovementParamMut<_Motion>| {
					let mut ctx =
						MovementParamMut::get_context_mut(&mut p, MovementMarker { entity })
							.unwrap();
					ctx.toggle_speed()
				})?;

		assert_eq!(
			(Some(&SpeedIndex(SpeedToggle::Right)), SpeedToggle::Right),
			(app.world().entity(entity).get::<SpeedIndex>(), toggled),
		);
		Ok(())
	}

	#[test]
	fn toggle_speed_back() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Config {
				speed: MovementSpeed::Variable([
					UnitsPerSecond::from_u8(1),
					UnitsPerSecond::from_u8(2),
				]),
				..default()
			})
			.id();

		let toggled =
			app.world_mut()
				.run_system_once(move |mut p: MovementParamMut<_Motion>| {
					let mut ctx =
						MovementParamMut::get_context_mut(&mut p, MovementMarker { entity })
							.unwrap();
					ctx.toggle_speed();
					ctx.toggle_speed()
				})?;

		assert_eq!(
			(Some(&SpeedIndex(SpeedToggle::Left)), SpeedToggle::Left),
			(app.world().entity(entity).get::<SpeedIndex>(), toggled),
		);
		Ok(())
	}

	#[test]
	fn keep_toggle_left_if_speed_is_fixed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Config {
				speed: MovementSpeed::Fixed(UnitsPerSecond::from_u8(1)),
				..default()
			})
			.id();

		let toggled =
			app.world_mut()
				.run_system_once(move |mut p: MovementParamMut<_Motion>| {
					let mut ctx =
						MovementParamMut::get_context_mut(&mut p, MovementMarker { entity })
							.unwrap();
					ctx.toggle_speed()
				})?;

		assert_eq!(
			(Some(&SpeedIndex(SpeedToggle::Left)), SpeedToggle::Left),
			(app.world().entity(entity).get::<SpeedIndex>(), toggled),
		);
		Ok(())
	}

	#[test]
	fn set_speed_toggle_to_left_when_speed_fixed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				SpeedIndex(SpeedToggle::Right),
				Config {
					speed: MovementSpeed::Fixed(UnitsPerSecond::from_u8(1)),
					..default()
				},
			))
			.id();

		let toggled =
			app.world_mut()
				.run_system_once(move |mut p: MovementParamMut<_Motion>| {
					let mut ctx =
						MovementParamMut::get_context_mut(&mut p, MovementMarker { entity })
							.unwrap();
					ctx.toggle_speed()
				})?;

		assert_eq!(
			(Some(&SpeedIndex(SpeedToggle::Left)), SpeedToggle::Left),
			(app.world().entity(entity).get::<SpeedIndex>(), toggled),
		);
		Ok(())
	}
}
