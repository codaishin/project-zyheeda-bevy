pub mod children;
pub mod colors;
pub mod get_node;
pub mod set;

use bevy::{
	asset::Handle,
	hierarchy::ChildBuilder,
	prelude::KeyCode,
	render::texture::Image,
	text::TextStyle,
	ui::{
		node_bundles::{NodeBundle, TextBundle},
		Style,
		UiRect,
		Val,
	},
	utils::default,
};
use children::Children;
use colors::DEFAULT_PANEL_COLORS;
use get_node::GetNode;

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

impl<T: Clone> GetNode for Tooltip<SkillDescriptor<KeyCode, T>> {
	fn node(&self) -> NodeBundle {
		NodeBundle {
			style: Style {
				top: Val::Px(-25.0),
				padding: UiRect::all(Val::Px(5.0)),
				..default()
			},
			background_color: DEFAULT_PANEL_COLORS.text.into(),
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
