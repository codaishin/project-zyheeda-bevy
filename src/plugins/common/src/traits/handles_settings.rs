use super::{
	iterate::Iterate,
	key_mappings::{GetInput, TryGetAction},
};
use crate::tools::action_key::{ActionKey, user_input::UserInput};
use bevy::prelude::*;

pub trait HandlesSettings {
	type TKeyMap<TAction>: Resource
		+ GetInput<TAction, UserInput>
		+ TryGetAction<UserInput, TAction>
		+ UpdateKey<TAction, UserInput>
		+ for<'a> Iterate<'a, TItem = (&'a ActionKey, &'a UserInput)>
	where
		ActionKey: From<TAction>,
		TAction: InvalidInput<UserInput> + TryFrom<ActionKey> + Copy,
		UserInput: From<TAction> + PartialEq;
}

pub trait UpdateKey<TAction, TInput> {
	fn update_key(&mut self, action: TAction, input: TInput);
}

pub trait InvalidInput<TInput> {
	fn invalid_input(&self) -> &[TInput];
}
