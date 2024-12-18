use super::{start_game::StartGame, start_menu_button::StartMenuButton};
use crate::traits::{colors::DEFAULT_PANEL_COLORS, insert_ui_content::InsertUiContent, LoadUi};
use bevy::prelude::*;

#[derive(Component)]
#[require(Node(full_screen))]
pub(crate) struct StartMenu;

fn full_screen() -> Node {
	Node {
		width: Val::Vw(100.0),
		height: Val::Vh(100.0),
		align_items: AlignItems::Center,
		justify_content: JustifyContent::Center,
		flex_direction: FlexDirection::Column,
		..default()
	}
}

impl LoadUi<AssetServer> for StartMenu {
	fn load_ui(_: &mut AssetServer) -> Self {
		Self
	}
}

impl InsertUiContent for StartMenu {
	fn insert_ui_content(&self, parent: &mut ChildBuilder) {
		parent
			.spawn(Node {
				margin: UiRect::bottom(Val::Px(30.)),
				..default()
			})
			.with_children(|parent| {
				parent.spawn((
					Text::new("Project Zyheeda"),
					TextFont {
						font_size: 64.0,
						..default()
					},
					TextColor(DEFAULT_PANEL_COLORS.text),
				));
			});
		parent
			.spawn((
				Button,
				Node {
					width: Val::Px(300.0),
					height: Val::Px(100.0),
					margin: UiRect::all(Val::Px(2.0)),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					..default()
				},
				StartMenuButton,
				StartGame,
			))
			.with_children(|parent| {
				parent.spawn((
					Text::new("New Game"),
					TextFont {
						font_size: 32.0,
						..default()
					},
					TextColor(DEFAULT_PANEL_COLORS.text),
				));
			});
	}
}
