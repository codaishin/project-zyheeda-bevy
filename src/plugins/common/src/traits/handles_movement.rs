use crate::{
	tools::{Units, UnitsPerSecond},
	traits::{
		accessors::get::{GetContext, GetContextMut, View, ViewField},
		system_set_definition::SystemSetDefinition,
	},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

pub trait HandlesMovement: SystemSetDefinition {
	type TMovement<'w, 's>: SystemParam
		+ for<'c> GetContext<Movement, TContext<'c>: CurrentMovement>;
	type TMovementMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<ConfiguredMovement, TContext<'c>: ControlMovement>;
	type TMovementConfig<'w, 's>: SystemParam
		+ for<'c> GetContextMut<NotConfiguredMovement, TContext<'c>: ConfigureMovement>;
}

pub type MovementSystemParam<'w, 's, T> = <T as HandlesMovement>::TMovement<'w, 's>;
pub type MovementSystemParamMut<'w, 's, T> = <T as HandlesMovement>::TMovementMut<'w, 's>;
pub type MovementSystemConfigParam<'w, 's, T> = <T as HandlesMovement>::TMovementConfig<'w, 's>;

pub trait ControlMovement: StartMovement + StopMovement + ToggleSpeed + CurrentMovement {}

impl<T> ControlMovement for T where T: StartMovement + StopMovement + ToggleSpeed + CurrentMovement {}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum MovementTarget {
	Dir(Dir3),
	Point(Vec3),
}

impl From<Dir3> for MovementTarget {
	fn from(dir: Dir3) -> Self {
		Self::Dir(dir)
	}
}

impl From<Vec3> for MovementTarget {
	fn from(point: Vec3) -> Self {
		Self::Point(point)
	}
}

impl ViewField for MovementTarget {
	type TValue<'a> = Self;
}

pub trait ConfigureMovement {
	fn configure(&mut self, speed: MovementSpeed, required_clearance: Units, ground_offset: Vec3);
}

impl<T> ConfigureMovement for T
where
	T: DerefMut<Target: ConfigureMovement>,
{
	fn configure(&mut self, speed: MovementSpeed, required_clearance: Units, ground_offset: Vec3) {
		self.deref_mut()
			.configure(speed, required_clearance, ground_offset);
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum MovementSpeed {
	Fixed(UnitsPerSecond),
	Variable([UnitsPerSecond; 2]),
}

impl MovementSpeed {
	pub fn with_fastest_left(self) -> Self {
		match self {
			Self::Variable([slow, fast]) if *slow < *fast => Self::Variable([fast, slow]),
			speed => speed,
		}
	}
}

impl Default for MovementSpeed {
	fn default() -> Self {
		Self::Fixed(UnitsPerSecond::from_u8(1))
	}
}

pub trait StartMovement {
	fn start<T>(&mut self, target: T)
	where
		T: Into<MovementTarget> + 'static;
}

impl<T> StartMovement for T
where
	T: DerefMut<Target: StartMovement>,
{
	fn start<TTarget>(&mut self, target: TTarget)
	where
		TTarget: Into<MovementTarget> + 'static,
	{
		self.deref_mut().start(target)
	}
}

pub trait StopMovement {
	fn stop(&mut self);
}

impl<T> StopMovement for T
where
	T: DerefMut<Target: StopMovement>,
{
	fn stop(&mut self) {
		self.deref_mut().stop();
	}
}

pub trait ToggleSpeed {
	fn toggle_speed(&mut self) -> SpeedToggle;
}

impl<T> ToggleSpeed for T
where
	T: DerefMut<Target: ToggleSpeed>,
{
	fn toggle_speed(&mut self) -> SpeedToggle {
		self.deref_mut().toggle_speed()
	}
}

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub enum SpeedToggle {
	#[default]
	Left,
	Right,
}

impl ViewField for SpeedToggle {
	type TValue<'a> = Self;
}

pub trait CurrentMovement: View<Option<MovementTarget>> + View<SpeedToggle> {}

impl<T> CurrentMovement for T where T: View<Option<MovementTarget>> + View<SpeedToggle> {}

pub struct Movement {
	pub entity: Entity,
}

impl From<Movement> for Entity {
	fn from(Movement { entity }: Movement) -> Self {
		entity
	}
}

pub struct ConfiguredMovement {
	pub entity: Entity,
}

impl From<ConfiguredMovement> for Entity {
	fn from(ConfiguredMovement { entity }: ConfiguredMovement) -> Self {
		entity
	}
}

pub struct NotConfiguredMovement {
	pub entity: Entity,
}

impl From<NotConfiguredMovement> for Entity {
	fn from(NotConfiguredMovement { entity }: NotConfiguredMovement) -> Self {
		entity
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	mod movement_speed {
		use super::*;

		#[test]
		fn with_fastest_left_switch() {
			let movement_speed =
				MovementSpeed::Variable([UnitsPerSecond::from_u8(11), UnitsPerSecond::from_u8(22)]);

			let sorted = movement_speed.with_fastest_left();

			assert_eq!(
				MovementSpeed::Variable([UnitsPerSecond::from_u8(22), UnitsPerSecond::from_u8(11)]),
				sorted,
			);
		}

		#[test]
		fn with_fastest_left_unchanged() {
			let movement_speed =
				MovementSpeed::Variable([UnitsPerSecond::from_u8(22), UnitsPerSecond::from_u8(11)]);

			let sorted = movement_speed.with_fastest_left();

			assert_eq!(
				MovementSpeed::Variable([UnitsPerSecond::from_u8(22), UnitsPerSecond::from_u8(11)]),
				sorted,
			);
		}

		#[test]
		fn with_fastest_left_fixed() {
			let movement_speed = MovementSpeed::Fixed(UnitsPerSecond::from_u8(22));

			let sorted = movement_speed.with_fastest_left();

			assert_eq!(MovementSpeed::Fixed(UnitsPerSecond::from_u8(22)), sorted);
		}
	}
}
