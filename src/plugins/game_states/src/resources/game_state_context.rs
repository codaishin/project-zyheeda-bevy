use crate::{states::command_state::CommandState, system_params::ui_states::UIStates};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use common::prelude::*;
use std::collections::HashSet;

#[derive(Resource)]
pub struct GameStateContext {
	pub(crate) command_state: CommandState,
	pub(crate) ui: HashSet<IngameUI>,
}

impl FromWorld for GameStateContext {
	fn from_world(world: &mut World) -> Self {
		Self {
			command_state: *world.resource::<State<CommandState>>().get(),
			ui: world
				.run_system_once(|p: UIStates| HashSet::from(&p))
				.unwrap_or_default(),
		}
	}
}
