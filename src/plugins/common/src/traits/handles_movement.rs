use std::ops::DerefMut;

use crate::{
	tools::{Units, UnitsPerSecond},
	traits::{accessors::get::EntityContextMut, animation::Animation},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};

pub trait HandlesMovement {
	type TMovementMut<'w, 's>: SystemParam
		+ for<'c> EntityContextMut<Movement, TContext<'c>: ControlMovement>;
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
		T: Into<MovementTarget> + 'static;
}

impl<T> StartMovement for T
where
	T: DerefMut<Target: StartMovement>,
{
	fn start<TTarget>(
		&mut self,
		target: TTarget,
		radius: Units,
		speed: UnitsPerSecond,
		animation: Option<Animation>,
	) where
		TTarget: Into<MovementTarget> + 'static,
	{
		self.deref_mut().start(target, radius, speed, animation)
	}
}

pub trait UpdateMovement {
	fn update(&mut self, speed: UnitsPerSecond, animation: Option<Animation>);
}

impl<T> UpdateMovement for T
where
	T: DerefMut<Target: UpdateMovement>,
{
	fn update(&mut self, speed: UnitsPerSecond, animation: Option<Animation>) {
		self.deref_mut().update(speed, animation);
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

pub struct Movement;
