use crate::traits::{LoadUi, insert_ui_content::InsertUiContent};
use bevy::prelude::*;
use common::traits::{
	handles_load_tracking::{AssetsProgress, DependenciesProgress, Progress},
	handles_localization::LocalizeToken,
};
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
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &mut TLocalization,
		parent: &mut ChildBuilder,
	) where
		TLocalization: LocalizeToken,
	{
		let label = localize.localize_token("loading-assets").or_token();

		parent.spawn((
			Text::new(label),
			TextFont {
				font_size: 32.,
				..default()
			},
		));
	}
}

impl InsertUiContent for LoadingScreen<DependenciesProgress> {
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &mut TLocalization,
		parent: &mut ChildBuilder,
	) where
		TLocalization: LocalizeToken,
	{
		let label = localize.localize_token("resolving-dependencies").or_token();

		parent.spawn((
			Text::new(label),
			TextFont {
				font_size: 32.,
				..default()
			},
		));
	}
}
