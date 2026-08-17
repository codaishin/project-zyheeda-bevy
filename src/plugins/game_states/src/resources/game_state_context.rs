use crate::{states::activity::ActivityState, system_params::ui_states::UIStates};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use common::prelude::*;
use std::collections::HashSet;

#[derive(Resource)]
pub struct GameStateContext {
	pub(crate) activity: Activity,
	pub(crate) ui: HashSet<IngameUI>,
}

impl FromWorld for GameStateContext {
	fn from_world(world: &mut World) -> Self {
		Self {
			activity: Activity::from(world.resource::<State<ActivityState>>().get()),
			ui: world
				.run_system_once(|p: UIStates| HashSet::from(&p))
				.unwrap_or_default(),
		}
	}
}
