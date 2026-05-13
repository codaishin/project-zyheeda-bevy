use crate::traits::accessors::get::GetContextMut;
use bevy::{ecs::system::SystemParam, prelude::*};

pub trait HandlesInteractive {
	type TInteractiveMut: SystemParam
		+ for<'c> GetContextMut<SetInteractive, TContext<'c>: SetInteractiveRole>;
}

pub trait SetInteractiveRole {
	fn set_interactive_role(&mut self, role: Interactive);
}

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
