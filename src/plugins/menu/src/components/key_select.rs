use super::{
	combo_overview::ComboOverview,
	skill_button::Horizontal,
	SkillSelectDropdownInsertCommand,
};
use crate::traits::{
	ui_components::GetUIComponents,
	update_children::UpdateChildren,
	GetComponent,
	GetKey,
};
use bevy::prelude::*;
use skills::slot_key::SlotKey;

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

impl<TExtra> GetUIComponents for KeySelect<TExtra> {
	fn ui_components(&self) -> (Node, BackgroundColor) {
		(default(), default())
	}
}

impl<TExtra> UpdateChildren for KeySelect<TExtra>
where
	TExtra: GetKey<SlotKey>,
	KeySelect<TExtra>: GetComponent,
{
	fn update_children(&self, parent: &mut ChildBuilder) {
		let Some(bundle) = self.bundle() else {
			return;
		};
		let Some(key) = self.extra.get_key(&self.key_path) else {
			return;
		};

		parent
			.spawn((bundle, ComboOverview::skill_key_button()))
			.with_children(|parent| {
				parent.spawn(ComboOverview::skill_key_text(*key));
			});
	}
}

impl GetComponent for KeySelect<ReKeySkill> {
	type TComponent = KeySelect<ReKeySkill>;

	fn bundle(&self) -> Option<Self::TComponent> {
		Some(self.clone())
	}
}

impl<TKey: Copy + Sync + Send + 'static> GetComponent for KeySelect<AppendSkill<TKey>, TKey> {
	type TComponent = SkillSelectDropdownInsertCommand<TKey, Horizontal>;

	fn bundle(&self) -> Option<Self::TComponent> {
		Some(SkillSelectDropdownInsertCommand::new(
			[self.key_path.clone(), vec![self.extra.on]].concat(),
		))
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
			Some(SkillSelectDropdownInsertCommand::<_Key, Horizontal>::new(
				vec![_Key::A, _Key::B, _Key::C]
			)),
			select.bundle()
		)
	}
}
