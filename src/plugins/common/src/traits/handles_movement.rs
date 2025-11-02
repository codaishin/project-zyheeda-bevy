use crate::{
	tools::{Units, UnitsPerSecond},
	traits::{accessors::get::EntityContextMut, animation::Animation},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};

pub trait HandlesMovement {
	type TMovementMut<'w, 's>: SystemParam
		+ for<'c> EntityContextMut<Movement, TContext<'c>: UpdateMovement>;
}

pub type MovementSystemParamMut<'w, 's, T> = <T as HandlesMovement>::TMovementMut<'w, 's>;

pub trait ControlMovement: StartMovement + UpdateMovement + StopMovement {}

impl<T> ControlMovement for T where T: StartMovement + UpdateMovement + StopMovement {}

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

pub trait StartMovement {
	fn start<T>(
		&mut self,
		target: T,
		radius: Units,
		speed: UnitsPerSecond,
		animation: Option<Animation>,
	) where
		T: Into<MovementTarget>;
}

pub trait UpdateMovement {
	fn update(&mut self, speed: UnitsPerSecond, animation: Option<Animation>);
}

pub trait StopMovement {
	fn stop(&mut self);
}

pub struct Movement;
