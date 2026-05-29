use crate::traits::{
	accessors::get::{TryGetContext, TryGetContextMut, View, ViewField},
	handles_map_generation::InteractiveType,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use macros::EntityKey;

pub trait HandlesInteractive {
	type TInteractive: SystemParam
		+ for<'c> TryGetContext<Interactive, TContext<'c>: InspectInteractive>;

	type TInteractiveMut: SystemParam
		+ for<'c> TryGetContextMut<Interactive, TContext<'c>: SetInteractiveState>;
}

pub trait InspectInteractive: View<InteractiveType> + View<InteractiveState> {}

impl<T> InspectInteractive for T where T: View<InteractiveType> + View<InteractiveState> {}

pub trait SetInteractiveState: View<InteractiveState> {
	fn set_interactive_state(&mut self, interactive_state: InteractiveState);
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
