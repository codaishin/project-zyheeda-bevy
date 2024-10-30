use super::{start_game::StartGame, start_menu_button::StartMenuButton};
use crate::traits::{
	colors::DEFAULT_PANEL_COLORS,
	get_node::GetNode,
	instantiate_content_on::InstantiateContentOn,
	LoadUi,
};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct StartMenu;

impl LoadUi<AssetServer> for StartMenu {
	fn load_ui(_: &mut AssetServer) -> Self {
		Self
	}
}

impl GetNode for StartMenu {
	fn node(&self) -> NodeBundle {
		NodeBundle {
			style: Style {
				width: Val::Vw(100.0),
				height: Val::Vh(100.0),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		}
	}
}

impl InstantiateContentOn for StartMenu {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent
			.spawn(NodeBundle {
				style: Style {
					margin: UiRect::bottom(Val::Px(30.)),
					..default()
				},
				..default()
			})
			.with_children(|parent| {
				parent.spawn(TextBundle::from_section(
					"Project Zyheeda",
					TextStyle {
						font_size: 64.0,
						color: DEFAULT_PANEL_COLORS.text,
						..default()
					},
				));
			});
		parent
			.spawn((
				ButtonBundle {
					style: Style {
						width: Val::Px(300.0),
						height: Val::Px(100.0),
						margin: UiRect::all(Val::Px(2.0)),
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
						..default()
					},
					..default()
				},
				StartMenuButton,
				StartGame,
			))
			.with_children(|parent| {
				parent.spawn(TextBundle::from_section(
					"New Game",
					TextStyle {
						font_size: 32.0,
						color: DEFAULT_PANEL_COLORS.text,
						..default()
					},
				));
			});
	}
}
