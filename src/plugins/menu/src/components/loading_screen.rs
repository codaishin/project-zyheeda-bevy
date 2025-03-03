use crate::traits::{LoadUi, insert_ui_content::InsertUiContent};
use bevy::prelude::*;
use common::traits::handles_load_tracking::{AssetsProgress, DependenciesProgress, Progress};
use std::marker::PhantomData;

#[derive(Component)]
#[require(
	Node(full_screen),
	BackgroundColor(black),
	ZIndex(ZIndex::max),
	GlobalZIndex(GlobalZIndex::max)
)]
pub(crate) struct LoadingScreen<T>(PhantomData<T>)
where
	T: Progress;

fn full_screen() -> Node {
	Node {
		width: Val::Vw(100.),
		height: Val::Vh(100.),
		flex_direction: FlexDirection::ColumnReverse,
		padding: UiRect::bottom(Val::Px(100.)).with_left(Val::Px(50.)),
		..default()
	}
}

fn black() -> BackgroundColor {
	BackgroundColor(Color::BLACK)
}

trait Max {
	fn max() -> Self;
}

impl Max for ZIndex {
	fn max() -> Self {
		Self(i32::MAX)
	}
}

impl Max for GlobalZIndex {
	fn max() -> Self {
		Self(i32::MAX)
	}
}

impl<T> LoadUi<AssetServer> for LoadingScreen<T>
where
	T: Progress,
{
	fn load_ui(_: &mut AssetServer) -> Self {
		LoadingScreen(PhantomData)
	}
}

impl InsertUiContent for LoadingScreen<AssetsProgress> {
	fn insert_ui_content(&self, parent: &mut ChildBuilder) {
		parent.spawn((
			Text::new("Loading Assets ..."),
			TextFont {
				font_size: 32.,
				..default()
			},
		));
	}
}

impl InsertUiContent for LoadingScreen<DependenciesProgress> {
	fn insert_ui_content(&self, parent: &mut ChildBuilder) {
		parent.spawn((
			Text::new("Resolving Dependencies ..."),
			TextFont {
				font_size: 32.,
				..default()
			},
		));
	}
}
