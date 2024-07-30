use super::combo_overview::ComboOverview;
use crate::traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn, GetKey};
use bevy::prelude::{BuildChildren, ChildBuilder, Component, Entity, KeyCode, NodeBundle};
use common::traits::get_ui_text::{English, GetUiText, UIText};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct EmptySkill {
	pub(crate) button_entity: Entity,
}

impl GetKey<KeyCode> for EmptySkill {
	fn get_key<'a>(&'a self, key_path: &'a [KeyCode]) -> Option<&'a KeyCode> {
		key_path.last()
	}
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ReKeySkill<TKey = KeyCode> {
	pub(crate) to: TKey,
}

impl GetKey<KeyCode> for ReKeySkill {
	fn get_key<'a>(&'a self, _: &'a [KeyCode]) -> Option<&'a KeyCode> {
		Some(&self.to)
	}
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

impl<TExtra> InstantiateContentOn for KeySelect<TExtra>
where
	TExtra: Clone + Sync + Send + 'static + GetKey<KeyCode>,
{
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let key = match self.extra.get_key(&self.key_path).map(English::ui_text) {
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
