use super::{
	iterate::Iterate,
	key_mappings::{GetInput, TryGetAction},
};
use crate::tools::action_key::{ActionKey, user_input::UserInput};
use bevy::prelude::*;

pub trait HandlesSettings {
	type TKeyMap<TAction>: Resource
		+ GetInput<TAction>
		+ TryGetAction<TAction>
		+ UpdateKey<TAction>
		+ for<'a> Iterate<'a, TItem = (&'a ActionKey, &'a UserInput)>
	where
		TAction: Copy + InvalidUserInput + TryFrom<ActionKey> + Into<ActionKey> + Into<UserInput>;
}

pub trait UpdateKey<TAction> {
	fn update_key(&mut self, action: TAction, input: UserInput);
}

pub trait InvalidUserInput {
	fn invalid_input(&self) -> &[UserInput];
}
