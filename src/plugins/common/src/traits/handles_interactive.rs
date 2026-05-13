use crate::traits::accessors::get::GetContextMut;
use bevy::{ecs::system::SystemParam, prelude::*};
use macros::EntityKey;
use std::ops::DerefMut;

pub trait HandlesInteractive {
	type TInteractiveMut: SystemParam
		+ for<'c> GetContextMut<SetInteractive, TContext<'c>: SetInteractiveRole>;
}

pub trait SetInteractiveRole {
	fn set_interactive_role(&mut self, role: Interactive);
}

impl<T> SetInteractiveRole for T
where
	T: DerefMut<Target: SetInteractiveRole>,
{
	fn set_interactive_role(&mut self, role: Interactive) {
		self.deref_mut().set_interactive_role(role);
	}
}

#[derive(EntityKey)]
pub struct SetInteractive {
	pub entity: Entity,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Interactive {
	Door(Door),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Door {
	SlideDoor,
}
