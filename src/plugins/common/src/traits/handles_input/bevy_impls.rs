//! Implementations for common bevy system parameters to simplify test setups

use crate::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::handles_input::{GetInput, GetInputState, UpdateKey},
};
use bevy::prelude::*;
use std::ops::{Deref, DerefMut};

impl<T> GetInput for Res<'_, T>
where
	T: GetInput + Resource,
{
	fn get_input<TAction>(&self, action: TAction) -> UserInput
	where
		TAction: Into<ActionKey> + 'static,
	{
		self.deref().get_input(action)
	}
}

impl<T> GetInput for ResMut<'_, T>
where
	T: GetInput + Resource,
{
	fn get_input<TAction>(&self, action: TAction) -> UserInput
	where
		TAction: Into<ActionKey> + 'static,
	{
		self.deref().get_input(action)
	}
}

impl<T> GetInputState for Res<'_, T>
where
	T: GetInputState + Resource,
{
	fn get_input_state<TAction>(&self, action: TAction) -> super::InputState
	where
		TAction: Into<ActionKey> + 'static,
	{
		self.deref().get_input_state(action)
	}
}

impl<T> GetInputState for ResMut<'_, T>
where
	T: GetInputState + Resource,
{
	fn get_input_state<TAction>(&self, action: TAction) -> super::InputState
	where
		TAction: Into<ActionKey> + 'static,
	{
		self.deref().get_input_state(action)
	}
}

impl<T> UpdateKey for ResMut<'_, T>
where
	T: UpdateKey + Resource,
{
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<ActionKey> + 'static,
	{
		self.deref_mut().update_key(action, input)
	}
}
