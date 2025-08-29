use super::start_menu_button::StartMenuButton;
use crate::traits::{LoadUi, colors::PanelColors, insert_ui_content::InsertUiContent};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{
	states::{game_state::GameState, save_state::SaveState},
	traits::handles_localization::LocalizeToken,
};

#[derive(Component)]
#[require(Node = Self::full_screen())]
pub(crate) struct StartMenu;

impl StartMenu {
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
}

impl LoadUi<AssetServer> for StartMenu {
	fn load_ui(_: &mut AssetServer) -> Self {
		Self
	}
}

impl InsertUiContent for StartMenu {
	fn insert_ui_content<TLocalization>(
		&self,
		localization: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: LocalizeToken,
	{
		let title = localization
			.localize_token("project-zyheeda")
			.or_string(|| "Project Zyheeda");
		let new_game = localization
			.localize_token("start-menu-new-game")
			.or_token();
		let continue_game = localization
			.localize_token("start-menu-continue-game")
			.or_token();

		parent
			.spawn(Node {
				margin: UiRect::bottom(Val::Px(30.)),
				..default()
			})
			.with_children(|parent| {
				parent.spawn((
					Text::new(title),
					TextFont {
						font_size: 64.0,
						..default()
					},
					TextColor(PanelColors::DEFAULT.filled.text),
				));
			});
		parent.spawn(StartMenuButton {
			label: new_game,
			trigger_state: GameState::NewGame,
		});
		parent.spawn(StartMenuButton {
			label: continue_game,
			trigger_state: GameState::Save(SaveState::AttemptLoad),
		});
	}
}
