use super::{
	iterate::Iterate,
	key_mappings::{GetInput, TryGetAction},
};
use crate::tools::action_key::{ActionKey, user_input::UserInput};
use bevy::prelude::*;

pub trait HandlesSettings {
	type TKeyMap<TAction>: Resource
		+ GetInput<TAction, TInput = UserInput>
		+ TryGetAction<TAction, TInput = UserInput>
		+ UpdateKey<TAction, TInput = UserInput>
		+ for<'a> Iterate<'a, TItem = (&'a ActionKey, &'a UserInput)>
	where
		TAction: Copy
			+ InvalidInput<TInput = UserInput>
			+ TryFrom<ActionKey>
			+ Into<ActionKey>
			+ Into<UserInput>;
}

pub trait UpdateKey<TAction> {
	type TInput;

	fn update_key(&mut self, action: TAction, input: Self::TInput);
}

pub trait InvalidInput {
	type TInput;

	fn invalid_input(&self) -> &[Self::TInput];
}
