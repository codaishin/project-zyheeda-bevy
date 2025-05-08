use super::{
	iterate::Iterate,
	key_mappings::{GetUserInput, TryGetKey},
};
use crate::tools::action_key::{ActionKey, user_input::UserInput};
use bevy::prelude::*;

pub trait HandlesSettings {
	type TKeyMap<TKey>: Resource
		+ GetUserInput<TKey, UserInput>
		+ TryGetKey<UserInput, TKey>
		+ UpdateKey<TKey, UserInput>
		+ for<'a> Iterate<'a, TItem = (&'a ActionKey, &'a UserInput)>
	where
		ActionKey: From<TKey>,
		TKey: TryFrom<ActionKey> + Copy,
		UserInput: From<TKey> + PartialEq;
}

pub trait UpdateKey<TKey, TUserInput> {
	fn update_key(&mut self, key: TKey, user_input: TUserInput);
}
