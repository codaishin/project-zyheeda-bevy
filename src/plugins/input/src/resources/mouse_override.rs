use bevy::prelude::*;
use common::{tools::action_key::ActionKey, traits::handles_input::InputState};

#[derive(Resource, Default, Debug, PartialEq)]
pub(crate) enum MouseOverride {
	#[default]
	Idle,
	Ui {
		panel: Entity,
	},
	World {
		panel: Entity,
		action: ActionKey,
		input_state: InputState,
	},
}
