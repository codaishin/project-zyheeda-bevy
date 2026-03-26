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
	pub(crate) collider_radius: Units,
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
			collider_radius: Units::from(0.5),
			speed: MovementSpeed::FixedRun(UnitsPerSecond::from(1.0)),
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
		self.collider_radius
	}
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum MovementSpeed {
	FixedRun(UnitsPerSecond),
	FixedWalk(UnitsPerSecond),
	Variable(VariableSpeed),
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum CurrentSpeed {
	Walk,
	Run,
}
