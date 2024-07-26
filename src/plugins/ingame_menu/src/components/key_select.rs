use crate::traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn};
use bevy::prelude::{BuildChildren, ChildBuilder, Component, Entity, KeyCode, NodeBundle};
use common::traits::get_ui_text::{English, GetUiText, UIText};

use super::combo_overview::ComboOverview;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct KeySelect<TKey = KeyCode> {
	pub(crate) target: Entity,
	pub(crate) key_path: Vec<TKey>,
}

impl GetNode for KeySelect {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl InstantiateContentOn for KeySelect {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let key = match self.key_path.last().map(English::ui_text) {
			Some(UIText::String(key)) => key,
			_ => "?".to_owned(),
		};

		parent
			.spawn(ComboOverview::skill_key_button_bundle())
			.with_children(|parent| {
				parent.spawn(ComboOverview::skill_key_text(&key));
			});
	}
}
