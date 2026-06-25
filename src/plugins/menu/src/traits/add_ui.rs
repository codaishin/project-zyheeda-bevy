use super::{LoadUi, insert_ui_content::InsertUiContent};
use crate::systems::{despawn::despawn, spawn::spawn, update_children::update_children};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{
	accessors::get::GetContextMut,
	handles_graphics::{CameraHandle, RenderUi},
	handles_localization::Localize,
	thread_safe::ThreadSafe,
};

pub(crate) trait AddUI<TState> {
	fn add_ui<TComponent, TLocalizationServer, TUICamera>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TUICamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>;
}

impl<TState> AddUI<TState> for App
where
	TState: States + Copy,
{
	fn add_ui<TComponent, TLocalizationServer, TUICamera>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TUICamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>,
	{
		self.add_systems(
			OnEnter(on_state),
			spawn::<TComponent, AssetServer, TUICamera>,
		)
		.add_systems(OnExit(on_state), despawn::<TComponent>)
		.add_systems(Update, update_children::<TComponent, TLocalizationServer>)
	}
}
