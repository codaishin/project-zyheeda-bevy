use crate::resources::{activity_context::ActivityContext, ui_context::UIContext};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::handles_game_states::{GameStateCollection, GameStates};

#[derive(SystemParam)]
pub struct GameStatesRead<'w> {
	activity_ctx: Res<'w, ActivityContext>,
	ui_ctx: Res<'w, UIContext>,
}

impl GameStates for GameStatesRead<'_> {
	fn game_states(&self) -> GameStateCollection<'_> {
		GameStateCollection {
			activity: self.activity_ctx.activity,
			ui: &self.ui_ctx.ui,
		}
	}
}
