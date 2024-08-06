use super::{
	combo_overview::ComboOverview,
	skill_descriptor::Horizontal,
	SkillSelectDropdownInsertCommand,
};
use crate::traits::{
	get_node::GetNode,
	instantiate_content_on::InstantiateContentOn,
	GetBundle,
	GetKey,
};
use bevy::{
	prelude::{BuildChildren, ChildBuilder, Component, NodeBundle},
	ui::{Style, Val},
	utils::default,
};
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
		let key_button_dimensions = ComboOverview::KEY_BUTTON_DIMENSIONS;

		NodeBundle {
			style: Style {
				width: Val::from(key_button_dimensions.width),
				height: Val::from(key_button_dimensions.height),
				..default()
			},
			..default()
		}
	}
}

impl<TExtra> InstantiateContentOn for KeySelect<TExtra>
where
	TExtra: GetKey<SlotKey>,
	KeySelect<TExtra>: GetBundle,
{
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let Some(key) = self.extra.get_key(&self.key_path) else {
			return;
		};

		parent
			.spawn((self.bundle(), ComboOverview::skill_key_button_bundle()))
			.with_children(|parent| {
				parent.spawn(ComboOverview::skill_key_text(*key));
			});
	}
}

impl GetBundle for KeySelect<ReKeySkill> {
	type TBundle = KeySelect<ReKeySkill>;

	fn bundle(&self) -> Self::TBundle {
		self.clone()
	}
}

impl<TKey: Copy + Sync + Send + 'static> GetBundle for KeySelect<AppendSkill<TKey>, TKey> {
	type TBundle = SkillSelectDropdownInsertCommand<TKey, Horizontal>;

	fn bundle(&self) -> Self::TBundle {
		SkillSelectDropdownInsertCommand::new([self.key_path.clone(), vec![self.extra.on]].concat())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn key_select_append_skill_get_bundle() {
		#[derive(Debug, PartialEq, Clone, Copy)]
		enum _Key {
			A,
			B,
			C,
		}

		let select = KeySelect {
			extra: AppendSkill { on: _Key::C },
			key_path: vec![_Key::A, _Key::B],
		};

		assert_eq!(
			SkillSelectDropdownInsertCommand::<_Key, Horizontal>::new(vec![
				_Key::A,
				_Key::B,
				_Key::C
			]),
			select.bundle()
		)
	}
}
