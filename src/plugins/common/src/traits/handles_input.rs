mod bevy_impls;

use crate::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		accessors::get::{GetProperty, Property},
		iteration::{Iter, IterFinite},
	},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use std::hash::Hash;

pub trait HandlesActionKeyButton {
	/// Controls triggering of actions through mouse left clicking the associated
	/// button.
	type TActionKeyButton: Component + From<ActionKey> + GetProperty<LeftMouseOverridden>;
}

/// Indicate wether left mouse behavior is overridden for an associated instance that
/// implements [`GetInputState`].
pub struct LeftMouseOverridden;

impl Property for LeftMouseOverridden {
	type TValue<'a> = bool;
}

pub trait HandlesInput {
	type TInput<'world, 'state>: SystemParam
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInput + InputSetupChanged>
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetRawUserInput + GetInputState>;
}

pub trait HandlesInputMut {
	type TInputMut<'world, 'state>: SystemParam
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInput + InputSetupChanged>
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetRawUserInput + GetInputState>
		+ for<'w, 's> SystemParam<Item<'w, 's>: UpdateKey>;
}

/// Helper type to designate [`HandlesInput::TInput`] as a [`SystemParam`] implementation for a
/// given generic system constraint
pub type InputSystemParam<'w, 's, T> = <T as HandlesInput>::TInput<'w, 's>;

/// Helper type to designate [`HandlesInputMut::TInputMut`] as a [`SystemParam`] implementation for a
/// given generic system constraint
pub type InputMutSystemParam<'w, 's, T> = <T as HandlesInputMut>::TInputMut<'w, 's>;

pub trait UpdateKey {
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<ActionKey> + 'static;
}

pub trait InvalidUserInput {
	fn invalid_input(&self) -> &[UserInput];
}

pub trait InputSetupChanged {
	fn input_setup_changed(&self) -> bool;
}

pub trait GetInput {
	fn get_input<TAction>(&self, action: TAction) -> UserInput
	where
		TAction: Into<ActionKey> + 'static;
}

pub trait GetAllInputs {
	fn get_all_inputs(&self) -> impl Iterator<Item = (ActionKey, UserInput)>;
}

impl<T> GetAllInputs for T
where
	T: GetInput,
{
	fn get_all_inputs(&self) -> impl Iterator<Item = (ActionKey, UserInput)> {
		IterInputs {
			input: self,
			actions: ActionKey::iterator(),
		}
	}
}

pub struct IterInputs<'a, T>
where
	T: GetInput,
{
	input: &'a T,
	actions: Iter<ActionKey>,
}

impl<'a, T> Iterator for IterInputs<'a, T>
where
	T: GetInput + 'a,
{
	type Item = (ActionKey, UserInput);

	fn next(&mut self) -> Option<Self::Item> {
		let action_key = self.actions.next()?;
		Some((action_key, self.input.get_input(action_key)))
	}
}

