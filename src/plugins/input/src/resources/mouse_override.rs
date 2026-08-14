use bevy::prelude::*;
use common::prelude::*;

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
