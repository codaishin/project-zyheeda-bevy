use crate::resources::key_map::KeyMap;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::tools::action_key::user_input::UserInput;

#[derive(SystemParam)]
pub struct Input<'w> {
	pub(crate) key_map: Res<'w, KeyMap>,
	pub(crate) input: Res<'w, ButtonInput<UserInput>>,
}

#[derive(SystemParam)]
pub struct InputMut<'w> {
	pub(crate) key_map: ResMut<'w, KeyMap>,
	pub(crate) input: Res<'w, ButtonInput<UserInput>>,
}
