use super::{LoadUi, insert_ui_content::InsertUiContent};
use crate::systems::{despawn::despawn, spawn::spawn, update_children::update_children};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::*;

pub(crate) trait AddGui<TState> {
	fn add_gui<TComponent, TLocalizationServer, TUICamera, TGameStates>(
		&mut self,
		state: TState,
	) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TUICamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>,
		TGameStates: AddGameStateSystem<TState>;
}

impl<TState> AddGui<TState> for App
where
	TState: Copy,
{
	fn add_gui<TComponent, TLocalizationServer, TUICamera, TGameStates>(
		&mut self,
		state: TState,
	) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TUICamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>,
		TGameStates: AddGameStateSystem<TState>,
	{
		TGameStates::add_game_state_systems(
			self,
			OnStateTransition::Enter(state),
			spawn::<TComponent, AssetServer, TUICamera>,
		);
		TGameStates::add_game_state_systems(
			self,
			OnStateTransition::Exit(state),
			despawn::<TComponent>,
		);

		self.add_systems(Update, update_children::<TComponent, TLocalizationServer>)
	}
}

pub(crate) trait AddLoadGui {
	fn add_load_gui<TComponent, TLocalizationServer, TUICamera, TLoading>(
		&mut self,
		load_group: impl LoadGroup<TLoading::TLoadAssetState>,
	) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TUICamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>,
		TLoading: HandlesLoadTracking;
}

impl AddLoadGui for App {
	fn add_load_gui<TComponent, TLocalizationServer, TUICamera, TLoading>(
		&mut self,
		load_group: impl LoadGroup<TLoading::TLoadAssetState>,
	) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TUICamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>,
		TLoading: HandlesLoadTracking,
	{
		TLoading::add_loading_systems(
			self,
			OnStateTransition::Enter(load_group),
			spawn::<TComponent, AssetServer, TUICamera>,
		);
		TLoading::add_loading_systems(
			self,
			OnStateTransition::Exit(load_group),
			despawn::<TComponent>,
		);

		self.add_systems(Update, update_children::<TComponent, TLocalizationServer>)
	}
}
