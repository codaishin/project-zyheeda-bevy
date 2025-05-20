use super::{Quickbar, input_label::InputLabel, quickbar_panel::QuickbarPanel};
use crate::{
	tools::PanelState,
	traits::{LoadUi, colors::PanelColors, insert_ui_content::InsertUiContent},
};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{tools::action_key::slot::SlotKey, traits::iteration::IterFinite};

#[derive(Component)]
#[require(Node = full_screen())]
pub struct UIOverlay;

fn full_screen() -> Node {
	Node {
		width: Val::Percent(100.0),
		height: Val::Percent(100.0),
		flex_direction: FlexDirection::ColumnReverse,
		..default()
	}
}

impl LoadUi<AssetServer> for UIOverlay {
	fn load_ui(_: &mut AssetServer) -> Self {
		UIOverlay
	}
}

impl InsertUiContent for UIOverlay {
	fn insert_ui_content<TLocalization>(
		&self,
		_: &mut TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) {
		add_quickbar(parent);
	}
}

fn add_quickbar(parent: &mut RelatedSpawnerCommands<ChildOf>) {
	parent
		.spawn((
			Quickbar,
			Node {
				padding: UiRect::all(Val::Px(20.)),
				..default()
			},
		))
		.with_children(|quickbar| {
			for slot_key in SlotKey::iterator() {
				add_slot(quickbar, &slot_key);
			}
		});
}

fn add_slot(quickbar: &mut RelatedSpawnerCommands<ChildOf>, key: &SlotKey) {
	quickbar
		.spawn(Node {
			width: Val::Px(70.0),
			height: Val::Px(70.0),
			margin: UiRect::all(Val::Px(10.0)),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		})
		.with_children(|background| {
			background
				.spawn(get_quickbar_panel(key))
				.with_children(|parent| {
					let font_size = 22.;
					let size = 30.;
					let border = 2.;
					let offset = -size / 2. - border;
					parent
						.spawn((
							Node {
								position_type: PositionType::Absolute,
								left: Val::Px(offset),
								top: Val::Px(offset),
								width: Val::Px(size),
								height: Val::Px(size),
								border: UiRect::all(Val::Px(border)),
								..default()
							},
							BorderColor::from(PanelColors::DEFAULT.text),
							BackgroundColor::from(PanelColors::DEFAULT.filled),
						))
						.with_child((
							TextFont {
								font_size,
								..default()
							},
							InputLabel::<SlotKey> { key: *key },
						));
				});
		});
}

fn get_quickbar_panel(key: &SlotKey) -> (QuickbarPanel, Button, Node) {
	(
		QuickbarPanel {
			key: *key,
			state: PanelState::Empty,
		},
		Button,
		Node {
			width: Val::Percent(100.),
			height: Val::Percent(100.),
			justify_content: JustifyContent::Start,
			align_items: AlignItems::Start,
			..default()
		},
	)
}
