use super::{Quickbar, input_label::InputLabel, quickbar_panel::QuickbarPanel};
use crate::traits::{LoadUi, colors::PanelColors, insert_ui_content::InsertUiContent};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{tools::action_key::slot::PlayerSlot, traits::iteration::IterFinite};

#[derive(Component)]
#[require(Node = Self::full_screen())]
pub struct UIOverlay;

impl UIOverlay {
	fn full_screen() -> Node {
		Node {
			width: Val::Percent(100.0),
			height: Val::Percent(100.0),
			flex_direction: FlexDirection::ColumnReverse,
			..default()
		}
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
		_: &TLocalization,
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
			for slot_key in PlayerSlot::iterator() {
				add_slot(quickbar, slot_key);
			}
		});
}

fn add_slot(quickbar: &mut RelatedSpawnerCommands<ChildOf>, key: PlayerSlot) {
	let slot_desc_text_size = 22.;
	let slot_desc_size = 30.;
	let slot_desc_border = 2.;
	let slot_desc_offset = -slot_desc_size / 2. - slot_desc_border;

	quickbar.spawn((
		Node {
			width: Val::Px(70.0),
			height: Val::Px(70.0),
			margin: UiRect::all(Val::Px(10.0)),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		},
		children![(
			QuickbarPanel::from(key),
			Button,
			Node {
				width: Val::Percent(100.),
				height: Val::Percent(100.),
				justify_content: JustifyContent::Start,
				align_items: AlignItems::Start,
				..default()
			},
			children![(
				Node {
					position_type: PositionType::Absolute,
					left: Val::Px(slot_desc_offset),
					top: Val::Px(slot_desc_offset),
					width: Val::Px(slot_desc_size),
					height: Val::Px(slot_desc_size),
					border: UiRect::all(Val::Px(slot_desc_border)),
					..default()
				},
				BorderColor::from(PanelColors::DEFAULT.filled.text),
				BackgroundColor::from(PanelColors::DEFAULT.filled.background),
				children![(
					Node {
						width: Val::Px(slot_desc_size - 2. * slot_desc_border),
						height: Val::Px(slot_desc_size - 2. * slot_desc_border),
						..default()
					},
					TextLayout {
						justify: Justify::Center,
						..default()
					},
					TextFont {
						font_size: slot_desc_text_size,
						..default()
					},
					TextColor(PanelColors::DEFAULT.filled.text),
					InputLabel { key },
				)],
			)],
		)],
	));
}
