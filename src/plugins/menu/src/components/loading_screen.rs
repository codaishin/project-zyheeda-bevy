use crate::traits::{
	ui_components::{GetUIComponents, GetZIndex, GetZIndexGlobal},
	update_children::UpdateChildren,
	LoadUi,
};
use bevy::prelude::*;
use common::traits::handles_load_tracking::{AssetsProgress, DependenciesProgress, Progress};
use std::marker::PhantomData;

#[derive(Component)]
pub(crate) struct LoadingScreen<T>(PhantomData<T>)
where
	T: Progress;

impl<T> LoadUi<AssetServer> for LoadingScreen<T>
where
	T: Progress,
{
	fn load_ui(_: &mut AssetServer) -> Self {
		LoadingScreen(PhantomData)
	}
}

impl<T> GetZIndex for LoadingScreen<T>
where
	T: Progress,
{
	fn z_index(&self) -> Option<ZIndex> {
		Some(ZIndex(i32::MAX))
	}
}

impl<T> GetZIndexGlobal for LoadingScreen<T>
where
	T: Progress,
{
	fn z_index_global(&self) -> Option<GlobalZIndex> {
		Some(GlobalZIndex(i32::MAX))
	}
}

impl<T> GetUIComponents for LoadingScreen<T>
where
	T: Progress,
{
	fn ui_components(&self) -> (Node, BackgroundColor) {
		(
			Node {
				width: Val::Vw(100.),
				height: Val::Vh(100.),
				flex_direction: FlexDirection::ColumnReverse,
				padding: UiRect::bottom(Val::Px(100.)).with_left(Val::Px(50.)),
				..default()
			},
			Color::BLACK.into(),
		)
	}
}

impl UpdateChildren for LoadingScreen<AssetsProgress> {
	fn update_children(&self, parent: &mut ChildBuilder) {
		parent.spawn((
			Text::new("Loading Assets ..."),
			TextFont {
				font_size: 32.,
				..default()
			},
		));
	}
}

impl UpdateChildren for LoadingScreen<DependenciesProgress> {
	fn update_children(&self, parent: &mut ChildBuilder) {
		parent.spawn((
			Text::new("Resolving Dependencies ..."),
			TextFont {
				font_size: 32.,
				..default()
			},
		));
	}
}
