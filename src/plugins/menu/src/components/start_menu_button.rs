use crate::{
	tools::PanelState,
	traits::{
		colors::{HasPanelColors, PanelColors},
		trigger_game_state::TriggerState,
	},
};
use bevy::prelude::*;
use common::{
	errors::Error,
	states::game_state::GameState,
	traits::{
		accessors::get::GetterRef,
		handles_localization::localized::Localized,
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Debug, PartialEq)]
#[require(Button, Node = Self::node())]
pub(crate) struct StartMenuButton {
	pub(crate) label: Localized,
	pub(crate) trigger_state: GameState,
}

impl StartMenuButton {
	fn node() -> Node {
		Node {
			width: Val::Px(300.0),
			height: Val::Px(100.0),
			margin: UiRect::all(Val::Px(2.0)),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		}
	}
}

impl GetterRef<PanelState> for StartMenuButton {
	fn get(&self) -> &PanelState {
		&PanelState::Filled
	}
}

impl HasPanelColors for StartMenuButton {
	const PANEL_COLORS: PanelColors = PanelColors::DEFAULT;
}

impl TriggerState for StartMenuButton {
	type TState = GameState;

	fn trigger_state(&self) -> Self::TState {
		self.trigger_state
	}
}

impl Prefab<()> for StartMenuButton {
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Error> {
		entity.with_child((
			Text::new(self.label.clone()),
			TextFont {
				font_size: 32.0,
				..default()
			},
			TextColor(PanelColors::DEFAULT.text),
		));
		Ok(())
	}
}
