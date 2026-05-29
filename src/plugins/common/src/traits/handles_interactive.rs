use crate::traits::{
	accessors::get::{TryGetContext, TryGetContextMut, View, ViewField},
	handles_map_generation::InteractiveType,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use macros::EntityKey;
use std::ops::DerefMut;

pub trait HandlesInteractive {
	type TInteractive: SystemParam
		+ for<'c> TryGetContext<Interactive, TContext<'c>: InspectInteractive>;

	type TInteractiveMut: SystemParam
		+ for<'c> TryGetContextMut<Interactive, TContext<'c>: SetInteractiveState>;
}

pub trait InspectInteractive: View<InteractiveType> + View<InteractiveState> {}

impl<T> InspectInteractive for T where T: View<InteractiveType> + View<InteractiveState> {}

pub trait SetInteractiveState: InspectInteractive {
	fn set_interactive_state(&mut self, interactive_state: InteractiveState);
}

impl<T> SetInteractiveState for T
where
	T: DerefMut<Target: SetInteractiveState>,
{
	fn set_interactive_state(&mut self, interactive_state: InteractiveState) {
		self.deref_mut().set_interactive_state(interactive_state);
	}
}

#[derive(EntityKey)]
pub struct Interactive {
	pub entity: Entity,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InteractiveState {
	Active,
	Inactive,
}

impl ViewField for InteractiveState {
	type TValue<'a> = Self;
}
