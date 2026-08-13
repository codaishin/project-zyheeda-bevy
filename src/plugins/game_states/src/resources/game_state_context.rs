use crate::{states::activity::Activity, system_params::ui_states::UIStates};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use common::traits::handles_game_states::{ActivityState, UIState};
use std::collections::HashSet;

#[derive(Resource)]
pub(crate) struct GameStatesContext {
	pub(crate) activity: ActivityState,
	pub(crate) ui: HashSet<UIState>,
}

impl FromWorld for GameStatesContext {
	fn from_world(world: &mut World) -> Self {
		Self {
			activity: ActivityState::from(world.resource::<State<Activity>>().get()),
			ui: world
				.run_system_once(|ui: UIStates| HashSet::from(&ui))
				.unwrap_or_else(|_| HashSet::default()),
		}
	}
}
