mod get_all_input_states;
mod get_all_inputs;
mod get_input;
mod get_input_state;
mod get_raw_user_input;
mod input_setup_changed;
mod update_key;

use crate::resources::mouse_override::MouseOverride;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};

#[derive(SystemParam)]
pub struct Input<'w, 's, TKeyMap>
where
	TKeyMap: SystemParam + 'static,
{
	keys: Res<'w, ButtonInput<KeyCode>>,
	mouse: Res<'w, ButtonInput<MouseButton>>,
	mouse_override: Res<'w, MouseOverride>,
	key_map: StaticSystemParam<'w, 's, TKeyMap>,
}