pub trait GetRawUserInput {
	fn get_raw_user_input(&self, state: RawInputState) -> impl Iterator<Item = UserInput>;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RawInputState {
	JustPressed,
	Held,
	JustReleased,
}

pub trait GetInputState {
	fn get_input_state<TAction>(&self, action: TAction) -> InputState
	where
		TAction: Into<ActionKey> + 'static;
}

pub trait GetAllInputStates {
	fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
	where
		TAction: Into<ActionKey> + IterFinite + 'static;
}

impl<T> GetAllInputStates for T
where
	T: GetInputState,
{
	fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
	where
		TAction: Into<ActionKey> + IterFinite + 'static,
	{
		IterInputStates {
			input: self,
			actions: TAction::iterator(),
		}
	}
}

pub struct IterInputStates<'a, TInput, TAction>
where
	TInput: GetInputState,
	TAction: Into<ActionKey> + IterFinite + 'static,
{
	input: &'a TInput,
	actions: Iter<TAction>,
}

impl<'a, TInput, TAction> Iterator for IterInputStates<'a, TInput, TAction>
where
	TInput: GetInputState + 'a,
	TAction: Into<ActionKey> + IterFinite + 'static,
{
	type Item = (TAction, InputState);

	fn next(&mut self) -> Option<Self::Item> {
		let action_key = self.actions.next()?;
		Some((action_key, self.input.get_input_state(action_key)))
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputState {
	Pressed { just_now: bool },
	Released { just_now: bool },
}

impl InputState {
	pub const fn pressed() -> Self {
		Self::Pressed { just_now: false }
	}

	pub const fn just_pressed() -> Self {
		Self::Pressed { just_now: true }
	}

	pub const fn released() -> Self {
		Self::Released { just_now: false }
	}

	pub const fn just_released() -> Self {
		Self::Released { just_now: true }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		states::menu_state::MenuState,
		tools::action_key::{movement::MovementKey, slot::PlayerSlot},
	};
	use std::collections::HashMap;

	mod get_all_inputs {
		use bevy::input::keyboard::KeyCode;

		use super::*;

		struct _Input(HashMap<ActionKey, UserInput>);

		impl GetInput for _Input {
			fn get_input<TAction>(&self, action: TAction) -> UserInput
			where
				TAction: Into<ActionKey> + 'static,
			{
				let action = action.into();
				self.0
					.get(&action)
					.copied()
					.unwrap_or(UserInput::from(action))
			}
		}

		#[test]
		fn all_input_states() {
			let input = _Input(HashMap::from([
				(
					ActionKey::from(PlayerSlot::LOWER_R),
					UserInput::KeyCode(KeyCode::KeyA),
				),
				(
					ActionKey::from(MenuState::Inventory),
					UserInput::KeyCode(KeyCode::KeyB),
				),
				(
					ActionKey::from(MovementKey::Left),
					UserInput::KeyCode(KeyCode::KeyC),
				),
			]));

			assert_eq!(
				ActionKey::iterator()
					.map(|a| match a {
						ActionKey::Slot(PlayerSlot::LOWER_R) =>
							(a, UserInput::KeyCode(KeyCode::KeyA)),
						ActionKey::Menu(MenuState::Inventory) =>
							(a, UserInput::KeyCode(KeyCode::KeyB)),
						ActionKey::Movement(MovementKey::Left) =>
							(a, UserInput::KeyCode(KeyCode::KeyC)),
						_ => (a, UserInput::from(a)),
					})
					.collect::<Vec<_>>(),
				input.get_all_inputs().collect::<Vec<_>>()
			);
		}
	}

	mod get_all_input_states {
		use super::*;

		struct _Input(HashMap<ActionKey, InputState>);

		impl GetInputState for _Input {
			fn get_input_state<TAction>(&self, action: TAction) -> InputState
			where
				TAction: Into<ActionKey> + 'static,
			{
				match self.0.get(&action.into()) {
					Some(input_state) => *input_state,
					None => InputState::Released { just_now: false },
				}
			}
		}

		#[test]
		fn all_input_states() {
			let input = _Input(HashMap::from([
				(
					ActionKey::from(PlayerSlot::LOWER_R),
					InputState::Pressed { just_now: true },
				),
				(
					ActionKey::from(MenuState::Inventory),
					InputState::Pressed { just_now: false },
				),
				(
					ActionKey::from(MovementKey::Left),
					InputState::Released { just_now: true },
				),
			]));

			assert_eq!(
				ActionKey::iterator()
					.map(|a| match a {
						ActionKey::Slot(PlayerSlot::LOWER_R) =>
							(a, InputState::Pressed { just_now: true }),
						ActionKey::Menu(MenuState::Inventory) =>
							(a, InputState::Pressed { just_now: false }),
						ActionKey::Movement(MovementKey::Left) =>
							(a, InputState::Released { just_now: true }),
						_ => (a, InputState::Released { just_now: false }),
					})
					.collect::<Vec<_>>(),
				input.get_all_input_states().collect::<Vec<_>>()
			);
		}
	}
}
