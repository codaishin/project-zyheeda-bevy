use crate::{states::activity::Activity, system_params::ui_states::UIStates};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use common::prelude::*;
use std::collections::HashSet;

#[derive(Resource)]
pub struct GameStateContext {
	pub(crate) activity: ActivityState,
	pub(crate) ui: HashSet<UIState>,
}

impl FromWorld for GameStateContext {
	fn from_world(world: &mut World) -> Self {
		Self {
			activity: ActivityState::from(world.resource::<State<Activity>>().get()),
			ui: world
				.run_system_once(|p: UIStates| HashSet::from(&p))
				.unwrap_or_default(),
		}
	}
}
