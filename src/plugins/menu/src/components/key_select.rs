use super::{
	SkillSelectDropdownInsertCommand,
	combo_overview::ComboOverview,
	combo_skill_button::Horizontal,
};
use crate::traits::{GetComponent, GetKey, insert_ui_content::InsertUiContent};
use bevy::prelude::*;
use common::tools::action_key::slot::SlotKey;

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
#[require(Node)]
pub(crate) struct KeySelect<TExtra, TKey = SlotKey> {
	pub(crate) extra: TExtra,
	pub(crate) key_path: Vec<TKey>,
}

impl<TExtra> InsertUiContent for KeySelect<TExtra>
where
	TExtra: GetKey<SlotKey>,
	KeySelect<TExtra>: GetComponent<TInput = ()>,
{
	fn insert_ui_content<TLocalization>(&self, _: &mut TLocalization, parent: &mut ChildBuilder) {
		let Some(component) = self.component(()) else {
			return;
		};
		let Some(key) = self.extra.get_key(&self.key_path) else {
			return;
		};

		parent
			.spawn((component, ComboOverview::skill_key_button()))
			.with_children(|parent| {
				parent.spawn(ComboOverview::skill_key_text(*key));
			});
	}
}

impl<TKey: Copy + Sync + Send + 'static> GetComponent for KeySelect<AppendSkill<TKey>, TKey> {
	type TComponent = SkillSelectDropdownInsertCommand<TKey, Horizontal>;
	type TInput = ();

	fn component(&self, _: ()) -> Option<Self::TComponent> {
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
			select.component(())
		)
	}
}
