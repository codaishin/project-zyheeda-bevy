use crate::traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn, LoadUi};
use bevy::prelude::*;
use loading::traits::progress::{AssetsProgress, DependenciesProgress, Progress};
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

impl<T> GetNode for LoadingScreen<T>
where
	T: Progress,
{
	fn node(&self) -> NodeBundle {
		NodeBundle {
			style: Style {
				width: Val::Vw(100.),
				height: Val::Vh(100.),
				flex_direction: FlexDirection::ColumnReverse,
				padding: UiRect::bottom(Val::Px(100.)).with_left(Val::Px(50.)),
				..default()
			},
			background_color: Color::BLACK.into(),
			z_index: ZIndex::Global(i32::MAX),
			..default()
		}
	}
}

impl InstantiateContentOn for LoadingScreen<AssetsProgress> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent.spawn(TextBundle::from_section(
			"Loading Assets ...",
			TextStyle {
				font_size: 32.,
				..default()
			},
		));
	}
}

impl InstantiateContentOn for LoadingScreen<DependenciesProgress> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent.spawn(TextBundle::from_section(
			"Resolving Dependencies ...",
			TextStyle {
				font_size: 32.,
				..default()
			},
		));
	}
}
