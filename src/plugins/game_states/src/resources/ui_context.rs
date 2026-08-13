use crate::system_params::ui_states::UIStates;
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use common::traits::handles_game_states::UIState;
use std::collections::HashSet;

#[derive(Resource)]
pub(crate) struct UIContext {
	pub(crate) ui: HashSet<UIState>,
}

impl FromWorld for UIContext {
	fn from_world(world: &mut World) -> Self {
		Self {
			ui: world
				.run_system_once(|p: UIStates| HashSet::from(&p))
				.unwrap_or_default(),
		}
	}
}
