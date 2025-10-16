use bevy::prelude::*;
use common::{tools::action_key::ActionKey, traits::handles_input::InputState};

#[derive(Resource, Default, Debug, PartialEq)]
pub(crate) enum MouseOverride {
	#[default]
	Idle,
	Active {
		panel: Entity,
		action: ActionKey,
		input_state: Option<InputState>,
	},
}
