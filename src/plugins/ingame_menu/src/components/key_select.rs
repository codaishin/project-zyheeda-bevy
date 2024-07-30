use super::combo_overview::ComboOverview;
use crate::traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn};
use bevy::prelude::{BuildChildren, ChildBuilder, Component, Entity, KeyCode, NodeBundle};
use common::traits::get_ui_text::{English, GetUiText, UIText};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct EmptySkillButton {
	pub(crate) entity: Entity,
}

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct KeySelect<TExtra, TKey = KeyCode> {
	pub(crate) extra: TExtra,
	pub(crate) key_button: Entity,
	pub(crate) key_path: Vec<TKey>,
}

impl<TExtra> GetNode for KeySelect<TExtra> {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl<TExtra: Clone + Sync + Send + 'static> InstantiateContentOn for KeySelect<TExtra> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let key = match self.key_path.last().map(English::ui_text) {
			Some(UIText::String(key)) => key,
			_ => "?".to_owned(),
		};

		parent
			.spawn((self.clone(), ComboOverview::skill_key_button_bundle()))
			.with_children(|parent| {
				parent.spawn(ComboOverview::skill_key_text(&key));
			});
	}
}
