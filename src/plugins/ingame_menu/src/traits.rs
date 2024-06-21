pub mod children;
pub mod colors;
pub mod get_style;
pub mod set;

use bevy::{
	asset::Handle,
	hierarchy::ChildBuilder,
	prelude::KeyCode,
	render::{color::Color, texture::Image},
	text::TextStyle,
	ui::{node_bundles::TextBundle, Style, UiRect, Val},
	utils::default,
};
use children::Children;
use colors::{HasBackgroundColor, DEFAULT_PANEL_COLORS};
use get_style::GetStyle;

use crate::components::tooltip::Tooltip;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SkillDescriptor<TKey, TIcon: Clone> {
	pub name: &'static str,
	pub key: TKey,
	pub icon: Option<TIcon>,
}

pub(crate) type CombosDescriptor<TKey, TIcon> = Vec<Vec<SkillDescriptor<TKey, TIcon>>>;

pub(crate) trait UpdateCombos<TKey> {
	fn update_combos(&mut self, combos: CombosDescriptor<TKey, Handle<Image>>);
}

impl<T: Clone> GetStyle for Tooltip<SkillDescriptor<KeyCode, T>> {
	fn style(&self) -> Style {
		Style {
			top: Val::Px(-25.0),
			padding: UiRect::all(Val::Px(5.0)),
			..default()
		}
	}
}

impl<T: Clone> Children for Tooltip<SkillDescriptor<KeyCode, T>> {
	fn children(&self, parent: &mut ChildBuilder) {
		parent.spawn(TextBundle::from_section(
			self.0.name,
			TextStyle {
				font_size: 20.0,
				color: DEFAULT_PANEL_COLORS.filled,
				..default()
			},
		));
	}
}

impl<T: Clone> HasBackgroundColor for Tooltip<SkillDescriptor<KeyCode, T>> {
	const BACKGROUND_COLOR: Option<Color> = Some(DEFAULT_PANEL_COLORS.text);
}
