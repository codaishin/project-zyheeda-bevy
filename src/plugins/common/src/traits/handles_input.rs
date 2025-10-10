use super::{
	iterate::Iterate,
	key_mappings::{GetInput, TryGetAction},
};
use crate::tools::action_key::{ActionKey, user_input::UserInput};
use bevy::prelude::*;

pub trait HandlesInput {
	type TKeyMap: Resource
		+ GetInput
		+ TryGetAction
		+ UpdateKey
		+ for<'a> Iterate<'a, TItem = (&'a ActionKey, &'a UserInput)>;
}

pub trait UpdateKey {
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<ActionKey> + Into<UserInput> + 'static;
}

pub trait InvalidUserInput {
	fn invalid_input(&self) -> &[UserInput];
}
