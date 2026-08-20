use super::{LoadUi, insert_ui_content::InsertUiContent};
use crate::systems::{despawn::despawn, spawn::spawn, update_children::update_children};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::*;

pub(crate) trait AddUI {
	fn add_ui<TComponent, TLocalizationServer, TUICamera, TGameStates>(
		&mut self,
		state: impl Into<GameState>,
	) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TUICamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>,
		TGameStates: AddGameStateSystem;
}

impl AddUI for App {
	fn add_ui<TComponent, TLocalizationServer, TUICamera, TGameStates>(
		&mut self,
		state: impl Into<GameState>,
	) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TUICamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>,
		TGameStates: AddGameStateSystem,
	{
		let state = state.into();

		TGameStates::add_game_state_systems(
			self,
			OnGameState::Enter(state),
			spawn::<TComponent, AssetServer, TUICamera>,
		);
		TGameStates::add_game_state_systems(self, OnGameState::Exit(state), despawn::<TComponent>);

		self.add_systems(Update, update_children::<TComponent, TLocalizationServer>)
	}
}
