use crate::traits::{
	children::Children,
	colors::{HasBackgroundColor, DEFAULT_PANEL_COLORS},
	get_style::GetStyle,
};
use bevy::{
	hierarchy::{BuildChildren, ChildBuilder},
	prelude::Component,
	render::color::Color,
	text::TextStyle,
	ui::{
		node_bundles::{NodeBundle, TextBundle},
		AlignItems,
		FlexDirection,
		JustifyContent,
		Style,
		Val,
	},
	utils::default,
};

use super::ComboList;

#[derive(Component, Default)]
pub(crate) struct ComboOverview;

impl GetStyle for ComboOverview {
	fn style(&self) -> Style {
		Style {
			width: Val::Vw(100.0),
			height: Val::Vh(100.0),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		}
	}
}

impl HasBackgroundColor for ComboOverview {
	const BACKGROUND_COLOR: Option<Color> = Some(Color::rgba(0.5, 0.5, 0.5, 0.5));
}

impl Children for ComboOverview {
	fn children(&self, parent: &mut ChildBuilder) {
		parent
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Row,
					align_items: AlignItems::Start,
					..default()
				},
				..default()
			})
			.with_children(|parent| {
				add_title(parent, "Combo");
				add_combo_list(parent);
			});
	}
}

fn add_title(parent: &mut ChildBuilder, title: &str) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Center,
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			parent.spawn(TextBundle::from_section(
				title,
				TextStyle {
					font_size: 40.0,
					color: DEFAULT_PANEL_COLORS.text,
					..default()
				},
			));
		});
}

fn add_combo_list(parent: &mut ChildBuilder) {
	parent.spawn(ComboList);
}
