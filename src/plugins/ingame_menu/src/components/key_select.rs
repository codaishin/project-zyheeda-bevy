use super::combo_overview::ComboOverview;
use crate::traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn, GetKey};
use bevy::prelude::{BuildChildren, ChildBuilder, Component, NodeBundle};
use skills::items::slot_key::SlotKey;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ReKeySkill<TKey = SlotKey> {
	pub(crate) to: TKey,
}

impl GetKey<SlotKey> for ReKeySkill {
	fn get_key<'a>(&'a self, _: &'a [SlotKey]) -> Option<&'a SlotKey> {
		Some(&self.to)
	}
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct AppendSkill<TKey = SlotKey> {
	pub(crate) on: TKey,
}

impl<TKey> GetKey<TKey> for AppendSkill<TKey> {
	fn get_key<'a>(&'a self, _: &'a [TKey]) -> Option<&'a TKey> {
		Some(&self.on)
	}
}

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct KeySelect<TExtra, TKey = SlotKey> {
	pub(crate) extra: TExtra,
	pub(crate) key_path: Vec<TKey>,
}

impl<TExtra> GetNode for KeySelect<TExtra> {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl<TExtra> InstantiateContentOn for KeySelect<TExtra>
where
	TExtra: Clone + Sync + Send + 'static + GetKey<SlotKey>,
{
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let Some(key) = self.extra.get_key(&self.key_path) else {
			return;
		};

		parent
			.spawn((self.clone(), ComboOverview::skill_key_button_bundle()))
			.with_children(|parent| {
				parent.spawn(ComboOverview::skill_key_text(*key));
			});
	}
}
