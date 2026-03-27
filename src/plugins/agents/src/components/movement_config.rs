use bevy::prelude::*;
use common::{
	tools::{Units, UnitsPerSecond, speed::Speed},
	traits::{accessors::get::View, handles_movement::RequiredClearance},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[savable_component(id = "movement config")]
pub struct MovementConfig {
	pub(crate) required_clearance: Units,
	pub(crate) speed: MovementSpeed,
}

#[cfg(test)]
impl MovementConfig {
	pub(crate) fn with_speed(speed: impl Into<MovementSpeed>) -> Self {
		Self {
			speed: speed.into(),
			..default()
		}
	}
}

impl Default for MovementConfig {
	fn default() -> Self {
		Self {
			required_clearance: Units::from(0.5),
			speed: MovementSpeed::default(),
		}
	}
}

impl View<Speed> for MovementConfig {
	fn view(&self) -> UnitsPerSecond {
		match &self.speed {
			MovementSpeed::FixedRun(speed) | MovementSpeed::FixedWalk(speed) => *speed,
			MovementSpeed::Variable(variable) => variable.speed(),
		}
	}
}

impl View<RequiredClearance> for MovementConfig {
	fn view(&self) -> Units {
		self.required_clearance
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum MovementSpeed {
	FixedRun(UnitsPerSecond),
	FixedWalk(UnitsPerSecond),
	Variable(VariableSpeed),
}

impl Default for MovementSpeed {
	fn default() -> Self {
		Self::FixedRun(UnitsPerSecond::from(1.0))
	}
}

impl From<UnitsPerSecond> for MovementSpeed {
	fn from(speed: UnitsPerSecond) -> Self {
		Self::FixedRun(speed)
	}
}

impl From<VariableSpeed> for MovementSpeed {
	fn from(speed: VariableSpeed) -> Self {
		Self::Variable(speed)
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct VariableSpeed {
	pub(crate) current: CurrentSpeed,
	pub(crate) run: UnitsPerSecond,
	pub(crate) walk: UnitsPerSecond,
}

impl VariableSpeed {
	#[cfg(test)]
	pub(crate) fn from_current(current: CurrentSpeed) -> Self {
		Self {
			current,
			run: UnitsPerSecond::from_u8(2),
			walk: UnitsPerSecond::from_u8(1),
		}
	}

	fn speed(&self) -> UnitsPerSecond {
		match self.current {
			CurrentSpeed::Walk => self.walk,
			CurrentSpeed::Run => self.run,
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum CurrentSpeed {
	Walk,
	Run,
}
