//! Implementations for common bevy system parameters to forward traits
//! from the inner to the outer type.

use super::{GetInput, GetInputState, GetRawUserInput, RawInputState, UpdateKey};
use crate::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{handles_input::GetAllInputStates, iteration::IterFinite},
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

impl<T> GetRawUserInput for Res<'_, T>
where
	T: GetRawUserInput + Resource,
{
	fn get_raw_user_input(&self, state: RawInputState) -> impl Iterator<Item = UserInput> {
		self.deref().get_raw_user_input(state)
	}
}

impl<T> GetRawUserInput for ResMut<'_, T>
where
	T: GetRawUserInput + Resource,
{
	fn get_raw_user_input(&self, state: RawInputState) -> impl Iterator<Item = UserInput> {
		self.deref().get_raw_user_input(state)
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

impl<T> GetAllInputStates for Res<'_, T>
where
	T: GetAllInputStates + Resource,
{
	fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, super::InputState)>
	where
		TAction: Into<ActionKey> + IterFinite + 'static,
	{
		self.deref().get_all_input_states()
	}
}

impl<T> GetAllInputStates for ResMut<'_, T>
where
	T: GetAllInputStates + Resource,
{
	fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, super::InputState)>
	where
		TAction: Into<ActionKey> + IterFinite + 'static,
	{
		self.deref().get_all_input_states()
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
