use super::{LoadUi, insert_ui_content::InsertUiContent};
use crate::systems::{despawn::despawn, spawn::spawn, update_children::update_children};
use bevy::prelude::*;
use common::traits::{handles_graphics::StaticRenderLayers, handles_localization::LocalizeToken};

pub(crate) trait AddUI<TState> {
	fn add_ui<TComponent, TLocalizationServer, TGraphics>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: Resource + LocalizeToken,
		TGraphics: StaticRenderLayers + 'static;
}

impl<TState> AddUI<TState> for App
where
	TState: States + Copy,
{
	fn add_ui<TComponent, TLocalizationServer, TGraphics>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: Resource + LocalizeToken,
		TGraphics: StaticRenderLayers + 'static,
	{
		self.add_systems(
			OnEnter(on_state),
			spawn::<TComponent, AssetServer, TGraphics>,
		)
		.add_systems(OnExit(on_state), despawn::<TComponent>)
		.add_systems(Update, update_children::<TComponent, TLocalizationServer>)
	}
}
