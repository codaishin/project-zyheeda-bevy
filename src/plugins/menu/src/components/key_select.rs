use super::{
	SkillSelectDropdownCommand,
	combo_overview::ComboOverview,
	combo_skill_button::Horizontal,
};
use crate::traits::{GetComponent, GetKey, insert_ui_content::InsertUiContent};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::tools::action_key::slot::{PlayerSlot, SlotKey};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct AppendSkill {
	pub(crate) on: SlotKey,
}

impl GetKey<SlotKey> for AppendSkill {
	fn get_key<'a>(&'a self, _: &'a [SlotKey]) -> Option<&'a SlotKey> {
		Some(&self.on)
	}
}

#[derive(Component, Debug, PartialEq, Clone)]
#[require(Node)]
pub(crate) struct KeySelect<TExtra> {
	pub(crate) extra: TExtra,
	pub(crate) key_path: Vec<SlotKey>,
}

impl<TExtra> InsertUiContent for KeySelect<TExtra>
where
	TExtra: GetKey<SlotKey>,
	KeySelect<TExtra>: GetComponent<TInput = ()>,
{
	fn insert_ui_content<TLocalization>(
		&self,
		_: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) {
		let Some(component) = self.component(()) else {
			return;
		};
		let Some(key) = self.extra.get_key(&self.key_path) else {
			return;
		};
		let Ok(player_slot) = PlayerSlot::try_from(*key) else {
			return;
		};

		parent
			.spawn((component, ComboOverview::skill_key_button()))
			.with_children(|parent| {
				parent.spawn(ComboOverview::skill_key_text(player_slot));
			});
	}
}

impl GetComponent for KeySelect<AppendSkill> {
	type TComponent = SkillSelectDropdownCommand<Horizontal>;
	type TInput = ();

	fn component(&self, _: ()) -> Option<Self::TComponent> {
		Some(SkillSelectDropdownCommand::new(
			[self.key_path.clone(), vec![self.extra.on]].concat(),
		))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn key_select_append_skill_get_bundle() {
		let select = KeySelect {
			extra: AppendSkill { on: SlotKey(204) },
			key_path: vec![SlotKey(0), SlotKey(11)],
		};

		assert_eq!(
			Some(SkillSelectDropdownCommand::<Horizontal>::new(vec![
				SlotKey(0),
				SlotKey(11),
				SlotKey(204)
			])),
			select.component(())
		)
	}
}
