use super::{iterate::Iterate, key_mappings::TryGetAction};
use crate::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::iteration::{Iter, IterFinite},
};
use bevy::{ecs::system::SystemParam, prelude::*};

pub trait HandlesInput {
	type TKeyMap: Resource
		+ GetInput
		+ TryGetAction
		+ UpdateKey
		+ for<'a> Iterate<'a, TItem = (&'a ActionKey, &'a UserInput)>;
	type TInput<'world, 'state>: SystemParam
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInput>
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInputState>;
}

pub trait HandlesInputMut {
	type TInputMut<'world, 'state>: SystemParam
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInput>
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInputState>
		+ for<'w, 's> SystemParam<Item<'w, 's>: UpdateKey>;
}

/// Helper type to designate [`HandlesInput::TInput`] as a [`SystemParam`] constraint for a
/// given generic system
pub type InputSystemParam<T> = <T as HandlesInput>::TInput<'static, 'static>;

/// Helper type to designate [`HandlesInputMut::TInputMut`] as a [`SystemParam`] constraint for a
/// given generic system
pub type InputMutSystemParam<T> = <T as HandlesInputMut>::TInputMut<'static, 'static>;

pub trait UpdateKey {
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<ActionKey> + 'static;
}

pub trait InvalidUserInput {
	fn invalid_input(&self) -> &[UserInput];
}

pub trait GetInput {
	fn get_input<TAction>(&self, action: TAction) -> UserInput
	where
		TAction: Into<ActionKey> + 'static;
}

pub trait GetInputState {
	fn get_input_state<TAction>(&self, action: TAction) -> InputState
	where
		TAction: Into<ActionKey> + 'static;
}

impl<'a, T> Iterate<'a> for T
where
	T: GetInputState + 'a,
{
	type TItem = (ActionKey, InputState);
	type TIter = IterInputStates<'a, Self>;

	fn iterate(&'a self) -> Self::TIter {
		IterInputStates {
			input: self,
			actions: ActionKey::iterator(),
		}
	}
}

pub struct IterInputStates<'a, T>
where
	T: GetInputState,
{
	input: &'a T,
	actions: Iter<ActionKey>,
}

impl<'a, T> Iterator for IterInputStates<'a, T>
where
	T: GetInputState + 'a,
{
	type Item = (ActionKey, InputState);

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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		states::menu_state::MenuState,
		tools::action_key::{movement::MovementKey, slot::PlayerSlot},
	};
	use std::collections::HashMap;

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
			input.iterate().collect::<Vec<_>>()
		);
	}
}
