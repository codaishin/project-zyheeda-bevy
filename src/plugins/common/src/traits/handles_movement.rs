use crate::{
	tools::{Units, speed::Speed},
	traits::accessors::get::{GetContext, GetContextMut, View, ViewField},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

pub trait HandlesMovement {
	type TMovement<'w, 's>: SystemParam
		+ for<'c> GetContext<Movement, TContext<'c>: CurrentMovement>;
	type TMovementMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<Movement, TContext<'c>: ControlMovement>;

	/// Register movement execution via the provided movement definition.
	/// Without doing this an implementing plugin might not execute any movement
	/// at all.
	fn register_movement<TMovementDefinition>(app: &mut App)
	where
		TMovementDefinition: Component + View<Speed> + View<RequiredClearance>;
}

pub type MovementSystemParam<'w, 's, T> = <T as HandlesMovement>::TMovement<'w, 's>;
pub type MovementSystemParamMut<'w, 's, T> = <T as HandlesMovement>::TMovementMut<'w, 's>;

pub trait ControlMovement: StartMovement + StopMovement + CurrentMovement {}

impl<T> ControlMovement for T where T: StartMovement + StopMovement + CurrentMovement {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RequiredClearance(pub Units);

impl ViewField for RequiredClearance {
	type TValue<'a> = Units;
}

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

pub trait CurrentMovement {
	fn current_movement(&self) -> Option<MovementTarget>;
}

impl<T> CurrentMovement for T
where
	T: Deref<Target: CurrentMovement>,
{
	fn current_movement(&self) -> Option<MovementTarget> {
		self.deref().current_movement()
	}
}

pub struct Movement {
	pub entity: Entity,
}

impl From<Movement> for Entity {
	fn from(Movement { entity }: Movement) -> Self {
		entity
	}
}
