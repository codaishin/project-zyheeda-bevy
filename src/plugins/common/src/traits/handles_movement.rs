use crate::{
	tools::{Units, UnitsPerSecond},
	traits::{accessors::get::EntityContextMut, animation::Animation},
};
use bevy::ecs::system::SystemParam;

pub trait HandlesMovement {
	type TMovementMut<'w, 's>: SystemParam
		+ for<'c> EntityContextMut<Movement, TContext<'c>: UpdateMovement>;
}

pub type MovementSystemParamMut<'w, 's, T> = <T as HandlesMovement>::TMovementMut<'w, 's>;

pub trait ControlMovement: StartMovement + UpdateMovement + StopMovement {}

impl<T> ControlMovement for T where T: StartMovement + UpdateMovement + StopMovement {}

pub trait UpdateMovement {
	fn update(&mut self, speed: UnitsPerSecond, animation: Option<Animation>);
}

pub trait StartMovement {
	fn start(&mut self, radius: Units, speed: UnitsPerSecond, animation: Option<Animation>);
}

pub trait StopMovement {
	fn stop(&mut self);
}

pub struct Movement;
