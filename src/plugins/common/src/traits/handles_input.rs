use super::{
	iterate::Iterate,
	key_mappings::{GetInput, TryGetAction},
};
use crate::tools::action_key::{ActionKey, user_input::UserInput};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};

pub trait HandlesInput {
	type TKeyMap: Resource
		+ GetInput
		+ TryGetAction
		+ UpdateKey
		+ for<'a> Iterate<'a, TItem = (&'a ActionKey, &'a UserInput)>;
	type TInput<'world, 'state>: SystemParam + GetActionInputState;
}

pub trait HandlesInputMut {
	type TInputMut<'world, 'state>: SystemParam + UpdateKey + GetActionInputState;
}

pub type InputSystemParam<'world, 'state, 'world_self, 'state_self, T> =
	StaticSystemParam<'world, 'state, <T as HandlesInput>::TInput<'world_self, 'state_self>>;

pub type InputMutSystemParam<'world, 'state, 'world_self, 'state_self, T> =
	StaticSystemParam<'world, 'state, <T as HandlesInputMut>::TInputMut<'world_self, 'state_self>>;

pub trait UpdateKey {
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<ActionKey> + 'static;
}

pub trait InvalidUserInput {
	fn invalid_input(&self) -> &[UserInput];
}

pub trait GetActionInputState {
	fn get_action_input_state<TAction>(&self, action: TAction) -> InputState
	where
		TAction: Into<ActionKey> + 'static;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputState {
	Pressed { just_now: bool },
	Released { just_now: bool },
}
